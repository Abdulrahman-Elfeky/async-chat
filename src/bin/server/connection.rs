use std::sync::Arc;

use async_chat::{
    utils::{recieve_as_json, send_as_json},
    ChatResult, FromClient, FromServer,
};
use async_std::{
    io::{BufRead, BufReader, WriteExt},
    net::TcpStream,
    stream::StreamExt,
    sync::Mutex,
};

use crate::group_table::GroupTable;

pub async fn serve(connection: TcpStream, groups: Arc<GroupTable>) -> ChatResult<()> {
    let outbound = Arc::new(Outbound::new(connection.clone()));
    let buffered = BufReader::new(connection);
    let mut from_client = recieve_as_json::<_, FromClient>(buffered).await;
    while let Some(packet) = from_client.next().await {
        let packet = packet?;
        let result = match packet {
            FromClient::Post {
                group_name,
                message,
            } => match groups.get(group_name.clone()) {
                Some(group) => {
                    group.post(message);
                    Ok(())
                }
                None => Err(format!("Group '{}' deosn't exist", group_name)),
            },
            FromClient::Join { group_name } => {
                let group = groups.get_or_create(group_name);
                group.join(outbound.clone());
                Ok(())
            }
        };
        if let Err(message) = result {
            outbound.send(FromServer::Error(message)).await?;
        }
    }

    Ok(())
}

pub struct Outbound(Mutex<TcpStream>);

impl Outbound {
    pub fn new(to_client: TcpStream) -> Outbound {
        Outbound(Mutex::new(to_client))
    }
    pub async fn send(&self, packet: FromServer) -> ChatResult<()> {
        let mut guard = self.0.lock().await;
        send_as_json(&mut *guard, &packet).await?;
        guard.flush().await?;
        Ok(())
    }
}
