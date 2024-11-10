use mongodb::{
    bson::{doc, Document},
    Client, Collection,
};

const DEFAULT_LAST_UPDATE: &str = "2020-01-01T00:00:00Z";

pub async fn get_last_update(client: &Client) -> String {
    let collection: Collection<Document> = client.database("application").collection("last_update");
    let last_update = collection.find_one(doc! {}).await;
    match last_update {
        Ok(Some(document)) => document
            .get_str("last_update")
            .map(|s| s.to_string())
            .unwrap_or(get_biggest_update_at_from_profiles(client).await),
        _ => get_biggest_update_at_from_profiles(client).await,
    }
}

pub async fn update_last_update(client: &Client) {
    let collection: Collection<Document> = client.database("application").collection("last_update");
    let last_update = get_biggest_update_at_from_profiles(client).await;
    collection
        .replace_one(
            doc! {"_id": 1},
            doc! { "_id": 1, "last_update": last_update },
        )
        .upsert(true)
        .await
        .unwrap();
}

async fn get_biggest_update_at_from_profiles(client: &Client) -> String {
    let collection: Collection<Document> = client.database("42").collection("profiles");
    let last_update = collection
        .find_one(doc! {})
        .sort(doc! { "updated_at": -1 })
        .projection(doc! { "updated_at": 1 })
        .await;
    if let Ok(Some(document)) = last_update {
        document
            .get_str("updated_at")
            .unwrap_or(DEFAULT_LAST_UPDATE)
            .to_string()
    } else {
        DEFAULT_LAST_UPDATE.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson::doc;
    use mongodb::{options::ClientOptions, Client};
    use std::error::Error;
    use testcontainers::{
        core::IntoContainerPort, runners::AsyncRunner, ContainerAsync, GenericImage,
    };

    pub async fn get_test_mongo_client() -> (Client, ContainerAsync<GenericImage>) {
        let container = match GenericImage::new("mongo", "latest")
            .with_exposed_port(27017.tcp())
            .start()
            .await
        {
            Ok(c) => c,
            Err(e) => {
                panic!("Failed to start MongoDB container: {}", e);
            }
        };

        // Get the host and port
        let port = match container.get_host_port_ipv4(27017).await {
            Ok(p) => p,
            Err(e) => {
                panic!("Failed to get MongoDB container port: {}", e);
            }
        };

        let client_uri = format!("mongodb://localhost:{}/", port);
        let options = ClientOptions::parse(&client_uri).await.unwrap();
        let client = Client::with_options(options).unwrap();

        let db = client.database("admin");
        for _ in 0..10 {
            match db.run_command(doc! {"ping": 1}).await {
                Ok(_) => break,
                Err(e) => {
                    eprintln!("Waiting for MongoDB to be ready: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }

        (client, container)
    }

    #[tokio::test]
    async fn test_get_last_update() -> Result<(), Box<dyn Error>> {
        // Start the Docker daemon
        let (client, container) = get_test_mongo_client().await;

        // Insert test data into the 'application.last_update' collection
        let collection: Collection<Document> =
            client.database("application").collection("last_update");
        collection
            .insert_one(doc! { "last_update": "2023-01-01T00:00:00Z" })
            .await?;

        // Call the function to test
        let last_update = get_last_update(&client).await;

        // Assert the expected result
        assert_eq!(last_update, "2023-01-01T00:00:00Z");
        container.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_biggest_update_at_from_profiles() -> Result<(), Box<dyn Error>> {
        // Start the Docker daemon
        let (client, container) = get_test_mongo_client().await;

        // Insert test data into the '42.profiles' collection
        let collection: Collection<Document> = client.database("42").collection("profiles");
        collection
            .insert_many(vec![
                doc! { "updated_at": "2023-01-02T00:00:00Z" },
                doc! { "updated_at": "2023-01-03T00:00:00Z" },
                doc! { "updated_at": "2023-01-01T00:00:00Z" },
            ])
            .await?;

        // Call the function to test
        let last_update = get_biggest_update_at_from_profiles(&client).await;

        // Assert the expected result
        assert_eq!(last_update, "2023-01-03T00:00:00Z");
        container.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_last_update() -> Result<(), Box<dyn Error>> {
        // Start the Docker daemon
        let (client, container) = get_test_mongo_client().await;

        // Insert test data into the '42.profiles' collection
        let collection: Collection<Document> = client.database("42").collection("profiles");
        collection
            .insert_many(vec![
                doc! { "updated_at": "2023-01-02T00:00:00Z" },
                doc! { "updated_at": "2023-01-03T00:00:00Z" },
                doc! { "updated_at": "2023-01-01T00:00:00Z" },
            ])
            .await?;

        // Call the function to test
        update_last_update(&client).await;

        // Assert the expected result
        let collection: Collection<Document> =
            client.database("application").collection("last_update");
        let last_update = collection.find_one(doc! {}).await?;
        assert_eq!(
            last_update.unwrap().get_str("last_update").unwrap(),
            "2023-01-03T00:00:00Z"
        );
        container.stop().await?;
        Ok(())
    }
}
