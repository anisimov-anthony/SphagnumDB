// SphagnumDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use std::{any::Any, error::Error, fmt};

use super::data_types::{data_type::DataType, string::StringStore};
use crate::core::commands::Command;

#[derive(Debug)]
pub enum DataStorageError {
    InitializationError,
    DataRetrievalError,
    DataModificationError,
}

impl fmt::Display for DataStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataStorageError::InitializationError => write!(f, "Failed to initialize DataStorage"),
            DataStorageError::DataRetrievalError => write!(f, "Failed to retrieve data"),
            DataStorageError::DataModificationError => write!(f, "Failed to modify data"),
        }
    }
}

impl Error for DataStorageError {}

/// To work with the data that will be stored on the node.
/// At this stage, it's a simple mock, which is still far from a hashmap, but it's enough for the
/// initial stage.
pub struct DataStorage {
    storage: Box<dyn DataType>,
}

impl DataStorage {
    pub fn new() -> Result<Self, DataStorageError> {
        let store = StringStore::new().map_err(|_| DataStorageError::InitializationError)?;
        Ok(Self {
            storage: Box::new(store),
        })
    }

    pub fn handle_command(&mut self, command: Command) -> Result<Box<dyn Any>, DataStorageError> {
        self.storage
            .handle_command(command)
            .map_err(|_| DataStorageError::DataModificationError)
    }
}
