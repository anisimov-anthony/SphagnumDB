// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use libp2p::{ping, request_response};
use libp2p_swarm_derive::NetworkBehaviour;

use super::req_resp_codec::{SproutRequest, SproutResponse};

#[derive(NetworkBehaviour)]
pub struct SproutBehaviour {
    pub ping: ping::Behaviour,
    pub request_response: request_response::json::Behaviour<SproutRequest, SproutResponse>, // firstly, codec is only json
}
