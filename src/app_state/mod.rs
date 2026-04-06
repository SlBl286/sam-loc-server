use std::sync::Arc;

use crate::{player, room};

pub struct AppState {
    pub id_generator: Arc<room::room_manager::IdGenerator>,
    pub session_manager: Arc<player::session_manager::SessionManager>,
    pub room_manager: Arc<room::room_manager::RoomManager>,
}


