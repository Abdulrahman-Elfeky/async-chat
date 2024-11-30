use std::{env, result, sync::Arc};

use async_chat::ChatResult;
use async_std::{eprint, net::TcpListener, stream::StreamExt, task};
use connection::serve;
use group_table::GroupTable;

mod connection;
mod group;
mod group_table;
fn main() -> ChatResult<()> {
    let address = env::args().nth(1).expect("Usage: server ADDRESS");
    let chat_table = Arc::new(GroupTable::new());
    task::block_on(async {
        let listener = TcpListener::bind(address).await?;
        let mut new_connections = listener.incoming();
        while let Some(socket_result) = new_connections.next().await {
            let socket = socket_result?;
            let groups = chat_table.clone();
            task::spawn(async { log_error(serve(socket, groups).await) });
        }
        Ok(())
    })
}

fn log_error(result: ChatResult<()>) {
    if let Err(e) = result {
        std::eprintln!("{}", e);
    }
}
