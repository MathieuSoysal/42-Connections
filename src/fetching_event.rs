use log::error;
use log::info;
use mongodb::Client;
use oauth2::AccessToken;

use crate::ft_api;
use crate::ft_mongodb_app_indexor::get_an_id;
use crate::ft_mongodb_app_indexor::insert_id;
use crate::ft_mongodb_events;

const COLLECTION_NAME: &str = "events_ids";

pub async fn double_fetch_event_from_42_to_mongo(
    client: &Client,
    token1: &AccessToken,
    token2: &AccessToken,
) -> Result<(), Box<dyn std::error::Error>> {
    futures::future::try_join(
        fetch_event_from_42_to_mongo(client, token1),
        fetch_event_from_42_to_mongo(client, token2),
    )
    .await?;
    Ok(())
}

pub async fn fetch_event_from_42_to_mongo(
    client: &Client,
    token: &AccessToken,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_id = get_an_id(client, COLLECTION_NAME).await?;
    let event_node = ft_api::request_event(&token, &event_id).await;
    if event_node.is_err() {
        error!("Event recuperation failed for event_id: {}", event_id);
        insert_id(client, event_id, COLLECTION_NAME).await?;
        return Ok(());
    }
    let event_node = event_node?;
    if event_node.is_null() {
        info!("All event is get for event_id: {}", event_id);
        return Ok(());
    }
    ft_mongodb_events::insert_event_in_mongodb(client, event_id, &event_node).await?;
    info!("Insertion succed in MongoDB for event: {}", event_id);
    Ok(())
}
