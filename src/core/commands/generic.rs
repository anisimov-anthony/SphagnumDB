// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenericCommand {
    Exists { keys: Vec<String> },
    Delete { keys: Vec<String> },
    // TODO
}
