use crate::websocket::{
    handler::websocket_handler,
    structures::Clients
};

use axum::{
    Extension,
    Router, 
    routing::get
};
use common::utils::log::{
    LogFile,
    LogLevel
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex}
};
use tokio::net::TcpListener;

pub async fn launch_websocket_server(address: String) -> Result<(), Box<dyn std::error::Error>> {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .layer(Extension(clients));

    let listener = TcpListener::bind(&address).await;

    if let Err(e) = listener {
        LogFile::add_log(LogLevel::Error, &format!("Failed to bind to {}: {}", address, e)).ok();

        return Err(Box::new(e));
    }

    let listener = listener.unwrap();

    if let Err(e) = axum::serve(listener, app).await {
        LogFile::add_log(LogLevel::Error, &format!("Failed to start server: {}", e)).ok();

        return Err(Box::new(e));
    }

    LogFile::add_log(LogLevel::Info, &format!("WebSocket server running on {}", address)).ok();

    Ok(())
}