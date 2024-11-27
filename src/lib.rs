use std::{error::Error, sync::Arc};

use serde::{Deserialize, Serialize};

pub type ChatError = Box<dyn Error + Send + Sync + 'static>;
pub type ChatResult<T> = Result<T, ChatError>;

pub mod utils;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum FromClient {
    Join {
        group_name: Arc<String>,
    },
    Post {
        group_name: Arc<String>,
        message: Arc<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum FromServer {
    Message {
        group_name: Arc<String>,
        message: Arc<String>,
    },
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_client_json() {
        let from_client = FromClient::Post {
            group_name: Arc::new("Dogs".to_string()),
            message: Arc::new("soemmessage".to_string()),
        };
        let json = serde_json::to_string(&from_client).unwrap();
        //println!("{}", json);
        assert_eq!(
            json,
            r#"{"Post":{"group_name":"Dogs","message":"soemmessage"}}"#
        );
        assert_eq!(
            serde_json::from_str::<FromClient>(&json).unwrap(),
            from_client
        );
    }
}
