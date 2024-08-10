use std::io;

use redis_clone::{commands::RedisCommand, resp::RESPValues};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let port = 6379;
    let server = TcpListener::bind(("127.0.0.1", port)).await?;

    loop {
        match server.accept().await {
            Err(_) => eprintln!("Error at accepting connection"),
            Ok((stream, _)) => {
                tokio::spawn(accept_connection(stream));
            }
        }
    }
}

async fn accept_connection(conn: TcpStream) -> io::Result<()> {
    loop {
        let mut buf = [0; 512];
        match conn.try_read(&mut buf) {
            Ok(0) => break,
            Ok(_) => {
                let command = String::from_utf8_lossy(&buf).to_string();
                parse_command(command);
                conn.try_write("+PONG\r\n".as_bytes())
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e),
        }
        .unwrap();
    }

    Ok(())
}

fn parse_command(command: String) {
    let client_input = RESPValues::try_from(command.as_str()).expect("couldn't parse client input");
    let command =
        RedisCommand::try_from(client_input.clone()).expect("couldn't parse client command");
    println!("Input: {client_input:?}");
    println!("Command: {command:?}");
}
