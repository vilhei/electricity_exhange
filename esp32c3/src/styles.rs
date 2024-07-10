use embedded_graphics::{
    mono_font::{iso_8859_3::FONT_10X20, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::{Rgb565, RgbColor},
};

pub const TEXT_STYLE: MonoTextStyle<'static, Rgb565> = MonoTextStyleBuilder::new()
    .font(&FONT_10X20)
    .text_color(Rgb565::RED)
    .build();

// pub const TEXT_STYLE_BOLD: MonoTextStyle<'static, Rgb565> = MonoTextStyleBuilder::new()
//     .font(&FONT_8X13)
//     .text_color(BinaryColor::On)
//     .build();

// pub const OUTER_RECT_STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
//     .stroke_color(BinaryColor::On)
//     .stroke_width(1)
//     .fill_color(BinaryColor::Off)
//     .build();

// pub const FILL_RECT_STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
//     .fill_color(BinaryColor::On)
//     .build();
