// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use std::error::Error;

use libp2p::Multiaddr;
use sproutdb::core::sprout::Sprout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut sp_1 = Sprout::new()?;
    let mut sp_2 = Sprout::new()?;

    println!("PeerID of sp_1: {}", sp_1.peer_id()?);
    println!("PeerID of sp_2: {}", sp_2.peer_id()?);

    let listen_addr_2: Multiaddr = "/ip4/127.0.0.1/tcp/4002".parse()?;

    sp_2.listen_on(listen_addr_2)?;

    sp_1.dial("/ip4/127.0.0.1/tcp/4002")?;

    let handle_1 = tokio::spawn(async move {
        if let Err(e) = sp_1.run().await {
            eprintln!("Error in sp_1: {}", e);
        }
    });

    let handle_2 = tokio::spawn(async move {
        if let Err(e) = sp_2.run().await {
            eprintln!("Error in sp_2: {}", e);
        }
    });

    let _ = tokio::join!(handle_1, handle_2);

    Ok(())
}
