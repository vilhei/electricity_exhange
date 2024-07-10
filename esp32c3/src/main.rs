#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::str::FromStr;

use display_interface_spi::SPIInterface;
use electricity_exhange::wifi;
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::{Channel, Receiver, Sender},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    clock::ClockControl,
    gpio::{Io, Level, Output, NO_PIN},
    interrupt::Priority,
    peripherals::Peripherals,
    prelude::*,
    rng::Rng,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    system::SystemControl,
    timer::timg::TimerGroup,
};
use esp_hal_embassy::InterruptExecutor;
// use esp_println::println;
use heapless::String;
use shared::{DisplayMessage, Message, Response};
use static_cell::{ConstStaticCell, StaticCell};

use esp_backtrace as _; // Panic behaviour

/// Send incoming messages to this channel for broker task to handle
static BROKER_CHANNEL: ConstStaticCell<Channel<NoopRawMutex, Message, 10>> =
    ConstStaticCell::new(Channel::new());

/// Send responses to this channel for the serial_write task to handle
static WRITER_CHANNEL: ConstStaticCell<Channel<NoopRawMutex, Response, 10>> =
    ConstStaticCell::new(Channel::new());

static DISPLAY_CHANNEL: ConstStaticCell<Channel<CriticalSectionRawMutex, DisplayMessage, 10>> =
    ConstStaticCell::new(Channel::new());

// static DMA_DESCRIPTORS: ConstStaticCell<([DmaDescriptor; 5], [DmaDescriptor; 5])> =
// ConstStaticCell::new(dma_descriptors!(16384));

static HIGH_PRIO_EXECUTOR: StaticCell<InterruptExecutor<2>> = StaticCell::new();

// #[embassy_executor::task]
// async fn read_serial(
//     mut rx: UartRx<'static, UART0, Async>,
//     sender: Sender<'static, NoopRawMutex, Message, 10>,
// ) {
//     let mut message = Vec::<u8, MESSAGE_SIZE>::new();
//     let mut buf = [0; 1];

//     loop {
//         let _ = rx.read_exact(&mut buf).await;
//         if message.is_full() {
//             panic!("Message buffer is full")
//         }
//         message.push(buf[0]).unwrap();
//         if buf[0] == corncobs::ZERO {
//             let deserialized = deserialize_crc_cobs(&message);
//             message.clear();
//             sender.send(deserialized).await;
//         }
//     }
// }

#[embassy_executor::task]
async fn broker(
    receiver: Receiver<'static, NoopRawMutex, Message, 10>,
    writer_sender: Sender<'static, NoopRawMutex, Response, 10>,
) {
    loop {
        let message = receiver.receive().await;

        match message {
            Message::Wifi(_) => writer_sender.send(Response::Ok).await,
            Message::FingridApiKey(_) => todo!(),
            Message::EntsoeApiKey(_) => todo!(),
        }
    }
}

// #[embassy_executor::task]
// async fn write_serial(
//     receiver: Receiver<'static, NoopRawMutex, Response, 10>,
//     mut tx: UartTx<'static, UART0, Async>,
// ) {
//     loop {
//         let response = receiver.receive().await;
//         let mut buf = [0; RESPONSE_SIZE];
//         let serialized = serialize_crc_cobs::<Response, RESPONSE_SIZE>(response, &mut buf);
//         tx.write_async(serialized).await.unwrap();
//     }
// }

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
        .send(DisplayMessage::StatusUpdate(
            String::from_str("Wifi init").unwrap(),
        ))
        .await;

    let stack = wifi::connect(
        &spawner,
        rng,
        peripherals.SYSTIMER,
        peripherals.RADIO_CLK,
        &clocks,
        peripherals.WIFI,
    )
    .await
    .unwrap();

    display_sender
        .send(DisplayMessage::StatusUpdate(
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
        .send(DisplayMessage::StatusUpdate(
            String::from_str("Starting broker task").unwrap(),
        ))
        .await;
    spawner
        .spawn(broker(broker_channel.receiver(), writer_channel.sender()))
        .expect("Failed to spawn read serial");
    // println!("Starting serial setup");

    display_sender
        .send(DisplayMessage::StatusUpdate(
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
        .send(DisplayMessage::StatusUpdate(
            String::from_str("Serial init done").unwrap(),
        ))
        .await;
    // println!("Done serial setup");

    // let mut buf = [0; 32];
    // let mut s_buf = String::<32>::new();
    // loop {
    //     display.clear(Rgb565::WHITE).unwrap();

    //     Text::with_alignment(
    //         "reading",
    //         Point { x: 100, y: 100 },
    //         TEXT_STYLE,
    //         Alignment::Center,
    //     )
    //     .draw(&mut display)
    //     .unwrap();

    //     s_buf.clear();
    //     // write!(s_buf, "{}", buf[0]).unwrap();
    //     display.clear(Rgb565::WHITE).unwrap();

    //     Text::with_alignment(
    //         &s_buf,
    //         Point { x: 100, y: 100 },
    //         TEXT_STYLE,
    //         Alignment::Center,
    //     )
    //     .draw(&mut display)
    //     .unwrap();
    //     Timer::after(Duration::from_millis(500)).await;
    // }
}
