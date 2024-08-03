use embassy_net::Stack;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use serde::{Deserialize, Serialize};
use static_cell::StaticCell;

use crate::client::{Client, Ready};

static CLIENT: StaticCell<Client<Ready>> = StaticCell::new();

pub fn setup(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) -> Result<(), Error> {
    CLIENT.init(Client::new(stack));
    Ok(())
}

pub fn perform_get_request() {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Error {
    FailedSetup,
}
