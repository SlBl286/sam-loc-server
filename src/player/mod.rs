use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

pub type Tx = mpsc::UnboundedSender<Message>;

#[derive(Clone)]
pub struct Session  {
    pub user_id: i64,
    pub room_id: Option<u32>,
    pub sender : Tx,
}

impl Session {
    pub fn new(user_id:i64,room_id: Option<u32>,sender:Tx)-> Self{
        Session { user_id, room_id, sender}
    }
}

pub mod session_manager;