// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use std::{error::Error, time::Duration};

use futures::prelude::*;
use libp2p::{
    noise, ping,
    request_response::{self, OutboundRequestId, ProtocolSupport},
    swarm::{Swarm, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol,
};

use std::collections::HashSet;

use super::{
    commands::{generic::GenericCommand, string::StringCommand, Command},
    data_storage::DataStorage,
    passport::Passport,
    req_resp_codec::{SproutRequest, SproutResponse},
    sprout_behaviour::{SproutBehaviour, SproutBehaviourEvent},
};

/// Reminder: in this project, the nodes are called sprouts. Thus, this structure is a node
/// structure. At this stage, this is a highly simplified representation of the node, and it will be
/// further refined.
pub struct Sprout {
    data_storage: DataStorage,
    passport: Passport,
    pub swarm: Swarm<SproutBehaviour>, // todo remove pub
    pub connected_peers: HashSet<PeerId>,
}

impl Sprout {
    pub fn new() -> Result<Sprout, Box<dyn Error>> {
        let behaviours = Self::configure_behaviours()?;

        let swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| behaviours)?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX))
            })
            .build();

        Ok(Sprout {
            data_storage: DataStorage::new()?,
            passport: Passport::new()?,
            swarm,
            connected_peers: HashSet::new(),
        })
    }

    fn configure_behaviours() -> Result<SproutBehaviour, Box<dyn Error>> {
        let ping = ping::Behaviour::default();
        let request_response = request_response::json::Behaviour::new(
            [(
                StreamProtocol::new("/sproutdb/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default(),
        );

        Ok(SproutBehaviour {
            ping,
            request_response,
        })
    }

    pub fn listen_on(&mut self, listen_addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm.listen_on(listen_addr)?;
        Ok(())
    }

    pub fn listeners(&self) -> impl Iterator<Item = &Multiaddr> {
        self.swarm.listeners()
    }

    pub fn peer_id(&self) -> Result<PeerId, Box<dyn Error>> {
        Ok(*self.swarm.local_peer_id())
    }

    pub fn get_data_storage(&self) -> Result<&DataStorage, Box<dyn Error>> {
        Ok(&self.data_storage)
    }

    pub fn get_passport(&self) -> Result<&Passport, Box<dyn Error>> {
        Ok(&self.passport)
    }

    pub async fn handle_event(&mut self) -> Result<(), Box<dyn Error>> {
        match self.swarm.select_next_some().await {
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in,
            } => {
                self.connected_peers.insert(peer_id);
                if endpoint.is_dialer() {
                    println!(
                        "Node {} successfully dialed {} (connection_id: {:?})",
                        self.swarm.local_peer_id(),
                        peer_id,
                        connection_id
                    );
                } else {
                    println!("Node {} accepted connection from {} (connection_id: {:?}, num_established: {}, established_in: {:?})", 
                        self.swarm.local_peer_id(), peer_id, connection_id, num_established, established_in);
                }
                if let Some(errors) = concurrent_dial_errors {
                    for (addr, err) in errors {
                        println!("Dial attempt to {:?} failed with error: {:?}", addr, err);
                    }
                }
                println!(
                    "Total number of established connections with peer {}: {}",
                    peer_id, num_established
                );
                println!("Connection established in: {:?}", established_in);
                Ok(())
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            } => {
                self.connected_peers.remove(&peer_id);
                println!("Node {} closed connection with {} (connection_id: {:?}, endpoint: {:?}, num_established: {})", 
                    self.swarm.local_peer_id(), peer_id, connection_id, endpoint, num_established);
                if let Some(err) = cause {
                    println!("Cause of disconnection: {:?}", err);
                }
                Ok(())
            }
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => {
                println!(
                    "Node {} is now listening on {:?} with listener ID: {:?}",
                    self.swarm.local_peer_id(),
                    address,
                    listener_id
                );
                Ok(())
            }
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => {
                println!(
                    "Listener {} closed. Addresses: {:?}",
                    listener_id, addresses
                );
                match reason {
                    Ok(_) => println!("Listener closed successfully."),
                    Err(err) => println!("Listener closed with error: {:?}", err),
                }
                Ok(())
            }
            SwarmEvent::ListenerError { listener_id, error } => {
                println!("Listener {} encountered an error: {:?}", listener_id, error);
                Ok(())
            }
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => {
                println!(
                    "Node {} is dialing peer {:?} (connection_id: {:?})",
                    self.swarm.local_peer_id(),
                    peer_id,
                    connection_id
                );
                Ok(())
            }
            SwarmEvent::NewExternalAddrCandidate { address } => {
                println!(
                    "Node {} discovered a new external address: {:?}",
                    self.swarm.local_peer_id(),
                    address
                );
                Ok(())
            }
            SwarmEvent::ExternalAddrConfirmed { address } => {
                println!(
                    "Node {} confirmed external address: {:?}",
                    self.swarm.local_peer_id(),
                    address
                );
                Ok(())
            }
            SwarmEvent::ExternalAddrExpired { address } => {
                println!(
                    "Node {} detected the expiration of external address: {:?}",
                    self.swarm.local_peer_id(),
                    address
                );
                Ok(())
            }
            SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {
                println!(
                    "Node {} discovered a new address for peer {:?}: {:?}",
                    self.swarm.local_peer_id(),
                    peer_id,
                    address
                );
                Ok(())
            }
            SwarmEvent::Behaviour(SproutBehaviourEvent::Ping(event)) => {
                let ping::Event { peer, result, .. } = event; // Исправлено
                match result {
                    Ok(rtt) => println!(
                        "Node {} received ping from {}: {:?}",
                        self.swarm.local_peer_id(),
                        peer,
                        rtt
                    ),
                    Err(e) => println!(
                        "Node {} failed to ping {}: {:?}",
                        self.swarm.local_peer_id(),
                        peer,
                        e
                    ),
                }
                Ok(())
            }
            SwarmEvent::Behaviour(SproutBehaviourEvent::RequestResponse(event)) => {
                match event {
                    request_response::Event::Message {
                        peer,
                        connection_id,
                        message,
                    } => match message {
                        request_response::Message::Request {
                            request_id,
                            request,
                            channel,
                        } => {
                            println!("Node {} received request from {} (connection: {:?}, request_id: {:?}): {:?}", 
                                    self.swarm.local_peer_id(), peer, connection_id, request_id, request);

                            let response = match request.command {
                                Command::String(StringCommand::Set { key, value }) => {
                                    match self.data_storage.handle_command(Command::String(
                                        StringCommand::Set { key, value },
                                    )) {
                                        Ok(result) => {
                                            if let Some(ok) = result.downcast_ref::<String>() {
                                                if ok.as_str() == "OK" {
                                                    SproutResponse {
                                                        payload: "OK".to_string(),
                                                    }
                                                } else {
                                                    SproutResponse {
                                                        payload: "Unexpected response".to_string(),
                                                    }
                                                }
                                            } else {
                                                SproutResponse {
                                                    payload: "Unexpected response".to_string(),
                                                }
                                            }
                                        }
                                        Err(e) => SproutResponse {
                                            payload: format!("Error setting value: {:?}", e),
                                        },
                                    }
                                }
                                Command::String(StringCommand::Get { key }) => {
                                    match self
                                        .data_storage
                                        .handle_command(Command::String(StringCommand::Get { key }))
                                    {
                                        Ok(result) => {
                                            if let Some(value) =
                                                result.downcast_ref::<Option<String>>()
                                            {
                                                SproutResponse {
                                                    payload: value
                                                        .clone()
                                                        .unwrap_or("nil".to_string()),
                                                }
                                            } else {
                                                SproutResponse {
                                                    payload: "Unexpected response".to_string(),
                                                }
                                            }
                                        }
                                        Err(e) => SproutResponse {
                                            payload: format!("Error getting value: {:?}", e),
                                        },
                                    }
                                }
                                Command::String(StringCommand::Append { key, value }) => {
                                    match self.data_storage.handle_command(Command::String(
                                        StringCommand::Append { key, value },
                                    )) {
                                        Ok(result) => {
                                            if let Some(len) = result.downcast_ref::<u64>() {
                                                SproutResponse {
                                                    payload: len.to_string(),
                                                }
                                            } else {
                                                SproutResponse {
                                                    payload: "Unexpected response".to_string(),
                                                }
                                            }
                                        }
                                        Err(e) => SproutResponse {
                                            payload: format!("Error appending value: {:?}", e),
                                        },
                                    }
                                }
                                Command::Generic(GenericCommand::Exists { keys }) => {
                                    match self.data_storage.handle_command(Command::Generic(
                                        GenericCommand::Exists { keys },
                                    )) {
                                        Ok(result) => {
                                            if let Some(count) = result.downcast_ref::<u64>() {
                                                SproutResponse {
                                                    payload: count.to_string(),
                                                }
                                            } else {
                                                SproutResponse {
                                                    payload: "Unexpected response".to_string(),
                                                }
                                            }
                                        }
                                        Err(e) => SproutResponse {
                                            payload: format!("Error checking existence: {:?}", e),
                                        },
                                    }
                                }
                                Command::Generic(GenericCommand::Delete { keys }) => {
                                    match self.data_storage.handle_command(Command::Generic(
                                        GenericCommand::Delete { keys },
                                    )) {
                                        Ok(result) => {
                                            if let Some(count) = result.downcast_ref::<u64>() {
                                                SproutResponse {
                                                    payload: count.to_string(),
                                                }
                                            } else {
                                                SproutResponse {
                                                    payload: "Unexpected response".to_string(),
                                                }
                                            }
                                        }
                                        Err(e) => SproutResponse {
                                            payload: format!("Error deleting keys: {:?}", e),
                                        },
                                    }
                                }
                            };
                            self.swarm
                                .behaviour_mut()
                                .request_response
                                .send_response(channel, response)
                                .unwrap();
                        }
                        request_response::Message::Response {
                            request_id,
                            response,
                        } => {
                            println!("Node {} received response from {} (connection: {:?}, request_id: {:?}): {:?}", 
                                    self.swarm.local_peer_id(), peer, connection_id, request_id, response);
                        }
                    },
                    request_response::Event::OutboundFailure {
                        peer,
                        connection_id,
                        request_id,
                        error,
                    } => {
                        println!("Node {} outbound request to {} (connection: {:?}, request: {:?}) failed: {:?}", 
                            self.swarm.local_peer_id(), peer, connection_id, request_id, error);
                    }
                    request_response::Event::InboundFailure {
                        peer,
                        connection_id,
                        request_id,
                        error,
                    } => {
                        println!("Node {} inbound request from {} (connection: {:?}, request: {:?}) failed: {:?}", 
                            self.swarm.local_peer_id(), peer, connection_id, request_id, error);
                    }
                    request_response::Event::ResponseSent {
                        peer,
                        connection_id,
                        request_id,
                    } => {
                        println!(
                            "Node {} sent response to {} (connection: {:?}, request: {:?})",
                            self.swarm.local_peer_id(),
                            peer,
                            connection_id,
                            request_id
                        );
                    }
                }
                Ok(())
            }
            _ => {
                println!(
                    "Unhandled event for SwarmEvent: {:?}",
                    self.swarm.select_next_some().await
                );
                Ok(())
            }
        }
    }

    pub fn dial(&mut self, remote_addr: &str) -> Result<(), Box<dyn Error>> {
        let remote: Multiaddr = remote_addr.parse()?;
        self.swarm.dial(remote)?;
        Ok(())
    }

    pub fn send_request_to_sprout(
        &mut self,
        peer_id: PeerId,
        command: Command,
    ) -> Result<OutboundRequestId, Box<dyn Error>> {
        let request = SproutRequest {
            command,
            payload: String::new(),
        };
        let request_id = self
            .swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, request);
        Ok(request_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let sprout = Sprout::new();
        assert!(sprout.is_ok(), "Sprout::new should return Ok");
        let sprout = sprout.unwrap();
        assert_eq!(
            sprout.connected_peers.len(),
            0,
            "New sprout should have no connected peers"
        );
        assert_eq!(
            sprout.listeners().count(),
            0,
            "New sprout should have no listeners initially"
        );
    }

    #[tokio::test]
    async fn test_listen_on_valid_addr() {
        let mut sprout = Sprout::new().unwrap();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        let result = sprout.listen_on(addr.clone());
        assert!(
            result.is_ok(),
            "listen_on with valid address should succeed"
        );
    }

    #[test]
    fn test_peer_id() {
        let sprout = Sprout::new().unwrap();
        let peer_id = sprout.peer_id();
        assert!(peer_id.is_ok(), "peer_id should return Ok");
        assert_eq!(
            peer_id.unwrap(),
            *sprout.swarm.local_peer_id(),
            "peer_id should match swarm's local_peer_id"
        );
    }

    #[test]
    fn test_get_data_storage() {
        let sprout = Sprout::new().unwrap();
        let data_storage = sprout.get_data_storage();
        assert!(data_storage.is_ok(), "get_data_storage should return Ok");
        assert!(
            std::ptr::eq(data_storage.unwrap(), &sprout.data_storage),
            "get_data_storage should return reference to internal data_storage"
        );
    }

    #[test]
    fn test_get_passport() {
        let sprout = Sprout::new().unwrap();
        let passport = sprout.get_passport();
        assert!(passport.is_ok(), "get_passport should return Ok");
        assert!(
            std::ptr::eq(passport.unwrap(), &sprout.passport),
            "get_passport should return reference to internal passport"
        );
    }

    #[tokio::test]
    async fn test_dial_valid_addr() {
        let mut sprout = Sprout::new().unwrap();
        let valid_addr = "/ip4/127.0.0.1/tcp/12345";
        let result = sprout.dial(valid_addr);
        assert!(result.is_ok(), "dial with valid address should succeed");
    }

    #[tokio::test]
    async fn test_dial_invalid_addr() {
        let mut sprout = Sprout::new().unwrap();
        let invalid_addr = "invalid_addr";
        let result = sprout.dial(invalid_addr);
        assert!(result.is_err(), "dial with invalid address should fail");
    }

    #[tokio::test]
    async fn test_send_request_to_sprout() {
        let mut sprout = Sprout::new().unwrap();
        let peer_id = PeerId::random();
        let command = Command::String(StringCommand::Set {
            key: "key".to_string(),
            value: "value".to_string(),
        });
        let result = sprout.send_request_to_sprout(peer_id, command);
        assert!(result.is_ok(), "send_request_to_sprout should return Ok");
        let request_id = result.unwrap();
        assert!(
            request_id.to_string().len() > 0,
            "Request ID should be non-empty"
        );
    }
}
