use std::{str::FromStr, thread::sleep, time::Duration};

use shared::{deserialize_crc_cobs, serialize_crc_cobs, Message, Response, WifiInfo, MESSAGE_SIZE};

fn main() {
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
    let mut out_buf = [0u8; MESSAGE_SIZE];
    let serialized_obj = serialize_crc_cobs::<Message, MESSAGE_SIZE>(test_object, &mut out_buf);
    // let mut s_buf = String::new();
    loop {
        let _ = port.write(serialized_obj);
        let mut read_buf = [0u8; 1];
        let mut response_buf = Vec::new();
        loop {
            if port.read_exact(&mut read_buf).is_ok() {
                response_buf.push(read_buf[0]);
                if read_buf[0] == corncobs::ZERO {
                    break;
                }
            }
        }
        let response = deserialize_crc_cobs::<Response>(&response_buf);
        println!("{response:?}");

        sleep(Duration::from_millis(200));
        i = (i + 1) % 255;
    }
}
