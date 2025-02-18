// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

/// The passport of this sprout node. 
/// At this stage, it represents a highly simplified implementation, we believe in the authenticity of this data at its word.
pub struct Passport {
    name: String
}

impl Passport {
    pub fn new() -> Self {
        Self {
            name: String::from("default name")
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns() {
        let ds = Passport::new();
        assert_eq!(ds.get_name(), "default name");
    }
}
