// SphagnumDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use crate::core::commands::{
    generic::GenericCommand, string::StringCommand, Command, CommandResult,
};
use crate::core::data_types::data_type::{DataType, GenericOperations};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
pub struct StringStore {
    data: HashMap<String, String>,
}

impl DataType for StringStore {
    fn new() -> Result<Self, Box<dyn Error>> {
        Ok(StringStore {
            data: HashMap::new(),
        })
    }

    fn handle_command(&mut self, command: Command) -> Result<CommandResult, Box<dyn Error>> {
        match command {
            Command::String(cmd) => match cmd {
                StringCommand::Set { key, value } => {
                    self.set(&key, &value)?;
                    Ok(CommandResult::String("OK".to_string()))
                }
                StringCommand::Get { key } => {
                    let result = self.get(&key)?;
                    Ok(result.map_or(CommandResult::Nil, |s| CommandResult::String(s.to_string())))
                }
                StringCommand::Append { key, value } => {
                    let len = self.append(&key, &value)?;
                    Ok(CommandResult::Int(len))
                }
            },
            Command::Generic(cmd) => match cmd {
                GenericCommand::Exists { keys } => {
                    let keys_ref: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
                    let result = self.exists(keys_ref)?;
                    Ok(CommandResult::Int(result))
                }
                GenericCommand::Delete { keys } => {
                    let keys_ref: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
                    let result = self.delete(keys_ref)?;
                    Ok(CommandResult::Int(result))
                }
            },
            #[allow(unreachable_patterns)]
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Command not supported by StringStore",
            ))),
        }
    }
}

impl StringStore {
    fn set(&mut self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        self.data.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<&str>, Box<dyn Error>> {
        Ok(self.data.get(key).map(|s| s.as_str()))
    }

    fn append(&mut self, key: &str, value: &str) -> Result<u64, Box<dyn Error>> {
        let entry = self.data.entry(key.to_string()).or_default();
        entry.push_str(&value);
        Ok(entry.len() as u64)
    }
}

impl GenericOperations for StringStore {
    fn exists(&self, keys: Vec<&str>) -> Result<u64, Box<dyn Error>> {
        let count = keys
            .iter()
            .filter(|&&key| self.data.contains_key(key))
            .count() as u64;
        Ok(count)
    }

    fn delete(&mut self, keys: Vec<&str>) -> Result<u64, Box<dyn Error>> {
        let mut count = 0;
        for key in keys {
            if self.data.remove(key).is_some() {
                count += 1;
            }
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        // Arrange
        let store = StringStore::new();
        // Act
        // Assert
        assert!(store.is_ok());
    }

    #[test]
    fn test_set_operation_with_valid_key_and_valid_value() {
        // Arrange
        let mut store = StringStore::new().unwrap();

        // Act
        let result = store.set("key", "value");

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_operation_with_non_existent_key() {
        // Arrange
        let store = StringStore::new().unwrap();

        // Act
        let result = store.get("key");

        // Assert
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_get_operation_with_existent_key() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        store.set("key", "value").unwrap();

        // Act
        let result = store.get("key");

        // Assert
        assert_eq!(result.unwrap(), Some("value"));
    }

    #[test]
    fn test_get_operations_with_same_existent_keys() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        store.set("key", "value").unwrap();

        // Act
        let result_1 = store.get("key");
        let result_2 = store.get("key");

        // Assert
        assert_eq!(result_1.unwrap(), Some("value"));
        assert_eq!(result_2.unwrap(), Some("value"));
    }

    #[test]
    fn test_set_operations_changes_the_values_for_the_same_keys() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        let key = "key";
        let value_1 = "value1";
        let value_2 = "value2";

        // Act & Assert
        store.set(key, value_1).unwrap();
        let result_1 = store.get(key);
        assert_eq!(result_1.unwrap(), Some(value_1));

        store.set(key, value_2).unwrap();
        let result_2 = store.get(key);
        assert_eq!(result_2.unwrap(), Some(value_2));
    }

    #[test]
    fn test_set_operations_for_the_different_keys() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        let key_1 = "key1";
        let key_2 = "key2";
        let value_1 = "value1";
        let value_2 = "value2";

        // Act
        store.set(key_1, value_1).unwrap();
        store.set(key_2, value_2).unwrap();
        let result_1 = store.get(key_1);
        let result_2 = store.get(key_2);

        // Assert
        assert_eq!(result_1.unwrap(), Some(value_1));
        assert_eq!(result_2.unwrap(), Some(value_2));
    }

    #[test]
    fn test_append_command_creates_new_key_on_non_existent_key() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        store.append("key", "value").unwrap();

        // Act
        let result = store.get("key");

        // Assert
        assert_eq!(result.unwrap(), Some("value"));
    }

    #[test]
    fn test_append_operation_with_valid_key_and_valid_value_appends_this_value() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        let key = "key";
        let value_1 = "value1";
        let value_2 = "value2";

        // Act & Assert
        store.append(key, value_1).unwrap();
        let result_1 = store.get(key);
        assert_eq!(result_1.unwrap(), Some(value_1));

        store.append(key, value_2).unwrap();
        let result_2 = store.get(key);
        assert_eq!(
            result_2.unwrap(),
            Some(format!("{}{}", value_1, value_2).as_str())
        );
    }

    #[test]
    fn test_exists_command_for_non_existent_key() {
        // Arrange
        let store = StringStore::new().unwrap();

        // Act
        let result = store.exists(vec!["key"]).unwrap();

        // Assert
        assert_eq!(result, 0);
    }

    #[test]
    fn test_exists_command_for_existent_key() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        store.set("key", "value").unwrap();

        // Act
        let result = store.exists(vec!["key"]).unwrap();

        // Assert
        assert_eq!(result, 1);
    }

    #[test]
    fn test_exists_command_for_ranges_of_existent_keys() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        store.set("key1", "value1").unwrap();
        store.set("key2", "value2").unwrap();
        store.set("key3", "value3").unwrap();
        store.set("key4", "value4").unwrap();
        store.set("key5", "value5").unwrap();

        // Act
        let result_1 = store
            .exists(vec!["key1", "key2", "key3", "key4", "key5"])
            .unwrap();
        let result_2 = store.exists(vec!["key1", "key2", "key3", "key4"]).unwrap();
        let result_3 = store.exists(vec!["key1", "key2", "key3"]).unwrap();
        let result_4 = store.exists(vec!["key1", "key2"]).unwrap();
        let result_5 = store.exists(vec!["key1"]).unwrap();
        let result_6 = store.exists(vec![]).unwrap();

        // Assert
        assert_eq!(result_1, 5);
        assert_eq!(result_2, 4);
        assert_eq!(result_3, 3);
        assert_eq!(result_4, 2);
        assert_eq!(result_5, 1);
        assert_eq!(result_6, 0);
    }

    #[test]
    fn test_exists_command_for_ranges_of_existent_and_non_existent_keys() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        store.set("key1", "value1").unwrap();
        store.set("key2", "value2").unwrap();
        store.set("key3", "value3").unwrap();

        // Act
        let result_1 = store.exists(vec!["key1", "key2"]).unwrap();
        let result_2 = store.exists(vec!["key1", "key2", "key3", "key4"]).unwrap();
        let result_3 = store.exists(vec!["key3", "key4", "key5"]).unwrap();
        let result_4 = store.exists(vec!["key4", "key5"]).unwrap();
        let result_5 = store.exists(vec!["key1", "key5"]).unwrap();

        // Assert
        assert_eq!(result_1, 2);
        assert_eq!(result_2, 3);
        assert_eq!(result_3, 1);
        assert_eq!(result_4, 0);
        assert_eq!(result_5, 1);
    }

    #[test]
    fn test_delete_command_for_non_existent_key() {
        // Arrange
        let mut store = StringStore::new().unwrap();

        // Act
        let result = store.delete(vec!["key"]).unwrap();

        // Assert
        assert_eq!(result, 0);
    }

    #[test]
    fn test_delete_command_for_existent_key() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        store.set("key", "value").unwrap();

        // Act
        let result = store.delete(vec!["key"]).unwrap();

        // Assert
        assert_eq!(result, 1);
    }

    #[test]
    fn test_delete_command_for_ranges_of_existent_keys() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        store.set("key1", "value1").unwrap();
        store.set("key2", "value2").unwrap();
        store.set("key3", "value3").unwrap();
        store.set("key4", "value4").unwrap();
        store.set("key5", "value5").unwrap();
        store.set("key6", "value6").unwrap();

        // Act
        let result_1 = store.delete(vec!["key1", "key2", "key3"]).unwrap();
        let result_2 = store.delete(vec!["key4", "key5"]).unwrap();
        let result_3 = store.delete(vec!["key6"]).unwrap();

        // Assert
        assert_eq!(result_1, 3);
        assert_eq!(result_2, 2);
        assert_eq!(result_3, 1);
    }

    #[test]
    fn test_delete_command_for_ranges_of_existent_and_non_existent_keys() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        store.set("key1", "value1").unwrap();
        store.set("key2", "value2").unwrap();
        store.set("key3", "value3").unwrap();

        // Act
        let result_1 = store.delete(vec!["key1", "key2"]).unwrap();
        let result_2 = store.delete(vec!["key1", "key2", "key3", "key4"]).unwrap();
        let result_3 = store.delete(vec!["key3", "key4", "key5"]).unwrap();
        let result_4 = store.delete(vec!["key1", "key5"]).unwrap();

        // Assert
        assert_eq!(result_1, 2);
        assert_eq!(result_2, 1);
        assert_eq!(result_3, 0);
        assert_eq!(result_4, 0);
    }

    // After the tests for private methods are completed, we do the tests for the public command handler.
    #[test]
    fn test_handle_command_set_with_valid_key_and_value() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        let command = Command::String(StringCommand::Set {
            key: "key".to_string(),
            value: "value".to_string(),
        });

        // Act
        let result = store.handle_command(command).unwrap();
        let get_result = store.get("key").unwrap();

        // Assert
        assert_eq!(result, CommandResult::String("OK".to_string()));
        assert_eq!(get_result, Some("value"));
    }

    #[test]
    fn test_handle_command_get_with_existent_key() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        let command = Command::String(StringCommand::Get {
            key: "key".to_string(),
        });

        // Act
        store.set("key", "value").unwrap();
        let result = store.handle_command(command).unwrap();

        // Assert
        assert_eq!(result, CommandResult::String("value".to_string()));
    }

    #[test]
    fn test_handle_command_get_with_non_existent_key() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        let command = Command::String(StringCommand::Get {
            key: "key".to_string(),
        });

        // Act
        let result = store.handle_command(command).unwrap();

        // Assert
        assert_eq!(result, CommandResult::Nil);
    }

    #[test]
    fn test_handle_command_append_with_non_existent_key() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        let command = Command::String(StringCommand::Append {
            key: "key".to_string(),
            value: "value".to_string(),
        });

        // Act
        let result = store.handle_command(command).unwrap();
        let get_result = store.get("key").unwrap();

        // Assert
        assert_eq!(result, CommandResult::Int("value".len() as u64));
        assert_eq!(get_result, Some("value"));
    }

    #[test]
    fn test_handle_command_append_with_existent_key() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        let command = Command::String(StringCommand::Append {
            key: "key".to_string(),
            value: "value2".to_string(),
        });

        // Act
        store.set("key", "value1").unwrap();
        let result = store.handle_command(command).unwrap();
        let get_result = store.get("key").unwrap();

        // Assert
        assert_eq!(result, CommandResult::Int("value1value2".len() as u64));
        assert_eq!(get_result, Some("value1value2"));
    }

    #[test]
    fn test_handle_command_exists_with_existent_and_non_existent_keys() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        let command = Command::Generic(GenericCommand::Exists {
            keys: vec![
                "key1".to_string(),
                "key2".to_string(),
                "key3".to_string(),
                "key4".to_string(),
            ],
        });

        // Act
        store.set("key1", "value1").unwrap();
        store.set("key2", "value2").unwrap();
        let result = store.handle_command(command).unwrap();

        // Assert
        assert_eq!(result, CommandResult::Int(2));
    }

    #[test]
    fn test_handle_command_delete_with_existent_and_non_existent_keys() {
        // Arrange
        let mut store = StringStore::new().unwrap();
        let command = Command::Generic(GenericCommand::Delete {
            keys: vec!["key1".to_string(), "key2".to_string(), "key3".to_string()],
        });

        // Act
        store.set("key1", "value1").unwrap();
        store.set("key2", "value2").unwrap();
        let result = store.handle_command(command).unwrap();

        // Assert
        assert_eq!(result, CommandResult::Int(2));
        assert_eq!(store.get("key1").unwrap(), None);
        assert_eq!(store.get("key2").unwrap(), None);
        assert_eq!(store.get("key3").unwrap(), None);
    }
}
