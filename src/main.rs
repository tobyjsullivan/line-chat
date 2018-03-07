#[macro_use]
extern crate futures;
extern crate tokio;
extern crate nix;

use std::{env, io};
use std::net::SocketAddr;
use std::os::unix::io::AsRawFd;
use futures::future::{lazy, empty};

use tokio::prelude::*;
use tokio::executor::current_thread;
use tokio::net::UdpSocket;

use nix::sys::socket;
use nix::sys::socket::sockopt::Broadcast;

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
            // let bc_addr = "255.255.255.255:1234".parse().unwrap();
            // let amt = try_ready!(self.socket.poll_send_to(&self.buf[..size], &bc_addr));
            let payload = String::from_utf8(self.buf[..size].to_vec());
            println!("Echoed {}/{} bytes to {} ({:?})", amt, size, peer, payload);
        }
    }
}

fn main() {
    let addr = "0.0.0.0:5555".parse().unwrap();
    let socket1 = UdpSocket::bind(&addr).unwrap();
    println!("Listening on: {}", socket1.local_addr().unwrap());

    socket1.set_broadcast(true).expect("error setting broadcast");
    // socket::setsockopt(socket1.as_raw_fd(), Broadcast, &true).expect("setsockopt failed");
 
    let mut server = Server {
        socket: socket1,
        buf: vec![0; 1024],
        to_send: None,
    };

    current_thread::block_on_all(lazy(|| {
        let msg = String::from("init");
        let bc_addr = "255.255.255.255:1234".parse().unwrap();
        server.socket.poll_send_to(&msg.into_bytes().as_slice(), &bc_addr);
        
        println!("Sent init!");

        if false {
            return Err(Async::Ready(()))
        }

        Ok(Async::Ready(()))
    })).expect("Error on init");

    tokio::run(server.map_err(|e| println!("server error = {:?}", e)));
    
}
