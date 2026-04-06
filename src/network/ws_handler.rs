use std::{net::SocketAddr, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, protocol::frame::coding::CloseCode},
};

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
            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    if let Err(e) = write.send(msg).await {
                        println!("Send error: {}", e);
                        break;
                    }
                }
            });
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // parse JSON
                        let parsed = serde_json::from_str::<ClientMessage>(&text);
                        println!("Parsed message: {:?}", parsed);
                        let response = match parsed {
                            Ok(ClientMessage::Connected { user_id }) => {
                                println!("user {} connected", user_id);
                                ctx.player_id = Some(user_id);
                                state
                                    .session_manager
                                    .add_session(Session::new(user_id, tx.clone()));
                                ServerMessage::RoomList {
                                    rooms: state.room_manager.get_rooms().await,
                                }
                            }
                            Ok(ClientMessage::Disconnected) => {
                                if let Some(uid) = ctx.player_id {
                                    if let Some(rid) = ctx.room_id {
                                        state.room_manager.remove_player_from_room(rid, uid).await;
                                    }
                                    state.session_manager.remove_session(uid);
                                    println!("user {} disconnected", uid);
                                }
                                ServerMessage::Error {
                                    message: "Disconnected".to_string(),
                                }
                            }

                            Ok(ClientMessage::CreateRoom {
                                room_name,
                                max_players,
                            }) => {
                                let room_id = state.id_generator.generate_id();
                                state
                                    .room_manager
                                    .create_room(room_id, room_name, max_players)
                                    .await;
                                let room_list_msg = ServerMessage::RoomList {
                                    rooms: state.room_manager.get_rooms().await,
                                };
                                let json = serde_json::to_string(&room_list_msg).unwrap();
                                state
                                    .session_manager
                                    .broadcast_all(Message::Text(json.into()));
                                if let Some(player_id) = ctx.player_id {
                                    let seat_index = state
                                        .room_manager
                                        .add_player_to_room(room_id, player_id)
                                        .await;
                                    ctx.room_id = Some(room_id);
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
                            Ok(_) => ServerMessage::Error {
                                message: "Unknown message type".to_string(),
                            },
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
                            if let Some(rid) = ctx.room_id {
                                state.room_manager.remove_player_from_room(rid, uid).await;
                            }
                            state.session_manager.remove_session(uid);
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
                                if let Some(rid) = ctx.room_id {
                                    state.room_manager.remove_player_from_room(rid, uid).await;
                                    let room = state.room_manager.get_room(&rid).await;
                                    if let Some(r) = room {
                                        println!("{}",r.get_num_players())
                                    }
                                }
                                state.session_manager.remove_session(uid);
                            }
                            break;
                        }

                        e => {
                            println!("Error: {}", e);
                            if let Some(uid) = ctx.player_id {
                                if let Some(rid) = ctx.room_id
                                {
                                    state.room_manager.remove_player_from_room(rid, uid).await;
                                }
                                state.session_manager.remove_session(uid);
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
