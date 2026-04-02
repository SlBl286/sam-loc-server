use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;



pub struct Player {
    pub user_id: u32,
    pub sender : mpsc::UnboundedSender<Message>,
}