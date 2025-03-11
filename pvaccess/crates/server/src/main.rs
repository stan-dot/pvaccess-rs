use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Atomic flag to control beacon timing.
    let udp_active = Arc::new(AtomicBool::new(true));
    
    // 1. Start the UDP beacon in the background
    let udp_active_clone = Arc::clone(&udp_active);
    tokio::spawn(async move {
        send_udp_beacons(udp_active_clone).await;
    });

    // 2. Start the TCP server
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    println!("Server listening on 127.0.0.1:8000");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New client connected: {}", addr);

        // Handle each client connection asynchronously
        tokio::spawn(handle_client(socket));
    }
}

async fn handle_client(mut socket: TcpStream) {
    let hello_message = b"Hello, welcome to the server!\n";
    
    // Send "Hello" message to client
    if let Err(e) = socket.write_all(hello_message).await {
        eprintln!("Failed to send hello message: {:?}", e);
        return;
    }
    println!("Sent hello message to client");

    let mut buffer = vec![0; 1024];

    // Optionally, read data from the client (if needed)
    match socket.read(&mut buffer).await {
        Ok(0) => println!("Client disconnected"),
        Ok(n) => println!("Received: {}", String::from_utf8_lossy(&buffer[..n])),
        Err(e) => eprintln!("Failed to read from client: {:?}", e),
    }
}

async fn send_udp_beacons(active: Arc<AtomicBool>) {
    // Set the initial beacon interval to 15 seconds
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    let beacon_addr = "127.0.0.1:12345"; // UDP server address

    loop {
        if active.load(Ordering::Relaxed) {
            // Send the beacon message
            let beacon_message = b"UDP Beacon - Hello from server!";
            if let Err(e) = socket.send_to(beacon_message, beacon_addr).await {
                eprintln!("Failed to send UDP beacon: {:?}", e);
            } else {
                println!("Sent UDP beacon to {}", beacon_addr);
            }
            
            // Change the interval after 15 seconds to send every 60 seconds
            interval.tick().await;
            sleep(Duration::from_secs(60)).await;
        } else {
            break; // If beaconing is no longer active, stop
        }
    }
}
