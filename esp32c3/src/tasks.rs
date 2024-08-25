use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::{Receiver, Sender},
    mutex::Mutex,
};
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
        display_sender.send("Serial message received".into()).await;

        match message {
            Message::Wifi(_) => {
                serial_writer_sender.send(Response::Ok).await;
                display_sender.send("Wifi info got".into()).await;
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

#[embassy_executor::task]
pub async fn perform_http_request() {}

#[embassy_executor::task]
pub async fn get_price_from_entsoe() {}
