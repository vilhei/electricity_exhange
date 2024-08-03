use core::{fmt::Error, str::FromStr};

use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Sender};
use embassy_time::{Duration, Timer};
use esp_hal::timer::systimer::SystemTimer;
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
use shared::DisplayUpdate;
use static_cell::StaticCell;

use crate::generate_rand_u64;
use crate::storage::{NonVolatileKey, NonVolatileStorage};

static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
static STACK: StaticCell<Stack<WifiDevice<'static, WifiStaDevice>>> = StaticCell::new();

pub struct WifiPeripherals<'a> {
    pub systimer: SYSTIMER,
    pub radio_clk: RADIO_CLK,
    pub clocks: &'a Clocks<'a>,
    pub wifi: WIFI,
}

pub async fn connect(
    spawner: &Spawner,
    mut rng: Rng,
    wifi_peripherals: WifiPeripherals<'_>,
    display_sender: Sender<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
    nvs_storage: &'static Mutex<NoopRawMutex, NonVolatileStorage>,
) -> Result<&'static Stack<WifiDevice<'static, WifiStaDevice>>, Error> {
    display_sender
        .send(DisplayUpdate::StatusUpdate(
            String::from_str("started Wifi init").unwrap(),
        ))
        .await;

    let seed = generate_rand_u64(&mut rng);

    // WIFI stuff
    let timer = SystemTimer::new(wifi_peripherals.systimer).alarm0;
    let init = esp_wifi::initialize(
        EspWifiInitFor::Wifi,
        timer,
        rng,
        wifi_peripherals.radio_clk,
        wifi_peripherals.clocks,
    )
    .unwrap();

    let (wifi_controller, controller) =
        wifi::new_with_mode(&init, wifi_peripherals.wifi, WifiStaDevice).unwrap();

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

        // Todo remove showing wifi credentials in final build
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
    display_sender
        .send(DisplayUpdate::StatusUpdate(
            String::from_str("Wifi init done").unwrap(),
        ))
        .await;

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

    // Todo send error message to display and/or serial if wifi is not connected within certain timeframe?
    loop {
        if wifi::get_wifi_state() == wifi::WifiState::StaConnected {
            // Wait until device is no longer connected to the wifi
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            // Wait before trying to connect again?
            Timer::after(Duration::from_millis(3000)).await
        }

        if !matches!(controller.is_started(), Ok(true)) {
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
