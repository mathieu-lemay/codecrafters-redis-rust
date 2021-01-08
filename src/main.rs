use std::io::Write;
use std::net::TcpListener;

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:6379") {
        Ok(l) => {
            println!("Listening on {:?}", l.local_addr().unwrap());
            l
        }
        Err(e) => panic!("Unable to start listener: {:?}", e),
    };

    match listener.accept() {
        Ok((mut socket, addr)) => {
            println!("accepted new client: {:?}", addr);
            socket
                .write_all(b"+PONG\r\n")
                .expect("Unable to write to socket");
        }
        Err(e) => println!("couldn't accept client: {:?}", e),
    }
}
