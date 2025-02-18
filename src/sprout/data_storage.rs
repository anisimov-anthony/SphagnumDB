// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

/// To work with the data that will be stored on the node.
/// At this stage, it's a simple mock, which is still far from a hashmap, but it's enough for the initial stage.
pub struct DataStorage{
    number: i32, // mock
}

impl DataStorage {
    pub fn new() -> Self {
        Self {
            number: 0
        }
    }

    pub fn get_number(&self) -> i32 {
        self.number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns() {
        let ds = DataStorage::new();
        assert_eq!(ds.get_number(), 0);
    }
}
