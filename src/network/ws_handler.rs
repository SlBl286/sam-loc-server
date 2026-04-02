use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::{Error, Message}};

pub async fn accept_connection(stream: TcpStream, addr: SocketAddr) {
    match handle_connection(stream, addr).await {
        Err(e) => match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8(_) => (),
            err => eprintln!("Error processing connection: {}", err),
        },
        _ => (),
    }
}

pub async fn handle_connection(stream: TcpStream, addr: SocketAddr) -> Result<(), Error> {
    match accept_async(stream).await {
        Ok(ws_stream) => {
            
            let (mut write, mut read) = ws_stream.split();

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
