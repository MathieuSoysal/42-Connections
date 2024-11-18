use log::error;
use log::info;
use mongodb::Client;
use oauth2::AccessToken;

use crate::ft_api;
use crate::ft_mongodb_app_indexor::{
    get_an_user_id_and_page_number, insert_user_id_and_page_number,
};
use crate::ft_mongodb_locations;

// {{url}}/v2/users/165216/events?page[size]=100&page[number]=1

const COLLECTION_NAME: &str = "events_participation_index";

pub async fn double_fetch_events_participation_from_42_to_mongo(
    client: &Client,
    token1: &AccessToken,
    token2: &AccessToken,
) -> Result<(), Box<dyn std::error::Error>> {
    futures::future::try_join(
        fetch_events_participation_from_42_to_mongo(client, token1),
        fetch_events_participation_from_42_to_mongo(client, token2),
    )
    .await?;
    Ok(())
}

pub async fn fetch_events_participation_from_42_to_mongo(
    client: &Client,
    token: &AccessToken,
) -> Result<(), Box<dyn std::error::Error>> {
    let (user_id, page_number) = get_an_user_id_and_page_number(client, COLLECTION_NAME).await?;
    let location_node = ft_api::request_location(&token, &user_id, &page_number).await;
    if location_node.is_err() {
        error!(
            "Location failed for user_id: {} page_number : {}",
            user_id, page_number
        );
        insert_user_id_and_page_number(client, user_id, page_number, COLLECTION_NAME).await?;
        return Ok(());
    }
    let location_node = location_node?;
    if location_node.as_array().is_none() || location_node.as_array().unwrap().is_empty() {
        info!("All locations is get for user_id: {}", user_id);
        return Ok(());
    }
    let nb_insert =
        ft_mongodb_locations::insert_user_locations_in_mongodb(client, user_id, &location_node)
            .await?;
    info!(
        "{} Locations inserted in MongoDB for user_id: {}",
        nb_insert, user_id
    );
    if nb_insert < 100 {
        info!(
            "All locations is get for user_id: {} location index was not reinserted",
            user_id
        );
        return Ok(());
    }
    insert_user_id_and_page_number(client, user_id, page_number + 1, COLLECTION_NAME).await?;
    Ok(())
}
