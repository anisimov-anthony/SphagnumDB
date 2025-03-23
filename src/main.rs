use clap::{Arg, Command}; // Используем `Command` вместо `App`
use libp2p::Multiaddr;
use sproutdb::core::{
    commands::{generic::GenericCommand, string::StringCommand, Command as SproutCommand},
    sprout::Sprout,
};
use std::error::Error;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("SproutDB Node")
        .arg(
            Arg::new("addr")
                .long("addr")
                .value_name("ADDRESS")
                .help("Address of another node to connect to"),
        )
        .get_matches();

    let mut sprout = Sprout::new()?;

    println!("PeerID: {}", sprout.peer_id()?);

    let listen_addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse()?;
    sprout.listen_on(listen_addr.clone())?;

    if let Some(addr) = sprout.listeners().next() {
        println!("Node is listening on: {}", addr);
    }

    if let Some(addr) = matches.get_one::<String>("addr") {
        sprout.dial(addr)?;
        println!("Connected to node at: {}", addr);
    };

    let sprout_arc = Arc::new(Mutex::new(sprout));

    let handle_events = {
        let sprout_arc = Arc::clone(&sprout_arc);
        tokio::spawn(async move {
            loop {
                let mut sprout = sprout_arc.lock().await;
                if let Err(e) = sprout.handle_event().await {
                    eprintln!("Error handling event: {}", e);
                }
            }
        })
    };

    let handle_input = {
        let sprout_arc = Arc::clone(&sprout_arc);
        tokio::spawn(async move {
            let stdin = io::stdin();
            let mut reader = BufReader::new(stdin).lines();

            while let Some(line) = reader.next_line().await.unwrap_or(None) {
                let mut parts = line.split_whitespace();
                if let Some(command) = parts.next() {
                    match command.to_lowercase().as_str() {
                        "get" => {
                            if let Some(key) = parts.next() {
                                let mut sprout = sprout_arc.lock().await;
                                if let Some(peer_id) = sprout.connected_peers.iter().next().copied()
                                {
                                    let cmd = SproutCommand::String(StringCommand::Get {
                                        key: key.to_string(),
                                    });
                                    match sprout.send_request_to_sprout(peer_id, cmd) {
                                        Ok(_) => println!("Get request sent for key: {}", key),
                                        Err(e) => eprintln!("Failed to send Get request: {}", e),
                                    }
                                } else {
                                    eprintln!("Not connected to any node.");
                                }
                            } else {
                                eprintln!("Usage: get <key>");
                            }
                        }
                        "set" => {
                            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                                let mut sprout = sprout_arc.lock().await;
                                if let Some(peer_id) = sprout.connected_peers.iter().next().copied()
                                {
                                    let cmd = SproutCommand::String(StringCommand::Set {
                                        key: key.to_string(),
                                        value: value.to_string(),
                                    });
                                    match sprout.send_request_to_sprout(peer_id, cmd) {
                                        Ok(_) => println!(
                                            "Set request sent with key: {}, value: {}",
                                            key, value
                                        ),
                                        Err(e) => eprintln!("Failed to send Set request: {}", e),
                                    }
                                } else {
                                    eprintln!("Not connected to any node.");
                                }
                            } else {
                                eprintln!("Usage: set <key> <value>");
                            }
                        }
                        "append" => {
                            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                                let mut sprout = sprout_arc.lock().await;
                                if let Some(peer_id) = sprout.connected_peers.iter().next().copied()
                                {
                                    let cmd = SproutCommand::String(StringCommand::Append {
                                        key: key.to_string(),
                                        value: value.to_string(),
                                    });
                                    match sprout.send_request_to_sprout(peer_id, cmd) {
                                        Ok(_) => println!(
                                            "Append request sent with key: {}, value: {}",
                                            key, value
                                        ),
                                        Err(e) => eprintln!("Failed to send Append request: {}", e),
                                    }
                                } else {
                                    eprintln!("Not connected to any node.");
                                }
                            } else {
                                eprintln!("Usage: append <key> <value>");
                            }
                        }
                        "exists" => {
                            let keys: Vec<String> = parts.map(|s| s.to_string()).collect();
                            if keys.is_empty() {
                                eprintln!("Usage: exists <key> [key ...]");
                            } else {
                                let mut sprout = sprout_arc.lock().await;
                                if let Some(peer_id) = sprout.connected_peers.iter().next().copied()
                                {
                                    let cmd =
                                        SproutCommand::Generic(GenericCommand::Exists { keys });
                                    match sprout.send_request_to_sprout(peer_id, cmd) {
                                        Ok(_) => println!("Exists request sent for keys"),
                                        Err(e) => eprintln!("Failed to send Exists request: {}", e),
                                    }
                                } else {
                                    eprintln!("Not connected to any node.");
                                }
                            }
                        }
                        "del" => {
                            let keys: Vec<String> = parts.map(|s| s.to_string()).collect();
                            if keys.is_empty() {
                                eprintln!("Usage: del <key> [key ...]");
                            } else {
                                let mut sprout = sprout_arc.lock().await;
                                if let Some(peer_id) = sprout.connected_peers.iter().next().copied()
                                {
                                    let cmd =
                                        SproutCommand::Generic(GenericCommand::Delete { keys });
                                    match sprout.send_request_to_sprout(peer_id, cmd) {
                                        Ok(_) => println!("Delete request sent for keys"),
                                        Err(e) => eprintln!("Failed to send Delete request: {}", e),
                                    }
                                } else {
                                    eprintln!("Not connected to any node.");
                                }
                            }
                        }
                        "enable_pinging_output" => {
                            let mut sprout = sprout_arc.lock().await;
                            sprout.enable_pinging_output();
                        }
                        "disable_pinging_output" => {
                            let mut sprout = sprout_arc.lock().await;
                            sprout.disable_pinging_output();
                        }
                        _ => {
                            eprintln!("Unknown command: {}", command);
                        }
                    }
                }
            }
        })
    };

    let _ = tokio::join!(handle_events, handle_input);

    Ok(())
}
