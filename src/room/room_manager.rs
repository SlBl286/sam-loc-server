use std::sync::atomic::AtomicU32;

use dashmap::DashMap;
use tokio_tungstenite::tungstenite::Message;

use crate::{player::session_manager::SessionManager, room::Room};

pub struct RoomManager {
    rooms: DashMap<u32, Room>,
}
pub struct IdGenerator {
    last_id: AtomicU32,
}

impl IdGenerator {
    pub fn new() -> Self {
        IdGenerator {
            last_id: AtomicU32::new(0),
        }
    }

    pub fn generate_id(&self) -> u32 {
        self.last_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

impl RoomManager {
    pub fn new() -> Self {
        RoomManager {
            rooms: DashMap::new(),
        }
    }

    pub async fn get_rooms(&self) -> Vec<Room> {
        self.rooms
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
    pub async fn create_room(&self, room_id: u32, room_name: String, max_players: Option<u8>) {
        self.rooms.insert(
            room_id,
            Room::new(room_id, room_name, max_players.unwrap_or(5)),
        );
    }

    pub async fn get_room(&self, room_id: &u32) -> Option<Room> {
        self.rooms.get(room_id).map(|entry| entry.clone())
    }
    pub async fn add_player_to_room(&self, room_id: u32, player_id: u64) -> usize {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            room.players.push(player_id);
            return room.players.len() - 1;
        }
        0
    }

    pub fn remove_room(&self, room_id: u32) {
        self.rooms.remove(&room_id);
        println!("Removed room {}", room_id);
    }
    pub async fn remove_player_from_room(&self, room_id: u32, player_id: u64) {
        let mut remove_room = false;
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            room.players.retain(|s| *s != player_id);

            if room.players.is_empty() {
                remove_room = true;
            }
        }
        if remove_room {
            self.remove_room(room_id);
        }
    }
    pub fn broadcast_room(
        session_manager: &SessionManager,
        players: &[u64],
        msg: &impl serde::Serialize,
    ) {
        let text = serde_json::to_string(msg).unwrap();
        session_manager.broadcast(players, Message::Text(text.into()));
    }
}
