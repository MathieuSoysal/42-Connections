use log::{debug, error};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AccessToken, AuthUrl, ClientId, ClientSecret,
    TokenResponse, TokenUrl,
};
use reqwest::header::AUTHORIZATION;
use std::error::Error;

const TOKEN_URL: &str = "https://api.intra.42.fr/oauth/token";

pub async fn request_profil(
    token: &AccessToken,
    user_id: &u32,
) -> Result<serde_json::Value, Box<dyn Error>> {
    debug!("Requesting profil from API for user_id: {}", user_id);
    let url = format!("https://api.intra.42.fr/v2/users/{}", user_id);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", token.secret()))
        .send()
        .await
        .map_err(|e| {
            error!("HTTP request failed for user_id {}: {}", user_id, e);
            e
        })?;
    if response.status() != 404 && response.status() != 200 {
        response.error_for_status_ref().map_err(|e| {
            error!(
                "Received error status from API for user_id {}: {}",
                user_id, e
            );
            e
        })?;
    }
    let response_json: serde_json::Value = response.json().await.map_err(|e| {
        error!("Failed to parse JSON for user_id {}: {}", user_id, e);
        e
    })?;
    debug!("Received profil from API for user_id: {}", user_id);
    Ok(response_json)
}

pub async fn request_location(
    token: &AccessToken,
    user_id: &u32,
    page_number: &u32,
) -> Result<serde_json::Value, Box<dyn Error>> {
    debug!("Requesting location from API for user_id: {}", user_id);
    let url = format!(
        "https://api.intra.42.fr/v2/users/{}/locations?page[size]=100&page[number]={}",
        user_id, page_number
    );
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", token.secret()))
        .send()
        .await
        .map_err(|e| {
            error!("HTTP request failed for user_id {}: {}", user_id, e);
            e
        })?;
    if response.status() != 404 && response.status() != 200 {
        response.error_for_status_ref().map_err(|e| {
            error!(
                "Received error status from API for user_id {}: {}",
                user_id, e
            );
            e
        })?;
    }
    let response_json: serde_json::Value = response.json().await.map_err(|e| {
        error!("Failed to parse JSON for user_id {}: {}", user_id, e);
        e
    })?;
    debug!("Received location from API for user_id: {}", user_id);
    Ok(response_json)
}

pub async fn generate_access_token(
    secret_uid: &str,
    secret_key: &str,
) -> Result<AccessToken, Box<dyn Error>> {
    let client = BasicClient::new(
        ClientId::new(secret_uid.to_string()),
        Some(ClientSecret::new(secret_key.to_string())),
        AuthUrl::new(TOKEN_URL.to_string())?,
        Some(TokenUrl::new(TOKEN_URL.to_string())?),
    );
    let token_result = client
        .exchange_client_credentials()
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            error!("Failed to obtain access token: {}", e);
            e
        })?;
    Ok(token_result.access_token().clone())
}
