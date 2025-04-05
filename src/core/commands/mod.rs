// SphagnumDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use crate::core::commands::{generic::GenericCommand, string::StringCommand};
use serde::{Deserialize, Serialize};

pub mod generic;
pub mod string;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    String(StringCommand),
    Generic(GenericCommand),
    // TODO
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResult {
    // todo: check other docs
    String(String),
    Int(u64),
    Bool(bool),
    Nil,
    Error(String),
}
