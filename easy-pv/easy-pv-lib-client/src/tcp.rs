use crate::config::ClientConfig;
use easy_pv_datatypes::{
    codec::PvAccessDecoder,
    frame::{PvAccessEncoder},
    header::Command,
    messages::{
        into::IntoPvAccessFrame, pv_echo::{EchoMessage, EchoResponse}, pv_validation::{ConnectionQoS, ConnectionValidationRequest, ConnectionValidationResponse}
    },
};

use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio_util::codec::{FramedRead, FramedWrite};

pub async fn handle_tcp_session(stream: TcpStream, config: &ClientConfig) -> anyhow::Result<()> {
    let (reader, writer): (OwnedReadHalf, OwnedWriteHalf) = stream.into_split();

    let mut framed_read = FramedRead::new(reader, PvAccessDecoder);
    let mut framed_write = FramedWrite::new(writer, PvAccessEncoder);

    // 🔹 Step 1: Expect ConnectionValidationRequest from server
    let Some(Ok(request_frame)) = framed_read.next().await else {
        anyhow::bail!("Failed to receive connection validation request frame");
    };

    let header = request_frame.0;
    if header.message_command != Command::ConnectionValidation {
        anyhow::bail!(
            "Unexpected command: expected validation, got {:?}",
            header.message_command
        );
    }

    let request = ConnectionValidationRequest::from_bytes(&request_frame.1)?;
    println!("📩 Received validation request: {:?}", request);

    // 🔸 Step 2: Respond with ConnectionValidationResponse
    let response = ConnectionValidationResponse::new(
        config.buffer_size,
        config.introspection_registry_max_size.try_into()?,
        ConnectionQoS::PRIORITY_MASK,
        "".to_string(),
    );

    let response_frame = response.into_frame(Command::ConnectionValidation, 0)?;
    framed_write.send(response_frame).await?;
    println!("📤 Sent validation response");

    // 🔁 Step 3: Process messages
    while let Some(frame_result) = framed_read.next().await {
        let ( header, payload ) = frame_result?;

        println!("📦 Received message: {:?}", header.message_command);
        let is_big_endian = header.is_big_endian();

        match header.message_command {
            Command::Echo => {
                let echo = EchoMessage::from_bytes(&payload, is_big_endian)?;
                println!("🟡 Echo message: {:?}", echo);

                let response = EchoResponse {
                    repeated_bytes: echo.random_bytes.clone(),
                };
                let response_frame = response.into_frame(Command::Echo, header.flags.bits())?;
                framed_write.send(response_frame).await?;
            }
            other => {
                println!("⚠️ Unhandled message command: {:?}", other);
            }
        }
    }

    println!("🔌 Server closed the connection.");
    Ok(())
}
