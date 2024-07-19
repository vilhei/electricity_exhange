#![no_std]
// #![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::str::FromStr;

use display_interface_spi::SPIInterface;
use electricity_exhange::{storage::NonVolatileStorage, wifi};
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::{Channel, Receiver, Sender},
    mutex::Mutex,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    clock::ClockControl,
    gpio::{Gpio9, Input, Io, Level, Output, NO_PIN},
    interrupt::Priority,
    peripherals::Peripherals,
    prelude::*,
    rng::Rng,
    spi::{master::Spi, SpiMode},
    system::SystemControl,
    timer::timg::TimerGroup,
};
use esp_hal_embassy::InterruptExecutor;
// use esp_println::println;
use heapless::String;
use shared::{DisplayBrightness, DisplayUpdate, Message, Response};
use static_cell::{ConstStaticCell, StaticCell};

use esp_backtrace as _; // Panic behaviour

/// Send incoming messages to this channel for broker task to handle
static BROKER_CHANNEL: ConstStaticCell<Channel<NoopRawMutex, Message, 10>> =
    ConstStaticCell::new(Channel::new());

/// Send responses to this channel for the serial_write task to handle
static WRITER_CHANNEL: ConstStaticCell<Channel<NoopRawMutex, Response, 10>> =
    ConstStaticCell::new(Channel::new());

/// Send updates to display
static DISPLAY_CHANNEL: ConstStaticCell<Channel<CriticalSectionRawMutex, DisplayUpdate, 10>> =
    ConstStaticCell::new(Channel::new());

/// Can be used to access non-volatile storage
static NVS_STORAGE: StaticCell<Mutex<NoopRawMutex, NonVolatileStorage>> = StaticCell::new();

/// Executor used by display task
static HIGH_PRIO_EXECUTOR: StaticCell<InterruptExecutor<2>> = StaticCell::new();

#[embassy_executor::task]
async fn broker(
    broker_receiver: Receiver<'static, NoopRawMutex, Message, 10>,
    serial_writer_sender: Sender<'static, NoopRawMutex, Response, 10>,
) {
    loop {
        let message = broker_receiver.receive().await;

        match message {
            Message::Wifi(_) => serial_writer_sender.send(Response::Ok).await,
            Message::FingridApiKey(_) => todo!(),
            Message::EntsoeApiKey(_) => todo!(),
            Message::Display(_) => todo!(),
        }
    }
}

#[embassy_executor::task]
async fn button_task(
    mut button: Input<'static, Gpio9>,
    display_sender: Sender<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
) {
    let mut display_on = 0;
    display_sender
        .send(DisplayUpdate::StatusUpdate(
            String::from_str("Started brightness task").unwrap(),
        ))
        .await;

    loop {
        button.wait_for_falling_edge().await;

        display_on = (display_on + 1) % 3;
        let brightness = match display_on {
            0 => DisplayBrightness::Low,
            1 => DisplayBrightness::Normal,
            2 => DisplayBrightness::High,
            _ => unreachable!(),
        };

        display_sender
            .send(DisplayUpdate::SetBrightness(brightness))
            .await;
    }
}

#[main]
async fn main(spawner: Spawner) {
    // println!("Init!");
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    esp_hal_embassy::init(&clocks, timg0);

    // let mut delay = esp_hal::delay::Delay::new(&clocks);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let sclk = io.pins.gpio12;
    let mosi = io.pins.gpio13;
    let cs = io.pins.gpio10;

    // let dma = Dma::new(peripherals.DMA);

    // let descriptors = DMA_DESCRIPTORS.take();

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

    let display_channel = DISPLAY_CHANNEL.take();

    let high_prio_executor = HIGH_PRIO_EXECUTOR.init(InterruptExecutor::new(
        system.software_interrupt_control.software_interrupt2,
    ));

    let high_prio_spawner = high_prio_executor.start(Priority::Priority3);

    electricity_exhange::display::setup(&high_prio_spawner, di, rst, display_channel.receiver());

    // let mut display = mipidsi::Builder::new(ST7789, di)
    //     .reset_pin(rst)
    //     .display_size(240, 320)
    //     .orientation(Orientation::new().rotate(Rotation::Deg90))
    //     .invert_colors(ColorInversion::Inverted)
    //     .init(&mut Delay)
    //     .unwrap();

    // display.clear(Rgb565::WHITE).unwrap();
    // let a = peripherals.UART0;
    // let uart = Uart::new_async(peripherals.UART0, &clocks);
    // let (tx, rx) = uart.split();

    let rng = Rng::new(peripherals.RNG);
    // dbg!("");
    // println!("Starting wifi connect");

    let display_sender = display_channel.sender();

    display_sender
        .send(DisplayUpdate::StatusUpdate(
            String::from_str("started Wifi init").unwrap(),
        ))
        .await;

    let nvs = NonVolatileStorage::take();
    let nvs_storage = NVS_STORAGE.init(Mutex::new(nvs));
    // {
    //     println!("Saving to nvs storage");
    //     let mut guard = nvs_storage.lock().await;
    //     guard
    //         .store(
    //             electricity_exhange::storage::NonVolatileKey::WifiSsid,
    //             NonVolatileItem::new("VHouHou2.4"),
    //         )
    //         .await
    //         .unwrap();

    //     guard
    //         .store(
    //             electricity_exhange::storage::NonVolatileKey::WifiPassword,
    //             NonVolatileItem::new("M98p26a10s"),
    //         )
    //         .await
    //         .unwrap();
    //     println!("Saved to nvs storage");
    // }

    let stack = wifi::connect(
        &spawner,
        rng,
        peripherals.SYSTIMER,
        peripherals.RADIO_CLK,
        &clocks,
        peripherals.WIFI,
        display_sender,
        nvs_storage,
    )
    .await
    .unwrap();

    display_sender
        .send(DisplayUpdate::StatusUpdate(
            String::from_str("Wifi init done").unwrap(),
        ))
        .await;

    // println!("Building client");
    let mut _client = electricity_exhange::client::Client::new(stack);
    // println!("Requesting");
    // let res = client.request().await;
    // println!("Done Requesting");

    let broker_channel = BROKER_CHANNEL.take();
    let writer_channel = WRITER_CHANNEL.take();

    // println!("{:#?}", res);

    display_sender
        .send(DisplayUpdate::StatusUpdate(
            String::from_str("Starting broker task").unwrap(),
        ))
        .await;
    spawner
        .spawn(broker(broker_channel.receiver(), writer_channel.sender()))
        .expect("Failed to spawn read serial");
    // println!("Starting serial setup");

    display_sender
        .send(DisplayUpdate::StatusUpdate(
            String::from_str("Serial init").unwrap(),
        ))
        .await;

    electricity_exhange::serial::setup(
        &spawner,
        peripherals.UART0,
        &clocks,
        broker_channel.sender(),
        writer_channel.receiver(),
    )
    .unwrap();

    display_sender
        .send(DisplayUpdate::StatusUpdate(
            String::from_str("Serial init done").unwrap(),
        ))
        .await;

    display_sender
        .send(DisplayUpdate::StatusUpdate(
            String::from_str("Device init done!").unwrap(),
        ))
        .await;

    // let button = Input::new(io.pins.gpio9, esp_hal::gpio::Pull::Up);

    // spawner.must_spawn(button_task(button, display_sender));

    // loop {

    //     Timer::after(Duration::from_millis(500)).await;
    // }
}
