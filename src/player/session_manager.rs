use dashmap::DashMap;
use tokio_tungstenite::tungstenite::Message;

use crate::player::Session;

pub struct SessionManager {
    pub sessions: DashMap<i64, Session>, // player_id -> session
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: DashMap::new(),
        }
    }
    pub fn add_session(&self, session: Session) {
        let user_id = session.user_id;

        // nếu player đã tồn tại → kick session cũ
        if let Some(old) = self.sessions.insert(user_id, session) {
            println!("Player {} reconnected → closing old session", user_id);

            let _ = old.sender.send(Message::Close(None));
        }
    }
    pub fn remove_session(&self, user_id: i64) {
        self.sessions.remove(&user_id);
        println!("Removed session {}", user_id);
    }
     pub fn get_session(&self, user_id: &i64) -> Option<Session> {
        self.sessions.get(user_id).map(|s| s.clone())
    }

    // ===== SEND TO ONE PLAYER =====
    pub fn send_to(&self, user_id: i64, msg: Message) {
        if let Some(session) = self.sessions.get(&user_id) {
            if let Err(_) = session.sender.send(msg) {
                println!("Send failed → removing session {}", user_id);
                self.sessions.remove(&user_id);
            }
        }
    }

    // ===== BROADCAST TO MANY =====
    pub fn broadcast(&self, players: &[i64], msg: Message) {
        for user_id in players {
            self.send_to(*user_id, msg.clone());
        }
    }

    // ===== CHECK ONLINE =====
    pub fn is_online(&self, user_id: i64) -> bool {
        self.sessions.contains_key(&user_id)
    }

    // ===== COUNT =====
    pub fn count(&self) -> usize {
        self.sessions.len()
    }
}
