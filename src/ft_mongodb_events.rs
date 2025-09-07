use std::error::Error;

use log::{error, info};
use mongodb::{
    Client,
    bson::{Document, doc},
};

const DATABASE_NAME: &str = "42";
const COLLECTION_NAME: &str = "events";

pub async fn insert_event_in_mongodb(
    client: &Client,
    event_id: i64,
    event_node: &serde_json::Value,
) -> Result<usize, Box<dyn Error>> {
    info!("Inserting event in MongoDB for user {}.", event_id);
    let event = map_event_to_bson_document(event_node, event_id);
    insert_event_into_mongodb(client, event_id, event).await;
    Ok(event_id as usize)
}

async fn insert_event_into_mongodb(client: &Client, event_id: i64, event: Document) {
    if event.is_empty() {
        error!("No event to insert in MongoDB.");
        return;
    }
    let colletion = client
        .database(DATABASE_NAME)
        .collection::<Document>(COLLECTION_NAME);
    colletion
        .replace_one(doc! {"_id": event_id}, event)
        .upsert(true)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to insert event in MongoDB: {}", e);
            panic!("Failed to insert event in MongoDB: {}", e);
        });
}

fn map_event_to_bson_document(event_node: &serde_json::Value, event_id: i64) -> Document {
    let bson_value = mongodb::bson::to_bson(event_node).unwrap();
    if let mongodb::bson::Bson::Document(mut doc) = bson_value {
        doc.insert("_id", event_id);
        return doc;
    } else {
        error!("Expected a document but got a different BSON type.");
        panic!("Expected a document but got a different BSON type.");
    }
}
