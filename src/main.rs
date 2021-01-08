use std::net::TcpListener;

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:6379") {
        Ok(l) => l,
        Err(e) => panic!("Unable to start listener: {:?}", e),
    };

    match listener.accept() {
        Ok((_socket, addr)) => println!("accepted new client: {:?}", addr),
        Err(e) => println!("couldn't accept client: {:?}", e),
    }
}
