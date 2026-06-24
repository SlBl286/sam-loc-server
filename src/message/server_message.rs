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
    RoomInfo { room: Room },

    Error { message: String },

    PlayerReadyUpdated { user_id: u64, ready: bool },
    GameStarted { hand: Vec<u8> },
    TurnUpdated { 
        active_player_id: u64, 
        last_played_cards: Vec<u8>, 
        last_played_by: Option<u64>,
        player_card_counts: std::collections::HashMap<u64, usize>,
        passed_players: Vec<u64>,
        is_sam_phase: bool,
        player_golds: std::collections::HashMap<u64, i64>,
        sam_choices: std::collections::HashMap<u64, bool>,
    },
    SamAnnounced { player_id: u64 },
    GameEnded { 
        winner_id: u64, 
        reason: String, 
        hands: std::collections::HashMap<u64, Vec<u8>>,
        sam_announcer: Option<u64>,
    },
}
