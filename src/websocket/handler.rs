use crate::websocket::structures::{ClientRole, Clients, Role};

use axum::{
    Extension,
    extract::ws::{
        Message,
        WebSocket,
        WebSocketUpgrade
    },
    response::IntoResponse,
};
use futures::{
    SinkExt,
    StreamExt
};
use std::sync::{Arc, Mutex};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedSender},
    select,
    task,
};
use uuid::Uuid;

static SENDER: Mutex<Option<UnboundedSender<Message>>> = Mutex::new(None);

pub async fn websocket_handler(ws: WebSocketUpgrade, Extension(clients): Extension<Clients>, role: Role) -> impl IntoResponse {
    let role = role.0;
    ws.on_upgrade(move |socket| handle_websocket(socket, role, clients))
}

pub async fn handle_websocket(socket: WebSocket, role: ClientRole, clients: Clients) {
    let (tx, mut rx) = unbounded_channel::<Message>();

    let ping_tx = tx.clone();

    let client_id = Uuid::new_v4();

    let role_clone = role.clone();

    if role == ClientRole::Receiver {
        clients.lock().unwrap().insert(client_id, tx.clone());
    } else {
        if let Some(_) = SENDER.lock().unwrap().as_ref() {
            // We don't allow multiple senders
            // The only sender is our algorithm
            // So we can refuse the new one

            return;
        } else {
            SENDER.lock().unwrap().replace(tx.clone());
        }
    }

    let (mut ws_sender, mut ws_receiver) = socket.split();

    let receiving_clients = Arc::clone(&clients);

    let mut send_task = task::spawn(async move {
        while let Some(message) = rx.recv().await {
            if ws_sender.send(message.clone()).await.is_err() {
                break;
            }
        }
    });

    let mut receive_task = task::spawn(async move {
        while let Some(Ok(message)) = ws_receiver.next().await {
            match message {
                Message::Ping(payload) => {
                    if let Err(e) = tx.send(Message::Pong(payload)) {
                        eprintln!("Failed to send pong response: {:?}", e);
                        break;
                    }
                },
                Message::Text(text) => {
                    if role == ClientRole::Receiver {
                        // If the client is a receiver
                        // We kick him out to avoid overloading the server
                        break;
                    }
                    
                    send_message_to_all_clients(&receiving_clients, Message::Text(text)).await;
                },
                Message::Close(_) => {
                    break;
                },
                _ => {}
            }
        }
    });

    let mut ping_task = task::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            if ping_tx.send(Message::Ping(Vec::new().into())).is_err() {
                break;
            }
        }

    });

    select! {
        _ = &mut send_task => {
            receive_task.abort();
            ping_task.abort();
        },
        _ = &mut receive_task => {
            send_task.abort();
            ping_task.abort();
        },
        _ = &mut ping_task => {
            send_task.abort();
            receive_task.abort();
        },
    }

    if role_clone == ClientRole::Sender {
        let mut sender_lock = SENDER.lock().unwrap();
        sender_lock.take();
    } else {
        clients.lock().unwrap().remove(&client_id);
    }
}

pub async fn send_message_to_all_clients(clients: &Clients, message: Message) {
    let connected_clients: Vec<UnboundedSender<Message>> = {
        let guard = clients.lock().unwrap();

        guard.values().cloned().collect()
    };

    for tx in connected_clients {
        if let Err(e) = tx.send(message.clone()) {
            eprintln!("Failed to send message to a client: {:?}", e);
        }
    }
}