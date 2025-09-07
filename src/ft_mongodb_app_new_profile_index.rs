use std::error::Error;

use log::{error, info};
use mongodb::{
    Client,
    bson::{Document, doc},
};

pub async fn insert_profile_ids_in_mongo(
    client: &Client,
    profiles_ids: Vec<u64>,
) -> Result<usize, Box<dyn Error>> {
    info!("Inserting profiles IDs in MongoDB.");
    // BSON has no u64, normalize to i64 as per project convention.
    let documents: Vec<Document> = profiles_ids
        .into_iter()
        .filter_map(|id| match i64::try_from(id) {
            Ok(v) => Some(doc! { "_id": v }),
            Err(_) => {
                error!("profile id {} exceeds i64 range; skipping", id);
                None
            }
        })
        .collect();
    let nb_profiles = documents.len();
    if nb_profiles == 0 {
        info!("No valid profile IDs to insert.");
        return Ok(0);
    }
    let collection = client
        .database("application")
        .collection::<Document>("profile_index_to_be_updated");
    collection
        .insert_many(documents)
        .ordered(false)
        .await
        .or_else(|e| {
            error!("Failed to insert profiles IDs in MongoDB: {}", e);
            Err(e)
        })?;
    info!("Inserted {} profiles IDs in MongoDB.", nb_profiles);
    Ok(nb_profiles)
}
