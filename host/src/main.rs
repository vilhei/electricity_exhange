#![feature(trait_upcasting)]

mod model;
mod tracing;
mod ui_event;
mod ui_render;
mod update;

use std::{
    error::Error,
    io::{Read, Write},
    str::FromStr,
    thread::sleep,
    time::Duration,
};

use host::{init_terminal, install_panic_hook, restore_terminal};
// use log::{info, warn, LevelFilter};
use ::tracing::info;
use model::{Model, RunningState};
use shared::{
    deserialize_crc_cobs, serialize_crc_cobs, Message, Response, WifiInfo, MESSAGE_SIZE,
    RESPONSE_SIZE,
};
use tracing::initialize_logging;
use ui_event::handle_event;
// use update::UiMessage;

fn main() -> Result<(), Box<dyn Error>> {
    initialize_logging()?;
    install_panic_hook().unwrap();

    info!("program start");

    let mut terminal = init_terminal()?;
    let mut model = Model::new();
    let quit_state = std::mem::discriminant(&RunningState::ForceQuit);

    while quit_state != std::mem::discriminant(&model.running_state) {
        terminal.draw(|frame| ui_render::view(&mut model, frame))?;

        let mut current_msg = handle_event(&model)?;

        while let Some(msg) = current_msg {
            current_msg = update::update(&mut model, &msg);
        }
    }

    restore_terminal()?;
    Ok(())
}

#[allow(unused)]
fn main2() {
    let ports = serialport::available_ports().expect("No ports found");
    // for p in ports {
    //     println!("{}", p.port_name);
    // }
    let port_name = &ports[0].port_name;
    let mut port = serialport::new(port_name, 115200)
        .timeout(Duration::from_millis(50))
        .open()
        .unwrap_or_else(|_| panic!("Failed to connect to {}", port_name));
    port.set_data_bits(serialport::DataBits::Eight).unwrap();
    port.set_stop_bits(serialport::StopBits::One).unwrap();
    port.set_parity(serialport::Parity::None).unwrap();

    let mut buf: Vec<u8> = vec![0; MESSAGE_SIZE];
    let mut i = 0;

    // let ssid = String::from_str("MyWifi");

    let test_object = shared::Message::Wifi(WifiInfo::new(
        heapless::String::<64>::from_str("Mywifi").unwrap(),
        heapless::String::<64>::from_str("1234").unwrap(),
    ));
    // let mut reader = BufReader::new(port.try_clone().unwrap());
    // let mut writer = BufWriter::new(port);

    let mut out_buf = [0u8; MESSAGE_SIZE];
    // let mut response_buf = Vec::with_capacity(RESPONSE_SIZE);
    let serialized_obj = serialize_crc_cobs::<Message, MESSAGE_SIZE>(test_object, &mut out_buf);

    // loop {
    //     let _ = writer.write_all(serialized_obj);
    //     println!("Looping");
    //     let mut read_buf = [0u8; 1];
    //     match reader.read_until(corncobs::ZERO, &mut response_buf) {
    //         Ok(byte_count) => {
    //             let response = deserialize_crc_cobs::<Response>(&response_buf);
    //             println!("{response:?}");
    //             response_buf.clear();
    //         }
    //         Err(e) => println!("Error : {e}")
    //     }

    //     sleep(Duration::from_millis(2000));
    //     i = (i + 1) % 255;
    // }
    let mut out_buf = [0u8; MESSAGE_SIZE];
    // let mut s_buf = String::new();

    let mut reader: Box<dyn Read> = port.try_clone().unwrap();
    let mut writer: Box<dyn Write> = port;

    loop {
        let _ = writer.write(serialized_obj);
        let mut read_buf = [0u8; 1];
        let mut response_buf = Vec::new();
        loop {
            if reader.read_exact(&mut read_buf).is_ok() {
                response_buf.push(read_buf[0]);
                if read_buf[0] == corncobs::ZERO {
                    break;
                }
            }
        }
        let response = deserialize_crc_cobs::<Response>(&response_buf);
        println!("{response:?}");

        sleep(Duration::from_millis(2000));
        i = (i + 1) % 255;
    }
}
