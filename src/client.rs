use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    signal,
    sync::broadcast,
};
pub struct Client {
    socket: TcpStream,
    address: SocketAddr,
    tx: broadcast::Sender<(String, SocketAddr)>,
    rx: broadcast::Receiver<(String, SocketAddr)>,
}

impl Client {
    pub async fn new(
        socket: TcpStream,
        address: SocketAddr,
        tx: broadcast::Sender<(String, SocketAddr)>,
    ) -> Self {
        let rx = tx.subscribe();
        Client {
            socket,
            address,
            tx,
            rx,
        }
    }

    pub async fn handle_client(mut self) {
        let (reader, mut writer) = self.socket.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            tokio::select! {
                result = reader.read_line(&mut line) => {
                    //utiliser match
                    match result {
                        Ok(0) => break,
                        Ok(_) => {
                            self.tx.send((line.clone(), self.address)).unwrap();
                            line.clear();
                        },
                        Err(_) => break,
                    }

                },
                result = self.rx.recv() => {
                    let (msg, other_addr) = result.unwrap();
                    if self.address != other_addr {
                        writer.write_all(msg.as_bytes()).await.unwrap();
                    }
                }
                _ = signal::ctrl_c() => {

                    println!("Signal de fermeture reçu, by client...");
                    break;
                },
            }
        }
    }
}
