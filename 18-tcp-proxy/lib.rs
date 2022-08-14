#![forbid(unsafe_code)]

use std::net::{TcpListener, TcpStream};
use std::{thread, time};

use std::io::prelude::*;

fn send(mut from: TcpStream, mut to: TcpStream) {
    loop {
        let mut buffer: [u8; 10001] = [0; 10001];
        let read_result = from.read(&mut buffer);
        if let Ok(bytes_count) = read_result {
            let send_result = to.write_all(&buffer[..(bytes_count)]);
            if send_result.is_err() {
                break;
            }
        } else {
            break;
        }
    }
}

fn connection_handler(from: TcpStream, to_addr: String) {
    let connection = TcpStream::connect(to_addr);
    if let Ok(to) = connection {
        let send_from = from.try_clone().unwrap();
        let send_to = to.try_clone().unwrap();
        let receive_from = to.try_clone().unwrap();
        let receive_to = from.try_clone().unwrap();
        thread::spawn(move || send(send_from, send_to));
        thread::spawn(move || send(receive_from, receive_to));
    }
}

fn launch_proxy_server(accept_tcp: TcpListener, destination: String) {
    for accept in accept_tcp.incoming().flatten() {
        let to_addr = destination.clone();
        thread::spawn(move || connection_handler(accept, to_addr));
    }
}

pub fn run_proxy(port: u32, destination: String) {
    let server_addr: String = format!("127.0.0.1:{}", port);
    let listen_socket = TcpListener::bind(server_addr).expect("RunProxy TcpListener::Bind");
    thread::spawn(move || launch_proxy_server(listen_socket, destination));
    thread::sleep(time::Duration::from_secs(3));
}
