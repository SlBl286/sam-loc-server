use serde::Serialize;

use crate::{player::{UserInfo}, room::Room};

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    PlayerConnected { user_id: u64 },
    PlayerJoinedRoom { user_id: u64, room_id: u32,seat_index: usize },
    PlayerLeftRoom { user_id: u64, room_id: u32 },
    RemovePlayer { room_id: u32, user_id: u64 },
    RoomList { rooms: Vec<Room>},
    PlayerList { players: Vec<UserInfo> },

    Error { message: String },
}
