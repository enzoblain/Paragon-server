use axum::response::Json;
use serde_json::{json, Value};

pub async fn hello_rest() -> Json<Value> {
    Json(json!({"message": "Hello, REST!"}))
}