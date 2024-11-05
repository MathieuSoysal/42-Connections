use log::error;
use log::info;
use mongodb::Client;
use oauth2::AccessToken;

use crate::ft_api;
use crate::ft_mongodb_locations;
use crate::ft_mongodb_locations::insert_user_id_and_page_number;

pub async fn fetch_location_from_42_to_mongo(
    client: &Client,
    token: &AccessToken,
) -> Result<(), Box<dyn std::error::Error>> {
    let (user_id, page_number) =
        ft_mongodb_locations::get_an_user_id_and_page_number(client).await?;
    let location_node = ft_api::request_location(&token, &user_id, &page_number).await;
    if location_node.is_err() {
        error!(
            "Location failed for user_id: {} page_number : {}",
            user_id, page_number
        );
        insert_user_id_and_page_number(client, user_id, page_number).await?;
        return Ok(());
    }
    let location_node = location_node?;
    if location_node.as_array().is_none() {
        info!("All locations is get for user_id: {}", user_id);
        return Ok(());
    }
    ft_mongodb_locations::insert_user_locations_in_mongodb(client, user_id, &location_node).await?;
    insert_user_id_and_page_number(client, user_id, page_number + 1).await?;
    info!("Locations inserted in MongoDB for user_id: {}", user_id);
    Ok(())
}
