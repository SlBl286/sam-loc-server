use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;

use crate::player::Tx;

pub fn send_json<T: Serialize>(tx: &Tx, data: &T) {
    if let Ok(text) = serde_json::to_string(data) {
        let _ = tx.send(Message::Text(text.into()));
    }
}