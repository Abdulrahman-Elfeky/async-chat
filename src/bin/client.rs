use std::sync::Arc;

use async_chat::utils::{recieve_as_json, send_as_json};
use async_chat::{ChatResult, FromClient, FromServer};
use async_std::net::{self, TcpStream};
use async_std::task;
use async_std::{io::BufReader, prelude::*, println};
#[allow(dead_code)]

async fn send_commands(mut to_server: net::TcpStream) -> ChatResult<()> {
    println!(
        "Commands:\n\
                   join GROUP\n\
                   post GROUP MESSAGE\n\
                   Type Control-D (on Unix) or Control-Z (on Windows)
\
to close the connection."
    )
    .await;
    let mut commands = BufReader::new(async_std::io::stdin()).lines();
    while let Some(command_result) = commands.next().await {
        let command = command_result?;
        let request = match parse_command(&command) {
            Some(request) => request,
            None => continue,
        };
        send_as_json(&mut to_server, &request).await?;
        to_server.flush().await?;
    }
    Ok(())
}

async fn handle_replies(from_server: net::TcpStream) -> ChatResult<()> {
    let buffered = BufReader::new(from_server);
    let mut reply_stream = recieve_as_json(buffered).await;
    while let Some(replay) = reply_stream.next().await {
        match replay? {
            FromServer::Message {
                group_name,
                message,
            } => {
                println!("message posted to {}: {}", group_name, message).await;
            }
            FromServer::Error(message) => {
                println!("error from server: {}", message).await;
            }
        }
    }
    Ok(())
}

fn main() -> ChatResult<()> {
    let address = std::env::args().nth(1).expect("Usage: client ADDRESS:PORT");
    task::block_on(async {
        let socket = TcpStream::connect(address).await?;
        socket.set_nodelay(true)?;

        let to_server = send_commands(socket.clone());
        let from_server = handle_replies(socket);
        from_server.race(to_server).await?;
        Ok(())
    })
}

fn parse_command(line: &str) -> Option<FromClient> {
    let (command, rest) = get_next_token(line)?;

    if command == "post" {
        let (group, rest) = get_next_token(rest)?;
        let message = rest.trim_start().to_string();

        return Some(FromClient::Post {
            group_name: Arc::new(group.to_string()),
            message: Arc::new(message),
        });
    } else if command == "join" {
        let (group, rest) = get_next_token(rest)?;
        if !rest.trim_start().is_empty() {
            return None;
        }
        return Some(FromClient::Join {
            group_name: Arc::new(group.to_string()),
        });
    } else {
        eprintln!("Unrecognized command {:?}", line);
        return None;
    }
}

fn get_next_token(mut input: &str) -> Option<(&str, &str)> {
    input = input.trim_start();
    if input.is_empty() {
        return None;
    }
    match input.find(char::is_whitespace) {
        Some(space) => Some((&input[..space], &input[space..])),
        None => Some((input, "")),
    }
}
