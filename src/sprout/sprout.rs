// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use crate::sprout::data_storage::DataStorage;
use crate::sprout::passport::Passport;

/// Reminder: in this project, the nodes are called sprouts. Thus, this structure is a node structure.
/// At this stage, this is a highly simplified representation of the node, and it will be further refined.
pub struct Sprout {

    /// Data storage for the sprout node.
    data_storage: DataStorage,

    /// Information for identifying this node
    passport: Passport
}

impl Sprout {

    /// Creates a new `Sprout` with default data storage and configuration.
    pub fn new() -> Sprout {
        Sprout {
            data_storage: DataStorage::new(),
            passport: Passport::new()
    
        }
    }

    pub fn get_data_storage(&self) -> &DataStorage {
        &self.data_storage
    }
    pub fn get_passport(&self) -> &Passport {
        &self.passport
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns() {
        let sp = Sprout::new();
        assert_eq!(sp.get_data_storage().get_number(), 0);
        assert_eq!(sp.get_passport().get_name(), "default name");
    }
}