use core::{fmt::Error, str::FromStr};

use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Sender};
use embassy_time::{Duration, Timer};
use esp_hal::{
    clock::Clocks,
    peripherals::{RADIO_CLK, SYSTIMER, WIFI},
    rng::Rng,
};
use esp_println::println;
use esp_wifi::{
    wifi::{self, WifiController, WifiDevice, WifiEvent, WifiStaDevice},
    EspWifiInitFor,
};
use heapless::String;
use shared::DisplayUpdate;
use static_cell::StaticCell;

use crate::storage::{NonVolatileKey, NonVolatileStorage};

static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
static STACK: StaticCell<Stack<WifiDevice<'static, WifiStaDevice>>> = StaticCell::new();

/// SSID for WiFi network
// const WIFI_SSID: &str = env!("WIFI_SSID");

/// Password for WiFi network
// const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

#[allow(clippy::too_many_arguments)]
pub async fn connect(
    spawner: &Spawner,
    mut rng: Rng,
    systimer: SYSTIMER,
    radio_clk: RADIO_CLK,
    clocks: &Clocks<'_>,
    wifi: WIFI,
    display_sender: Sender<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
    nvs_storage: &'static Mutex<NoopRawMutex, NonVolatileStorage>,
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

    // Use scope so that mutex guard is dropped
    {
        let mut guard = nvs_storage.lock().await;
        let wifi_password = guard
            .fetch(NonVolatileKey::WifiPassword)
            .await
            .unwrap()
            .unwrap()
            .0;
        let wifi_ssid = guard
            .fetch(NonVolatileKey::WifiSsid)
            .await
            .unwrap()
            .unwrap()
            .0;

        let mut msg = String::<64>::new();
        write!(msg, "{wifi_ssid}\n{wifi_password}").unwrap();
        display_sender.send(DisplayUpdate::StatusUpdate(msg)).await;

        spawner.must_spawn(connection(controller, wifi_password, wifi_ssid));
    }

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
async fn connection(
    mut controller: WifiController<'static>,
    wifi_password: String<64>,
    wifi_ssid: String<64>,
) {
    let wifi_ssid = String::from_str(wifi_ssid.as_ref()).unwrap();

    println!("{wifi_ssid}\n{wifi_password}");

    // Todo send error message to display and/or serial if wifi is not connected within certain timeframe?
    loop {
        if wifi::get_wifi_state() == wifi::WifiState::StaConnected {
            // Wait until device is no longer connected to the wifi
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            // Wait before trying to connect again?
            Timer::after(Duration::from_millis(3000)).await
        }

        if !matches!(controller.is_started(), Ok(true)) {
            // let client_config = wifi::Configuration::Client(wifi::ClientConfiguration {
            //     ssid: String::from_str(WIFI_SSID).unwrap(),
            //     password: String::from_str(WIFI_PASSWORD).unwrap(),
            //     ..Default::default()
            // });

            let client_config = wifi::Configuration::Client(wifi::ClientConfiguration {
                ssid: wifi_ssid.clone(),
                password: wifi_password.clone(),
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
