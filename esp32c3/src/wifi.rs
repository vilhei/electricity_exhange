use core::fmt::Error;

use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use esp_hal::{
    clock::Clocks,
    peripheral::Peripheral,
    peripherals::{RADIO_CLK, RNG, SYSTIMER, WIFI},
    rng::Rng,
};
use esp_wifi::{
    wifi::{WifiDevice, WifiStaDevice},
    EspWifiInitFor,
};
use static_cell::StaticCell;

static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
static STACK: StaticCell<Stack<WifiDevice<'static, WifiStaDevice>>> = StaticCell::new();

/// SSID for WiFi network
const WIFI_SSID: &str = env!("WIFI_SSID");

/// Password for WiFi network
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

pub async fn connect(
    spawner: &Spawner,
    rng: impl Peripheral<P = RNG>,
    systimer: SYSTIMER,
    radio_clk: RADIO_CLK,
    clocks: &Clocks<'_>,
    wifi: WIFI,
) -> Result<&'static Stack<WifiDevice<'static, WifiStaDevice>>, Error> {
    let mut rng = Rng::new(rng);
    let seed1 = rng.random();
    let seed2 = rng.random();

    let mut bytes = [0u8; 8];
    bytes[0..4].copy_from_slice(&seed1.to_le_bytes());
    bytes[4..].copy_from_slice(&seed2.to_le_bytes());

    let seed = u64::from_le_bytes(bytes);

    // WIFI stuff
    let timer = esp_hal::timer::systimer::SystemTimer::new(systimer).alarm0;
    let init = esp_wifi::initialize(EspWifiInitFor::Wifi, timer, rng, radio_clk, clocks).unwrap();

    let (wifi_controller, controller) =
        esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

    let config = embassy_net::Config::dhcpv4(Default::default());

    let stack_resources = STACK_RESOURCES.init(StackResources::new());

    let stack = Stack::new(wifi_controller, config, stack_resources, seed);
    let stack = STACK.init(stack);

    spawner.spawn(net_task(stack)).unwrap();

    todo!()
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}
