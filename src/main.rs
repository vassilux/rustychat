use clap::Parser;
use std::sync::Arc;
use tokio::{
    io::{self, AsyncBufReadExt},
    sync::mpsc,
};

mod client;
mod manager;
mod server;

use crate::server::ChatServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<String>();

    // Initialiser le serveur de chat
    let server = Arc::new(ChatServer::new("localhost:1661").await);

    // Démarrer le serveur dans une tâche distincte
    {
        let server_arc = Arc::clone(&server);
        tokio::spawn(async move {
            server_arc.run().await;
        });
    }

    // Tâche pour gérer les commandes
    {
        let server_arc = Arc::clone(&server);
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                handle_command(Arc::clone(&server_arc), cmd).await;
            }
        });
    }

    // Continuous input loop
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        if line.trim() == "/exit" {
            break;
        }
        cmd_tx.send(line.to_string())?;
    }

    Ok(())
}

async fn handle_command(server: Arc<ChatServer>, cmd: String) {
    match cmd.as_str() {
        "/exit" => {
            println!("Exiting...");
            std::process::exit(0);
        }
        _ => {
            server.broadcast_message(cmd).await;
        }
    }
}
