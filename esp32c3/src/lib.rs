#![no_std]
#![feature(type_alias_impl_trait)]

use esp_hal::rng::Rng;

pub mod client;
pub mod display;
pub mod serial;
pub mod storage;
pub mod styles;
pub mod tasks;
pub mod wifi;

// use esp_hal::o

// pub fn configure_usb() {
//     let usb = Usb
// }

fn generate_rand_u64(rng: &mut Rng) -> u64 {
    let seed1 = rng.random();
    let seed2 = rng.random();

    let mut bytes = [0u8; 8];
    bytes[0..4].copy_from_slice(&seed1.to_le_bytes());
    bytes[4..].copy_from_slice(&seed2.to_le_bytes());

    u64::from_le_bytes(bytes)
}
