use crate::{
    database::structures::{
        Permission,
        PermissionLevel
    },
    utils::auth::{
        Claim, 
        verify_jwt
    },
};

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::errors::{
    Error, 
    ErrorKind
};

pub fn auth_access(claim: Claim) -> Result<PermissionLevel, Error> {
    if let Some(permissionlevel) = claim.permissionlevel {
        match permissionlevel.as_str() {
            "user" => {
                // TODO: If user is in the db then send
                Ok(PermissionLevel::User)
            }
            "admin" => {
                Ok(PermissionLevel::Admin)
            },
            _ => Err(Error::from(ErrorKind::InvalidToken)),
        }
    } else {
        Err(Error::from(ErrorKind::InvalidToken))
    }
}

// Extractor to get the permission level from the request
impl<S> FromRequestParts<S> for Permission
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

        let permission = auth_access(token_data.claims)
            .map_err(|_| (StatusCode::FORBIDDEN, "Permission denied"))?;

        Ok(Permission(permission))
    }
}