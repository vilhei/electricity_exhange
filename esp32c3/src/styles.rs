use u8g2_fonts::{
    fonts::{u8g2_font_helvR14_tf, u8g2_font_helvR18_tf},
    FontRenderer,
};

pub const FONT1_NORMAL: u8g2_fonts::FontRenderer = FontRenderer::new::<u8g2_font_helvR18_tf>();
pub const FONT1_SMALL: u8g2_fonts::FontRenderer = FontRenderer::new::<u8g2_font_helvR14_tf>();
pub const FONT1_BOLD: u8g2_fonts::FontRenderer =
    FontRenderer::new::<u8g2_fonts::fonts::u8g2_font_helvB18_tf>();
