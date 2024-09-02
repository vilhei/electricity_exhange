use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::{Receiver, Sender},
    mutex::Mutex,
};
use embassy_time::Duration;
use heapless::String;
use shared::{DisplayUpdate, Message, Response};

use crate::storage::{NonVolatileKey, NonVolatileStorage};

#[embassy_executor::task]
pub async fn broker(
    broker_receiver: Receiver<'static, CriticalSectionRawMutex, Message, 10>,
    serial_writer_sender: Sender<'static, CriticalSectionRawMutex, Response, 10>,
    display_sender: Sender<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
    nvs_storage: &'static Mutex<NoopRawMutex, NonVolatileStorage>,
) {
    loop {
        let message = broker_receiver.receive().await;

        match message {
            Message::Wifi(_) => {
                serial_writer_sender.send(Response::Ok).await;
            }
            Message::FingridApiKey(key) => {
                let mut nvs_guard = nvs_storage.lock().await;
                nvs_guard
                    .store(NonVolatileKey::FingridApiKey, key)
                    .await
                    .unwrap();
            }
            Message::EntsoeApiKey(key) => {
                let mut nvs_guard = nvs_storage.lock().await;
                nvs_guard
                    .store(NonVolatileKey::EntsoeApiKey, key)
                    .await
                    .unwrap();
            }
            Message::Display(s) => {
                display_sender.send(s.into()).await;
            }
        }
    }
}

pub async fn fetch_spot_price(
    spawner: &Spawner,
    client: &'static Mutex<CriticalSectionRawMutex, crate::client::Client<'_>>,
    nvs_storage: &Mutex<NoopRawMutex, NonVolatileStorage>,
) {
    let entsoe_token;
    {
        let mut nvs_guard = nvs_storage.lock().await;
        entsoe_token = nvs_guard.fetch(NonVolatileKey::EntsoeApiKey).await;
    }
    match entsoe_token {
        Ok(Some(token)) => {
            spawner.must_spawn(update_spot_price(client, token.0));
        }
        Ok(None) => todo!(),
        Err(_) => todo!(),
    }
}

#[allow(unused)]
#[embassy_executor::task]
pub async fn update_spot_price(
    client: &'static Mutex<CriticalSectionRawMutex, crate::client::Client<'_>>,
    entsoe_token: String<64>,
) {
    let mut ticker = embassy_time::Ticker::every(Duration::from_secs(900));
    loop {
        ticker.next().await;
    }
}

#[embassy_executor::task]
pub async fn perform_http_request() {}

#[embassy_executor::task]
pub async fn get_price_from_entsoe() {}
