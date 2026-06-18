use std::{net::SocketAddr, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, protocol::frame::coding::CloseCode},
};
use tokio_util::sync::CancellationToken;

use crate::{
    app_state::AppState,
    message::{client_message::ClientMessage, server_message::ServerMessage},
    player::{
        Session,
        session_manager::{self, SessionManager},
    },
    room::room_manager::RoomManager,
};
struct ConnectionContext {
    player_id: Option<u64>,
    room_id: Option<u32>,
}
pub async fn accept_connection(stream: TcpStream, addr: SocketAddr, app_state: Arc<AppState>) {
    match handle_connection(stream, addr, app_state).await {
        Err(e) => match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8(_) => (),
            err => eprintln!("Error processing connection: {}", err),
        },
        _ => (),
    }
}

pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    state: Arc<AppState>,
) -> Result<(), Error> {
    match accept_async(stream).await {
        Ok(ws_stream) => {
            let mut ctx = ConnectionContext {
                player_id: None,
                room_id: None,
            };
            let (mut write, mut read) = ws_stream.split();
            let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
            let cancel = CancellationToken::new();

            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    if let Err(e) = write.send(msg).await {
                        println!("Send error: {}", e);
                        break;
                    }
                }
            });
            while let Some(msg) = read.next().await {
                if cancel.is_cancelled() {
                    break;
                }
                match msg {
                    Ok(Message::Text(text)) => {
                        // parse JSON
                        let parsed = serde_json::from_str::<ClientMessage>(&text);
                        println!("Parsed message: {:?}", parsed);
                        let response = match parsed {
                            Ok(ClientMessage::Connected { user_id }) => {
                                println!("user {} connected", user_id);
                                ctx.player_id = Some(user_id);
                                state.session_manager.add_session(Session::new(
                                    user_id,
                                    tx.clone(),
                                    cancel.clone(),
                                ));
                                ServerMessage::RoomList {
                                    rooms: state.room_manager.get_rooms().await,
                                }
                            }
                            Ok(ClientMessage::Disconnected) => {
                                if let Some(uid) = ctx.player_id {
                                    handle_player_disconnect(&state, uid, ctx.room_id).await;
                                    ctx.player_id = None;
                                    ctx.room_id = None;
                                }
                                ServerMessage::Error {
                                    message: "Disconnected".to_string(),
                                }
                            }

                            Ok(ClientMessage::CreateRoom {
                                room_name,
                                max_players,
                                bet_size,
                                password,
                            }) => {
                                let room_id = state.id_generator.generate_id();
                                state
                                    .room_manager
                                    .create_room(room_id, room_name, max_players, bet_size, password)
                                    .await;

                                if let Some(player_id) = ctx.player_id {
                                    let seat_index = state
                                        .room_manager
                                        .add_player_to_room(room_id, player_id)
                                        .await;
                                    ctx.room_id = Some(room_id);
                                    let room_list_msg = ServerMessage::RoomList {
                                        rooms: state.room_manager.get_rooms().await,
                                    };
                                    let json = serde_json::to_string(&room_list_msg).unwrap();
                                    state
                                        .session_manager
                                        .broadcast_all(Message::Text(json.into()));

                                    if let Some(room) = state.room_manager.get_room(&room_id).await {
                                        let info_msg = ServerMessage::RoomInfo { room: room.clone() };
                                        RoomManager::broadcast_room(&state.session_manager, &room.players, &info_msg);
                                    }

                                    ServerMessage::PlayerJoinedRoom {
                                        user_id: player_id,
                                        room_id,
                                        seat_index,
                                    }
                                } else {
                                    ServerMessage::Error {
                                        message: "Create room failed".to_string(),
                                    }
                                }
                            }
                            Ok(ClientMessage::JoinRoom { room_id }) => {
                                if let Some(player_id) = ctx.player_id {
                                    let seat_index = state
                                        .room_manager
                                        .add_player_to_room(room_id, player_id)
                                        .await;
                                    ctx.room_id = Some(room_id);

                                    let room_list_msg = ServerMessage::RoomList {
                                        rooms: state.room_manager.get_rooms().await,
                                    };
                                    let json = serde_json::to_string(&room_list_msg).unwrap();
                                    state
                                        .session_manager
                                        .broadcast_all(Message::Text(json.into()));

                                    if let Some(room) = state.room_manager.get_room(&room_id).await {
                                        let info_msg = ServerMessage::RoomInfo { room: room.clone() };
                                        RoomManager::broadcast_room(&state.session_manager, &room.players, &info_msg);
                                    }

                                    ServerMessage::PlayerJoinedRoom {
                                        user_id: player_id,
                                        room_id,
                                        seat_index,
                                    }
                                } else {
                                    ServerMessage::Error {
                                        message: "Join room failed".to_string(),
                                    }
                                }
                            }
                            Ok(ClientMessage::LeaveRoom { room_id }) => {
                                if let Some(player_id) = ctx.player_id {
                                    state
                                        .room_manager
                                        .remove_player_from_room(room_id, player_id)
                                        .await;
                                    ctx.room_id = None;

                                    let room_list_msg = ServerMessage::RoomList {
                                        rooms: state.room_manager.get_rooms().await,
                                    };
                                    let json = serde_json::to_string(&room_list_msg).unwrap();
                                    state
                                        .session_manager
                                        .broadcast_all(Message::Text(json.into()));

                                    if let Some(room) = state.room_manager.get_room(&room_id).await {
                                        let info_msg = ServerMessage::RoomInfo { room: room.clone() };
                                        RoomManager::broadcast_room(&state.session_manager, &room.players, &info_msg);
                                    }

                                    ServerMessage::PlayerLeftRoom {
                                        user_id: player_id,
                                        room_id,
                                    }
                                } else {
                                    ServerMessage::Error {
                                        message: "Leave room failed".to_string(),
                                    }
                                }
                            }
                            Ok(ClientMessage::Ready) => {
                                if let (Some(player_id), Some(room_id)) = (ctx.player_id, ctx.room_id) {
                                    if let Some(room) = state.room_manager.set_player_ready(room_id, player_id, true).await {
                                        let info_msg = ServerMessage::RoomInfo { room: room.clone() };
                                        RoomManager::broadcast_room(&state.session_manager, &room.players, &info_msg);

                                        // Start game if everyone is ready and players count >= 2
                                        if room.ready_players.len() == room.players.len() && room.players.len() >= 2 {
                                            if let Some(start_res) = state.room_manager.start_room_game(room_id).await {
                                                match start_res {
                                                    Ok(toi_trang_opt) => {
                                                        if let Some(updated_room) = state.room_manager.get_room(&room_id).await {
                                                            if let Some(game_state) = &updated_room.game_state {
                                                                let playing_info_msg = ServerMessage::RoomInfo { room: updated_room.clone() };
                                                                RoomManager::broadcast_room(&state.session_manager, &updated_room.players, &playing_info_msg);

                                                                // Send private hands
                                                                for &p_id in &updated_room.players {
                                                                    if let Some(hand) = game_state.hands.get(&p_id) {
                                                                        let hand_msg = ServerMessage::GameStarted { hand: hand.clone() };
                                                                        let hand_json = serde_json::to_string(&hand_msg).unwrap();
                                                                        state.session_manager.send_to(p_id, Message::Text(hand_json.into()));
                                                                    }
                                                                }

                                                                // If tới trắng, end game instantly
                                                                if let Some((winner_id, reason)) = toi_trang_opt {
                                                                    let end_msg = ServerMessage::GameEnded {
                                                                        winner_id,
                                                                        reason: reason.to_string(),
                                                                        hands: game_state.hands.clone(),
                                                                    };
                                                                    RoomManager::broadcast_room(&state.session_manager, &updated_room.players, &end_msg);
                                                                } else {
                                                                    // Begin normal game turn
                                                                    let mut card_counts = std::collections::HashMap::new();
                                                                    for (&p_id, hand) in &game_state.hands {
                                                                        card_counts.insert(p_id, hand.len());
                                                                    }
                                                                    let turn_msg = ServerMessage::TurnUpdated {
                                                                        active_player_id: game_state.active_player,
                                                                        last_played_cards: Vec::new(),
                                                                        last_played_by: None,
                                                                        player_card_counts: card_counts,
                                                                        passed_players: Vec::new(),
                                                                    };
                                                                    RoomManager::broadcast_room(&state.session_manager, &updated_room.players, &turn_msg);
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        RoomManager::broadcast_room(&state.session_manager, &room.players, &ServerMessage::Error { message: e });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                ServerMessage::PlayerReadyUpdated { user_id: ctx.player_id.unwrap_or(0), ready: true }
                            }
                            Ok(ClientMessage::Unready) => {
                                if let (Some(player_id), Some(room_id)) = (ctx.player_id, ctx.room_id) {
                                    if let Some(room) = state.room_manager.set_player_ready(room_id, player_id, false).await {
                                        let info_msg = ServerMessage::RoomInfo { room: room.clone() };
                                        RoomManager::broadcast_room(&state.session_manager, &room.players, &info_msg);
                                    }
                                }
                                ServerMessage::PlayerReadyUpdated { user_id: ctx.player_id.unwrap_or(0), ready: false }
                            }
                            Ok(ClientMessage::PlayCards { cards }) => {
                                if let (Some(player_id), Some(room_id)) = (ctx.player_id, ctx.room_id) {
                                    if let Some(play_res) = state.room_manager.play_room_cards(room_id, player_id, &cards).await {
                                        match play_res {
                                            Ok(game_ended) => {
                                                if let Some(room) = state.room_manager.get_room(&room_id).await {
                                                    if let Some(game_state) = &room.game_state {
                                                        if game_ended {
                                                            let end_msg = ServerMessage::GameEnded {
                                                                winner_id: player_id,
                                                                reason: "Đã đánh hết bài".to_string(),
                                                                hands: game_state.hands.clone(),
                                                            };
                                                            RoomManager::broadcast_room(&state.session_manager, &room.players, &end_msg);
                                                        } else {
                                                            let mut card_counts = std::collections::HashMap::new();
                                                            for (&p_id, hand) in &game_state.hands {
                                                                card_counts.insert(p_id, hand.len());
                                                            }
                                                            let turn_msg = ServerMessage::TurnUpdated {
                                                                active_player_id: game_state.active_player,
                                                                last_played_cards: cards,
                                                                last_played_by: Some(player_id),
                                                                player_card_counts: card_counts,
                                                                passed_players: game_state.passed_players.clone(),
                                                            };
                                                            RoomManager::broadcast_room(&state.session_manager, &room.players, &turn_msg);
                                                        }
                                                    }
                                                }
                                            }
                                            Err(err_msg) => {
                                                let err_msg_struct = ServerMessage::Error { message: err_msg };
                                                let json = serde_json::to_string(&err_msg_struct).unwrap();
                                                state.session_manager.send_to(player_id, Message::Text(json.into()));
                                            }
                                        }
                                    }
                                }
                                ServerMessage::Error { message: "PlayCards processed".into() }
                            }
                            Ok(ClientMessage::PassTurn) => {
                                if let (Some(player_id), Some(room_id)) = (ctx.player_id, ctx.room_id) {
                                    if let Some(pass_res) = state.room_manager.pass_room_turn(room_id, player_id).await {
                                        match pass_res {
                                            Ok(_) => {
                                                if let Some(room) = state.room_manager.get_room(&room_id).await {
                                                    if let Some(game_state) = &room.game_state {
                                                        let mut card_counts = std::collections::HashMap::new();
                                                        for (&p_id, hand) in &game_state.hands {
                                                            card_counts.insert(p_id, hand.len());
                                                        }
                                                        let turn_msg = ServerMessage::TurnUpdated {
                                                            active_player_id: game_state.active_player,
                                                            last_played_cards: game_state.last_played_cards.clone(),
                                                            last_played_by: game_state.last_played_by,
                                                            player_card_counts: card_counts,
                                                            passed_players: game_state.passed_players.clone(),
                                                        };
                                                        RoomManager::broadcast_room(&state.session_manager, &room.players, &turn_msg);
                                                    }
                                                }
                                            }
                                            Err(err_msg) => {
                                                let err_msg_struct = ServerMessage::Error { message: err_msg };
                                                let json = serde_json::to_string(&err_msg_struct).unwrap();
                                                state.session_manager.send_to(player_id, Message::Text(json.into()));
                                            }
                                        }
                                    }
                                }
                                ServerMessage::Error { message: "PassTurn processed".into() }
                            }
                            Ok(ClientMessage::AnnounceSam) => {
                                if let (Some(player_id), Some(room_id)) = (ctx.player_id, ctx.room_id) {
                                    if let Some(sam_res) = state.room_manager.announce_room_sam(room_id, player_id).await {
                                        match sam_res {
                                            Ok(_) => {
                                                if let Some(room) = state.room_manager.get_room(&room_id).await {
                                                    if let Some(game_state) = &room.game_state {
                                                        let sam_msg = ServerMessage::SamAnnounced { player_id };
                                                        RoomManager::broadcast_room(&state.session_manager, &room.players, &sam_msg);

                                                        let mut card_counts = std::collections::HashMap::new();
                                                        for (&p_id, hand) in &game_state.hands {
                                                            card_counts.insert(p_id, hand.len());
                                                        }
                                                        let turn_msg = ServerMessage::TurnUpdated {
                                                            active_player_id: game_state.active_player,
                                                            last_played_cards: game_state.last_played_cards.clone(),
                                                            last_played_by: game_state.last_played_by,
                                                            player_card_counts: card_counts,
                                                            passed_players: game_state.passed_players.clone(),
                                                        };
                                                        RoomManager::broadcast_room(&state.session_manager, &room.players, &turn_msg);
                                                    }
                                                }
                                            }
                                            Err(err_msg) => {
                                                let err_msg_struct = ServerMessage::Error { message: err_msg };
                                                let json = serde_json::to_string(&err_msg_struct).unwrap();
                                                state.session_manager.send_to(player_id, Message::Text(json.into()));
                                            }
                                        }
                                    }
                                }
                                ServerMessage::Error { message: "AnnounceSam processed".into() }
                            }
                            Err(e) => ServerMessage::Error {
                                message: format!("Invalid message: {}", e),
                            },
                        };

                        // send response
                        let json = serde_json::to_string(&response).unwrap();
                        if let Err(e) = tx.send(Message::Text(json.into())) {
                            println!("Send error {}: {}", addr, e);
                            break;
                        }
                    }

                    Ok(Message::Close(frame_opt)) => {
                        if let Some(frame) = frame_opt {
                            match frame.code {
                                CloseCode::Abnormal => {}
                                _ => {}
                            }
                        }
                        println!("Client {:?} closed.", ctx.player_id.unwrap_or(0));
                        if let Some(uid) = ctx.player_id {
                            handle_player_disconnect(&state, uid, ctx.room_id).await;
                        }
                        break;
                    }
                    Ok(_) => {
                        println!("None Data");
                    }
                    Err(e) => match e {
                        Error::Protocol(_) => {
                            println!(
                                "Client {:?} disconnected unexpectedly",
                                ctx.player_id.unwrap_or(0)
                            );
                            if let Some(uid) = ctx.player_id {
                                handle_player_disconnect(&state, uid, ctx.room_id).await;
                            }
                            break;
                        }

                        e => {
                            println!("Error: {}", e);
                            if let Some(uid) = ctx.player_id {
                                handle_player_disconnect(&state, uid, ctx.room_id).await;
                            }
                            break;
                        }
                    },
                }
            }
        }
        Err(e) => {
            println!("Handshake failed: {:?}", e);
        }
    }

    println!("WebSocket connection closed: {}", addr);
    Ok(())
}

async fn handle_player_disconnect(
    state: &Arc<AppState>,
    player_id: u64,
    room_id: Option<u32>,
) {
    if let Some(rid) = room_id {
        state.room_manager.remove_player_from_room(rid, player_id).await;
        
        // Broadcast RoomList to lobby
        let rooms = state.room_manager.get_rooms().await;
        let list_msg = ServerMessage::RoomList { rooms };
        let json = serde_json::to_string(&list_msg).unwrap();
        state.session_manager.broadcast_all(Message::Text(json.into()));

        // Broadcast RoomInfo to remaining players
        if let Some(room) = state.room_manager.get_room(&rid).await {
            let info_msg = ServerMessage::RoomInfo { room: room.clone() };
            RoomManager::broadcast_room(&state.session_manager, &room.players, &info_msg);
        }
    }
    state.session_manager.remove_session(player_id);
}
