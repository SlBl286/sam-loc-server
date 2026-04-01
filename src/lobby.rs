use std::{net::SocketAddr, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    spawn,
    sync::{Mutex, mpsc},
};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message},
};

type Player = mpsc::UnboundedSender<Message>;

struct Lobby {
    players: Vec<Player>,
    lobby_name: String,
}

impl Lobby {
    fn new() -> Self {
        Lobby {
            players: Vec::new(),
            lobby_name: String::new(),
        }
    }
}
struct LobbyManager {
    lobbies: Arc<Mutex<std::collections::HashMap<String, Lobby>>>,
}

impl LobbyManager {
    fn new() -> Self {
        LobbyManager {
            lobbies: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    async fn get_or_create_lobby(&self, lobby_id: String) {
        let mut lobbies = self.lobbies.lock().await;
        if !lobbies.contains_key(&lobby_id) {
            lobbies.insert(lobby_id.clone(), Lobby::new());
        }
    }
    async fn add_player_from_lobby(&self, lobby_id: String, player: Player) {
        let mut lobbies = self.lobbies.lock().await;
        if let Some(lobby) = lobbies.get_mut(&lobby_id) {
            lobby.players.push(player);
        }
    }

    async fn remove_player_from_lobby(&self, lobby_id: String, player: &Player) {
        let mut lobbies = self.lobbies.lock().await;
        if let Some(lobby) = lobbies.get_mut(&lobby_id) {
            lobby
                .players
                .retain(|s| s as *const _ != player as *const _);
        }
    }

    async fn broadcast(&self, lobby_id: String, message: Message) {
        let mut lobbies = self.lobbies.lock().await;
        if let Some(lobby) = lobbies.get_mut(&lobby_id) {
            for player in lobby.players.iter() {
                player.send(message.clone()).expect("Faild to send message");
            }
        }
    }
}
async fn accept_connection(stream: TcpStream, addr: SocketAddr) {
    match handle_connection(stream, addr).await {
        Err(e) => match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8(_) => (),
            err => eprintln!("Error processing connection: {}", err),
        },
        _ => (),
    }
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr) -> Result<(), Error> {
    match accept_async(stream).await {
        Ok(ws_stream) => {
            println!("connected");
            let (mut write, mut read) = ws_stream.split();

            // spawn(async move {
            //     while let Some(msg) = rx.recv().await {
            //         write.send(msg).await.expect("Faild to send message");
            //     }
            // });

            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("recv: {}", text);
                       let _ = write.send(Message::text("")).await;
                    }

                    Ok(Message::Binary(_)) => {
                        println!("binary data");
                    }

                    Ok(Message::Close(frame)) => {
                        println!("Client {} closed: {:?}", addr, frame);
                        break;
                    }

                    Ok(Message::Ping(_)) => {
                        println!("Ping from {}", addr);
                    }

                    Ok(Message::Pong(_)) => {}
                    Ok(_) => {
                        println!("None Data");
                    }
                    Err(e) => match e {
                        Error::Protocol(e) => {
                            println!("Client {} disconnected unexpectedly: {}", addr, e);
                            break;
                        }

                        e => {
                            println!("Error: {}", e);

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

pub async fn start_ws_server(host: String) ->  Result<(), Error>  {
    let listener = TcpListener::bind(host.clone()).await.unwrap();

    println!("WebSocket server listen at ws://{}", host);

    let lobby_manager = Arc::new(LobbyManager::new());

    while let Ok((stream, addr)) = listener.accept().await {
        // let lobby_manager: Arc<LobbyManager> = lobby_manager.clone();
        println!("New WebSocket Connetion: {}", addr);

        spawn(accept_connection(stream, addr));
    }

    Ok(())
}
