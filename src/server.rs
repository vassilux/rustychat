use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{net::TcpListener, signal, sync::broadcast};

use crate::client::Client;
use crate::manager::ClientManager;

pub struct ChatServer {
    listener: TcpListener,
    tx: broadcast::Sender<(String, SocketAddr)>,
    client_manager: Arc<ClientManager>,
}

impl ChatServer {
    pub async fn new(addr: &str) -> Self {
        let listener = TcpListener::bind(addr).await.unwrap();
        let (tx, _rx) = broadcast::channel(10);
        let client_manager = Arc::new(ClientManager::new());
        ChatServer {
            listener,
            tx,
            client_manager,
        }
    }

    pub async fn run(&self) {
        loop {
            tokio::select! {
                Ok((socket, addr)) = self.listener.accept() => {
                    let client_manager = Arc::clone(&self.client_manager);
                    client_manager.add_client(addr).await;

                    let client = Client::new(socket, addr, self.tx.clone()).await;
                    tokio::spawn(async move {
                        client.handle_client().await;
                        client_manager.remove_client(&addr).await;
                    });
                },
                _ = signal::ctrl_c() => {
                    println!("Signal de fermeture reçu, arrêt du serveur...");
                    break;
                },
            }
        }
    }

    pub async fn broadcast_message(&self, message: String) {
        self.client_manager
            .broadcast_message(self.tx.clone(), message)
            .await;
    }
}
