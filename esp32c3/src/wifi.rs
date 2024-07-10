use core::{fmt::Error, str::FromStr};

use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::{
    clock::Clocks,
    peripherals::{RADIO_CLK, SYSTIMER, WIFI},
    rng::Rng,
};
use esp_wifi::{
    wifi::{self, WifiController, WifiDevice, WifiEvent, WifiStaDevice},
    EspWifiInitFor,
};
use heapless::String;
use static_cell::StaticCell;

static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
static STACK: StaticCell<Stack<WifiDevice<'static, WifiStaDevice>>> = StaticCell::new();

/// SSID for WiFi network
const WIFI_SSID: &str = env!("WIFI_SSID");

/// Password for WiFi network
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

pub async fn connect(
    spawner: &Spawner,
    mut rng: Rng,
    systimer: SYSTIMER,
    radio_clk: RADIO_CLK,
    clocks: &Clocks<'_>,
    wifi: WIFI,
) -> Result<&'static Stack<WifiDevice<'static, WifiStaDevice>>, Error> {
    let seed1 = rng.random();
    let seed2 = rng.random();

    let mut bytes = [0u8; 8];
    bytes[0..4].copy_from_slice(&seed1.to_le_bytes());
    bytes[4..].copy_from_slice(&seed2.to_le_bytes());

    let seed = u64::from_le_bytes(bytes);

    // WIFI stuff
    let timer = esp_hal::timer::systimer::SystemTimer::new(systimer).alarm0;
    let init = esp_wifi::initialize(EspWifiInitFor::Wifi, timer, rng, radio_clk, clocks).unwrap();

    let (wifi_controller, controller) = wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

    let config = embassy_net::Config::dhcpv4(Default::default());

    let stack_resources = STACK_RESOURCES.init(StackResources::new());

    let stack = Stack::new(wifi_controller, config, stack_resources, seed);
    let stack = &*STACK.init(stack);

    spawner.must_spawn(connection(controller));
    spawner.must_spawn(net_task(stack));

    while !stack.is_link_up() {
        Timer::after(Duration::from_millis(500)).await;
    }

    while stack.config_v4().is_none() {
        Timer::after(Duration::from_millis(500)).await;
    }

    Ok(stack)
}

// Todo error propagation to host program if wifi connection fails
#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    loop {
        if wifi::get_wifi_state() == wifi::WifiState::StaConnected {
            // Wait until device is no longer connected to the wifi
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            // Wait before trying to connect again?
            Timer::after(Duration::from_millis(3000)).await
        }

        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = wifi::Configuration::Client(wifi::ClientConfiguration {
                ssid: String::from_str(WIFI_SSID).unwrap(),
                password: String::from_str(WIFI_PASSWORD).unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            controller
                .start()
                .await
                .expect("Failed to start the controller");
        }

        match controller.connect().await {
            Ok(_) => {}
            Err(_) => {
                Timer::after(Duration::from_millis(3000)).await;
            }
        }
    }
}

/// This needs to be running in the background for wifi to work?
#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}
