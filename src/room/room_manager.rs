
use dashmap::DashMap;
use tokio_tungstenite::tungstenite::Message;

use crate::{player::session_manager::SessionManager, room::Room};

pub struct RoomManager {
    rooms: DashMap<u32, Room>,
}

impl RoomManager {
    pub fn new() -> Self {
        RoomManager {
            rooms: DashMap::new(),
        }
    }

    async fn get_or_create_room(&self, room_id: u32, max_players: u8) {
        if !self.rooms.contains_key(&room_id) {
            self.rooms.insert(room_id, Room::new(room_id, max_players));
        }
    }
    async fn add_player_to_room(&self, room_id: u32, player_id: i64) {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            room.players.push(player_id);
        }
    }

    async fn remove_player_from_room(&self, room_id: u32, player_id: i64) {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            room.players
                .retain(|s| s as *const _ != player_id as *const _);
        }
    }
    pub fn broadcast_room(
        session_manager: &SessionManager,
        players: &[u64],
        msg: &impl serde::Serialize,
    ) {
        let text = serde_json::to_string(msg).unwrap();
        session_manager.broadcast(players, Message::Text(text));
    }
}
