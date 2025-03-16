// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringCommand {
    Set { key: String, value: String },
    Get { key: String },
    Append { key: String, value: String },
    // TODO
}
