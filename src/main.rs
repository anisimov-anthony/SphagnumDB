// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use std::error::Error;
use std::sync::Arc;
use libp2p::Multiaddr;
use sproutdb::core::sprout::Sprout;
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut sp_1 = Sprout::new()?;
    let mut sp_2 = Sprout::new()?;

    println!("PeerID of sp_1: {}", sp_1.peer_id()?);
    println!("PeerID of sp_2: {}", sp_2.peer_id()?);

    let listen_addr_2: Multiaddr = "/ip4/127.0.0.1/tcp/4002".parse()?;
    sp_2.listen_on(listen_addr_2)?;

    sp_1.dial("/ip4/127.0.0.1/tcp/4002")?;

    let peer_id_2 = sp_2.peer_id()?;

    let sp_1_clone = Arc::new(Mutex::new(sp_1));
    
    let handle_events_1 = {
        let sp_1_clone = Arc::clone(&sp_1_clone);
        tokio::spawn(async move {
            loop {
                let mut sp_1 = sp_1_clone.lock().await;
                if let Err(e) = sp_1.handle_event().await {
                    eprintln!("Error in sp_1: {}", e);
                }
            }
        })
    };

    let handle_events_2 = tokio::spawn(async move {
        loop {
            if let Err(e) = sp_2.handle_event().await {
                eprintln!("Error in sp_2: {}", e);
            }
        }
    });

    let handle_greet_request = {
        let sp_1_clone = Arc::clone(&sp_1_clone);
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(3)).await; 
                let mut sp_1 = sp_1_clone.lock().await;
                match sp_1.send_request_to_sprout(peer_id_2, "Hello from sp_1".to_string()) {
                    Ok(_) => println!("Greeting request sent from sp_1 to sp_2"),
                    Err(e) => eprintln!("Failed to send greeting request: {}", e),
                }
            }
        })
    };

    let _ = tokio::join!(handle_events_1, handle_events_2, handle_greet_request);

    Ok(())
}
