use std::sync::Arc;

use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::tungstenite::Message;

type Player = mpsc::UnboundedSender<Message>;

struct Lobby {
    players: Vec<Player>,
    lobby_name: String,
}

impl Lobby {
    fn new() -> Self {
        Lobby { players: Vec::new(), lobby_name: String::new() }
    }
}
struct LobbyManager {
    lobbies: Arc<Mutex<std::collections::HashMap<String, Lobby>>>,
}

impl LobbyManager {
    fn new() -> Self {
        LobbyManager {
            lobbies: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    async fn get_or_create_lobby(&self, lobby_id: String) {
        let mut lobbies = self.lobbies.lock().await;
        if !lobbies.contains_key(&lobby_id) {
            lobbies.insert(lobby_id.clone(), Lobby::new());
        }
    }
    
    async  fn remove_player_from_lobby(&self, lobby_id: String, players: &Player) {
        let mut lobbies = self.lobbies.lock().await;

        
    }
}
