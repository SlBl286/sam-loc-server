use std::sync::Arc;

use tokio::{net::TcpListener, spawn};
use tokio_tungstenite::tungstenite::Error;

use crate::{
    app_state, network::ws_handler::accept_connection, player::session_manager::SessionManager, room::room_manager::{IdGenerator, RoomManager}
};

pub async fn start_ws_server(host: String) -> Result<(), Error> {
    let listener = TcpListener::bind(host.clone()).await.unwrap();

    println!("WebSocket server listen at ws://{}", host);

    let app_state = Arc::new(app_state::AppState {
        session_manager: Arc::new(SessionManager::new()),
        room_manager: Arc::new(RoomManager::new()),
        id_generator: Arc::new(IdGenerator::new()),
    });

    while let Ok((stream, addr)) = listener.accept().await {
        // let lobby_manager: Arc<RoomManager> = lobby_manager.clone();
        println!("New WebSocket Connetion: {}", addr);
        spawn(accept_connection(stream, addr, app_state.clone()));
    }

    Ok(())
}
