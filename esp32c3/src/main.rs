#![no_std]
// #![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(type_alias_impl_trait)]

use display_interface_spi::SPIInterface;
use electricity_exhange::{
    storage::NonVolatileStorage,
    tasks::broker,
    wifi::{self, WifiPeripherals},
};
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::Channel,
    mutex::Mutex,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    clock::ClockControl,
    gpio::{Io, Level, Output, NO_PIN},
    interrupt::Priority,
    peripherals::Peripherals,
    prelude::*,
    rng::Rng,
    spi::{master::Spi, SpiMode},
    system::SystemControl,
    timer::timg::TimerGroup,
};
use esp_hal_embassy::InterruptExecutor;
use shared::{DisplayUpdate, Message, Response};
use static_cell::StaticCell;

use esp_backtrace as _; // Panic behaviour

/// Send incoming messages to this channel for broker task to handle
static BROKER_CHANNEL: Channel<CriticalSectionRawMutex, Message, 10> = Channel::new();

/// Send responses to this channel for the serial_write task to handle
static WRITER_CHANNEL: Channel<CriticalSectionRawMutex, Response, 10> = Channel::new();

/// Send updates to display
static DISPLAY_CHANNEL: Channel<CriticalSectionRawMutex, DisplayUpdate, 10> = Channel::new();

/// Can be used to access non-volatile storage
static NVS_STORAGE: StaticCell<Mutex<NoopRawMutex, NonVolatileStorage>> = StaticCell::new();

/// Executor used by display task
static HIGH_PRIO_EXECUTOR: StaticCell<InterruptExecutor<2>> = StaticCell::new();

#[main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    esp_hal_embassy::init(&clocks, timg0);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let sclk = io.pins.gpio12;
    let mosi = io.pins.gpio13;
    let cs = io.pins.gpio10;

    let spi = Spi::new(peripherals.SPI2, 80.MHz(), SpiMode::Mode0, &clocks).with_pins(
        Some(sclk),
        Some(mosi),
        NO_PIN,
        NO_PIN,
    );

    let cs = Output::new(cs, Level::High);
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let dc = Output::new(io.pins.gpio7, Level::High);
    let di = SPIInterface::new(spi_device, dc);

    let rst = Output::new(io.pins.gpio8, Level::High);

    // let display_channel = DISPLAY_CHANNEL.take();

    let high_prio_executor = HIGH_PRIO_EXECUTOR.init(InterruptExecutor::new(
        system.software_interrupt_control.software_interrupt2,
    ));

    let high_prio_spawner = high_prio_executor.start(Priority::Priority3);

    electricity_exhange::display::setup(&high_prio_spawner, di, rst, DISPLAY_CHANNEL.receiver());

    let rng = Rng::new(peripherals.RNG);

    let display_sender = DISPLAY_CHANNEL.sender();

    let nvs_storage: &'static Mutex<NoopRawMutex, NonVolatileStorage> =
        &*NVS_STORAGE.init(Mutex::new(NonVolatileStorage::take()));

    let wifi_peripherals = WifiPeripherals {
        systimer: peripherals.SYSTIMER,
        radio_clk: peripherals.RADIO_CLK,
        clocks: &clocks,
        wifi: peripherals.WIFI,
    };

    let stack = wifi::connect(&spawner, rng, wifi_peripherals, display_sender, nvs_storage)
        .await
        .unwrap();

    // let broker_channel = BROKER_CHANNEL.take();
    // let writer_channel = WRITER_CHANNEL.take();

    spawner.must_spawn(broker(
        BROKER_CHANNEL.receiver(),
        WRITER_CHANNEL.sender(),
        display_sender,
        nvs_storage,
    ));

    electricity_exhange::serial::setup(
        &spawner,
        peripherals.UART0,
        &clocks,
        BROKER_CHANNEL.sender(),
        WRITER_CHANNEL.receiver(),
        display_sender,
    )
    .await
    .unwrap();

    let mut client = electricity_exhange::client::Client::new(stack);

    electricity_exhange::datetime::setup_datetime(&spawner, &mut client, display_sender).await;

    display_sender.send("Device init done.".into()).await;

    // loop {

    //     Timer::after(Duration::from_millis(500)).await;
    // }
}
