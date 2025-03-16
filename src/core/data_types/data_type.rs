// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use crate::core::commands::Command;
use std::any::Any;
use std::error::Error;

/// Generic methods for all Data Types.
pub trait GenericOperations {
    /// Returns the number of keys that exist from those specified.
    /// If the same key is provided multiple times, it is counted multiple times.
    fn exists(&self, keys: Vec<&str>) -> Result<u64, Box<dyn Error>>;

    /// Deletes the specified keys and returns the number of keys that were removed.
    /// A key is ignored if it does not exist.
    fn delete(&mut self, keys: Vec<&str>) -> Result<u64, Box<dyn Error>>;
}

/// Base trait for all Data Types.
pub trait DataType: std::fmt::Debug + Send + GenericOperations {
    /// Creates a new instance of the data type.
    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;

    /// Handles a command and returns the result.
    fn handle_command(&mut self, command: Command) -> Result<Box<dyn Any>, Box<dyn Error>>;
    // TODO
}
