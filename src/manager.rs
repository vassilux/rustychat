use std::net::SocketAddr;
use tokio::sync::{broadcast, Mutex};

pub struct ClientManager {
    clients: Mutex<Vec<SocketAddr>>,
}

impl ClientManager {
    pub fn new() -> Self {
        ClientManager {
            clients: Mutex::new(Vec::new()),
        }
    }

    pub async fn add_client(&self, addr: SocketAddr) {
        let mut clients = self.clients.lock().await;
        clients.push(addr);
    }

    pub async fn remove_client(&self, addr: &SocketAddr) {
        let mut clients = self.clients.lock().await;
        clients.retain(|&x| x != *addr);
    }

    pub async fn broadcast_message(
        &self,
        tx: broadcast::Sender<(String, SocketAddr)>,
        message: String,
    ) {
        let clients = self.clients.lock().await;
        for &client in clients.iter() {
            tx.send((message.clone(), client)).unwrap();
        }
    }
}
