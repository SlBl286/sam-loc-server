use serde::Serialize;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

pub type Tx = mpsc::UnboundedSender<Message>;

#[derive(Clone, Debug)]
pub struct Session {
    pub user_id: u64,
    pub sender: Tx,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub user_id: u64,
}

impl Session {
    pub fn new(user_id: u64, sender: Tx) -> Self {
        Session {
            user_id,
            sender,
        }
    }
}

pub mod session_manager;
