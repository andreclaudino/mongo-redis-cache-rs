extern crate redis;

use std::marker::PhantomData;
use std::fmt::Debug;
use redis::aio::Connection;
use serde::{Serialize, de::DeserializeOwned};

struct RedisClient<T>
where
	T: Serialize + DeserializeOwned + Unpin + Debug + Clone + Send + Sync
{
	connection: Connection,
	ttl: Option<usize>,
	phantom: PhantomData<T>
}

impl<T> RedisClient<T>
where
	T: Serialize + DeserializeOwned + Unpin + Debug + Clone + Send + Sync
{
	pub async fn new<A>(redis_url: A, ttl: Option<usize>) -> anyhow::Result<RedisClient<T>>
	where
		A: Into<String>
	{
		let connection_string = redis_url.into();
		let client = redis::Client::open(connection_string)?;
    let connection = client.get_async_connection().await?;
		
		let redis_client = RedisClient {
			connection,
			ttl,
			phantom: PhantomData
		};

		Ok(redis_client)
	}

	pub async fn put<K>(&mut self, key: K, value: T) -> anyhow::Result<()>
	where
		K: Into<String>
	{
		let redis_arg = serde_json::to_value(value)?.to_string();
		match self.ttl {
			Some(ttl) => {
				redis::pipe()
					.set_ex(key.into(), redis_arg, ttl)
					.query_async(&mut self.connection).await?;
			},
			None => {
				redis::pipe()
					.set(key.into(), redis_arg)
					.query_async(&mut self.connection).await?;
			}				
		};
		

		Ok(())
	}

	pub async fn get<K>(&mut self, key: K) -> anyhow::Result<T>
	where
		K: Into<String>
	{
		todo!()
	}
}