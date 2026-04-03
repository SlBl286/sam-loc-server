use tokio::{net::TcpListener, spawn};
use tokio_tungstenite::tungstenite::Error;

use crate::{
    network::ws_handler::accept_connection,
    player::{Session, session_manager::SessionManager},
    room::{self, room_manager::RoomManager},
};

pub async fn start_ws_server(host: String) -> Result<(), Error> {
    let listener = TcpListener::bind(host.clone()).await.unwrap();

    println!("WebSocket server listen at ws://{}", host);

    let session_manager = SessionManager::new();

    while let Ok((stream, addr)) = listener.accept().await {
        // let lobby_manager: Arc<RoomManager> = lobby_manager.clone();
        println!("New WebSocket Connetion: {}", addr);
        spawn(accept_connection(stream, addr));
    }

    Ok(())
}
