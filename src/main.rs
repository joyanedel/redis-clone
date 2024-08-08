use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() {
    let port = 6379;
    let server_connection = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(v) => v,
        Err(e) => panic!("Couldn't connect to 127.0.0.1:{port}. {}", e),
    };

    for stream in server_connection.incoming() {
        if let Err(_) = stream {
            eprintln!("Connection couldn't be resolved");
            continue;
        }

        let mut stream = stream.unwrap();
        let mut buf = [0; 512];
        stream.read(&mut buf).unwrap();
        stream.write(b"+PONG\r\n").unwrap();
    }
}
