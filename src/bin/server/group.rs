use std::sync::Arc;

use async_chat::FromServer;
use async_std::task;
use tokio::sync::broadcast::{self, error::RecvError};

use crate::connection::Outbound;

pub struct Group {
    group_name: Arc<String>,
    sender: broadcast::Sender<Arc<String>>,
}

impl Group {
    pub fn new(group_name: Arc<String>) -> Group {
        let (sender, _reciever) = broadcast::channel(1000);
        Group { group_name, sender }
    }

    pub fn join(&self, outbound: Arc<Outbound>) {
        let reciever = self.sender.subscribe();
        task::spawn(handle_subscriber(
            self.group_name.clone(),
            reciever,
            outbound,
        ));
    }
    pub fn post(&self, message: Arc<String>) {
        let _ignored = self.sender.send(message);
    }
}

async fn handle_subscriber(
    group_name: Arc<String>,
    mut reciever: broadcast::Receiver<Arc<String>>,
    outbound: Arc<Outbound>,
) {
    loop {
        let packet = match reciever.recv().await {
            Ok(message) => FromServer::Message {
                group_name: group_name.clone(),
                message,
            },
            Err(RecvError::Lagged(n)) => {
                FromServer::Error(format!("Dropped {} messages from {}", n, group_name))
            }
            Err(RecvError::Closed) => break,
        };
        if outbound.send(packet).await.is_err() {
            break;
        }
    }
}
