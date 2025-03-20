use redis::{AsyncCommands, Client};

async fn store_client_in_redis(client_id: &str, channel: &str) {
    let client = Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_async_connection().await.unwrap();
    let _: () = con.set(client_id, channel).await.unwrap();
}

async fn get_client_from_redis(client_id: &str) -> Option<String> {
    let client = Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_async_connection().await.unwrap();
    con.get(client_id).await.ok()
}
