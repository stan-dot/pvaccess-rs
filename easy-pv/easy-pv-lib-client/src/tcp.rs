use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::config::ClientConfig;
use easy_pv_datatypes::{
    header::{Command, PvAccessHeader},
    messages::{
        pv_echo::{EchoMessage, EchoResponse},
        pv_validation::{ConnectionQoS, ConnectionValidationRequest, ConnectionValidationResponse},
    },
};

use tokio::net::{
    TcpStream,
    tcp::{OwnedReadHalf, OwnedWriteHalf},
};

pub async fn handle_tcp_session(stream: TcpStream, config: &ClientConfig) -> anyhow::Result<()> {
    let (mut reader, mut writer): (OwnedReadHalf, OwnedWriteHalf) = stream.into_split();

    // 🔹 Read initial validation request
    let mut buffer = vec![0u8; config.buffer_size as usize];
    let n = reader.read(&mut buffer).await?;
    let request = ConnectionValidationRequest::from_bytes(&buffer[..n])?;
    println!("📩 Received validation request: {:?}", request);

    // 🔸 Send validation response
    let response = ConnectionValidationResponse::new(
        config.buffer_size,
        config.introspection_registry_max_size.try_into().unwrap(),
        ConnectionQoS::PRIORITY_MASK,
        "authz".to_string(),
    );
    let response_bytes = response.to_bytes()?;
    writer.write_all(&response_bytes).await?;
    println!("📤 Sent validation response.");

    // 🔁 Start message loop
    let mut frame_buf = vec![0u8; 1500];
    loop {
        let n = match reader.read(&mut frame_buf).await {
            Ok(0) => {
                println!("🔌 Connection closed by server.");
                break;
            }
            Ok(n) => n,
            Err(e) => {
                println!("❗ Read error: {}", e);
                break;
            }
        };

        // parse message header here
        let header = PvAccessHeader::from_bytes(&frame_buf[..8])?;
        let is_big_endian = header.is_big_endian();

        println!("📦 Received message command: {:?}", header.message_command);

        match header.message_command {
            Command::Echo => {
                let echo = EchoMessage::from_bytes(&frame_buf[8..n], is_big_endian)?;
                println!("🟡 Echo message: {:?}", echo);

                let response = EchoResponse {
                    repeated_bytes: echo.random_bytes.clone(),
                };

                let response_bytes = response.to_bytes(is_big_endian)?;
                writer.write_all(&response_bytes).await?;
            }
            _ => {
                println!(
                    "⚠️ Unknown or unhandled message command: {:?}",
                    header.message_command
                );
            }
        }
    }

    Ok(())
}
