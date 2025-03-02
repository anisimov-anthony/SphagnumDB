// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SproutRequest {
    pub payload : String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SproutResponse {
    pub payload : String,
}
