use std::io;

use redis_clone::{
    commands::{RedisCommand, RedisCommandError},
    resp::RESPValues,
};
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
        let command = match conn.try_read(&mut buf) {
            Ok(0) => break,
            Ok(_) => {
                let command = String::from_utf8_lossy(&buf).to_string();
                let parsed_command = parse_command(command);
                parsed_command
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e),
        };

        if let Err(error) = command {
            reply_error_to_client(error, &conn).expect("couldn't reply to client");
        } else {
            reply_command_to_client(command.ok().unwrap(), &conn)
                .expect("couldn't respond to client");
        }

        // responds_to_client(command, &conn).expect("couldn't respond to client");
    }

    Ok(())
}

fn parse_command(command: String) -> Result<RedisCommand, RedisCommandError> {
    let client_input = RESPValues::try_from(command.as_str()).expect("couldn't parse client input");
    RedisCommand::try_from(client_input.clone())
}

fn reply_command_to_client(command: RedisCommand, conn: &TcpStream) -> io::Result<usize> {
    match command {
        RedisCommand::Ping(Some(v)) => conn.try_write(format!("+\"{v}\"\r\n").as_bytes()),
        RedisCommand::Ping(_) => conn.try_write("+PONG\r\n".as_bytes()),
        RedisCommand::Echo(v) => conn.try_write(format!("+\"{v}\"\r\n").as_bytes()),
        _ => unimplemented!(),
    }
    // conn.try_write("+PONG\r\n".as_bytes())
}

fn reply_error_to_client(command_error: RedisCommandError, conn: &TcpStream) -> io::Result<usize> {
    match command_error {
        RedisCommandError::NotImplemented => {
            conn.try_write("+Command not implemented\r\n".as_bytes())
        }
    }
}
