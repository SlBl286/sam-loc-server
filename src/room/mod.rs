
use crate::{player::Player, room::room_status::RoomStatus};

pub struct Room {
    id: u32,
    players: Vec<Player>,
    max_players: u8,
    status: RoomStatus,
}

impl Room {
   pub fn new(id: u32, max_players: u8) -> Self {
        Room {
            id,
            players: Vec::new(),
            max_players,
            status: RoomStatus::Waiting,
        }
    }
}


pub mod room_manager;
pub mod room_status;