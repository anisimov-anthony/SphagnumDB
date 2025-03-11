// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Get,
    Set,
    // other commands will added soon
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SproutRequest {
    pub command : Command,
    pub payload : String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SproutResponse {
    pub payload : String,
}
