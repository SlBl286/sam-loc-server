use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;


#[tokio::main]
async fn main() -> Result<(), IoError> {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();

        println!("Client connected: {}", addr);

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(_) => return,
                };

                println!("Received: {:?}", &buf[..n]);

                socket.write_all(&buf[..n]).await.unwrap();
            }
        });
    }

}


mod lobby;