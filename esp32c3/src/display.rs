use display_interface_spi::SPIInterface;
use embassy_executor::SendSpawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver};
use embassy_time::Delay;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::{Rgb565, RgbColor},
    text::{Alignment, Text},
    Drawable,
};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_hal::{
    gpio::{Gpio10, Gpio7, Gpio8, Output},
    peripherals::SPI2,
    spi::{master::Spi, FullDuplexMode},
};
use mipidsi::{
    models::ST7789,
    options::{ColorInversion, Orientation, Rotation},
    Display,
};
use shared::DisplayMessage;

use crate::styles::TEXT_STYLE;

type ST7789Display = Display<DisplaySpiInterface, ST7789, Output<'static, Gpio8>>;

type DisplaySpiInterface = SPIInterface<
    ExclusiveDevice<Spi<'static, SPI2, FullDuplexMode>, Output<'static, Gpio10>, NoDelay>,
    Output<'static, Gpio7>,
>;

pub fn setup(
    spawner: &SendSpawner,
    di: DisplaySpiInterface,
    rst: Output<'static, Gpio8>,
    receiver: Receiver<'static, CriticalSectionRawMutex, DisplayMessage, 10>,
) {
    let mut display = mipidsi::Builder::new(ST7789, di)
        .reset_pin(rst)
        .display_size(240, 320)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .invert_colors(ColorInversion::Inverted)
        .init(&mut Delay)
        .unwrap();

    display.clear(Rgb565::WHITE).unwrap();

    Text::with_alignment(
        "Display init done",
        Point { x: 100, y: 100 },
        TEXT_STYLE,
        Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();

    spawner.spawn(update_display(display, receiver)).unwrap();
}

#[embassy_executor::task]
async fn update_display(
    mut display: ST7789Display,
    receiver: Receiver<'static, CriticalSectionRawMutex, DisplayMessage, 10>,
) {
    loop {
        let msg = receiver.receive().await;

        match msg {
            DisplayMessage::On => todo!(),
            DisplayMessage::Off => todo!(),
            DisplayMessage::StatusUpdate(s) => {
                display.clear(Rgb565::WHITE).unwrap();
                Text::with_alignment(&s, Point { x: 100, y: 100 }, TEXT_STYLE, Alignment::Center)
                    .draw(&mut display)
                    .unwrap();
            }

            DisplayMessage::Fill(color) => {
                display.clear(color).unwrap();
            }
        }
    }
}
