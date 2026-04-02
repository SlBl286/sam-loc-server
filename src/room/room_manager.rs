use std::sync::Arc;

use serde_json::json;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use crate::{player::Player, room::Room};

pub struct RoomManager {
    rooms: Arc<Mutex<std::collections::HashMap<u32, Room>>>,
}

impl RoomManager {
    pub fn new() -> Self {
        RoomManager {
            rooms: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    async fn get_or_create_room(&self, room_id: u32, max_players: u8) {
        let mut rooms = self.rooms.lock().await;
        if !rooms.contains_key(&room_id) {
            rooms.insert(room_id, Room::new(room_id, max_players));
        }
    }
    async fn add_player_to_room(&self, room_id: u32, player: Player) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            room.players.push(player);
        }
    }

    async fn remove_player_from_room(&self, room_id: u32, player: &Player) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            room
                .players
                .retain(|s| s as *const _ != player as *const _);
        }
    }

    pub async fn broadcast(&self, room_id: Option<u32>) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room_id) = room_id {
            if let Some(room) = rooms.get_mut(&room_id) {
                for player in room.players.iter() {
                    player.sender.send(Message::Text(json!({
                        "type": "room_list",
                        "data": {
                            "id": room_id,
                            "players": room.players.len(),
                            "max_players": room.max_players,
                            "status":  room.status,
                        }
                    }).to_string().into())).expect("Failed to send message");
                }
            }
            else {
                eprintln!("Room {} not found", room_id);
            }
        }
        else{
            for room in rooms.values() {
                for player in room.players.iter() {
                    player.sender.send(Message::Text(json!({
                        "type": "room_list",
                        "data": {
                            "id": room.id,
                            "players": room.players.len(),
                            "max_players": room.max_players,
                            "status":  room.status,
                        }
                    }).to_string().into())).expect("Failed to send message");
                }
            }
        }
    }
}
