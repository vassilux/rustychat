use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    // Initialiser le serveur de chat
    let mut server = ChatServer::new("localhost:1661").await;
    server.run().await;
}

// Structure représentant un client
struct Client {
    socket: TcpStream,
    address: SocketAddr,
    tx: broadcast::Sender<(String, SocketAddr)>,
    rx: broadcast::Receiver<(String, SocketAddr)>,
}

impl Client {
    async fn new(
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

    async fn handle_client(mut self) {
        let (reader, mut writer) = self.socket.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            tokio::select! {
                result = reader.read_line(&mut line) => {
                    if result.unwrap() == 0 {
                        break;
                    }

                    self.tx.send((line.clone(), self.address)).unwrap();
                    line.clear();
                },
                result = self.rx.recv() => {
                    let (msg, other_addr) = result.unwrap();
                    if self.address != other_addr {
                        writer.write_all(msg.as_bytes()).await.unwrap();
                    }
                }
            }
        }
    }
}

// Structure représentant le serveur de chat
struct ChatServer {
    listener: TcpListener,
    tx: broadcast::Sender<(String, SocketAddr)>,
}

impl ChatServer {
    async fn new(addr: &str) -> Self {
        let listener = TcpListener::bind(addr).await.unwrap();
        let (tx, _rx) = broadcast::channel(10);
        ChatServer { listener, tx }
    }

    async fn run(&mut self) {
        loop {
            let (socket, addr) = self.listener.accept().await.unwrap();
            let client = Client::new(socket, addr, self.tx.clone()).await;
            tokio::spawn(async move {
                client.handle_client().await;
            });
        }
    }
}
