use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
  Connected{user_id: u64},
  Disconnected,

  CreateRoom { room_name: String,max_players: Option<u8>},
  JoinRoom {room_id : u32},
  LeaveRoom {room_id : u32},

  PlayCards { cards: Vec<u8>},
  Ready,
  Unready,

}