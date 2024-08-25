use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{self, Sender};
use embedded_io_async::Read;
use esp_hal::peripherals::UART0;
use esp_hal::uart::{UartRx, UartTx};
use esp_hal::Async;
use esp_hal::{clock::Clocks, uart::Uart};
use heapless::Vec;
use shared::{
    deserialize_crc_cobs, serialize_crc_cobs, DisplayUpdate, Message, Response, MESSAGE_SIZE,
    RESPONSE_SIZE,
};

/// **TODO** Try to read multiple bytes at a time instead of one at a time?
///
/// Constructs Uart instance and starts serial read and write tasks
/// When a full [Message] has been received in [read_serial] it is sent to `broker_sender` channel
/// When `writer_receiver` receives [Response] it will encode and write it to serial
///
/// # Errors
///
/// This function will return an error if spawning a task fails.
pub async fn setup(
    spawner: &Spawner,
    uart: UART0,
    clocks: &Clocks<'_>,
    broker_sender: channel::Sender<'static, CriticalSectionRawMutex, Message, 10>,
    writer_receiver: channel::Receiver<'static, CriticalSectionRawMutex, Response, 10>,
    display_sender: Sender<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
) -> Result<(), SerialError> {
    display_sender.send("Serial init".into()).await;

    let uart = Uart::new_async(uart, clocks);
    let (tx, rx) = uart.split();

    spawner.spawn(read_serial(rx, broker_sender, display_sender))?;
    spawner.spawn(write_serial(tx, writer_receiver))?;

    display_sender.send("Serial init done".into()).await;

    Ok(())
}

#[embassy_executor::task]
async fn read_serial(
    mut rx: UartRx<'static, UART0, Async>,
    broker_sender: channel::Sender<'static, CriticalSectionRawMutex, Message, 10>,
    display_sender: Sender<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
) {
    let mut message = Vec::<u8, MESSAGE_SIZE>::new();
    let mut buf = [0; 1];

    loop {
        let _ = rx.read_exact(&mut buf).await;
        if message.is_full() {
            display_sender
                .send("Message buffer is full - panicking".into())
                .await;
            panic!("Message buffer is full")
        }
        message.push(buf[0]).unwrap();
        if buf[0] == corncobs::ZERO {
            let deserialized = deserialize_crc_cobs(&message);
            broker_sender.send(deserialized).await;
            message.clear();
        }
    }
}

#[embassy_executor::task]
async fn write_serial(
    mut tx: UartTx<'static, UART0, Async>,
    writer_receiver: channel::Receiver<'static, CriticalSectionRawMutex, Response, 10>,
) {
    loop {
        let response = writer_receiver.receive().await;
        let mut buf = [0; RESPONSE_SIZE];
        let serialized = serialize_crc_cobs::<Response, RESPONSE_SIZE>(response, &mut buf);
        tx.write_async(serialized).await.unwrap();
    }
}

#[derive(Debug)]
pub enum SerialError {
    SpawnError,
}

impl From<embassy_executor::SpawnError> for SerialError {
    fn from(_: embassy_executor::SpawnError) -> Self {
        Self::SpawnError
    }
}
