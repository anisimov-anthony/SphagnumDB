// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use std::{error::Error, time::Duration};

use futures::prelude::*;
use libp2p::{
    noise, ping,
    swarm::{Swarm, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId,
};

use super::{data_storage::DataStorage, passport::Passport};

/// Reminder: in this project, the nodes are called sprouts. Thus, this structure is a node
/// structure. At this stage, this is a highly simplified representation of the node, and it will be
/// further refined.
pub struct Sprout {
    /// Data storage for the sprout node.
    data_storage: DataStorage,

    /// Information for identifying this node
    passport: Passport,

    swarm: Swarm<ping::Behaviour>, // firstly, only ping is implemented
}

impl Sprout {
    /// Creates a new `Sprout` with default data storage and configuration.
    pub fn new() -> Result<Sprout, Box<dyn Error>> {
        // Create a new Swarm with ping behavior
        let swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| ping::Behaviour::default())?
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

    /// Sets the address for the node to listen on.
    pub fn listen_on(&mut self, listen_addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm.listen_on(listen_addr)?;
        Ok(())
    }

    /// Returns the `PeerId` of this node.
    pub fn peer_id(&self) -> Result<PeerId, Box<dyn Error>> {
        Ok(*self.swarm.local_peer_id())
    }

    /// Returns a reference to the data storage.
    pub fn get_data_storage(&self) -> Result<&DataStorage, Box<dyn Error>> {
        Ok(&self.data_storage)
    }

    /// Returns a reference to the passport.
    pub fn get_passport(&self) -> Result<&Passport, Box<dyn Error>> {
        Ok(&self.passport)
    }

    /// Runs the Swarm event loop.
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(event) => {
                    println!("Ping event: {:?}", event);
                }
                _ => {}
            }
        }
    }

    /// Dials a remote peer.
    pub fn dial(&mut self, remote_addr: &str) -> Result<(), Box<dyn Error>> {
        let remote: Multiaddr = remote_addr.parse()?;
        self.swarm.dial(remote)?;
        println!("Dialed {}", remote_addr);
        Ok(())
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
