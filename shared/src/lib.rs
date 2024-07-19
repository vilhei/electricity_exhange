#![cfg_attr(not(test), no_std)]
#![feature(type_alias_impl_trait)]

use core::mem::size_of;

use corncobs::max_encoded_len;
use embedded_graphics::pixelcolor::Rgb565;
use heapless::String;
use mipidsi::dcs::DcsCommand;
use serde::{Deserialize, Serialize};

pub const MESSAGE_SIZE: usize = max_encoded_len(size_of::<Message>() + size_of::<u32>());
pub const RESPONSE_SIZE: usize = max_encoded_len(size_of::<Response>() + size_of::<u32>());

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub enum Message {
    Wifi(WifiInfo),
    FingridApiKey(String<64>),
    EntsoeApiKey(String<64>),
    Display(DisplayMessage),
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct WifiInfo {
    ssid: String<64>,
    password: String<64>,
}

impl WifiInfo {
    pub fn new(ssid: String<64>, password: String<64>) -> Self {
        Self { ssid, password }
    }

    pub fn get_ssid(&self) -> &str {
        &self.ssid
    }
    pub fn get_password(&self) -> &str {
        &self.password
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub enum Response {
    Ok,
    Error,
}

pub const CKSUM: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

/// Panics on all errors? Should return result?
pub fn serialize_crc_cobs<T: Serialize, const N: usize>(item: T, out_buf: &mut [u8]) -> &mut [u8] {
    let mut buf = [0u8; N];

    let buf = postcard::to_slice_crc32(&item, &mut buf, CKSUM.digest()).unwrap();
    let bytes_used = corncobs::encode_buf(buf, out_buf);

    &mut out_buf[0..bytes_used]
}

pub fn deserialize_crc_cobs<T: for<'a> Deserialize<'a>>(in_buf: &[u8]) -> T {
    let mut decoded_buf = [0u8; MESSAGE_SIZE];

    let bytes_used = corncobs::decode_buf(in_buf, &mut decoded_buf).unwrap();
    let decoded_buf =
        postcard::from_bytes_crc32(&decoded_buf[0..bytes_used], CKSUM.digest()).unwrap();

    decoded_buf
}

#[derive(Debug)]
pub enum DisplayUpdate {
    On,
    Off,
    StatusUpdate(String<64>),
    Fill(Rgb565),
    SetBrightness(DisplayBrightness),
}

impl From<DisplayMessage> for DisplayUpdate {
    fn from(value: DisplayMessage) -> Self {
        match value {
            DisplayMessage::On => DisplayUpdate::On,
            DisplayMessage::Off => DisplayUpdate::Off,
            DisplayMessage::StatusUpdate(s) => DisplayUpdate::StatusUpdate(s),
        }
    }
}

#[derive(Debug)]
pub enum DisplayBrightness {
    Low,
    Normal,
    High,
}

impl DcsCommand for DisplayBrightness {
    fn instruction(&self) -> u8 {
        0x51
    }

    fn fill_params_buf(&self, buffer: &mut [u8]) -> Result<usize, mipidsi::error::Error> {
        match self {
            DisplayBrightness::Low => buffer[0] = 75,
            DisplayBrightness::Normal => buffer[0] = 150,
            DisplayBrightness::High => buffer[0] = 255,
        }
        Ok(1)
    }
}

// trait DisplayBrightness {}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub enum DisplayMessage {
    On,
    Off,
    StatusUpdate(String<64>),
}

#[cfg(test)]
mod tests {}
