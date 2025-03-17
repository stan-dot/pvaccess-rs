use rmp_serde::{decode, encode};
use serde_json::json;
use shared::{ChannelRequest, ChannelResponse};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn send_request(request: ChannelRequest) -> ChannelResponse {
    let mut stream = TcpStream::connect("127.0.0.1:8000").await.unwrap();

    let mut buf = Vec::new();
    encode::write(&mut buf, &request).unwrap();
    stream.write_all(&buf).await.unwrap();

    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer).await.unwrap();

    decode::from_read::<_, ChannelResponse>(&buffer[..n]).unwrap()
}
