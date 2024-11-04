use futures::future;
use log::{debug, info, warn};
use mongodb::Client;
use oauth2::AccessToken;
use tokio::time::sleep_until;
use tokio::time::Instant;

use crate::{
    ft_api,
    ft_mongodb::{self, insert_failed_id_in_mongo, insert_ignoring_id_in_mongo},
};

pub const TIME_BETWEEN_REQUESTS: u32 = 4;

pub async fn fetching_data_from_42_to_mongo(
    client: &Client,
    user_id: u32,
    token_profil: &AccessToken,
    token_location: &AccessToken,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Fetching data from 42 API for user_id: {}", user_id);
    let inst = Instant::now();
    let fut1 = fetch_profil_from_42_to_mongo(client, user_id, token_profil);
    let fut2 = fetch_location_from_42_to_mongo(client, user_id, token_location);
    future::try_join(fut1, fut2).await?;
    sleep_until(inst + std::time::Duration::from_secs(TIME_BETWEEN_REQUESTS.into())).await;
    info!("Data fetched from 42 API for user_id: {}", user_id);
    Ok(())
}

pub async fn fetch_profil_from_42_to_mongo(
    client: &Client,
    user_id: u32,
    token: &AccessToken,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Fetching profile from 42 API for user_id: {}", user_id);
    let profile_node = ft_api::request_profil(&token, &user_id).await;
    if profile_node.is_err() {
        warn!("Profil failed for user_id: {}", user_id);
        insert_failed_id_in_mongo(client, user_id).await?;
        return Ok(());
    }
    let profile_node = profile_node?;
    if profile_node.get("id").is_none() {
        warn!("Profil not found for user_id: {}", user_id);
        insert_ignoring_id_in_mongo(client, user_id).await?;
        return Ok(());
    }
    ft_mongodb::insert_profile_in_mongo(client, &profile_node, user_id).await?;
    info!("Profile inserted in MongoDB for user_id: {}", user_id);
    Ok(())
}

pub async fn fetch_location_from_42_to_mongo(
    client: &Client,
    user_id: u32,
    token: &AccessToken,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Fetching location from 42 API for user_id: {}", user_id);
    let location_node = ft_api::request_location(&token, &user_id).await;
    if location_node.is_err() {
        insert_failed_id_in_mongo(client, user_id).await?;
        return Ok(());
    }
    let location_node = location_node?;
    if location_node.as_array().is_none() {
        warn!("Location not found for user_id: {}", user_id);
        insert_ignoring_id_in_mongo(client, user_id).await?;
        return Ok(());
    }
    ft_mongodb::insert_location_in_mongo(client, &location_node, user_id).await?;
    info!("Location inserted in MongoDB for user_id: {}", user_id);
    Ok(())
}
