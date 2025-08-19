use server::{
    launch_database, launch_websocket_server
};
use common::{Config, Secrets};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::global();
    let secrets = Secrets::global();

    let websocket_runner = tokio::spawn(async move {
        let websocket_address = format!("{}:{}", config.server.websocket.address, config.server.websocket.port);
        
        launch_websocket_server(websocket_address).await
            .map_err(|e| format!("WebSocket server error: {}", e))
    });

    let database_runner = tokio::spawn(async move {
        let database_address = format!("{}:{}", config.server.database.address, config.server.database.port);
        
        launch_database(database_address, secrets.server.database.url.clone()).await
            .map_err(|e| format!("Database error: {}", e))
    });

    tokio::select! {
        result = websocket_runner => match result {
            Ok(Ok(())) => {},
            Ok(Err(e)) => eprintln!("Error launching WebSocket server: {}", e),
            Err(e) => eprintln!("WebSocket task failed: {}", e),
        },
        result = database_runner => match result {
            Ok(Ok(())) => {},
            Ok(Err(e)) => eprintln!("Error launching Database: {}", e),
            Err(e) => eprintln!("Database task failed: {}", e),
        }
    }

    Ok(())
}
