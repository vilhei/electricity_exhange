use std::{str::FromStr, thread::sleep, time::Duration};

use shared::{serialize_crc_cobs, Message, WifiInfo, IN_SIZE};

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

    let mut buf: Vec<u8> = vec![0; IN_SIZE];
    let mut i = 0;

    // let ssid = String::from_str("MyWifi");

    let test_object = shared::Message::Wifi(WifiInfo::new(
        heapless::String::<64>::from_str("Mywifi").unwrap(),
        heapless::String::<64>::from_str("1234").unwrap(),
    ));
    let mut out_buf = [0u8; IN_SIZE];
    let serialized_obj = serialize_crc_cobs::<Message, IN_SIZE>(test_object, &mut out_buf);
    let mut s_buf = String::new();
    loop {
        if let Ok(m) = port.read(&mut buf) {
            println!(
                "Got message from esp : \n {}",
                String::from_utf8(buf.to_vec()).unwrap()
            );
        }

        // println!("{buf:?}");
        let _ = port.write(serialized_obj);
        println!("{i}");
        sleep(Duration::from_millis(200));
        i = (i + 1) % 255;
    }
}
