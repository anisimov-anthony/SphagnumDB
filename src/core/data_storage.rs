// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use std::{error::Error, fmt};

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
    number: Option<i32>, // mock
}

impl DataStorage {
    /// Creates a new `DataStorage` with a default number value.
    pub fn new() -> Result<Self, DataStorageError> {
        Ok(Self { number: Some(0) })
    }

    /// Returns the number that is stored
    pub fn get_number(&self) -> Result<i32, DataStorageError> {
        self.number.ok_or(DataStorageError::DataRetrievalError)
    }

    /// Sets the number to be stored
    pub fn set_number(&mut self, new_value: i32) -> Result<(), DataStorageError> {
        if let Some(num) = &mut self.number {
            *num = new_value;
            Ok(())
        } else {
            Err(DataStorageError::DataModificationError)
        }
    }

    /// Resetting the number
    pub fn reset_number(&mut self) {
        self.number = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let storage = DataStorage::new().unwrap();
        assert_eq!(storage.get_number().unwrap(), 0);
    }

    #[test]
    fn test_get_number() {
        let storage = DataStorage { number: Some(111) };
        assert_eq!(storage.get_number().unwrap(), 111);

        let storage = DataStorage { number: None };
        assert!(storage.get_number().is_err());
    }

    #[test]
    fn test_set_number() {
        let mut storage = DataStorage { number: Some(0) };
        assert!(storage.set_number(111).is_ok());
        assert_eq!(storage.get_number().unwrap(), 111);

        let mut storage = DataStorage { number: None };
        assert!(storage.set_number(111).is_err());
    }

    #[test]
    fn test_reset_number() {
        let mut storage = DataStorage { number: Some(111) };
        storage.reset_number();
        assert!(storage.get_number().is_err());
    }
}
