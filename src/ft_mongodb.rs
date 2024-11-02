use std::error::Error;

use log::{debug, error, info};
use mongodb::{
    bson::{doc, Bson, Document},
    options::ClientOptions,
    Client, Collection,
};

pub async fn connect_to_mongodb(mongodb_uri: &str) -> Client {
    info!("Connecting to MongoDB at {}", mongodb_uri);
    let client_options = ClientOptions::parse(mongodb_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    client
        .database("42")
        .run_command(doc! { "ping": 1 })
        .await
        .unwrap_or_else(|e| {
            error!("Failed to connect to MongoDB: {}", e);
            panic!();
        });
    info!("Successfully connected to MongoDB.");
    client
}

pub async fn fetch_current_index(client: &Client, nb_fetch: u32) -> Result<u32, Box<dyn Error>> {
    info!("Fetching current index from MongoDB.");
    let collection: Collection<Document> = client.database("application").collection("index");
    let found_doc = collection.find_one(doc! { "_id": 1 }).await?;
    let current_index = obtain_index(found_doc);
    increment_index_in_mongo(collection, current_index, nb_fetch).await?;
    info!("Current index is {}", current_index);
    Ok(current_index)
}

pub async fn insert_profile_in_mongo(
    client: &Client,
    profile_node: &serde_json::Value,
    user_id: u32,
) -> Result<(), Box<dyn Error>> {
    debug!("Inserting profile in MongoDB.");
    let collection: Collection<Document> = client.database("42").collection("profiles");
    let bson_value = mongodb::bson::to_bson(&profile_node)?;
    if let Bson::Document(mut doc) = bson_value {
        if let Some(id_value) = doc.get("id").cloned() {
            doc.insert("_id", user_id);
            let filter = doc! { "_id": user_id };
            collection
                .replace_one(filter, doc)
                .upsert(true)
                .await
                .or_else(|e| {
                    error!("Failed to insert profile in MongoDB: {}", e);
                    Err(e)
                })?;
            info!("Inserted/Updated document with _id: {:?}", id_value);
        } else {
            error!("Profil missing 'id' field: {:?}", doc);
        }
    } else {
        error!("Expected a document but got a different BSON type.");
    }
    debug!("Profile inserted in MongoDB.");
    Ok(())
}

pub async fn insert_location_in_mongo(
    client: &Client,
    location_node: &serde_json::Value,
    user_id: u32,
) -> Result<(), Box<dyn Error>> {
    debug!("Inserting location in MongoDB.");
    let collection: Collection<Document> = client.database("42").collection("locations");
    let bson_value = mongodb::bson::to_bson(&location_node)?;
    if let Bson::Array(ar) = bson_value {
        let filter = doc! { "_id": user_id };
        collection
            .replace_one(
                filter,
                doc! {
                    "_id": user_id,
                    "location": ar
                },
            )
            .upsert(true)
            .await?;
        debug!("Inserted/Updated document with _id: {:?}", user_id);
    } else {
        error!("Expected a array but got a different BSON type.");
    }
    debug!("location inserted in MongoDB.");
    Ok(())
}

pub async fn insert_ignoring_id_in_mongo(
    client: &Client,
    index: u32,
) -> Result<(), Box<dyn Error>> {
    debug!("Inserting in MongoDB ignoring id.");
    let collection: Collection<Document> = client.database("42").collection("ignoring_id");
    let bson_value = mongodb::bson::to_bson(&index)?;
    collection.insert_one(doc! {"_id": bson_value}).await?;
    debug!("Inserted in MongoDB ignoring id.");
    Ok(())
}

pub async fn insert_failed_id_in_mongo(client: &Client, index: u32) -> Result<(), Box<dyn Error>> {
    debug!("Inserting in MongoDB failed id.");
    let collection: Collection<Document> = client.database("42").collection("failed_id");
    let bson_value = mongodb::bson::to_bson(&index)?;
    collection.insert_one(doc! {"_id": bson_value}).await?;
    debug!("Inserted in MongoDB failed id.");
    Ok(())
}

async fn increment_index_in_mongo(
    collection: Collection<Document>,
    current_index: u32,
    nb_fetch: u32,
) -> Result<(), Box<dyn Error>> {
    collection
        .update_one(
            doc! {
                "_id": 1
            },
            doc! {
                "$set": {
                    "current_index": current_index + nb_fetch
                }
            },
        )
        .upsert(true)
        .await?;
    Ok(())
}

fn obtain_index(found_doc: Option<Document>) -> u32 {
    let current_index: u32 = match found_doc {
        Some(doc) => {
            if let Some(index) = doc.get("current_index").and_then(|v| v.as_i32()) {
                index as u32
            } else {
                28
            }
        }
        None => 28,
    };
    current_index
}
