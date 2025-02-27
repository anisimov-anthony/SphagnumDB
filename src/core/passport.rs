// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use std::{error::Error, fmt};

#[derive(Debug)]
pub enum PassportError {
    InitializationError,
    FieldRetrievalError,
    FieldModificationError,
}

impl fmt::Display for PassportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PassportError::InitializationError => write!(f, "Failed to initialize Passport"),
            PassportError::FieldRetrievalError => write!(f, "Failed to retrieve field"),
            PassportError::FieldModificationError => write!(f, "Failed to modify field"),
        }
    }
}

impl Error for PassportError {}

/// The passport of this sprout node.
/// At this stage, it represents a highly simplified implementation, we believe in the authenticity
/// of this data at its word.
pub struct Passport {
    field: String,
}

impl Passport {
    /// Creates a new `Passport` with a default field value.
    pub fn new() -> Result<Self, PassportError> {
        Ok(Self {
            field: "lawn".to_string(),
        })
    }

    /// Returns a reference to the field.
    pub fn get_field(&self) -> Result<&String, PassportError> {
        if self.field.is_empty() {
            Err(PassportError::FieldRetrievalError)
        } else {
            Ok(&self.field)
        }
    }

    /// Sets the field to a new value.
    pub fn set_field(&mut self, new_field: String) -> Result<(), PassportError> {
        if new_field.is_empty() {
            Err(PassportError::FieldModificationError)
        } else {
            self.field = new_field;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let passport = Passport::new();
        assert!(passport.is_ok());
        assert_eq!(passport.unwrap().field, "lawn");
    }

    #[test]
    fn test_get_field() {
        let passport = Passport {
            field: "test_field".to_string(),
        };
        assert_eq!(passport.get_field().unwrap(), "test_field");

        let passport = Passport {
            field: String::new(),
        };
        assert!(passport.get_field().is_err());
    }

    #[test]
    fn test_set_field() {
        let mut passport = Passport {
            field: "old_field".to_string(),
        };
        assert!(passport.set_field("new_field".to_string()).is_ok());
        assert_eq!(passport.field, "new_field");

        assert!(passport.set_field(String::new()).is_err());
    }
}
