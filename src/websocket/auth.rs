use crate::{ 
    utils::auth::{Claim, verify_jwt},
    websocket::structures::{ClientRole, Role}
};

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::errors::{
    Error, 
    ErrorKind
};

pub fn auth_access(claim: Claim) -> Result<ClientRole, Error> {
    if let Some(role) = claim.role {
        match role.as_str() {
            "receiver" => {
                // TODO: If user is in the db then send
                Ok(ClientRole::Receiver)
            }
            "sender" => {
                    Ok(ClientRole::Sender)
            },
            _ => Err(Error::from(ErrorKind::InvalidToken)),
        }
    } else {
        Err(Error::from(ErrorKind::InvalidToken))
    }
}

impl<S> FromRequestParts<S> for Role
where S: Send + Sync {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = parts.headers.get("authorization")
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header"))?;
        let header = header.to_str().map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid header"))?;

        let token = header.strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid Authorization header"))?;

        let token_data = verify_jwt(token)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

        let role = auth_access(token_data.claims)
            .map_err(|_| (StatusCode::FORBIDDEN, "Permission denied"))?;

        Ok(Role(role))
    }
}