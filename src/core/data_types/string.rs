// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use crate::core::commands::{generic::GenericCommand, string::StringCommand, Command};
use crate::core::data_types::data_type::{DataType, GenericOperations};
use std::any::Any;
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

    fn handle_command(&mut self, command: Command) -> Result<Box<dyn Any>, Box<dyn Error>> {
        match command {
            Command::String(cmd) => match cmd {
                StringCommand::Set { key, value } => {
                    self.set(key, value)?;
                    Ok(Box::new("OK".to_string()))
                }
                StringCommand::Get { key } => {
                    let result = self.get(&key)?;
                    Ok(Box::new(result))
                }
                StringCommand::Append { key, value } => {
                    let len = self.append(key, value)?;
                    Ok(Box::new(len))
                }
            },
            Command::Generic(cmd) => match cmd {
                GenericCommand::Exists { keys } => {
                    let keys_ref: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
                    let result = self.exists(keys_ref)?;
                    Ok(Box::new(result))
                }
                GenericCommand::Delete { keys } => {
                    let keys_ref: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
                    let result = self.delete(keys_ref)?;
                    Ok(Box::new(result))
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
    fn set(&mut self, key: String, value: String) -> Result<(), Box<dyn Error>> {
        self.data.insert(key, value);
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        Ok(self.data.get(key).cloned())
    }

    fn append(&mut self, key: String, value: String) -> Result<u64, Box<dyn Error>> {
        let entry = self.data.entry(key).or_insert_with(String::new);
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
