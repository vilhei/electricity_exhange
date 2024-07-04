#![cfg_attr(not(test), no_std)]

use core::{mem::size_of, panic};

use corncobs::max_encoded_len;
use heapless::String;
use serde::{Deserialize, Serialize};

// TODO replace with:  max_encoded_len(size_of::<STRUCTURE_NAME_HERE>() + size_of::<u32>());
pub const IN_SIZE: usize = max_encoded_len(size_of::<Message>() + size_of::<u32>());
pub const OUT_SIZE: usize = max_encoded_len(size_of::<Response>() + size_of::<u32>());

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub enum Message {
    Wifi(WifiInfo),
    FingridApiKey(String<64>),
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
    let mut decoded_buf = [0u8; IN_SIZE];

    let bytes_used = corncobs::decode_buf(in_buf, &mut decoded_buf).unwrap();
    let decoded_buf =
        postcard::from_bytes_crc32(&decoded_buf[0..bytes_used], CKSUM.digest()).unwrap();

    decoded_buf
}

#[cfg(test)]
mod tests {}
