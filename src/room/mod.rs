
use serde::Serialize;

use crate::{ room::room_status::RoomStatus};

#[derive(Clone,Debug, Serialize)]
pub struct Room {
    id: u32,
    name: String,
    players: Vec<u64>,
    max_players: u8,
    status: RoomStatus,
}

impl Room {
   pub fn new(id: u32, name: String, max_players: u8) -> Self {
        Room {
            id,
            name,
            players: Vec::new(),
            max_players,
            status: RoomStatus::Waiting,
        }
    }

    pub fn get_num_players(&self)-> usize {
        self.players.len()
    }
}


pub mod room_manager;
pub mod room_status;