use rmp_serde::{decode, encode};
use serde_json::json;
use shared::{ChannelRequest, ChannelResponse};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8000").await.unwrap();

    // 🔹 1️⃣ Create a new channel with a schema
    let schema = json!({
        "type": "object",
        "properties": {
            "temperature": { "type": "number" },
            "timestamp": { "type": "integer" }
        },
        "required": ["temperature", "timestamp"]
    });

    let request = ChannelRequest {
        action: "create".into(),
        name: "temperature".into(),
        schema: Some(schema.clone()),
        message: None,
    };

    let mut buf = Vec::new();
    encode::write(&mut buf, &request).unwrap();
    stream.write_all(&buf).await.unwrap();

    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer).await.unwrap();

    if let Ok(response) = decode::from_read::<_, ChannelResponse>(&buffer[..n]) {
        println!("Server response: {:?}", response);
    }

    // 🔹 2️⃣ Publish a message that matches the schema
    let valid_message = json!({
        "temperature": 22.5,
        "timestamp": 1712345678
    });

    let publish_request = ChannelRequest {
        action: "publish".into(),
        name: "temperature".into(),
        schema: None,
        message: Some(valid_message.clone()),
    };

    buf.clear();
    encode::write(&mut buf, &publish_request).unwrap();
    stream.write_all(&buf).await.unwrap();

    let n = stream.read(&mut buffer).await.unwrap();
    if let Ok(response) = decode::from_read::<_, ChannelResponse>(&buffer[..n]) {
        println!("Publish Response: {:?}", response);
    }

    // 🔹 3️⃣ Request metadata
    let metadata_request = ChannelRequest {
        action: "metadata".into(),
        name: "temperature".into(),
        schema: None,
        message: None,
    };

    buf.clear();
    encode::write(&mut buf, &metadata_request).unwrap();
    stream.write_all(&buf).await.unwrap();

    let n = stream.read(&mut buffer).await.unwrap();
    if let Ok(response) = decode::from_read::<_, ChannelResponse>(&buffer[..n]) {
        println!("Channel Metadata: {:?}", response);
    }
}
