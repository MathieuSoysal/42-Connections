use std::time::Duration;

use log::{info, warn};
use mongodb::Client;
use oauth2::AccessToken;
use tokio::time::{sleep_until, Instant};

use crate::{
    fetching_locations, ft_api,
    ft_mongodb_profiles::{self, insert_failed_id_in_mongo, insert_ignoring_id_in_mongo},
    NB_FETCH,
};

pub const TIME_BETWEEN_REQUESTS: u32 = 4;

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
    ft_mongodb_profiles::insert_profile_in_mongo(client, &profile_node, user_id).await?;
    info!("Profile inserted in MongoDB for user_id: {}", user_id);
    Ok(())
}

pub async fn fetch_locations_from_42_to_mongo(
    client: &Client,
    token_1: &AccessToken,
    token_2: &AccessToken,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("All users have been fetched.");
    for _i in 0..NB_FETCH {
        let current = Instant::now();
        futures::future::try_join(
            fetching_locations::fetch_location_from_42_to_mongo(&client, &token_1),
            fetching_locations::fetch_location_from_42_to_mongo(&client, &token_2),
        )
        .await?;
        sleep_until(current + Duration::from_secs(TIME_BETWEEN_REQUESTS.into())).await;
    }
    Ok(())
}

pub async fn fetch_profiles_from_42_to_mongodb(
    client: &Client,
    user_id: u32,
    api_key_1: &AccessToken,
    api_key_2: &AccessToken,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut i = user_id;
    while i < user_id + (NB_FETCH * 2) {
        let current = Instant::now();
        futures::future::try_join(
            fetch_profil_from_42_to_mongo(&client, i.clone(), &api_key_2),
            fetch_profil_from_42_to_mongo(&client, i.clone() + 1, &api_key_1),
        )
        .await?;
        i += 2;
        sleep_until(current + Duration::from_secs(TIME_BETWEEN_REQUESTS.into())).await;
    }
    Ok(())
}
