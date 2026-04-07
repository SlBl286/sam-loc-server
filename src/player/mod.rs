use serde::Serialize;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

pub type Tx = mpsc::UnboundedSender<Message>;

#[derive(Clone, Debug)]
pub struct Session {
    pub user_id: u64,
    pub sender: Tx,
    pub cancel: CancellationToken,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub user_id: u64,
}

impl Session {
    pub fn new(user_id: u64, sender: Tx, cancel: CancellationToken) -> Self {
        Session {
            user_id,
            sender,
            cancel
        }
    }
}

pub mod session_manager;
