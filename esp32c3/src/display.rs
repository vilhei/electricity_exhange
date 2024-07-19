use display_interface_spi::SPIInterface;
use embassy_executor::SendSpawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver};
use embassy_time::Delay;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Dimensions, Point},
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
    dcs::{SetDisplayOff, SetDisplayOn},
    models::ST7789,
    options::{ColorInversion, Orientation, Rotation},
    Display,
};
use shared::DisplayUpdate;
use u8g2_fonts::{
    fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
};

use crate::styles::FONT1_NORMAL;

type ST7789Display = Display<DisplaySpiInterface, ST7789, Output<'static, Gpio8>>;

type DisplaySpiInterface = SPIInterface<
    ExclusiveDevice<Spi<'static, SPI2, FullDuplexMode>, Output<'static, Gpio10>, NoDelay>,
    Output<'static, Gpio7>,
>;

// pub const TEXT_STYLE2: U8g2TextStyle<Rgb565> =
//     U8g2TextStyle::new(fonts::u8g2_font_helvR18_tf, Rgb565::RED);

pub fn setup(
    spawner: &SendSpawner,
    di: DisplaySpiInterface,
    rst: Output<'static, Gpio8>,
    receiver: Receiver<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
) {
    let mut display = mipidsi::Builder::new(ST7789, di)
        .reset_pin(rst)
        .display_size(240, 320)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .invert_colors(ColorInversion::Inverted)
        .init(&mut Delay)
        .unwrap();

    display.clear(Rgb565::WHITE).unwrap();

    // FONT1.render("Display init done", position, vertical_pos, color, display)

    FONT1_NORMAL
        .render_aligned(
            "Display init done. Spawning display task",
            display.bounding_box().center(),
            VerticalPosition::Center,
            HorizontalAlignment::Center,
            FontColor::Transparent(Rgb565::RED),
            &mut display,
        )
        .unwrap();

    spawner.spawn(update_display(display, receiver)).unwrap();
}

#[embassy_executor::task]
async fn update_display(
    mut display: ST7789Display,
    receiver: Receiver<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
) {
    loop {
        let msg = receiver.receive().await;
        // Safety : Rest  of the code is not aware of raw commands sent.
        // User must be sure that commands sent do not affect state of the device in a way that results in undefined behaviour.
        let dcs = unsafe { display.dcs() };

        match msg {
            DisplayUpdate::On => {
                dcs.write_command(SetDisplayOff).unwrap();
            }
            DisplayUpdate::Off => {
                dcs.write_command(SetDisplayOn).unwrap();
            }
            DisplayUpdate::StatusUpdate(s) => {
                display.clear(Rgb565::WHITE).unwrap();
                FONT1_NORMAL
                    .render_aligned(
                        s.as_str(),
                        display.bounding_box().center(),
                        VerticalPosition::Center,
                        HorizontalAlignment::Center,
                        FontColor::Transparent(Rgb565::RED),
                        &mut display,
                    )
                    .unwrap();
            }
            DisplayUpdate::Fill(color) => {
                display.clear(color).unwrap();
            }
            DisplayUpdate::SetBrightness(b) => {
                dcs.write_command(b).unwrap();
            }
        }
    }
}
