use log::{debug, error};
use oauth2::{
    AccessToken, AuthUrl, ClientId, ClientSecret, TokenResponse, TokenUrl, basic::BasicClient,
    reqwest::async_http_client,
};
use reqwest::header::AUTHORIZATION;
use std::error::Error;

const TOKEN_URL: &str = "https://api.intra.42.fr/oauth/token";
const API_URL: &str = "https://api.intra.42.fr/v2";

pub async fn request_profil(
    token: &AccessToken,
    user_id: &u32,
) -> Result<serde_json::Value, Box<dyn Error>> {
    debug!("Requesting profil from API for user_id: {}", user_id);
    let url = format!("{}/users/{}", API_URL, user_id);
    let response = send_http_request(&url, token).await?;
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
    user_id: &i64,
    page_number: &i32,
) -> Result<serde_json::Value, Box<dyn Error>> {
    debug!("Requesting location from API for user_id: {}", user_id);
    let url = format!(
        "{}/users/{}/locations?page[size]=100&page[number]={}",
        API_URL, user_id, page_number
    );
    let response = send_http_request(&url, token).await?;
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

pub async fn request_event_participations(
    token: &AccessToken,
    user_id: &i64,
    page_number: &i32,
) -> Result<serde_json::Value, Box<dyn Error>> {
    debug!(
        "Requesting event participations from API for user_id: {}",
        user_id
    );
    let url = format!(
        "{}/users/{}/events?page[size]=100&page[number]={}",
        API_URL, user_id, page_number
    );
    let response = send_http_request(&url, token).await?;
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
    debug!("Received event from API for user_id: {}", user_id);
    Ok(response_json)
}

pub async fn request_event(
    token: &AccessToken,
    event_id: &i64,
) -> Result<serde_json::Value, Box<dyn Error>> {
    debug!("Requesting event from API for event_id: {}", event_id);
    let url = format!("{}/events/{}", API_URL, event_id);
    let response = send_http_request(&url, token).await?;
    if response.status() != 404 && response.status() != 200 {
        response.error_for_status_ref().map_err(|e| {
            error!(
                "Received error status from API for event_id {}: {}",
                event_id, e
            );
            e
        })?;
    }
    let response_json: serde_json::Value = response.json().await.map_err(|e| {
        error!("Failed to parse JSON for event_id {}: {}", event_id, e);
        e
    })?;
    debug!("Received event from API for event_id: {}", event_id);
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

async fn send_http_request(
    url: &str,
    token: &AccessToken,
) -> Result<reqwest::Response, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", token.secret()))
        .send()
        .await
        .map_err(|e| {
            error!("HTTP request failed for url {}: {}", url, e);
            e
        })?;
    Ok(response)
}
