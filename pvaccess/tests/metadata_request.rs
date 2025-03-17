#[tokio::test]
async fn test_metadata_request() {
    let request = ChannelRequest {
        action: "metadata".into(),
        name: "test_channel".into(),
        schema: None,
        message: None,
    };

    let response = test_client::send_request(request).await;
    assert!(response.success, "Failed to retrieve channel metadata");
}
