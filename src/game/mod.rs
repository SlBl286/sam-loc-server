use std::collections::HashMap;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ToiTrangReason {
    DragonStraight,
    FourOfAKind2,
    SameColor,
    FivePairs,
    Range3to9,
}

impl ToiTrangReason {
    pub fn to_string(&self) -> String {
        match self {
            ToiTrangReason::DragonStraight => "Sảnh rồng (3 -> A liên tiếp)".into(),
            ToiTrangReason::FourOfAKind2 => "Tứ quý 2 (4 quân Heo)".into(),
            ToiTrangReason::SameColor => "10 lá cùng màu (Đồng hoa)".into(),
            ToiTrangReason::FivePairs => "5 đôi".into(),
            ToiTrangReason::Range3to9 => "Bài chỉ từ 3 đến 9".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CombinationType {
    Invalid,
    Single(u8),
    Pair(u8),
    Triple(u8),
    Quad(u8),
    Straight(usize, u8), // (length, max_weight)
}

// Get rank: 0 (3) to 12 (2)
pub fn get_rank(card: u8) -> u8 {
    card / 4
}

// Get suit: 0 (Spades ♠), 1 (Clubs ♣), 2 (Diamonds ♦), 3 (Hearts ♥)
pub fn get_suit(card: u8) -> u8 {
    card % 4
}

// Get color: 0 (Black - Spades & Clubs), 1 (Red - Diamonds & Hearts)
pub fn get_color(card: u8) -> u8 {
    let suit = get_suit(card);
    if suit == 0 || suit == 1 {
        0
    } else {
        1
    }
}

// Get weight: 3 (3) to 15 (2)
pub fn get_weight(card: u8) -> u8 {
    let rank = get_rank(card);
    if rank == 12 {
        15 // Heo (2) is largest
    } else if rank == 11 {
        14 // Át (A)
    } else {
        rank + 3
    }
}

// Instant Win check logic
pub fn check_toi_trang(hand: &[u8]) -> Option<ToiTrangReason> {
    if hand.len() != 10 {
        return None;
    }

    let mut weights: Vec<u8> = hand.iter().map(|&c| get_weight(c)).collect();
    weights.sort();

    // 1. Dragon Straight (10 consecutive cards, 3 to A, no 2s allowed in sảnh)
    let has_2 = weights.iter().any(|&w| w == 15);
    if !has_2 {
        let is_consecutive = weights.windows(2).all(|w| w[1] == w[0] + 1);
        if is_consecutive {
            return Some(ToiTrangReason::DragonStraight);
        }
    }

    // 2. Four of a Kind 2
    let count_2 = hand.iter().filter(|&&c| get_rank(c) == 12).count();
    if count_2 == 4 {
        return Some(ToiTrangReason::FourOfAKind2);
    }

    // 3. Same Color (all 10 black or all 10 red)
    let first_color = get_color(hand[0]);
    let same_color = hand.iter().all(|&c| get_color(c) == first_color);
    if same_color {
        return Some(ToiTrangReason::SameColor);
    }

    // 4. 5 Pairs
    let mut rank_counts = HashMap::new();
    for &c in hand {
        *rank_counts.entry(get_rank(c)).or_insert(0) += 1;
    }
    let pair_count = rank_counts.values().filter(|&&count| count >= 2).count();
    if pair_count == 5 {
        return Some(ToiTrangReason::FivePairs);
    }

    // 5. Only 3 to 9 (weights between 3 and 9 inclusive)
    let only_3to9 = weights.iter().all(|&w| w >= 3 && w <= 9);
    if only_3to9 {
        return Some(ToiTrangReason::Range3to9);
    }

    None
}

// Check if played cards form a valid straight
fn get_straight_weight(weights: &[u8]) -> Option<u8> {
    if weights.len() < 3 {
        return None;
    }

    // Sort weights
    let mut sorted = weights.to_vec();
    sorted.sort();

    // Normal straight check
    let normal_consecutive = sorted.windows(2).all(|w| w[1] == w[0] + 1);
    // 2 (weight 15) is not allowed in a normal straight
    let contains_2 = sorted.contains(&15);
    if normal_consecutive && !contains_2 {
        return Some(*sorted.last().unwrap());
    }

    // Special straight check: A-2-3 (mapped weights [14, 15, 3] -> [1, 2, 3])
    // Map: 14 (A) -> 1, 15 (2) -> 2, others remain weight.
    let mut mapped: Vec<u8> = sorted.iter().map(|&w| {
        if w == 14 {
            1
        } else if w == 15 {
            2
        } else {
            w
        }
    }).collect();
    mapped.sort();

    let special_consecutive = mapped.windows(2).all(|w| w[1] == w[0] + 1);
    if special_consecutive {
        // Return the highest card weight in the mapped sequence
        return Some(*mapped.last().unwrap());
    }

    None
}

// Analyze card combination
pub fn analyze_combination(cards: &[u8]) -> CombinationType {
    if cards.is_empty() {
        return CombinationType::Invalid;
    }

    let len = cards.len();
    let mut weights: Vec<u8> = cards.iter().map(|&c| get_weight(c)).collect();
    weights.sort();

    if len == 1 {
        return CombinationType::Single(weights[0]);
    }

    // Check same rank sets (Pair, Triple, Quad)
    let first_rank = get_rank(cards[0]);
    let all_same_rank = cards.iter().all(|&c| get_rank(c) == first_rank);
    if all_same_rank {
        match len {
            2 => return CombinationType::Pair(weights[0]),
            3 => return CombinationType::Triple(weights[0]),
            4 => return CombinationType::Quad(weights[0]),
            _ => return CombinationType::Invalid,
        }
    }

    // Check Straight (Sảnh)
    if let Some(max_weight) = get_straight_weight(&weights) {
        return CombinationType::Straight(len, max_weight);
    }

    CombinationType::Invalid
}

// Check if current play can block last play
pub fn can_block(last: &CombinationType, current: &CombinationType) -> bool {
    match (last, current) {
        (CombinationType::Single(w_last), CombinationType::Single(w_curr)) => w_curr > w_last,
        (CombinationType::Pair(w_last), CombinationType::Pair(w_curr)) => w_curr > w_last,
        (CombinationType::Triple(w_last), CombinationType::Triple(w_curr)) => w_curr > w_last,
        (CombinationType::Quad(w_last), CombinationType::Quad(w_curr)) => w_curr > w_last,
        (CombinationType::Straight(len_last, w_last), CombinationType::Straight(len_curr, w_curr)) => {
            *len_curr == *len_last && w_curr > w_last
        }
        // Special: Quad can block a single 2 (weight 15)
        (CombinationType::Single(15), CombinationType::Quad(_)) => true,
        _ => false,
    }
}
