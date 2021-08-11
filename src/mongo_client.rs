use std::fmt::Debug;

use mongodb::{Client, bson, error::Result, options::FindOptions};
use serde::{Serialize, de::DeserializeOwned};
use futures_util::stream::StreamExt;

struct MongoClient<T = bson::Document>
where
	T: Serialize + DeserializeOwned + Unpin + Debug + Clone + Send + Sync
{
	collection: mongodb::Collection<T>
}

impl<T> MongoClient<T>
where
	T: Serialize + DeserializeOwned + Unpin + Debug + Clone + Send + Sync {

	pub async fn new<A, B, C>(mongo_uri: A, database_name: B, collection_name: C) -> anyhow::Result<MongoClient<T>>
	where
		A: Into<String>,
		B: Into<String>,
		C: Into<String>,
	{
		let client = Client::with_uri_str(&mongo_uri.into()).await?;
		let database = client.database(&database_name.into());
		let collection = database.collection::<T>(&collection_name.into());

		let mongo_client = MongoClient {
			collection,
		};

		Ok(mongo_client)
	}
	
	pub async fn find<S>(&self, filter: &S, find_options: Option<FindOptions>) -> anyhow::Result<Vec<T>> 
	where
		S: Serialize
	{
		let bson_filter = bson::to_bson(filter)?;
		let document_filter = bson::ser::to_document(&bson_filter)?;
		let cursor = self.collection.find(document_filter, find_options).await?;
		let result_options: Vec<Result<T>> = cursor.collect().await;
		
		let result = result_options.iter().filter_map(|document_result|{
			match document_result {
				Ok(document) => {
					let value = document.clone();
					Some(value)
				},
				Err(_) => None
			}
		});
	
		Ok(result.collect())
	}	
}

#[cfg(test)]
mod tests {
    use super::MongoClient;
		use serde::{Serialize, Deserialize};

		#[derive(Serialize, Deserialize)]
		struct Filter {
			main_category_id: String,
		}

		#[derive(Serialize, Deserialize, Debug, Clone)]
		struct Entity {
			main_category_id: String,
			id: String,
			name: String,
			anchor: Vec<f32>
		}
    
		#[tokio::test]
    async fn load_from_mongo() {
			let default_mongo_uri = "mongodb://10.0.0.207/".to_string();
			let mongo_uri = std::env::var("MONGODB_CONNECTION").unwrap_or(default_mongo_uri);
			let database_name = "test";
			let collection_name = "test";

      let client = MongoClient::<Entity>::new(mongo_uri, database_name, collection_name).await.unwrap();
			let filter = Filter { main_category_id: "19".to_string() };
			let results = client.find(&filter, None).await.unwrap();
			
			assert!(results.first().is_some());
			
    }
}
