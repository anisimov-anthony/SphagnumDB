// SphagnumDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use libp2p::Multiaddr;
use sphagnumdb::core::{
    commands::{generic::GenericCommand, string::StringCommand, Command as SphagnumCommand},
    sphagnum::SphagnumNode,
};
use std::error::Error;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // creating 3 nodes for a future cluster
    let mut sp1 = SphagnumNode::new()?;
    let mut sp2 = SphagnumNode::new()?;
    let mut sp3 = SphagnumNode::new()?;

    // setting listening ports
    sp1.listen_on("/ip4/127.0.0.1/tcp/3301".parse::<Multiaddr>()?)?;
    sp2.listen_on("/ip4/127.0.0.1/tcp/3302".parse::<Multiaddr>()?)?;
    sp3.listen_on("/ip4/127.0.0.1/tcp/3303".parse::<Multiaddr>()?)?;

    // dialing nodes
    sp1.dial("/ip4/127.0.0.1/tcp/3302")?;
    sp1.dial("/ip4/127.0.0.1/tcp/3303")?;
    sp2.dial("/ip4/127.0.0.1/tcp/3301")?;
    sp2.dial("/ip4/127.0.0.1/tcp/3303")?;
    sp3.dial("/ip4/127.0.0.1/tcp/3301")?;
    sp3.dial("/ip4/127.0.0.1/tcp/3302")?;

    // config replication
    sp1.add_to_replica_set(sp2.peer_id()?)?;
    sp1.add_to_replica_set(sp3.peer_id()?)?;
    sp2.add_to_replica_set(sp1.peer_id()?)?;
    sp2.add_to_replica_set(sp3.peer_id()?)?;
    sp3.add_to_replica_set(sp1.peer_id()?)?;
    sp3.add_to_replica_set(sp2.peer_id()?)?;

    let sp_arc_1 = Arc::new(Mutex::new(sp1));
    let sp_arc_2 = Arc::new(Mutex::new(sp2));
    let sp_arc_3 = Arc::new(Mutex::new(sp3));

    let nodes = std::collections::HashMap::from([
        ("sp1".to_string(), Arc::clone(&sp_arc_1)),
        ("sp2".to_string(), Arc::clone(&sp_arc_2)),
        ("sp3".to_string(), Arc::clone(&sp_arc_3)),
    ]);

    let handle_events_1 = {
        let sp_arc_1 = Arc::clone(&sp_arc_1);
        tokio::spawn(async move {
            loop {
                let mut sphagnum = sp_arc_1.lock().await;
                if let Err(e) = sphagnum.handle_event().await {
                    eprintln!("Error handling event: {}", e);
                }
            }
        })
    };

    let handle_events_2 = {
        let sp_arc_2 = Arc::clone(&sp_arc_2);
        tokio::spawn(async move {
            loop {
                let mut sphagnum = sp_arc_2.lock().await;
                if let Err(e) = sphagnum.handle_event().await {
                    eprintln!("Error handling event: {}", e);
                }
            }
        })
    };

    let handle_events_3 = {
        let sp_arc_3 = Arc::clone(&sp_arc_3);
        tokio::spawn(async move {
            loop {
                let mut sphagnum = sp_arc_3.lock().await;
                if let Err(e) = sphagnum.handle_event().await {
                    eprintln!("Error handling event: {}", e);
                }
            }
        })
    };

    let handle_input = {
        let nodes = nodes.clone();
        tokio::spawn(async move {
            let stdin = io::stdin();
            let mut reader = BufReader::new(stdin).lines();

            while let Some(line) = reader.next_line().await.unwrap_or(None) {
                let mut parts = line.split_whitespace();
                if let Some(node_name) = parts.next() {
                    if let Some(node_arc) = nodes.get(node_name) {
                        if let Some(command) = parts.next() {
                            match command.to_lowercase().as_str() {
                                "get" => {
                                    if let Some(key) = parts.next() {
                                        let mut sphagnum = node_arc.lock().await;
                                        if let Some(peer_id) =
                                            sphagnum.connected_peers.iter().next().copied()
                                        {
                                            let cmd = SphagnumCommand::String(StringCommand::Get {
                                                key: key.to_string(),
                                            });
                                            match sphagnum
                                                .send_request_to_sphagnum(peer_id, cmd)
                                                .await
                                            {
                                                Ok(_) => {
                                                    println!("Get request sent for key: {}", key)
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to send Get request: {}", e)
                                                }
                                            }
                                        } else {
                                            eprintln!("Not connected to any node.");
                                        }
                                    } else {
                                        eprintln!("Usage: <node> get <key>");
                                    }
                                }
                                "set" => {
                                    if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                                        let mut sphagnum = node_arc.lock().await;
                                        if let Some(peer_id) =
                                            sphagnum.connected_peers.iter().next().copied()
                                        {
                                            let cmd = SphagnumCommand::String(StringCommand::Set {
                                                key: key.to_string(),
                                                value: value.to_string(),
                                            });
                                            match sphagnum
                                                .send_request_to_sphagnum(peer_id, cmd)
                                                .await
                                            {
                                                Ok(_) => println!(
                                                    "Set request sent with key: {}, value: {}",
                                                    key, value
                                                ),
                                                Err(e) => {
                                                    eprintln!("Failed to send Set request: {}", e)
                                                }
                                            }
                                        } else {
                                            eprintln!("Not connected to any node.");
                                        }
                                    } else {
                                        eprintln!("Usage: <node> set <key> <value>");
                                    }
                                }
                                "append" => {
                                    if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                                        let mut sphagnum = node_arc.lock().await;
                                        if let Some(peer_id) =
                                            sphagnum.connected_peers.iter().next().copied()
                                        {
                                            let cmd =
                                                SphagnumCommand::String(StringCommand::Append {
                                                    key: key.to_string(),
                                                    value: value.to_string(),
                                                });
                                            match sphagnum
                                                .send_request_to_sphagnum(peer_id, cmd)
                                                .await
                                            {
                                                Ok(_) => println!(
                                                    "Append request sent with key: {}, value: {}",
                                                    key, value
                                                ),
                                                Err(e) => eprintln!(
                                                    "Failed to send Append request: {}",
                                                    e
                                                ),
                                            }
                                        } else {
                                            eprintln!("Not connected to any node.");
                                        }
                                    } else {
                                        eprintln!("Usage: <node> append <key> <value>");
                                    }
                                }
                                "exists" => {
                                    let keys: Vec<String> = parts.map(|s| s.to_string()).collect();
                                    if keys.is_empty() {
                                        eprintln!("Usage: <node> exists <key> [key ...]");
                                    } else {
                                        let mut sphagnum = node_arc.lock().await;
                                        if let Some(peer_id) =
                                            sphagnum.connected_peers.iter().next().copied()
                                        {
                                            let cmd =
                                                SphagnumCommand::Generic(GenericCommand::Exists {
                                                    keys,
                                                });
                                            match sphagnum
                                                .send_request_to_sphagnum(peer_id, cmd)
                                                .await
                                            {
                                                Ok(_) => println!("Exists request sent for keys"),
                                                Err(e) => eprintln!(
                                                    "Failed to send Exists request: {}",
                                                    e
                                                ),
                                            }
                                        } else {
                                            eprintln!("Not connected to any node.");
                                        }
                                    }
                                }
                                "del" => {
                                    let keys: Vec<String> = parts.map(|s| s.to_string()).collect();
                                    if keys.is_empty() {
                                        eprintln!("Usage: <node> del <key> [key ...]");
                                    } else {
                                        let mut sphagnum = node_arc.lock().await;
                                        if let Some(peer_id) =
                                            sphagnum.connected_peers.iter().next().copied()
                                        {
                                            let cmd =
                                                SphagnumCommand::Generic(GenericCommand::Delete {
                                                    keys,
                                                });
                                            match sphagnum
                                                .send_request_to_sphagnum(peer_id, cmd)
                                                .await
                                            {
                                                Ok(_) => println!("Delete request sent for keys"),
                                                Err(e) => eprintln!(
                                                    "Failed to send Delete request: {}",
                                                    e
                                                ),
                                            }
                                        } else {
                                            eprintln!("Not connected to any node.");
                                        }
                                    }
                                }
                                "enable_pinging_output" => {
                                    let mut sphagnum = node_arc.lock().await;
                                    sphagnum.enable_pinging_output();
                                }
                                "disable_pinging_output" => {
                                    let mut sphagnum = node_arc.lock().await;
                                    sphagnum.disable_pinging_output();
                                }
                                _ => {
                                    eprintln!("Unknown command: {}", command);
                                }
                            }
                        } else {
                            eprintln!("Please specify a command after node name");
                        }
                    } else {
                        eprintln!(
                            "Unknown node: {}. Available nodes: sp1, sp2, sp3",
                            node_name
                        );
                    }
                }
            }
        })
    };

    let _ = tokio::join!(
        handle_events_1,
        handle_events_2,
        handle_events_3,
        handle_input
    );

    Ok(())
}
