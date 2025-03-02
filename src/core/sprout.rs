// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use std::{error::Error, time::Duration};

use futures::prelude::*;
use libp2p::{
    noise, ping,
    swarm::{Swarm, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId,
    request_response::{self, ProtocolSupport, OutboundRequestId},
    StreamProtocol,
};

use super::{
    data_storage::DataStorage, 
    passport::Passport,
    req_resp_codec::{SproutRequest, SproutResponse},
    sprout_behaviour::{SproutBehaviour, SproutBehaviourEvent}
};

/// Reminder: in this project, the nodes are called sprouts. Thus, this structure is a node
/// structure. At this stage, this is a highly simplified representation of the node, and it will be
/// further refined.
pub struct Sprout {
    data_storage: DataStorage,
    passport: Passport,
    swarm: Swarm<SproutBehaviour>,
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
        })
    }

    fn configure_behaviours() -> Result<SproutBehaviour, Box<dyn Error>> {
        let ping = ping::Behaviour::default();
        let request_response = request_response::json::Behaviour::new(
            [(
                StreamProtocol::new("/greet/1.0.0"),
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

            // Events for SwarmEvent are handled here.
            SwarmEvent::ConnectionEstablished { 
                peer_id, 
                connection_id, 
                endpoint, 
                num_established, 
                concurrent_dial_errors, 
                established_in } => {
                    if endpoint.is_dialer() {
                        println!(
                            "Node {} successfully dialed {} (connection_id: {:?})", 
                            self.swarm.local_peer_id(), peer_id, connection_id
                        );
                    } else {
                        println!(
                            "Node {} accepted connection from {} (connection_id: {:?}, num_established: {}, established_in: {:?})", 
                            self.swarm.local_peer_id(), peer_id, connection_id, num_established, established_in
                        );
                    }

                    if let Some(errors) = concurrent_dial_errors {
                        for (addr, err) in errors {
                            println!(
                                "Dial attempt to {:?} failed with error: {:?}", 
                                addr, err
                            );
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
                cause } => {
                    println!(
                        "Node {} closed connection with {} (connection_id: {:?}, endpoint: {:?}, num_established: {})", 
                        self.swarm.local_peer_id(), peer_id, connection_id, endpoint, num_established
                    );
                    if let Some(err) = cause {
                        println!("Cause of disconnection: {:?}", err);
                    }
                    Ok(())
                }
            SwarmEvent::NewListenAddr { 
                listener_id, 
                address } => {
                    println!(
                        "Node {} is now listening on {:?} with listener ID: {:?}", 
                        self.swarm.local_peer_id(), address, listener_id
                    );
                Ok(())
            }
            SwarmEvent::ListenerClosed { 
                listener_id, 
                addresses, 
                reason } => {
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
            SwarmEvent::ListenerError { 
                listener_id, 
                error } => {
                    println!(
                        "Listener {} encountered an error: {:?}", 
                        listener_id, error
                    );
                Ok(())
            }
            SwarmEvent::Dialing { 
                peer_id, 
                connection_id } => {
                    println!(
                        "Node {} is dialing peer {:?} (connection_id: {:?})", 
                        self.swarm.local_peer_id(), peer_id, connection_id
                    );
                Ok(())
            }
            SwarmEvent::NewExternalAddrCandidate { 
                address } => {
                    println!(
                        "Node {} discovered a new external address: {:?}", 
                        self.swarm.local_peer_id(), address
                    );
                Ok(())
            }
            SwarmEvent::ExternalAddrConfirmed { 
                address } => {
                    println!(
                        "Node {} confirmed external address: {:?}", 
                        self.swarm.local_peer_id(), address
                    );
                Ok(())
            }
            SwarmEvent::ExternalAddrExpired { 
                address } => {
                    println!(
                        "Node {} detected the expiration of external address: {:?}", 
                        self.swarm.local_peer_id(), address
                    );
                Ok(())
            }
            SwarmEvent::NewExternalAddrOfPeer { 
                peer_id, 
                address } => {
                    println!(
                        "Node {} discovered a new address for peer {:?}: {:?}", 
                        self.swarm.local_peer_id(), peer_id, address
                    );
                Ok(())
            }

            // All possible events (there is only one event) for SproutBehaviourEvent::Ping::Event are handled here.
            SwarmEvent::Behaviour(SproutBehaviourEvent::Ping(event)) => {
                match event {
                    ping::Event { peer, result, .. } => {
                        match result {
                            Ok(rtt) => println!(
                                "Node {} received ping from {}: {:?}", self.swarm.local_peer_id(), 
                                peer, rtt
                            ),
                            Err(e) => println!(
                                "Node {} failed to ping {}: {:?}", 
                                self.swarm.local_peer_id(), peer, e
                            ),
                        }
                    }
                }
                Ok(())
            }

            // All possible events for request_response::Event are handled here.
            SwarmEvent::Behaviour(SproutBehaviourEvent::RequestResponse(event)) => {
                match event {
                    request_response::Event::Message { 
                        peer, 
                        connection_id, 
                        message } => {
                            match message {
                                request_response::Message::Request { 
                                    request_id,
                                    request, 
                                    channel } => {
                                        println!(
                                            "Node {} received request from {} (connection: {:?}, request_id: {:?}): {:?}",
                                            self.swarm.local_peer_id(), peer, connection_id, request_id, request
                                        );

                                        // The simplest option is to echo responses. In the future, it will be possible to make more interesting behavior.
                                        let response = SproutResponse {
                                            payload : format!("Response to the request '{}'", request.payload ),
                                        };
                                        self.swarm
                                            .behaviour_mut()
                                            .request_response
                                            .send_response(channel, response)
                                            .unwrap();
                                    }
                                request_response::Message::Response { 
                                    request_id,
                                    response } => {
                                        println!(
                                            "Node {} received response from {} (connection: {:?}, request_id: {:?}): {:?}",
                                            self.swarm.local_peer_id(), peer, connection_id, request_id, response
                                        );
                                    }
                                }
                            }
                    request_response::Event::OutboundFailure { 
                        peer, 
                        connection_id, 
                        request_id, 
                        error } => {
                            println!(
                                "Node {} outbound request to {} (connection: {:?}, request: {:?}) failed: {:?}",
                                self.swarm.local_peer_id(), peer, connection_id, request_id, error
                            );
                        }
                    request_response::Event::InboundFailure { 
                        peer, 
                        connection_id, 
                        request_id, 
                        error } => {
                            println!(
                                "Node {} inbound request from {} (connection: {:?}, request: {:?}) failed: {:?}",
                                self.swarm.local_peer_id(), peer, connection_id, request_id, error
                            );
                        }
                    request_response::Event::ResponseSent { 
                        peer, 
                        connection_id, 
                        request_id } => {
                            println!(
                                "Node {} sent response to {} (connection: {:?}, request: {:?})",
                                self.swarm.local_peer_id(), peer, connection_id, request_id
                            );
                        }
                }
                Ok(())
            }

            _ => {
                println!("Unhandled event for SwarmEvent: {:?}", self.swarm.select_next_some().await);
                Ok(())
            }
        }
    }

    pub fn dial(&mut self, remote_addr: &str) -> Result<(), Box<dyn Error>> {
        let remote: Multiaddr = remote_addr.parse()?;
        self.swarm.dial(remote)?;
        Ok(())
    }

    pub fn send_request_to_sprout(&mut self, peer_id: PeerId, payload : String) -> Result<OutboundRequestId, Box<dyn Error>> {
        let request = SproutRequest { payload  };
        let request_id = self.swarm
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
    fn test_sprout_creation() {
        let sprout = Sprout::new();
        assert!(sprout.is_ok());
    }

    #[test]
    fn test_get_data_storage() {
        let sprout = Sprout::new().unwrap();
        assert!(sprout.get_data_storage().is_ok());
    }

    #[test]
    fn test_get_passport() {
        let sprout = Sprout::new().unwrap();
        assert!(sprout.get_passport().is_ok());
    }

    #[tokio::test]
    async fn test_listen_on() {
        let mut sprout = Sprout::new().unwrap();
        let listen_addr = "/ip4/127.0.0.1/tcp/4001".parse().unwrap();
        let result = sprout.listen_on(listen_addr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_peer_id() {
        let sprout = Sprout::new().unwrap();
        let peer_id = sprout.peer_id();
        assert!(peer_id.is_ok());
    }

    #[tokio::test]
    async fn test_dial() {
        let mut sprout = Sprout::new().unwrap();
        let remote_addr = "/ip4/127.0.0.1/tcp/4002";
        let result = sprout.dial(remote_addr);
        assert!(result.is_ok());
    }
}
