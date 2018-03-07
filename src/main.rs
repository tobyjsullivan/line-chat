#[macro_use]
extern crate futures;
extern crate tokio;

use std::{env, io};
use std::net::SocketAddr;

use tokio::prelude::*;
use tokio::executor::current_thread;
use tokio::net::UdpSocket;

struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,
}

impl Future for Server {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            let (size, peer) = try_ready!(self.socket.poll_recv_from(&mut self.buf));

            let amt = try_ready!(self.socket.poll_send_to(&self.buf[..size], &peer));
            let payload = String::from_utf8(self.buf[..size].to_vec());
            println!("Echoed {}/{} bytes to {} ({:?})", amt, size, peer, payload);
            self.to_send = None;
        }
    }
}

fn main() {
    let addr = "0.0.0.0:1234".parse().unwrap();
    let socket = UdpSocket::bind(&addr).unwrap();
    println!("Listening on: {}", socket.local_addr().unwrap());


    let mut server = Server {
        socket: socket,
        buf: vec![0; 1024],
        to_send: None,
    };

    tokio::run(server.map_err(|e| println!("server error = {:?}", e)));
    
}
