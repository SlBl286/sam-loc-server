use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message},
};

use crate::player::{Session, session_manager::SessionManager};

pub async fn accept_connection(stream: TcpStream, addr: SocketAddr, session_manager: &SessionManager ) {
    match handle_connection(stream, addr,session_manager).await {
        Err(e) => match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8(_) => (),
            err => eprintln!("Error processing connection: {}", err),
        },
        _ => (),
    }
}

pub async fn handle_connection(stream: TcpStream, addr: SocketAddr, session_manager: &SessionManager ) -> Result<(), Error> {
    match accept_async(stream).await {
        Ok(ws_stream) => {
            let (mut write, mut read) = ws_stream.split();
            let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
            session_manager.add_session(Session::new(user_id, None, tx));
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("recv: {}", text.as_str());
                        let _ = write.send(Message::text("")).await;
                    }

                    Ok(Message::Close(frame)) => {
                        println!("Client {} closed: {:?}", addr, frame);
                        break;
                    }
                    Ok(_) => {
                        println!("None Data");
                    }
                    Err(e) => match e {
                        Error::Protocol(e) => {
                            println!("Client {} disconnected unexpectedly", addr);
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
