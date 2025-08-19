use common::Secrets;

use chrono::Utc;
use jsonwebtoken::{
    Algorithm,
    decode, 
    DecodingKey,
    errors::Error,  
    TokenData, 
    Validation
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    pub sub: String,
    pub username: String,
    #[serde(default, skip_serializing_if = "Option::is_none")] // Only needs for websocket
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] // Only needs for websocket
    pub permissionlevel: Option<String>,
    pub exp: usize,
}

pub fn verify_jwt(token: &str) -> Result<TokenData<Claim>, Error> {
    let secrets = Secrets::global();
    let secret_key = secrets.server.common.secret_key.as_bytes();

    decode::<Claim>(
        token,
        &DecodingKey::from_secret(secret_key),
        &Validation::new(Algorithm::HS256),
    )
}

pub fn create_jwt(sub: String, username: String, role: Option<String>, permissionlevel: Option<String>, time: usize) -> Result<String, Error> {
    let secrets = Secrets::global();
    let secret_key = secrets.server.common.secret_key.as_bytes();

    let now = Utc::now().timestamp() as usize;
    let exp = now + time; // Add the required time for the expiration token

    let claim = Claim {
        sub,
        username,
        role,
        permissionlevel,
        exp,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claim,
        &jsonwebtoken::EncodingKey::from_secret(secret_key),
    )
}