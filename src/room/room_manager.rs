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
    pub async fn create_room(
        &self,
        room_id: u32,
        room_name: String,
        max_players: Option<u8>,
        bet_size: Option<u64>,
        password: Option<String>,
    ) {
        self.rooms.insert(
            room_id,
            Room::new(
                room_id,
                room_name,
                max_players.unwrap_or(5),
                bet_size.unwrap_or(1000),
                password,
            ),
        );
    }

    pub async fn get_room(&self, room_id: &u32) -> Option<Room> {
        self.rooms.get(room_id).map(|entry| entry.clone())
    }

    pub async fn set_player_ready(&self, room_id: u32, player_id: u64, ready: bool) -> Option<Room> {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            room.set_player_ready(player_id, ready);
            return Some(room.clone());
        }
        None
    }

    pub async fn start_room_game(&self, room_id: u32) -> Option<Result<Option<(u64, crate::game::ToiTrangReason)>, String>> {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            return Some(room.start_game());
        }
        None
    }

    pub async fn play_room_cards(&self, room_id: u32, player_id: u64, cards: &[u8]) -> Option<Result<bool, String>> {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            return Some(room.play_cards(player_id, cards));
        }
        None
    }

    pub async fn pass_room_turn(&self, room_id: u32, player_id: u64) -> Option<Result<(), String>> {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            return Some(room.pass_turn(player_id));
        }
        None
    }

    pub async fn announce_room_sam(&self, room_id: u32, player_id: u64) -> Option<Result<(), String>> {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            return Some(room.announce_sam(player_id));
        }
        None
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
