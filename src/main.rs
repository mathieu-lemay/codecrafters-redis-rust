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

        let cmd = match parse_command(&buffer[..n]) {
            Ok(c) => c,
            Err(e) => {
                socket
                    .write_all(format!("-{}\r\n", e).as_bytes())
                    .expect("Unable to write to socket");
                break;
            }
        };

        println!("command: {:?}", cmd);

        let resp = match cmd {
            Command::Ping => String::from("+PONG\r\n"),
            Command::Echo(v) => format!("${}\r\n{}\r\n", v.len(), v),
        };

        println!("returning resp: {:?}", resp);

        socket
            .write_all(&resp.as_bytes())
            .expect("Unable to write to socket");
    }
}

#[derive(Debug)]
enum Command {
    Ping,
    Echo(String),
}

fn parse_command(cmd: &[u8]) -> Result<Command, String> {
    let cmd = match str::from_utf8(cmd) {
        Ok(c) => c,
        Err(e) => panic!("Error parsing cmd: {:?}", e),
    };

    println!("raw command: {:?}", cmd);

    let splits = cmd
        .split("\r\n")
        .skip(2)
        .step_by(2)
        .map(|s| String::from(s.trim_end()))
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>();
    let cmd = &splits[0].to_lowercase();

    let args = if splits.len() > 1 {
        splits[1..].to_vec()
    } else {
        Vec::new()
    };

    match cmd.as_str() {
        "ping" => Ok(Command::Ping),
        "echo" => {
            if args.len() == 1 {
                Ok(Command::Echo(args[0].clone()))
            } else {
                Err(format!(
                    "ERR wrong number of arguments for '{}' command",
                    cmd
                ))
            }
        }
        _ => Err(format!("Invalid command: {}", cmd)),
    }
}
