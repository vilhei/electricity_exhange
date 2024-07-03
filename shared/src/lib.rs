#![cfg_attr(not(test), no_std)]

use core::{mem::size_of, panic};

use corncobs::max_encoded_len;
use heapless::String;
use serde::{Deserialize, Serialize};

// TODO replace with:  max_encoded_len(size_of::<STRUCTURE_NAME_HERE>() + size_of::<u32>());
pub const IN_SIZE: usize = max_encoded_len(size_of::<Message>() + size_of::<u32>());

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
        &self.ssid
    }
}

pub const CKSUM: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

/// Panics on all errors? Should return result?
pub fn serialize_crc_cobs<T: Serialize, const N: usize>(item: T, out_buf: &mut [u8]) -> &mut [u8] {
    let mut buf = [0u8; N];
    // let bytes_serialized = ssmarshal::serialize(&mut buf, &item).unwrap();

    let serialized = postcard::to_slice_crc32(&item, &mut buf, CKSUM.digest()).unwrap();
    let bytes_used = corncobs::encode_buf(serialized, out_buf);
    // let crc = CKSUM.checksum(&buf[0..bytes_serialized]);
    // let crc_bytes_serialized = ssmarshal::serialize(&mut buf[bytes_serialized..], &crc).unwrap();

    // let n = corncobs::encode_buf(&buf[0..bytes_serialized + crc_bytes_serialized], out_buf);

    &mut out_buf[0..bytes_used]
}

pub fn deserialize_crc_cobs<T: for<'a> Deserialize<'a>>(in_buf: &[u8]) -> T {
    let mut decoded_buf = [0u8; IN_SIZE];

    let bytes_used = corncobs::decode_buf(in_buf, &mut decoded_buf).unwrap();
    let deserialized_item =
        postcard::from_bytes_crc32(&decoded_buf[0..bytes_used], CKSUM.digest()).unwrap();
    // Deserializes into type T. Remaining bytes from bytes_used to end of the buffer should be the crc
    // let (deserialized_item, bytes_used_item) =
    //     ssmarshal::deserialize::<T>(&decoded_buf[0..bytes_used]).unwrap();

    // let (crc, bytes_used_crc) =
    //     ssmarshal::deserialize::<u32>(&decoded_buf[bytes_used_item..]).unwrap();

    // let pkg_crc = CKSUM.checksum(&in_buf[0..bytes_used_item]);

    // if crc != pkg_crc {
    //     panic!("CRC does not match");
    // }

    deserialized_item
}

#[cfg(test)]
mod tests {}
