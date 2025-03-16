// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License

use sproutdb::core::data_types::string_store::StringStore;
use sproutdb::core::data_types::data_type::DataType;
use sproutdb::core::commands::{
    Command,
    string_commands::StringCommand,
    generic_commands::GenericCommand,
};

#[test]
fn test_new() {
    let store = StringStore::new();
    assert!(store.is_ok(), "StringStore::new should return Ok");
}

#[test]
fn test_set_command() {
    let mut store = StringStore::new().unwrap();
    let command = Command::String(StringCommand::Set {
        key: "key1".to_string(),
        value: "value1".to_string(),
    });
    let result = store.handle_command(command);
    assert!(result.is_ok(), "Set command should succeed");
    let boxed_result = result.unwrap();
    let ok_str = boxed_result.downcast_ref::<String>().unwrap();
    assert_eq!(ok_str, "OK", "Set command should return 'OK'");
}

#[test]
fn test_get_command() {
    let mut store = StringStore::new().unwrap();
    let set_command = Command::String(StringCommand::Set {
        key: "key1".to_string(),
        value: "value1".to_string(),
    });
    store.handle_command(set_command).unwrap();

    let get_command = Command::String(StringCommand::Get {
        key: "key1".to_string(),
    });
    let result = store.handle_command(get_command);
    assert!(result.is_ok(), "Get command should succeed");
    let boxed_result = result.unwrap();
    let value = boxed_result.downcast_ref::<Option<String>>().unwrap();
    assert_eq!(value, &Some("value1".to_string()), "Get should return the set value");
}

#[test]
fn test_get_nonexistent_key() {
    let mut store = StringStore::new().unwrap();
    let get_command = Command::String(StringCommand::Get {
        key: "nonexistent".to_string(),
    });
    let result = store.handle_command(get_command);
    assert!(result.is_ok(), "Get command for nonexistent key should succeed");
    let boxed_result = result.unwrap();
    let value = boxed_result.downcast_ref::<Option<String>>().unwrap();
    assert_eq!(value, &None, "Get for nonexistent key should return None");
}

#[test]
fn test_append_command() {
    let mut store = StringStore::new().unwrap();
    let set_command = Command::String(StringCommand::Set {
        key: "key1".to_string(),
        value: "Hello".to_string(),
    });
    store.handle_command(set_command).unwrap();

    let append_command = Command::String(StringCommand::Append {
        key: "key1".to_string(),
        value: "World".to_string(),
    });
    let result = store.handle_command(append_command);
    assert!(result.is_ok(), "Append command should succeed");
    let boxed_result = result.unwrap();
    let len = boxed_result.downcast_ref::<u64>().unwrap();
    assert_eq!(*len, 10, "Append should return the new length (HelloWorld = 10)");

    let get_command = Command::String(StringCommand::Get {
        key: "key1".to_string(),
    });
    let get_result = store.handle_command(get_command).unwrap();
    let value = get_result.downcast_ref::<Option<String>>().unwrap();
    assert_eq!(value, &Some("HelloWorld".to_string()), "Get should return appended value");
}

#[test]
fn test_exists_command() {
    let mut store = StringStore::new().unwrap();
    let set_command = Command::String(StringCommand::Set {
        key: "key1".to_string(),
        value: "value1".to_string(),
    });
    store.handle_command(set_command).unwrap();

    let exists_command = Command::Generic(GenericCommand::Exists {
        keys: vec!["key1".to_string(), "key2".to_string()],
    });
    let result = store.handle_command(exists_command);
    assert!(result.is_ok(), "Exists command should succeed");
    let boxed_result = result.unwrap();
    let count = boxed_result.downcast_ref::<u64>().unwrap();
    assert_eq!(*count, 1, "Exists should return 1 for one existing key");
}

#[test]
fn test_delete_command() {
    let mut store = StringStore::new().unwrap();
    let set_command = Command::String(StringCommand::Set {
        key: "key1".to_string(),
        value: "value1".to_string(),
    });
    store.handle_command(set_command).unwrap();

    let delete_command = Command::Generic(GenericCommand::Delete {
        keys: vec!["key1".to_string(), "key2".to_string()],
    });
    let result = store.handle_command(delete_command);
    assert!(result.is_ok(), "Delete command should succeed");
    let boxed_result = result.unwrap();
    let count = boxed_result.downcast_ref::<u64>().unwrap();
    assert_eq!(*count, 1, "Delete should return 1 for one deleted key");

    let get_command = Command::String(StringCommand::Get {
        key: "key1".to_string(),
    });
    let get_result = store.handle_command(get_command).unwrap();
    let value = get_result.downcast_ref::<Option<String>>().unwrap();
    assert_eq!(value, &None, "Get after delete should return None");
}
