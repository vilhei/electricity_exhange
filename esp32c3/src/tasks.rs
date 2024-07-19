use core::str::FromStr;

use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::{Receiver, Sender},
};
use heapless::String;
use shared::{DisplayUpdate, Message, Response};

#[embassy_executor::task]
pub async fn broker(
    broker_receiver: Receiver<'static, NoopRawMutex, Message, 10>,
    serial_writer_sender: Sender<'static, NoopRawMutex, Response, 10>,
    display_sender: Sender<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
) {
    // display_sender
    //     .send(DisplayUpdate::StatusUpdate(
    //         String::from_str("Starting broker task").unwrap(),
    //     ))
    //     .await;

    loop {
        let message = broker_receiver.receive().await;

        match message {
            Message::Wifi(_) => serial_writer_sender.send(Response::Ok).await,
            Message::FingridApiKey(_) => todo!(),
            Message::EntsoeApiKey(_) => todo!(),
            Message::Display(_) => todo!(),
        }
    }
}
