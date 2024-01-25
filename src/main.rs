// Copyright 2024 Brandon Matthews <thenewwazoo@optimaltour.us>

use std::io::Write;
use std::{io, thread};

use improv_rs::{ImprovPacket, RPCCommand, WifiSettings};

fn usage() -> ! {
    panic!(
        "usage: {} <port> <ssid> <psk>",
        std::env::args().nth(0).unwrap()
    )
}

fn main() {
    let port_name = std::env::args().nth(1).unwrap_or_else(|| usage());

    let mut port = serialport::new(port_name, 115200)
        .open()
        .expect("Failed to open serial port");

    let mut outp = port.try_clone().expect("Failed to clone");

    let mut input: String = String::new();

    let packets = [
        ImprovPacket::RPCCommand(RPCCommand::RequestCurrentState),
        ImprovPacket::RPCCommand(RPCCommand::RequestDeviceInformation),
        ImprovPacket::RPCCommand(RPCCommand::RequestScannedWifiNetworks),
        ImprovPacket::RPCCommand(RPCCommand::SendWifiSettings(WifiSettings {
            ssid: String::from(std::env::args().nth(2).unwrap_or_else(|| usage())),
            psk: String::from(std::env::args().nth(3).unwrap_or_else(|| usage())),
        })),
    ];
    let mut i = 0;

    thread::spawn(move || loop {
        // block until we hit enter
        let _ = std::io::stdin().read_line(&mut input);

        let p = packets[i].clone();
        i += 1;
        i = i % packets.len();

        println!("sending!");
        outp.write_all(&<ImprovPacket as Into<Vec<u8>>>::into(p))
            .expect("Failed to write to serial port");

        // why is one more byte required? I don't know. any byte will do.
        let _ = outp.write_all(&[0x01]);
    });

    // print whatever comes down the pipe
    let mut buffer: [u8; 1024] = [0; 1024];
    loop {
        match port.read(&mut buffer) {
            Ok(bytes) => {
                io::stdout().write_all(&buffer[0..bytes]).unwrap();
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
