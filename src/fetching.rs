use log::{debug, info, warn};
use mongodb::Client;
use oauth2::AccessToken;

use crate::{
    ft_api,
    ft_mongodb::{self, insert_ignoring_id_in_mongo},
};

pub async fn fetch_profil_from_42_to_mongo(
    client: &Client,
    user_id: u32,
    token: &AccessToken,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Fetching profile from 42 API for user_id: {}", user_id);
    let profile_node = ft_api::request_profil(&token, &user_id).await?;
    if profile_node.get("id").is_none() {
        warn!("Profile not found for user_id: {}", user_id);
        return Ok(());
    }
    ft_mongodb::insert_profile_in_mongo(client, &profile_node).await?;
    info!("Profile inserted in MongoDB for user_id: {}", user_id);
    Ok(())
}

pub async fn fetch_location_from_42_to_mongo(
    client: &Client,
    user_id: u32,
    token: &AccessToken,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Fetching location from 42 API for user_id: {}", user_id);
    let location_node = ft_api::request_location(&token, &user_id).await?;
    if location_node.as_array().is_none() {
        warn!("Location not found for user_id: {}", user_id);
        insert_ignoring_id_in_mongo(client, user_id).await?;
        return Ok(());
    }
    ft_mongodb::insert_location_in_mongo(client, user_id, &location_node).await?;
    info!("Location inserted in MongoDB for user_id: {}", user_id);
    Ok(())
}
