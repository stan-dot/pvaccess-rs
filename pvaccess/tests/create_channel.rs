use serde_json::json;
use shared::ChannelRequest;
use tokio::test;
mod test_client; // Import the test client

#[tokio::test]
async fn test_create_channel() {
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
        name: "test_channel".into(),
        schema: Some(schema.clone()),
        message: None,
    };

    let response = test_client::send_request(request).await;
    assert!(response.success, "Failed to create channel: {:?}", response);
}
