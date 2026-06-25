use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
  Connected{user_id: u64},
  Disconnected,

  CreateRoom { 
    room_name: String,
    max_players: Option<u8>,
    bet_size: Option<u64>,
    password: Option<String>,
    turn_limit: Option<u32>,
  },
  JoinRoom {room_id : u32},
  LeaveRoom {room_id : u32},

  PlayCards { cards: Vec<u8>},
  Ready,
  StartGame,
  Unready,
  PassTurn,
  AnnounceSam,
  UpdateProfile {
      display_name: String,
      avatar_url: String,
  },
}