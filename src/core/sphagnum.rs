// SphagnumDB
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
    commands::{generic::GenericCommand, string::StringCommand, Command, CommandResult},
    data_storage::DataStorage,
    passport::Passport,
    req_resp_codec::{SphagnumRequest, SphagnumResponse},
    sphagnum_behaviour::{SphagnumBehaviour, SphagnumBehaviourEvent},
};

/// Reminder: in this project, the nodes are called sphagnums. Thus, this structure is a node
/// structure. At this stage, this is a highly simplified representation of the node, and it will be
/// further refined.
pub struct SphagnumNode {
    data_storage: DataStorage,
    passport: Passport,
    pub swarm: Swarm<SphagnumBehaviour>, // todo remove pub
    pub connected_peers: HashSet<PeerId>,
    is_pinging_output_enabled: bool,

    /// Multiple nodes to which data will be replicated
    replica_set: HashSet<PeerId>,
}

impl SphagnumNode {
    pub fn new() -> Result<SphagnumNode, Box<dyn Error>> {
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

        Ok(SphagnumNode {
            data_storage: DataStorage::new()?,
            passport: Passport::new()?,
            swarm,
            connected_peers: HashSet::new(),
            is_pinging_output_enabled: false,
            replica_set: HashSet::new(),
        })
    }

    fn configure_behaviours() -> Result<SphagnumBehaviour, Box<dyn Error>> {
        let ping = ping::Behaviour::default();
        let request_response = request_response::json::Behaviour::new(
            [(
                StreamProtocol::new("/SphagnumDB/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default(),
        );

        Ok(SphagnumBehaviour {
            ping,
            request_response,
        })
    }

    pub fn enable_pinging_output(&mut self) {
        if !self.is_pinging_output_enabled {
            self.is_pinging_output_enabled = true;
            println!("Pinging enabled");
        }
    }

    pub fn disable_pinging_output(&mut self) {
        if self.is_pinging_output_enabled {
            self.is_pinging_output_enabled = false;
            println!("Pinging disabled");
        }
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

    pub fn get_passport(&self) -> Result<&Passport, Box<dyn Error>> {
        Ok(&self.passport)
    }

    pub fn add_to_replica_set(&mut self, peer_id: PeerId) -> Result<(), Box<dyn Error>> {
        self.replica_set.insert(peer_id);
        Ok(())
    }

    // todo: async
    async fn send_to_replicas(&mut self, command: Command) -> Result<(), Box<dyn Error>> {
        let self_id = self.peer_id()?;
        let peers_to_replicate: Vec<PeerId> = self
            .replica_set
            .iter()
            .filter(|&peer_id| peer_id != &self_id && self.connected_peers.contains(peer_id))
            .copied()
            .collect();

        for peer_id in peers_to_replicate {
            let request = SphagnumRequest {
                command: command.clone(),
                payload: String::new(),
                is_replication: true,
            };

            self.swarm
                .behaviour_mut()
                .request_response
                .send_request(&peer_id, request);
        }
        Ok(())
    }

    // todo: redesign
    // warning: no replication, if you want replication - use handle_event
    pub fn handle_command(&mut self, command: Command) -> Result<CommandResult, Box<dyn Error>> {
        self.data_storage
            .handle_command(command)
            .map_err(|e| Box::new(e) as Box<dyn Error>)
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
            SwarmEvent::Behaviour(SphagnumBehaviourEvent::Ping(event)) => {
                if self.is_pinging_output_enabled {
                    let ping::Event { peer, result, .. } = event;
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
                }
                Ok(())
            }
            SwarmEvent::Behaviour(SphagnumBehaviourEvent::RequestResponse(event)) => {
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

                            let command_to_replicate = request.command.clone();
                            let response = match request.command {
                                Command::String(StringCommand::Set { key, value }) => {
                                    match self.data_storage.handle_command(Command::String(
                                        StringCommand::Set { key, value },
                                    )) {
                                        Ok(CommandResult::String(ok)) => {
                                            if ok == "OK" && !request.is_replication {
                                                if let Err(e) = self
                                                    .send_to_replicas(command_to_replicate)
                                                    .await
                                                {
                                                    println!("Replication failed: {:?}", e);
                                                }
                                            }
                                            SphagnumResponse { payload: ok }
                                        }
                                        Ok(_) => SphagnumResponse {
                                            payload: "Unexpected response".to_string(),
                                        },
                                        Err(e) => SphagnumResponse {
                                            payload: format!("Error setting value: {:?}", e),
                                        },
                                    }
                                }
                                Command::String(StringCommand::Get { key }) => {
                                    match self
                                        .data_storage
                                        .handle_command(Command::String(StringCommand::Get { key }))
                                    {
                                        Ok(CommandResult::String(value)) => {
                                            SphagnumResponse { payload: value }
                                        }
                                        Ok(CommandResult::Nil) => SphagnumResponse {
                                            payload: "nil".to_string(),
                                        },
                                        Ok(_) => SphagnumResponse {
                                            payload: "Unexpected response".to_string(),
                                        },
                                        Err(e) => SphagnumResponse {
                                            payload: format!("Error getting value: {:?}", e),
                                        },
                                    }
                                }
                                Command::String(StringCommand::Append { key, value }) => {
                                    match self.data_storage.handle_command(Command::String(
                                        StringCommand::Append { key, value },
                                    )) {
                                        Ok(CommandResult::Int(len)) => {
                                            if !request.is_replication {
                                                if let Err(e) = self
                                                    .send_to_replicas(command_to_replicate)
                                                    .await
                                                {
                                                    println!("Replication failed: {:?}", e);
                                                }
                                            }
                                            SphagnumResponse {
                                                payload: len.to_string(),
                                            }
                                        }
                                        Ok(_) => SphagnumResponse {
                                            payload: "Unexpected response".to_string(),
                                        },
                                        Err(e) => SphagnumResponse {
                                            payload: format!("Error appending value: {:?}", e),
                                        },
                                    }
                                }
                                Command::Generic(GenericCommand::Exists { keys }) => {
                                    match self.data_storage.handle_command(Command::Generic(
                                        GenericCommand::Exists { keys },
                                    )) {
                                        Ok(CommandResult::Int(count)) => {
                                            if !request.is_replication {
                                                if let Err(e) = self
                                                    .send_to_replicas(command_to_replicate)
                                                    .await
                                                {
                                                    println!("Replication failed: {:?}", e);
                                                }
                                            }
                                            SphagnumResponse {
                                                payload: count.to_string(),
                                            }
                                        }
                                        Ok(_) => SphagnumResponse {
                                            payload: "Unexpected response".to_string(),
                                        },
                                        Err(e) => SphagnumResponse {
                                            payload: format!("Error checking existence: {:?}", e),
                                        },
                                    }
                                }
                                Command::Generic(GenericCommand::Delete { keys }) => {
                                    match self.data_storage.handle_command(Command::Generic(
                                        GenericCommand::Delete { keys },
                                    )) {
                                        Ok(CommandResult::Int(count)) => {
                                            if !request.is_replication {
                                                if let Err(e) = self
                                                    .send_to_replicas(command_to_replicate)
                                                    .await
                                                {
                                                    println!("Replication failed: {:?}", e);
                                                }
                                            }
                                            SphagnumResponse {
                                                payload: count.to_string(),
                                            }
                                        }
                                        Ok(_) => SphagnumResponse {
                                            payload: "Unexpected response".to_string(),
                                        },
                                        Err(e) => SphagnumResponse {
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

    pub async fn send_request_to_sphagnum(
        &mut self,
        peer_id: PeerId,
        command: Command,
    ) -> Result<OutboundRequestId, Box<dyn Error>> {
        let request = SphagnumRequest {
            command,
            payload: String::new(),
            is_replication: false, // by default
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
        let sphagnum = SphagnumNode::new();
        assert!(sphagnum.is_ok(), "SphagnumNode::new should return Ok");
        let sphagnum = sphagnum.unwrap();
        assert_eq!(
            sphagnum.connected_peers.len(),
            0,
            "New sphagnum should have no connected peers"
        );
        assert_eq!(
            sphagnum.listeners().count(),
            0,
            "New sphagnum should have no listeners initially"
        );
    }

    #[tokio::test]
    async fn test_listen_on_valid_addr() {
        let mut sphagnum = SphagnumNode::new().unwrap();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        let result = sphagnum.listen_on(addr.clone());
        assert!(
            result.is_ok(),
            "listen_on with valid address should succeed"
        );
    }

    #[test]
    fn test_peer_id() {
        let sphagnum = SphagnumNode::new().unwrap();
        let peer_id = sphagnum.peer_id();
        assert!(peer_id.is_ok(), "peer_id should return Ok");
        assert_eq!(
            peer_id.unwrap(),
            *sphagnum.swarm.local_peer_id(),
            "peer_id should match swarm's local_peer_id"
        );
    }

    #[test]
    fn test_get_passport() {
        let sphagnum = SphagnumNode::new().unwrap();
        let passport = sphagnum.get_passport();
        assert!(passport.is_ok(), "get_passport should return Ok");
        assert!(
            std::ptr::eq(passport.unwrap(), &sphagnum.passport),
            "get_passport should return reference to internal passport"
        );
    }

    #[tokio::test]
    async fn test_dial_valid_addr() {
        let mut sphagnum = SphagnumNode::new().unwrap();
        let valid_addr = "/ip4/127.0.0.1/tcp/12345";
        let result = sphagnum.dial(valid_addr);
        assert!(result.is_ok(), "dial with valid address should succeed");
    }

    #[tokio::test]
    async fn test_dial_invalid_addr() {
        let mut sphagnum = SphagnumNode::new().unwrap();
        let invalid_addr = "invalid_addr";
        let result = sphagnum.dial(invalid_addr);
        assert!(result.is_err(), "dial with invalid address should fail");
    }

    #[tokio::test]
    async fn test_send_request_to_sphagnum() {
        let mut sphagnum = SphagnumNode::new().unwrap();
        let peer_id = PeerId::random();
        let command = Command::String(StringCommand::Set {
            key: "key".to_string(),
            value: "value".to_string(),
        });
        let result = sphagnum.send_request_to_sphagnum(peer_id, command).await;
        assert!(result.is_ok(), "send_request_to_sphagnum should return Ok");
        let request_id = result.unwrap();
        assert!(
            request_id.to_string().len() > 0,
            "Request ID should be non-empty"
        );
    }
}
