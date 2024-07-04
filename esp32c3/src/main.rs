#![no_std]
#![no_main]

//! Embassy SPI
//!
//! Folowing pins are used:
//! SCLK    GPIO0
//! MISO    GPIO2
//! MOSI    GPIO4
//! CS      GPIO5
//!
//! Depending on your target and the board you are using you have to change the
//! pins.
//!
//! Connect MISO and MOSI pins to see the outgoing data is read as incoming
//! data.
//!
//! This is an example of running the embassy executor with SPI.

//% CHIPS: esp32 esp32c2 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3
//% FEATURES: async embassy embassy-time-timg0 embassy-generic-timers

use core::any::type_name;
use display_interface_spi::SPIInterface;
use electricity_exhange::styles::TEXT_STYLE;
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    channel::{Channel, Receiver, Sender},
};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::{Rgb565, RgbColor},
    text::{Alignment, Text},
    Drawable,
};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io_async::Read;
use esp_hal::{
    clock::ClockControl,
    dma::Dma,
    dma_descriptors,
    gpio::{Io, Level, Output, NO_PIN},
    peripherals::{Peripherals, UART0},
    prelude::*,
    riscv::asm::wfi,
    rng::Rng,
    spi::{
        master::{prelude::*, Spi},
        SpiMode,
    },
    system::SystemControl,
    timer::timg::TimerGroup,
    uart::{Uart, UartRx, UartTx},
    Async,
};
use esp_println::println;
use heapless::{String, Vec};
use mipidsi::{
    models::ST7789,
    options::{ColorInversion, Orientation, Rotation},
};
use shared::{deserialize_crc_cobs, serialize_crc_cobs, Message, Response, IN_SIZE, OUT_SIZE};
use static_cell::StaticCell;

use esp_backtrace as _; // Panic behaviour

static BROKER_CHANNEL: StaticCell<Channel<NoopRawMutex, Message, 10>> = StaticCell::new();
static WRITER_CHANNEL: StaticCell<Channel<NoopRawMutex, Response, 10>> = StaticCell::new();

#[embassy_executor::task]
async fn read_serial(
    mut rx: UartRx<'static, UART0, Async>,
    sender: Sender<'static, NoopRawMutex, Message, 10>,
) {
    let mut message = Vec::<u8, IN_SIZE>::new();
    let mut buf = [0; 1];

    loop {
        let _ = rx.read_exact(&mut buf).await;
        if message.is_full() {
            panic!("Message buffer is full")
        }
        message.push(buf[0]).unwrap();
        if buf[0] == corncobs::ZERO {
            let deserialized = deserialize_crc_cobs(&message);
            message.clear();
            sender.send(deserialized).await;
        }
    }
}

#[embassy_executor::task]
async fn broker(
    receiver: Receiver<'static, NoopRawMutex, Message, 10>,
    writer_sender: Sender<'static, NoopRawMutex, Response, 10>,
) {
    loop {
        let message = receiver.receive().await;

        match message {
            Message::Wifi(_) => {
                // Todo update the wifi information
                writer_sender.send(Response::Ok).await;
            }
            Message::FingridApiKey(_) => todo!(),
        }
    }
}

#[embassy_executor::task]
async fn write_serial(
    receiver: Receiver<'static, NoopRawMutex, Response, 10>,
    mut tx: UartTx<'static, UART0, Async>,
) {
    loop {
        let response = receiver.receive().await;
        let mut buf = [0; OUT_SIZE];
        let serialized = serialize_crc_cobs::<Response, OUT_SIZE>(response, &mut buf);
        tx.write_async(serialized).await.unwrap();
    }
}

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
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

    let dma = Dma::new(peripherals.DMA);

    let (mut descriptors, mut rx_descriptors) = dma_descriptors!(16384);

    let spi = Spi::new(peripherals.SPI2, 80.MHz(), SpiMode::Mode0, &clocks)
        .with_pins(Some(sclk), Some(mosi), NO_PIN, NO_PIN)
        .with_dma(dma.channel0.configure(
            false,
            &mut descriptors,
            &mut rx_descriptors,
            esp_hal::dma::DmaPriority::Priority1,
        ));
    let cs = Output::new(cs, Level::High);
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let dc = Output::new(io.pins.gpio7, Level::High);
    let di = SPIInterface::new(spi_device, dc);

    let rst = Output::new(io.pins.gpio8, Level::High);

    let mut display = mipidsi::Builder::new(ST7789, di)
        .reset_pin(rst)
        .display_size(240, 320)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .invert_colors(ColorInversion::Inverted)
        .init(&mut Delay)
        .unwrap();
    display.clear(Rgb565::WHITE).unwrap();

    let uart = Uart::new_async(peripherals.UART0, &clocks);
    let (tx, rx) = uart.split();

    let broker_channel = BROKER_CHANNEL.init(Channel::new());
    let writer_channel = WRITER_CHANNEL.init(Channel::new());

    spawner
        .spawn(read_serial(rx, broker_channel.sender()))
        .expect("Failed to spawn read serial");

    spawner
        .spawn(broker(broker_channel.receiver(), writer_channel.sender()))
        .expect("Failed to spawn read serial");

    spawner
        .spawn(write_serial(writer_channel.receiver(), tx))
        .expect("Failed to spawn serial writer task");

    // let mut buf = [0; 32];
    let mut s_buf = String::<32>::new();
    loop {
        display.clear(Rgb565::WHITE).unwrap();

        Text::with_alignment(
            "reading",
            Point { x: 100, y: 100 },
            TEXT_STYLE,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();

        s_buf.clear();
        // write!(s_buf, "{}", buf[0]).unwrap();
        display.clear(Rgb565::WHITE).unwrap();

        Text::with_alignment(
            &s_buf,
            Point { x: 100, y: 100 },
            TEXT_STYLE,
            Alignment::Center,
        )
        .draw(&mut display)
        .unwrap();
        Timer::after(Duration::from_millis(500)).await;
    }
}
