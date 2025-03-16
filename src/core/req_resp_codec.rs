// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use super::commands::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SproutRequest {
    pub command: Command,
    pub payload: String, // leave it for compatibility, but maybe we don't use it yet
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SproutResponse {
    pub payload: String,
}
