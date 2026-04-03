use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
  Connected{user_id: i64},
  JoinRoom {room_id : u32},
  PlayCards { cards: Vec<u8>},
  Ready,
  Unready,

}