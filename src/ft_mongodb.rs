use log::{error, info};
use mongodb::{bson::doc, options::ClientOptions, Client};

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
