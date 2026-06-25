use std::sync::atomic::AtomicU32;

use dashmap::DashMap;
use tokio_tungstenite::tungstenite::Message;

use crate::{player::session_manager::SessionManager, room::Room};
use super::get_next_player;

#[derive(Debug)]
pub struct LeaveRoomResult {
    pub room_deleted: bool,
    pub game_ended: Option<(u64, String, std::collections::HashMap<u64, Vec<u8>>, Option<u64>)>, // (winner_id, reason, hands, sam_announcer)
    pub turn_updated: Option<(u64, u32)>, // (next_active_player, next_turn_count)
}

pub struct RoomManager {
    rooms: DashMap<u32, Room>,
    db: sqlx::PgPool,
}
pub struct IdGenerator {
    last_id: AtomicU32,
}

impl IdGenerator {
    pub fn new() -> Self {
        IdGenerator {
            last_id: AtomicU32::new(0),
        }
    }

    pub fn generate_id(&self) -> u32 {
        self.last_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

impl RoomManager {
    pub fn new(db: sqlx::PgPool) -> Self {
        RoomManager {
            rooms: DashMap::new(),
            db,
        }
    }

    pub async fn get_rooms(&self) -> Vec<Room> {
        self.rooms
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
    pub async fn create_room(
        &self,
        room_id: u32,
        room_name: String,
        max_players: Option<u8>,
        bet_size: Option<u64>,
        password: Option<String>,
        turn_limit: Option<u32>,
    ) {
        self.rooms.insert(
            room_id,
            Room::new(
                room_id,
                room_name,
                max_players.unwrap_or(5),
                bet_size.unwrap_or(1000),
                password,
                turn_limit,
            ),
        );
    }

    pub async fn get_room(&self, room_id: &u32) -> Option<Room> {
        self.rooms.get(room_id).map(|entry| entry.clone())
    }

    pub async fn set_player_ready(&self, room_id: u32, player_id: u64, ready: bool) -> Option<Room> {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            room.set_player_ready(player_id, ready);
            return Some(room.clone());
        }
        None
    }

    pub async fn start_room_game(&self, room_id: u32) -> Option<Result<Option<(u64, crate::game::ToiTrangReason)>, String>> {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            return Some(room.start_game());
        }
        None
    }

    pub async fn play_room_cards(&self, room_id: u32, player_id: u64, cards: &[u8]) -> Option<Result<bool, String>> {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            return Some(room.play_cards(player_id, cards));
        }
        None
    }

    pub async fn pass_room_turn(&self, room_id: u32, player_id: u64) -> Option<Result<bool, String>> {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            return Some(room.pass_turn(player_id));
        }
        None
    }

    pub async fn announce_room_sam(&self, room_id: u32, player_id: u64) -> Option<Result<bool, String>> {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            return Some(room.announce_sam(player_id));
        }
        None
    }

    pub async fn force_resolve_room_sam(&self, room_id: u32) -> Option<Result<(), String>> {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            return Some(room.force_resolve_sam());
        }
        None
    }

    pub async fn reset_room_game_state(&self, room_id: u32) {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            room.game_state = None;
        }
    }

    pub async fn add_player_to_room(&self, room_id: u32, player_id: u64) -> usize {
        let profile = crate::database::user_repo::get_user_profile(&self.db, player_id).await;
        let (name, avatar, gold) = match profile {
            Some(p) => (p.display_name, p.avatar_url, p.gold),
            None => (format!("Guest_{}", player_id), "".to_string(), 500000),
        };

        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            room.player_names.insert(player_id, name);
            room.player_avatars.insert(player_id, avatar);
            room.player_golds.insert(player_id, gold);
            if room.players.len() >= room.max_players as usize {
                if !room.spectators.contains(&player_id) {
                    room.spectators.push(player_id);
                }
                return 999;
            } else {
                if !room.players.contains(&player_id) {
                    room.players.push(player_id);
                }
                return room.players.len() - 1;
            }
        }
        0
    }

    pub fn remove_room(&self, room_id: u32) {
        self.rooms.remove(&room_id);
        println!("Removed room {}", room_id);
    }
    pub async fn remove_player_from_room(&self, room_id: u32, player_id: u64) -> LeaveRoomResult {
        let mut room_deleted = false;
        let mut game_ended = None;
        let mut turn_updated = None;

        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            let was_player = room.players.contains(&player_id);
            room.players.retain(|s| *s != player_id);
            room.spectators.retain(|s| *s != player_id);
            room.ready_players.retain(|s| *s != player_id);

            // Mid-game escape logic
            let mut escape_handled = false;
            let mut loss = 0;
            let mut remaining_active = Vec::new();
            let mut state_sam_announcer = None;
            let mut state_hands = std::collections::HashMap::new();

            if room.status == crate::room::room_status::RoomStatus::Playing {
                let bet = room.bet_size as i64;
                let room_players = room.players.clone();
                if let Some(state) = room.game_state.as_mut() {
                    if state.hands.contains_key(&player_id) {
                        loss = if state.sam_announcer.is_some() {
                            20 * bet
                        } else {
                            15 * bet
                        };
                        state_sam_announcer = state.sam_announcer;
                        
                        // Remove from active hands
                        state.hands.remove(&player_id);

                        // Remaining active players who are still in the room
                        remaining_active = state.hands.keys()
                            .copied()
                            .filter(|p| room_players.contains(p))
                            .collect();

                        state_hands = state.hands.clone();
                        escape_handled = true;
                    }
                }
            }

            if escape_handled {
                // Deduct gold from escaping player
                let entry = room.player_golds.entry(player_id).or_insert(500000);
                *entry = (*entry - loss).max(0);

                let room_players = room.players.clone();

                if remaining_active.len() <= 1 {
                    // Game ends immediately
                    room.status = crate::room::room_status::RoomStatus::Waiting;
                    room.ready_players.clear();

                    if let Some(&winner_id) = remaining_active.first() {
                        // Credit the winner
                        let win_entry = room.player_golds.entry(winner_id).or_insert(500000);
                        *win_entry += loss;

                        game_ended = Some((
                            winner_id,
                            "Đối thủ thoát - Bạn thắng cuộc!".to_string(),
                            state_hands,
                            state_sam_announcer,
                        ));
                    } else {
                        game_ended = Some((
                            0,
                            "Không còn người chơi trong ván".to_string(),
                            state_hands,
                            state_sam_announcer,
                        ));
                    }
                    room.game_state = None;
                } else {
                    // Game continues. Split the penalty among remaining active players.
                    let split_win = loss / (remaining_active.len() as i64);
                    for &w_id in &remaining_active {
                        let w_entry = room.player_golds.entry(w_id).or_insert(500000);
                        *w_entry += split_win;
                    }

                    // Reset the round if the escaping player was the last one who played cards
                    if let Some(state) = room.game_state.as_mut() {
                        if state.last_played_by == Some(player_id) {
                            state.last_played_cards.clear();
                            state.last_played_by = None;
                            state.passed_players.clear();
                        }
                    }

                    // If it was their turn, move to the next player
                    if let Some(state) = room.game_state.as_mut() {
                        if state.active_player == player_id {
                            let next_player = get_next_player(&room_players, player_id, &state.passed_players, &state.hands);
                            state.active_player = next_player;

                            if Some(next_player) == state.last_played_by {
                                state.last_played_cards.clear();
                                state.last_played_by = None;
                                state.passed_players.clear();
                            }

                            state.turn_count += 1;
                            turn_updated = Some((state.active_player, state.turn_count));
                        }
                    }
                }
            }

            // Save updated golds to database
            for (&p_id, &g) in &room.player_golds {
                let _ = sqlx::query!(
                    "UPDATE users SET gold = $1 WHERE id = $2",
                    g,
                    p_id as i64
                )
                .execute(&self.db)
                .await;
            }

            if room.players.is_empty() {
                room_deleted = true;
            } else if was_player && room.players.len() < room.max_players as usize && !room.spectators.is_empty() {
                let next_player = room.spectators.remove(0);
                room.players.push(next_player);
            }
        }

        if room_deleted {
            self.remove_room(room_id);
        }

        LeaveRoomResult {
            room_deleted,
            game_ended,
            turn_updated,
        }
    }
    pub fn broadcast_room(
        session_manager: &SessionManager,
        room: &Room,
        msg: &impl serde::Serialize,
    ) {
        let mut targets = room.players.clone();
        targets.extend(&room.spectators);
        let text = serde_json::to_string(msg).unwrap();
        session_manager.broadcast(&targets, Message::Text(text.into()));
    }

    pub async fn update_room_player_profile(&self, room_id: u32, player_id: u64, name: String, avatar: String) {
        if let Some(mut room) = self.rooms.get_mut(&room_id) {
            room.player_names.insert(player_id, name);
            room.player_avatars.insert(player_id, avatar);
        }
    }

    pub async fn save_room_golds(&self, room_id: u32) {
        if let Some(room) = self.rooms.get(&room_id) {
            for (&player_id, &gold) in &room.player_golds {
                let _ = sqlx::query!(
                    "UPDATE users SET gold = $1 WHERE id = $2",
                    gold,
                    player_id as i64
                )
                .execute(&self.db)
                .await;
            }
        }
    }
}
