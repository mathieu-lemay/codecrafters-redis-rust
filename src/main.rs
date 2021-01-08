use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str;
use std::thread;

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:6379") {
        Ok(l) => {
            println!("Listening on {:?}", l.local_addr().unwrap());
            l
        }
        Err(e) => panic!("Unable to start listener: {:?}", e),
    };

    loop {
        match listener.accept() {
            Ok((mut socket, addr)) => {
                thread::spawn(move || {
                    process(&mut socket, &addr);
                });
            }
            Err(e) => println!("couldn't accept client: {:?}", e),
        }
    }
}

fn process(socket: &mut TcpStream, addr: &SocketAddr) {
    println!("accepted new client: {:?}", addr);
    loop {
        let mut buffer = [0; 1024];

        let n = socket
            .read(&mut buffer)
            .expect("Unable to read from socket");

        if n == 0 {
            println!("client disconnected: {:?}", addr);
            break;
        }

        println!("Client sent: {:?}", str::from_utf8(&buffer[..n]));

        socket
            .write_all(b"+PONG\r\n")
            .expect("Unable to write to socket");
    }
}
