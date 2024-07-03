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
use core::fmt::Write;
use display_interface_spi::SPIInterface;
use electricity_exhange::styles::TEXT_STYLE;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::Drawable;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io_async::Read;
use esp_backtrace as _;
use esp_hal::dma::Dma;
use esp_hal::peripherals::UART0;
use esp_hal::riscv::asm::wfi;
use esp_hal::spi::master::prelude::_esp_hal_spi_master_dma_WithDmaSpi2;
use esp_hal::spi::master::Spi;
use esp_hal::uart::{Uart, UartRx, UartTx};
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{Io, Level, Output, NO_PIN},
    peripherals::Peripherals,
    prelude::*,
    spi::SpiMode,
    system::SystemControl,
    timer::timg::TimerGroup,
};
use esp_hal::{dma_descriptors, Async};
use esp_println::println;
use heapless::{String, Vec};
use mipidsi::models::ST7789;
use mipidsi::options::{Orientation, Rotation};
use shared::{deserialize_crc_cobs, Message, IN_SIZE};
use static_cell::StaticCell;

static BROKER_CHANNEL: StaticCell<Channel<NoopRawMutex, Message, 10>> = StaticCell::new();

#[embassy_executor::task]
async fn read_serial(
    mut rx: UartRx<'static, UART0, Async>,
    sender: Sender<'static, NoopRawMutex, Message, 10>,
) {
    // println!("Reading serial 3");
    let mut message = Vec::<u8, IN_SIZE>::new();
    let mut buf = [0; 1];

    loop {
        let _ = rx.read_exact(&mut buf).await;
        // println!("Reading serial");
        if message.is_full() {
            panic!("Message buffer is full")
        }
        message.push(buf[0]).unwrap();
        if buf[0] == corncobs::ZERO {
            // let a = message.into_array().unwrap();
            let deserialized = deserialize_crc_cobs(&message);
            message.clear();
            sender.send(deserialized).await;
        }
    }
}

#[embassy_executor::task]
async fn broker(
    receiver: Receiver<'static, NoopRawMutex, Message, 10>,
    mut tx: UartTx<'static, UART0, Async>,
) {
    loop {
        let message = receiver.receive().await;

        match message {
            Message::Wifi(w) => {
                tx.write_async(w.get_password().as_bytes()).await.unwrap();
            }
            Message::FingridApiKey(_) => todo!(),
        }
    }
}

fn test(a: i32) {
    todo!()
}

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

#[main]
async fn main(spawner: Spawner) {
    // println!("Init!");
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    esp_hal_embassy::init(&clocks, timg0);

    let mut delay = Delay::new(&clocks);

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
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .init(&mut delay)
        .unwrap();
    display.clear(Rgb565::WHITE).unwrap();

    let uart = Uart::new_async(peripherals.UART0, &clocks);
    let (tx, rx) = uart.split();
    let channel = BROKER_CHANNEL.init(Channel::new());

    spawner
        .spawn(read_serial(rx, channel.sender()))
        .expect("Failed to spawn read serial");

    spawner
        .spawn(broker(channel.receiver(), tx))
        .expect("Failed to spawn read serial");

    // let mut buf = [0; 32];
    let mut s_buf = String::<32>::new();
    loop {
        // let _ = rx.read_exact(&mut buf[0..1]).await;
        // println!("Reading serial");
        // tx.write_bytes(&[3u8]).unwrap();
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

        delay.delay_millis(500);
        // wfi();
    }
}
