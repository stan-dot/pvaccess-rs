// fn main() {
//     println!("Hello, world!");
// }

use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let listener = TcpListener::bind(addr).await?;

    loop {
        println!("Waiting for connection at {}", addr);
        let (socket, peer_addr): (TcpStream, SocketAddr) = listener.accept().await?;
        println!("Accepted connection from {}", peer_addr);
    }
}
