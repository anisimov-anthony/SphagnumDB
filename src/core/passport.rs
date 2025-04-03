// SphagnumDB
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

/// The passport of this sphagnum node.
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
        assert!(passport.is_ok(), "Passport::new should return Ok");
        let passport = passport.unwrap();
        assert_eq!(
            passport.field, "lawn",
            "New passport should have default field 'lawn'"
        );
    }

    #[test]
    fn test_get_field_empty() {
        let mut passport = Passport::new().unwrap();
        passport.field = String::new(); // Принудительно делаем поле пустым
        let result = passport.get_field();
        assert!(result.is_err(), "get_field should fail with empty field");
        match result {
            Err(PassportError::FieldRetrievalError) => (),
            _ => panic!("get_field should return FieldRetrievalError for empty field"),
        }
    }

    #[test]
    fn test_get_field_success() {
        let passport = Passport::new().unwrap();
        let field = passport.get_field();
        assert!(
            field.is_ok(),
            "get_field should succeed with non-empty field"
        );
        assert_eq!(field.unwrap(), &"lawn", "get_field should return 'lawn'");
    }

    #[test]
    fn test_set_field_empty() {
        let mut passport = Passport::new().unwrap();
        let result = passport.set_field("".to_string());
        assert!(result.is_err(), "set_field should fail with empty value");
        match result {
            Err(PassportError::FieldModificationError) => (),
            _ => panic!("set_field should return FieldModificationError for empty value"),
        }
        assert_eq!(
            passport.field, "lawn",
            "Field should remain 'lawn' after failed set"
        );
    }

    #[test]
    fn test_set_field_success() {
        let mut passport = Passport::new().unwrap();
        let result = passport.set_field("new_value".to_string());
        assert!(
            result.is_ok(),
            "set_field should succeed with non-empty value"
        );
        assert_eq!(
            passport.field, "new_value",
            "Field should be updated to 'new_value'"
        );
    }
}
