use std::collections::HashMap;
use serde::Serialize;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::{
    room::room_status::RoomStatus,
    game::{check_toi_trang, analyze_combination, can_block, ToiTrangReason}
};

#[derive(Clone, Debug, Serialize)]
pub struct GameState {
    pub hands: HashMap<u64, Vec<u8>>,
    pub active_player: u64,
    pub last_played_cards: Vec<u8>,
    pub last_played_by: Option<u64>,
    pub passed_players: Vec<u64>,
    pub sam_announcer: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Room {
    pub id: u32,
    pub name: String,
    pub players: Vec<u64>,
    pub max_players: u8,
    pub status: RoomStatus,
    pub bet_size: u64,
    pub password: Option<String>,

    pub ready_players: Vec<u64>,
    pub game_state: Option<GameState>,
}

impl Room {
    pub fn new(id: u32, name: String, max_players: u8, bet_size: u64, password: Option<String>) -> Self {
        Room {
            id,
            name,
            players: Vec::new(),
            max_players,
            status: RoomStatus::Waiting,
            bet_size,
            password,
            ready_players: Vec::new(),
            game_state: None,
        }
    }

    pub fn get_num_players(&self) -> usize {
        self.players.len()
    }

    pub fn set_player_ready(&mut self, player_id: u64, ready: bool) {
        if ready {
            if !self.ready_players.contains(&player_id) {
                self.ready_players.push(player_id);
            }
        } else {
            self.ready_players.retain(|&p| p != player_id);
        }
    }

    // Start Sâm Lốc game. Checks for Toi Trang win instantly.
    // Returns Ok(Some((winner_id, reason))) if someone wins instantly via Toi Trang.
    // Returns Ok(None) if the game starts normally.
    pub fn start_game(&mut self) -> Result<Option<(u64, ToiTrangReason)>, String> {
        if self.players.len() < 2 {
            return Err("Cần ít nhất 2 người chơi để bắt đầu!".into());
        }

        // 1. Shuffling deck
        let mut deck: Vec<u8> = (0..52).collect();
        let mut rng = thread_rng();
        deck.shuffle(&mut rng);

        // 2. Deal 10 cards to each player
        let mut hands = HashMap::new();
        for &player in &self.players {
            let mut hand = Vec::new();
            for _ in 0..10 {
                if let Some(card) = deck.pop() {
                    hand.push(card);
                }
            }
            hands.insert(player, hand);
        }

        // 3. Check for instant wins (Tới Trắng)
        for &player in &self.players {
            if let Some(hand) = hands.get(&player) {
                if let Some(reason) = check_toi_trang(hand) {
                    // Game ended immediately due to Toi Trang
                    self.status = RoomStatus::Waiting;
                    self.ready_players.clear();
                    self.game_state = Some(GameState {
                        hands,
                        active_player: player,
                        last_played_cards: Vec::new(),
                        last_played_by: None,
                        passed_players: Vec::new(),
                        sam_announcer: None,
                    });
                    return Ok(Some((player, reason)));
                }
            }
        }

        // 4. Start normal game state
        let owner_or_first = self.players[0];
        self.status = RoomStatus::Playing;
        self.game_state = Some(GameState {
            hands,
            active_player: owner_or_first,
            last_played_cards: Vec::new(),
            last_played_by: None,
            passed_players: Vec::new(),
            sam_announcer: None,
        });

        Ok(None)
    }

    // Play cards validator and transition logic.
    // Returns Ok(true) if player wins (game ended).
    pub fn play_cards(&mut self, player_id: u64, played: &[u8]) -> Result<bool, String> {
        let state = self.game_state.as_mut().ok_or("Trận đấu chưa bắt đầu!")?;

        if state.active_player != player_id {
            return Err("Không phải lượt của bạn!".into());
        }

        let hand = state.hands.get_mut(&player_id).ok_or("Không tìm thấy bài người chơi!")?;

        // Verify player actually holds these cards
        for &c in played {
            if !hand.contains(&c) {
                return Err("Quân bài không hợp lệ hoặc không có trên tay!".into());
            }
        }

        // Analyze played combination
        let current_comb = analyze_combination(played);
        if current_comb == crate::game::CombinationType::Invalid {
            return Err("Bộ bài đánh ra không đúng luật!".into());
        }

        // Verify it can block the last played cards
        if !state.last_played_cards.is_empty() {
            let last_comb = analyze_combination(&state.last_played_cards);
            if !can_block(&last_comb, &current_comb) {
                return Err("Bộ bài của bạn không đủ lớn để chặn!".into());
            }
        }

        // Remove played cards from hand
        for &c in played {
            if let Some(pos) = hand.iter().position(|&x| x == c) {
                hand.remove(pos);
            }
        }

        // Check victory
        if hand.is_empty() {
            self.status = RoomStatus::Waiting;
            self.ready_players.clear();
            return Ok(true); // Player won, game over
        }

        // Update round state
        state.last_played_cards = played.to_vec();
        state.last_played_by = Some(player_id);

        // Find next player who hasn't passed
        let next_player = get_next_player(&self.players, player_id, &state.passed_players);

        // If turn goes back to the last person who played, reset round
        if Some(next_player) == state.last_played_by {
            state.last_played_cards.clear();
            state.last_played_by = None;
            state.passed_players.clear();
        }

        state.active_player = next_player;
        Ok(false)
    }

    // Pass turn logic.
    pub fn pass_turn(&mut self, player_id: u64) -> Result<(), String> {
        let state = self.game_state.as_mut().ok_or("Trận đấu chưa bắt đầu!")?;

        if state.active_player != player_id {
            return Err("Không phải lượt của bạn!".into());
        }

        if state.last_played_cards.is_empty() {
            return Err("Bạn đang cầm cái, không được bỏ lượt!".into());
        }

        if !state.passed_players.contains(&player_id) {
            state.passed_players.push(player_id);
        }

        let next_player = get_next_player(&self.players, player_id, &state.passed_players);

        if Some(next_player) == state.last_played_by {
            state.last_played_cards.clear();
            state.last_played_by = None;
            state.passed_players.clear();
        }

        state.active_player = next_player;
        Ok(())
    }

    // Announce Sâm logic
    pub fn announce_sam(&mut self, player_id: u64) -> Result<(), String> {
        let state = self.game_state.as_mut().ok_or("Trận đấu chưa bắt đầu!")?;
        state.sam_announcer = Some(player_id);
        state.active_player = player_id;
        Ok(())
    }
}

fn get_next_player(players: &[u64], current: u64, passed: &[u64]) -> u64 {
    let pos = players.iter().position(|&p| p == current).unwrap_or(0);
    for i in 1..players.len() {
        let next_idx = (pos + i) % players.len();
        let next_player = players[next_idx];
        if !passed.contains(&next_player) {
            return next_player;
        }
    }
    current
}

pub mod room_manager;
pub mod room_status;