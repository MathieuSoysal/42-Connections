
use log::{info, warn};
use mongodb::Client;
use oauth2::AccessToken;

use crate::{ft_api, ft_mongodb_app_new_profile_index};

pub const TIME_BETWEEN_REQUESTS: u32 = 3;

pub async fn fetch_profils_ids_from_42_to_mongo(
    client: &Client,
    token: &AccessToken,
    page_number: &i32,
) -> Result<i32, Box<dyn std::error::Error>> {
    info!("Fetching profiles IDs from 42 API");
    let updated_at = "2024-11-06T12:04:39.139Z";
    let profiles_ids = ft_api::request_profiles_ids(&token, page_number, updated_at).await;
    if profiles_ids.is_err() {
        warn!("Failed to fetch profiles IDs");
        return Err(profiles_ids.err().unwrap());
    }
    let profiles_ids = profiles_ids?;
    if profiles_ids.as_array().unwrap().is_empty() {
        info!("No new profiles IDs to fetch");
        return Ok(0);
    }
    let mut user_ids = Vec::new();
    for profile_id in profiles_ids.as_array().unwrap() {
        let user_id = profile_id.get("id").unwrap().as_u64().unwrap();
        user_ids.push(user_id);
    }
    let inserted_count =
        ft_mongodb_app_new_profile_index::insert_profile_ids_in_mongo(client, user_ids).await?;
    info!("Profiles IDs inserted in MongoDB");
    Ok(inserted_count as i32)
}
