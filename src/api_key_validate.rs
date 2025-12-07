use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use dotenvy::dotenv;

use crate::{AppState, validate_api_key_hmac::validate_api_key_hmac};

pub struct ApiKey(pub String);

impl FromRequestParts<AppState> for ApiKey {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        dotenv().ok();
        let secret = std::env::var("HMAC_SECRET");
        let key = parts
            .headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());
        let api_key = key.clone().unwrap_or_default();
        let exist = state
            .redis_batcher
            .is_api_key_exist(&api_key)
            .await
            .unwrap();
        if exist {
            return Ok(ApiKey(api_key));
        }

        let valiadte = validate_api_key_hmac(api_key.as_str(), secret.unwrap().as_str());
        if !valiadte {
            return Err((StatusCode::UNAUTHORIZED, "Invalid API key".into()));
        }

        state
            .redis_batcher
            .add_validate_api_key(&api_key)
            .await
            .unwrap();

        Ok(ApiKey(api_key))
    }
}
