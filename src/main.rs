use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

type MemoryMap = Arc<RwLock<HashMap<String, (String, Option<Instant>)>>>;

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:6379") {
        Ok(l) => {
            println!("Listening on {:?}", l.local_addr().unwrap());
            l
        }
        Err(e) => panic!("Unable to start listener: {:?}", e),
    };

    // https://stackoverflow.com/a/50283931
    let memmap: MemoryMap = Arc::new(RwLock::new(HashMap::new()));

    loop {
        match listener.accept() {
            Ok((mut socket, addr)) => {
                let memmap = Arc::clone(&memmap);
                thread::spawn(move || {
                    process(&mut socket, &addr, memmap);
                });
            }
            Err(e) => println!("couldn't accept client: {:?}", e),
        }
    }
}

fn process(socket: &mut TcpStream, addr: &SocketAddr, memmap: MemoryMap) {
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
            Command::Get(k) => {
                let map = memmap.read().expect("Mutex poisoned");
                match map.get(&k) {
                    Some((v, ex)) => {
                        if ex.is_some() && ex.unwrap() < Instant::now() {
                            String::from("$-1\r\n")
                        } else {
                            format!("${}\r\n{}\r\n", v.len(), v)
                        }
                    }
                    None => String::from("$-1\r\n"),
                }
            }
            Command::Set(k, v, ex) => {
                let mut map = memmap.write().expect("Mutex poisoned");
                let ex = match ex {
                    Some(t) => Some(Instant::now() + Duration::from_millis(t)),
                    None => None,
                };
                map.insert(k, (v, ex));
                String::from("+OK\r\n")
            }
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
    Get(String),
    Set(String, String, Option<u64>),
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
        "get" => {
            if args.len() == 1 {
                Ok(Command::Get(args[0].clone()))
            } else {
                Err(format!(
                    "ERR wrong number of arguments for '{}' command",
                    cmd
                ))
            }
        }
        "set" => {
            if args.len() < 2 {
                return Err(format!(
                    "ERR wrong number of arguments for '{}' command",
                    cmd
                ));
            }

            let mut args = args.iter();
            let k = args.next().unwrap();
            let v = args.next().unwrap();

            let opt = args.next();
            let ex = match opt {
                Some(o) => {
                    if o != "px" {
                        return Err(String::from("ERR syntax error"));
                    }

                    let ex = args.next();

                    if ex.is_none() {
                        return Err(String::from("ERR syntax error"));
                    }

                    match ex.unwrap().parse::<u64>() {
                        Ok(i) => Some(i),
                        Err(_) => {
                            return Err(String::from("ERR syntax error"));
                        }
                    }
                }
                None => None,
            };

            Ok(Command::Set(k.to_string(), v.to_string(), ex))
        }
        _ => Err(format!("Invalid command: {}", cmd)),
    }
}
