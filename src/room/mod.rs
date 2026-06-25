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
    pub turn_count: u32,
    pub is_sam_phase: bool,
    pub starter: u64,
    pub sam_choices: HashMap<u64, bool>,
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
    pub turn_limit: u32,

    pub ready_players: Vec<u64>,
    pub game_state: Option<GameState>,
    pub first_player: Option<u64>,
    pub player_golds: HashMap<u64, i64>,
    pub spectators: Vec<u64>,
    pub player_names: HashMap<u64, String>,
    pub player_avatars: HashMap<u64, String>,
}

impl Room {
    pub fn new(id: u32, name: String, max_players: u8, bet_size: u64, password: Option<String>, turn_limit: Option<u32>) -> Self {
        Room {
            id,
            name,
            players: Vec::new(),
            max_players,
            status: RoomStatus::Waiting,
            bet_size,
            password,
            turn_limit: turn_limit.unwrap_or(15),
            ready_players: Vec::new(),
            game_state: None,
            first_player: None,
            player_golds: HashMap::new(),
            spectators: Vec::new(),
            player_names: HashMap::new(),
            player_avatars: HashMap::new(),
        }
    }

    pub fn get_num_players(&self) -> usize {
        self.players.len()
    }

    pub fn set_player_ready(&mut self, player_id: u64, ready: bool) {
        // Only allow changing ready status if game is waiting
        if self.status != RoomStatus::Waiting {
            return;
        }
        // Host does not ready up
        if let Some(&host_id) = self.players.first() {
            if player_id == host_id {
                return;
            }
        }
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
        let host_id = self.players.first().copied().ok_or("Không tìm thấy chủ phòng!")?;
        
        let ready_other_players: Vec<u64> = self.ready_players.iter()
            .copied()
            .filter(|&p| p != host_id)
            .collect();
            
        if ready_other_players.is_empty() {
            return Err("Cần ít nhất 1 người chơi khác sẵn sàng để bắt đầu!".into());
        }

        // Starter is self.first_player if present and active, else host_id
        let starter = if let Some(starter_id) = self.first_player {
            if self.players.contains(&starter_id) && (starter_id == host_id || ready_other_players.contains(&starter_id)) {
                starter_id
            } else {
                host_id
            }
        } else {
            host_id
        };

        // 1. Shuffling deck
        let mut deck: Vec<u8> = (0..52).collect();
        let mut rng = thread_rng();
        deck.shuffle(&mut rng);

        // 2. Deal 10 cards to active players
        let mut hands = HashMap::new();
        
        // Host gets cards
        let mut host_hand = Vec::new();
        for _ in 0..10 {
            if let Some(card) = deck.pop() {
                host_hand.push(card);
            }
        }
        hands.insert(host_id, host_hand);
        
        // Ready players get cards
        for &player in &ready_other_players {
            let mut hand = Vec::new();
            for _ in 0..10 {
                if let Some(card) = deck.pop() {
                    hand.push(card);
                }
            }
            hands.insert(player, hand);
        }

        // Active players in turn order starting from starter
        let mut active_players_in_order = Vec::new();
        let pos = self.players.iter().position(|&p| p == starter).unwrap_or(0);
        for i in 0..self.players.len() {
            let idx = (pos + i) % self.players.len();
            let p_id = self.players[idx];
            if p_id == host_id || ready_other_players.contains(&p_id) {
                active_players_in_order.push(p_id);
            }
        }

        // 3. Check for instant wins (Tới Trắng) in order
        for &player in &active_players_in_order {
            if let Some(hand) = hands.get(&player) {
                if let Some(reason) = check_toi_trang(hand) {
                    // Game ended immediately due to Toi Trang
                    self.status = RoomStatus::Waiting;
                    self.ready_players.clear();
                    self.first_player = Some(player); // Save winner
                    self.game_state = Some(GameState {
                        hands,
                        active_player: player,
                        last_played_cards: Vec::new(),
                        last_played_by: None,
                        passed_players: Vec::new(),
                        sam_announcer: None,
                        turn_count: 0,
                        is_sam_phase: false,
                        starter,
                        sam_choices: HashMap::new(),
                    });
                    self.update_payouts(player);
                    return Ok(Some((player, reason)));
                }
            }
        }

        // 4. Start game in Sâm Announce Phase
        self.status = RoomStatus::Playing;
        self.game_state = Some(GameState {
            hands,
            active_player: starter,
            last_played_cards: Vec::new(),
            last_played_by: None,
            passed_players: Vec::new(),
            sam_announcer: None,
            turn_count: 0,
            is_sam_phase: true,
            starter,
            sam_choices: HashMap::new(),
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

        // Rule: cannot play 2 as the last card/combination
        if played.len() == hand.len() && played.iter().any(|&c| crate::game::get_rank(c) == 12) {
            return Err("Không được đánh 2 cuối cùng!".into());
        }

        // Rule: cannot play cards that leave the hand with only 2s (thối hai)
        if played.len() < hand.len() {
            let remaining_count = hand.len() - played.len();
            let total_twos = hand.iter().filter(|&&c| crate::game::get_rank(c) == 12).count();
            let played_twos = played.iter().filter(|&&c| crate::game::get_rank(c) == 12).count();
            let remaining_twos = total_twos - played_twos;
            if remaining_twos == remaining_count {
                return Err("Không được đánh nước đi để lại toàn quân 2 (thối hai)!".into());
            }
        }

        // Rule: if player has only 2s left, they can only pass
        if hand.iter().all(|&c| crate::game::get_rank(c) == 12) {
            return Err("Bạn chỉ còn 2, chỉ được bỏ lượt!".into());
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

        // Check mid-game block penalties (Tứ quý chặn 2, Tứ quý chồng tiếp x2)
        if !state.last_played_cards.is_empty() {
            let last_comb = analyze_combination(&state.last_played_cards);
            if let Some(victim_id) = state.last_played_by {
                match (&last_comb, &current_comb) {
                    (crate::game::CombinationType::Single(15), crate::game::CombinationType::Quad(_)) => {
                        let penalty = 10 * (self.bet_size as i64);
                        let victim_gold = self.player_golds.entry(victim_id).or_insert(100000);
                        *victim_gold = (*victim_gold - penalty).max(0);

                        let blocker_gold = self.player_golds.entry(player_id).or_insert(100000);
                        *blocker_gold += penalty;
                    }
                    (crate::game::CombinationType::Quad(_), crate::game::CombinationType::Quad(_)) => {
                        let penalty = 20 * (self.bet_size as i64);
                        let victim_gold = self.player_golds.entry(victim_id).or_insert(100000);
                        *victim_gold = (*victim_gold - penalty).max(0);

                        let blocker_gold = self.player_golds.entry(player_id).or_insert(100000);
                        *blocker_gold += penalty;
                    }
                    _ => {}
                }
            }
        }

        // Check if Sâm announcer is blocked (Sâm fails)
        if state.sam_announcer.is_some() && !state.last_played_cards.is_empty() {
            state.last_played_cards = played.to_vec();
            state.last_played_by = Some(player_id);
            self.update_payouts(player_id);
            self.status = RoomStatus::Waiting;
            self.ready_players.clear();
            self.first_player = Some(player_id);
            return Ok(true); // Blocker wins immediately
        }

        // Check victory
        if hand.is_empty() {
            self.update_payouts(player_id);
            self.status = RoomStatus::Waiting;
            self.ready_players.clear();
            self.first_player = Some(player_id);
            return Ok(true); // Player won, game over
        }

        // Update round state
        state.last_played_cards = played.to_vec();
        state.last_played_by = Some(player_id);

        // Find next player who hasn't passed
        let next_player = get_next_player(&self.players, player_id, &state.passed_players, &state.hands);

        // If turn goes back to the last person who played, reset round
        if Some(next_player) == state.last_played_by {
            state.last_played_cards.clear();
            state.last_played_by = None;
            state.passed_players.clear();
        }

        state.active_player = next_player;
        state.turn_count += 1;
        Ok(false)
    }

    // Pass turn logic.
    pub fn pass_turn(&mut self, player_id: u64) -> Result<bool, String> {
        let state = self.game_state.as_mut().ok_or("Trận đấu chưa bắt đầu!")?;

        if state.is_sam_phase {
            if !state.hands.contains_key(&player_id) {
                return Err("Bạn không phải người chơi trong ván này!".into());
            }
            // Record choice
            state.sam_choices.insert(player_id, false);

            // Check if all players have chosen
            let all_chosen = state.sam_choices.len() == state.hands.len();
            if all_chosen {
                self.resolve_sam_phase();
                return Ok(true); // Sâm phase ended
            } else {
                return Ok(false); // Sâm phase continues
            }
        }

        if state.active_player != player_id {
            return Err("Không phải lượt của bạn!".into());
        }

        if state.last_played_cards.is_empty() {
            let hand = state.hands.get(&player_id).ok_or("Không tìm thấy bài người chơi!")?;
            let only_twos = !hand.is_empty() && hand.iter().all(|&c| crate::game::get_rank(c) == 12);
            if !only_twos {
                return Err("Bạn đang cầm cái, không được bỏ lượt!".into());
            }
        }

        if !state.passed_players.contains(&player_id) {
            state.passed_players.push(player_id);
        }

        let next_player = get_next_player(&self.players, player_id, &state.passed_players, &state.hands);

        if Some(next_player) == state.last_played_by {
            state.last_played_cards.clear();
            state.last_played_by = None;
            state.passed_players.clear();
        }

        state.active_player = next_player;
        state.turn_count += 1;
        Ok(false)
    }

    // Announce Sâm logic
    pub fn announce_sam(&mut self, player_id: u64) -> Result<bool, String> {
        let state = self.game_state.as_mut().ok_or("Trận đấu chưa bắt đầu!")?;
        if !state.is_sam_phase {
            return Err("Không phải trong giai đoạn báo Sâm!".into());
        }
        if !state.hands.contains_key(&player_id) {
            return Err("Bạn không phải người chơi trong ván này!".into());
        }

        // Record choice
        state.sam_choices.insert(player_id, true);

        // Check if all players have chosen
        let all_chosen = state.sam_choices.len() == state.hands.len();
        if all_chosen {
            self.resolve_sam_phase();
            Ok(true) // Sâm phase ended
        } else {
            Ok(false) // Sâm phase continues
        }
    }

    // Resolves choices at the end of Sâm phase
    pub fn resolve_sam_phase(&mut self) {
        let state = match self.game_state.as_mut() {
            Some(s) => s,
            None => return,
        };

        if !state.is_sam_phase {
            return;
        }

        // Active players in turn order starting from starter
        let mut active_players_in_order = Vec::new();
        let pos = self.players.iter().position(|&p| p == state.starter).unwrap_or(0);
        for i in 0..self.players.len() {
            let idx = (pos + i) % self.players.len();
            let p_id = self.players[idx];
            if state.hands.contains_key(&p_id) {
                active_players_in_order.push(p_id);
            }
        }

        // Find the first player in order who chose to announce Sam
        let mut sam_player = None;
        for &p_id in &active_players_in_order {
            if let Some(&chose_sam) = state.sam_choices.get(&p_id) {
                if chose_sam {
                    sam_player = Some(p_id);
                    break;
                }
            }
        }

        if let Some(p_id) = sam_player {
            state.sam_announcer = Some(p_id);
            state.active_player = p_id;
        } else {
            state.sam_announcer = None;
            state.active_player = state.starter;
        }

        state.is_sam_phase = false;
        state.turn_count += 1;
    }

    pub fn force_resolve_sam(&mut self) -> Result<(), String> {
        let state = self.game_state.as_mut().ok_or("Trận đấu chưa bắt đầu!")?;
        if !state.is_sam_phase {
            return Err("Không phải trong giai đoạn báo Sâm!".into());
        }

        // For any player who hasn't chosen, default to false
        let players: Vec<u64> = state.hands.keys().copied().collect();
        for p_id in players {
            state.sam_choices.entry(p_id).or_insert(false);
        }

        self.resolve_sam_phase();
        Ok(())
    }

    fn update_payouts(&mut self, winner_id: u64) {
        let state = match &self.game_state {
            Some(s) => s,
            None => return,
        };

        let bet = self.bet_size as i64;
        let mut payouts = std::collections::HashMap::new();

        let active_players: Vec<u64> = state.hands.keys().copied().collect();
        let other_active_count = (active_players.len() as i64) - 1;

        if let Some(announcer_id) = state.sam_announcer {
            if winner_id == announcer_id {
                let mut total_win = 0;
                for &p_id in &active_players {
                    if p_id != winner_id {
                        let loss = 20 * bet;
                        payouts.insert(p_id, -loss);
                        total_win += loss;
                    }
                }
                payouts.insert(winner_id, total_win);
            } else {
                let penalty = 20 * bet * other_active_count;
                payouts.insert(announcer_id, -penalty);
                payouts.insert(winner_id, penalty);
                for &p_id in &active_players {
                    if p_id != announcer_id && p_id != winner_id {
                        payouts.insert(p_id, 0);
                    }
                }
            }
        } else {
            let mut total_win = 0;
            for &p_id in &active_players {
                if p_id != winner_id {
                    let hand = state.hands.get(&p_id).cloned().unwrap_or_default();
                    let cards_count = hand.len() as i64;
                    let heo_count = hand.iter().filter(|&&c| (c / 4) == 12).count() as i64;

                    let loss = if cards_count == 10 {
                        (15 + heo_count * 5) * bet
                    } else {
                        (cards_count + heo_count * 5) * bet
                    };
                    payouts.insert(p_id, -loss);
                    total_win += loss;
                }
            }
            payouts.insert(winner_id, total_win);
        }

        for (p_id, diff) in payouts {
            let entry = self.player_golds.entry(p_id).or_insert(100000);
            *entry = (*entry + diff).max(0);
        }
    }
}

pub(crate) fn get_next_player(
    players: &[u64],
    current: u64,
    passed: &[u64],
    hands: &std::collections::HashMap<u64, Vec<u8>>,
) -> u64 {
    let pos = players.iter().position(|&p| p == current).unwrap_or(0);
    for i in 1..players.len() {
        let next_idx = (pos + i) % players.len();
        let next_player = players[next_idx];
        if hands.contains_key(&next_player) && !passed.contains(&next_player) {
            return next_player;
        }
    }
    current
}

pub mod room_manager;
pub mod room_status;