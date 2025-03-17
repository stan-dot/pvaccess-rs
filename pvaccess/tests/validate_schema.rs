use tokio::test;
use serde_json::json;
use shared::{ChannelRequest};
mod test_client;

#[tokio::test]
async fn test_publish_valid_message() {
    let valid_message = json!({
        "temperature": 22.5,
        "timestamp": 1712345678
    });

    let request = ChannelRequest {
        action: "publish".into(),
        name: "test_channel".into(),
        schema: None,
        message: Some(valid_message.clone()),
    };

    let response = test_client::send_request(request).await;
    assert!(response.success, "Message should have passed schema validation");
}

#[tokio::test]
async fn test_publish_invalid_message() {
    let invalid_message = json!({
        "temperature": "hot",  // Invalid type (should be number)
        "timestamp": 1712345678
    });

    let request = ChannelRequest {
        action: "publish".into(),
        name: "test_channel".into(),
        schema: None,
        message: Some(invalid_message.clone()),
    };

    let response = test_client::send_request(request).await;
    assert!(
        !response.success,
        "Invalid message should have failed validation"
    );
}
