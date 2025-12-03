use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

pub struct ApiKey(pub String);

impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let key = parts
            .headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        match key {
            Some(k) if k == "MY_SECRET_KEY" => Ok(ApiKey(k)),
            _ => Err((StatusCode::UNAUTHORIZED, "Invalid API key".into())),
        }
    }
}
