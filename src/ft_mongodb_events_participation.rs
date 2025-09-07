use std::error::Error;

use log::{error, info};
use mongodb::{
    Client,
    bson::{Document, doc},
};

const DATABASE_NAME: &str = "42";
const COLLECTION_NAME: &str = "event_participations";

pub async fn insert_user_events_in_mongodb(
    client: &Client,
    user_id: i64,
    events_node: &serde_json::Value,
) -> Result<usize, Box<dyn Error>> {
    info!("Inserting events in MongoDB for user {}.", user_id);
    let events = map_events_to_bson_documents(events_node, user_id);
    let nb_events = events.len();
    insert_user_events_into_mongodb(client, user_id, events).await;
    Ok(nb_events)
}

fn convert_json_event_to_bson(event_node: &serde_json::Value, user_id: i64) -> Document {
    let bson_value = mongodb::bson::to_bson(event_node).unwrap();
    if let mongodb::bson::Bson::Document(doc) = bson_value {
        return doc! {
            "event_id": doc.get_i64("id").unwrap(),
            "user_id": user_id,
        };
    } else {
        error!("Expected a document but got a different BSON type.");
        panic!("Expected a document but got a different BSON type.");
    }
}

async fn insert_user_events_into_mongodb(client: &Client, user_id: i64, events: Vec<Document>) {
    if events.is_empty() {
        error!("No events to insert in MongoDB.");
        return;
    }
    let colletion = client
        .database(DATABASE_NAME)
        .collection::<Document>(COLLECTION_NAME);
    let query = doc! {"_id": user_id};
    let update = doc! {
        "$push": {
            "events": {
                "$each": events
            }
        }
    };
    colletion
        .update_one(query, update)
        .upsert(true)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to insert events in MongoDB: {}", e);
            panic!("Failed to insert events in MongoDB: {}", e);
        });
}

fn map_events_to_bson_documents(locations_node: &serde_json::Value, user_id: i64) -> Vec<Document> {
    match locations_node.as_array() {
        Some(array) => array
            .iter()
            .map(|location_node| convert_json_event_to_bson(location_node, user_id))
            .collect(),
        None => {
            error!("Expected an array but got a different JSON type.");
            vec![]
        }
    }
}
