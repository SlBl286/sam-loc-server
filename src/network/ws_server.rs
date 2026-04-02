
use tokio::{net::TcpListener, spawn};
use tokio_tungstenite::tungstenite::Error;

use crate::{network::ws_handler::accept_connection, room::{self, room_manager::RoomManager}};


pub async fn start_ws_server(host: String) ->  Result<(), Error>  {
    let listener = TcpListener::bind(host.clone()).await.unwrap();

    println!("WebSocket server listen at ws://{}", host);

    let room_manager = RoomManager::new();

    while let Ok((stream, addr)) = listener.accept().await {
        // let lobby_manager: Arc<RoomManager> = lobby_manager.clone();
        println!("New WebSocket Connetion: {}", addr);
        room_manager.broadcast(None).await;
        spawn(accept_connection(stream, addr));
    }

    Ok(())
}
