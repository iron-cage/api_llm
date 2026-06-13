//! Dynamic Configuration Tests
//!
//! Comprehensive test suite for dynamic configuration functionality including:
//! - Configuration value operations
//! - Snapshot management and versioning
//! - Validation rules and error handling
//! - Configuration manager operations
//! - Change event notifications
//! - Atomic updates and backup/restore
//! - Concurrent access scenarios

#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::similar_names ) ] // Test variables often have similar names
#![ allow( clippy::useless_vec ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::must_use_candidate ) ]
#![ allow( clippy::missing_panics_doc ) ]
#![ allow( clippy::missing_errors_doc ) ]
#![ allow( clippy::doc_markdown ) ]

#[ cfg( test ) ]
mod dynamic_configuration_tests
{
  use api_openai::dynamic_configuration::*;
  use std::time::Duration;
  use tokio::time;

  // ===== CONFIG VALUE TESTS =====

  #[ tokio::test ]
  async fn test_config_value_creation()
  {
    let string_val = ConfigValue::String( "test".to_string() );
    let int_val = ConfigValue::Integer( 42 );
    let float_val = ConfigValue::Float( std::f64::consts::PI );
    let bool_val = ConfigValue::Boolean( true );
    let duration_val = ConfigValue::Duration( 5000 );

    assert_eq!( string_val.as_string(), Some( "test".to_string() ) );
    assert_eq!( int_val.as_integer(), Some( 42 ) );
    assert_eq!( float_val.as_float(), Some( std::f64::consts::PI ) );
    assert_eq!( bool_val.as_boolean(), Some( true ) );
    assert_eq!( duration_val.as_duration(), Some( Duration::from_secs( 5 ) ) );
  }

  #[ tokio::test ]
  async fn test_config_value_type_conversion()
  {
    let string_val = ConfigValue::String( "test".to_string() );

    // Should return None for wrong type conversions
    assert_eq!( string_val.as_integer(), None );
    assert_eq!( string_val.as_float(), None );
    assert_eq!( string_val.as_boolean(), None );
    assert_eq!( string_val.as_duration(), None );

    // Only string conversion should work
    assert_eq!( string_val.as_string(), Some( "test".to_string() ) );
  }

  #[ tokio::test ]
  async fn test_config_value_serialization()
  {
    let values = vec![
      ConfigValue::String( "test".to_string() ),
      ConfigValue::Integer( 42 ),
      ConfigValue::Float( std::f64::consts::PI ),
      ConfigValue::Boolean( true ),
      ConfigValue::Duration( 5000 ),
    ];

    for value in values
    {
      // Test serialization
      let serialized = serde_json::to_string( &value ).expect( "Failed to serialize value" );
      assert!( !serialized.is_empty() );

      // Test deserialization
      let deserialized : ConfigValue = serde_json::from_str( &serialized )
        .expect( "Failed to deserialize value" );

      assert_eq!( value, deserialized );
    }
  }

  // ===== CONFIG SNAPSHOT TESTS =====

  #[ tokio::test ]
  async fn test_config_snapshot_creation()
  {
    let snapshot = ConfigSnapshot::new();
    assert_eq!( snapshot.version, 1 );
    assert!( snapshot.values.is_empty() );

    let snapshot_default = ConfigSnapshot::default();
    assert_eq!( snapshot_default.version, 1 );
  }

  #[ tokio::test ]
  async fn test_config_snapshot_versioning()
  {
    let snapshot1 = ConfigSnapshot::new();
    let snapshot2 = snapshot1.next_version();

    assert_eq!( snapshot1.version, 1 );
    assert_eq!( snapshot2.version, 2 );
    assert!( snapshot2.created_at > snapshot1.created_at );
  }

  #[ tokio::test ]
  async fn test_config_snapshot_value_operations()
  {
    let snapshot = ConfigSnapshot::new()
      .with_value( "key1".to_string(), ConfigValue::String( "value1".to_string() ) )
      .with_value( "key2".to_string(), ConfigValue::Integer( 42 ) );

    assert_eq!( snapshot.get( "key1" ), Some( &ConfigValue::String( "value1".to_string() ) ) );
    assert_eq!( snapshot.get( "key2" ), Some( &ConfigValue::Integer( 42 ) ) );
    assert_eq!( snapshot.get( "key3" ), None );

    let snapshot_without = snapshot.without_value( "key1" );
    assert_eq!( snapshot_without.get( "key1" ), None );
    assert_eq!( snapshot_without.get( "key2" ), Some( &ConfigValue::Integer( 42 ) ) );
  }

  #[ tokio::test ]
  async fn test_config_snapshot_serialization()
  {
    let snapshot = ConfigSnapshot::new()
      .with_value( "key1".to_string(), ConfigValue::String( "value1".to_string() ) )
      .with_value( "key2".to_string(), ConfigValue::Integer( 42 ) );

    // Test serialization
    let serialized = serde_json::to_string( &snapshot ).expect( "Failed to serialize snapshot" );
    assert!( !serialized.is_empty() );

    // Test deserialization
    let deserialized : ConfigSnapshot = serde_json::from_str( &serialized )
      .expect( "Failed to deserialize snapshot" );

    assert_eq!( snapshot.version, deserialized.version );
    assert_eq!( snapshot.values, deserialized.values );
  }

  // ===== CONFIG VALIDATOR TESTS =====

  #[ tokio::test ]
  async fn test_config_validator_creation()
  {
    let _validator = ConfigValidator::new();
    let _validator_default = ConfigValidator::default();
    // Both should create successfully without panicking
  }

  #[ tokio::test ]
  async fn test_config_validator_required_rule()
  {
    let validator = ConfigValidator::new()
      .add_rule( "required_key".to_string(), ValidationRule::Required );

    let valid_snapshot = ConfigSnapshot::new()
      .with_value( "required_key".to_string(), ConfigValue::String( "present".to_string() ) );

    let invalid_snapshot = ConfigSnapshot::new()
      .with_value( "other_key".to_string(), ConfigValue::String( "present".to_string() ) );

    assert!( valid_snapshot.validate( &validator ).is_ok() );
    assert!( invalid_snapshot.validate( &validator ).is_err() );
  }

  #[ tokio::test ]
  async fn test_config_validator_integer_range()
  {
    let validator = ConfigValidator::new()
      .add_rule( "port".to_string(), ValidationRule::IntegerRange( 1, 65535 ) );

    let valid_snapshot = ConfigSnapshot::new()
      .with_value( "port".to_string(), ConfigValue::Integer( 8080 ) );

    let invalid_snapshot = ConfigSnapshot::new()
      .with_value( "port".to_string(), ConfigValue::Integer( 70000 ) );

    assert!( valid_snapshot.validate( &validator ).is_ok() );
    assert!( invalid_snapshot.validate( &validator ).is_err() );
  }

  #[ tokio::test ]
  async fn test_config_validator_string_pattern()
  {
    let validator = ConfigValidator::new()
      .add_rule( "url".to_string(), ValidationRule::StringPattern( "https".to_string() ) );

    let valid_snapshot = ConfigSnapshot::new()
      .with_value( "url".to_string(), ConfigValue::String( "https://api.example.com".to_string() ) );

    let invalid_snapshot = ConfigSnapshot::new()
      .with_value( "url".to_string(), ConfigValue::String( "http://api.example.com".to_string() ) );

    assert!( valid_snapshot.validate( &validator ).is_ok() );
    assert!( invalid_snapshot.validate( &validator ).is_err() );
  }

  #[ tokio::test ]
  async fn test_config_validator_duration_range()
  {
    let validator = ConfigValidator::new()
      .add_rule( "timeout".to_string(), ValidationRule::DurationRange( 1000, 30000 ) );

    let valid_snapshot = ConfigSnapshot::new()
      .with_value( "timeout".to_string(), ConfigValue::Duration( 5000 ) );

    let invalid_snapshot = ConfigSnapshot::new()
      .with_value( "timeout".to_string(), ConfigValue::Duration( 50000 ) );

    assert!( valid_snapshot.validate( &validator ).is_ok() );
    assert!( invalid_snapshot.validate( &validator ).is_err() );
  }

  #[ tokio::test ]
  async fn test_config_validator_type_mismatch()
  {
    let validator = ConfigValidator::new()
      .add_rule( "port".to_string(), ValidationRule::IntegerRange( 1, 65535 ) );

    let invalid_snapshot = ConfigSnapshot::new()
      .with_value( "port".to_string(), ConfigValue::String( "not_a_number".to_string() ) );

    let result = invalid_snapshot.validate( &validator );
    assert!( result.is_err() );
    assert!( result.unwrap_err().iter().any( | e | e.contains( "type doesn't match" ) ) );
  }

  // ===== CONFIG MANAGER TESTS =====

  #[ tokio::test ]
  async fn test_config_manager_creation()
  {
    let validator = ConfigValidator::new();
    let manager = ConfigManager::new( validator );

    let snapshot = manager.get_snapshot();
    assert_eq!( snapshot.version, 1 );
    assert!( snapshot.values.is_empty() );
  }

  #[ tokio::test ]
  async fn test_config_manager_update_value()
  {
    let validator = ConfigValidator::new();
    let manager = ConfigManager::new( validator );

    let result = manager.update_value( "key1".to_string(), ConfigValue::String( "value1".to_string() ) );
    assert!( result.is_ok() );

    let snapshot = manager.get_snapshot();
    assert_eq!( snapshot.get( "key1" ), Some( &ConfigValue::String( "value1".to_string() ) ) );
    assert_eq!( snapshot.version, 2 );
  }

  #[ tokio::test ]
  async fn test_config_manager_validation_failure()
  {
    let validator = ConfigValidator::new()
      .add_rule( "port".to_string(), ValidationRule::IntegerRange( 1, 65535 ) );

    let manager = ConfigManager::new( validator );

    let result = manager.update_value( "port".to_string(), ConfigValue::Integer( 70000 ) );
    assert!( result.is_err() );

    // Original snapshot should be unchanged
    let snapshot = manager.get_snapshot();
    assert_eq!( snapshot.version, 1 );
    assert!( snapshot.values.is_empty() );
  }

  #[ tokio::test ]
  async fn test_config_manager_remove_value()
  {
    let validator = ConfigValidator::new();
    let manager = ConfigManager::new( validator );

    // Add a value first
    manager.update_value( "key1".to_string(), ConfigValue::String( "value1".to_string() ) ).unwrap();

    // Remove the value
    let result = manager.remove_value( "key1" );
    assert!( result.is_ok() );

    let snapshot = manager.get_snapshot();
    assert_eq!( snapshot.get( "key1" ), None );
    assert_eq!( snapshot.version, 3 );
  }

  #[ tokio::test ]
  async fn test_config_manager_get_value()
  {
    let validator = ConfigValidator::new();
    let manager = ConfigManager::new( validator );

    assert_eq!( manager.get_value( "nonexistent" ), None );

    manager.update_value( "key1".to_string(), ConfigValue::String( "value1".to_string() ) ).unwrap();
    assert_eq!( manager.get_value( "key1" ), Some( ConfigValue::String( "value1".to_string() ) ) );
  }

  // ===== DYNAMIC CONFIG MANAGER TESTS =====

  #[ tokio::test ]
  async fn test_atomic_changes()
  {
    let validator = ConfigValidator::new();
    let manager = ConfigManager::new( validator );

    let changes = vec![
      ( "key1".to_string(), ConfigValue::String( "value1".to_string() ) ),
      ( "key2".to_string(), ConfigValue::Integer( 42 ) ),
      ( "key3".to_string(), ConfigValue::Boolean( true ) ),
    ];

    let result = DynamicConfigManager::apply_atomic_changes( &manager, changes );
    assert!( result.is_ok() );

    let snapshot = manager.get_snapshot();
    assert_eq!( snapshot.get( "key1" ), Some( &ConfigValue::String( "value1".to_string() ) ) );
    assert_eq!( snapshot.get( "key2" ), Some( &ConfigValue::Integer( 42 ) ) );
    assert_eq!( snapshot.get( "key3" ), Some( &ConfigValue::Boolean( true ) ) );
  }

  #[ tokio::test ]
  async fn test_atomic_changes_validation_failure()
  {
    let validator = ConfigValidator::new()
      .add_rule( "port".to_string(), ValidationRule::IntegerRange( 1, 65535 ) );

    let manager = ConfigManager::new( validator );

    let changes = vec![
      ( "key1".to_string(), ConfigValue::String( "value1".to_string() ) ),
      ( "port".to_string(), ConfigValue::Integer( 70000 ) ), // Invalid
    ];

    let result = DynamicConfigManager::apply_atomic_changes( &manager, changes );
    assert!( result.is_err() );

    // No changes should have been applied
    let snapshot = manager.get_snapshot();
    assert_eq!( snapshot.get( "key1" ), None );
    assert_eq!( snapshot.get( "port" ), None );
  }

  #[ tokio::test ]
  async fn test_backup_and_restore()
  {
    let snapshot = ConfigSnapshot::new()
      .with_value( "key1".to_string(), ConfigValue::String( "value1".to_string() ) )
      .with_value( "key2".to_string(), ConfigValue::Integer( 42 ) );

    let backup = DynamicConfigManager::create_backup( &snapshot );
    assert!( !backup.is_empty() );

    let restored = DynamicConfigManager::restore_from_backup( &backup );
    assert!( restored.is_ok() );

    let restored_snapshot = restored.unwrap();
    assert_eq!( restored_snapshot.values, snapshot.values );
  }

  #[ tokio::test ]
  async fn test_merge_snapshots()
  {
    let base = ConfigSnapshot::new()
      .with_value( "key1".to_string(), ConfigValue::String( "base_value".to_string() ) )
      .with_value( "key2".to_string(), ConfigValue::Integer( 42 ) );

    let overlay = ConfigSnapshot::new()
      .with_value( "key1".to_string(), ConfigValue::String( "overlay_value".to_string() ) )
      .with_value( "key3".to_string(), ConfigValue::Boolean( true ) );

    let merged = DynamicConfigManager::merge_snapshots( &base, overlay );

    // Should have overlay value for key1
    assert_eq!( merged.get( "key1" ), Some( &ConfigValue::String( "overlay_value".to_string() ) ) );
    // Should keep base value for key2
    assert_eq!( merged.get( "key2" ), Some( &ConfigValue::Integer( 42 ) ) );
    // Should have new value for key3
    assert_eq!( merged.get( "key3" ), Some( &ConfigValue::Boolean( true ) ) );
  }

  // ===== CHANGE EVENT TESTS =====

  #[ tokio::test ]
  async fn test_config_change_events()
  {
    let ( sender, mut receiver ) = DynamicConfigManager::create_change_watcher();

    let event = ConfigChangeEvent
    {
      key : "test_key".to_string(),
      old_value : Some( ConfigValue::String( "old".to_string() ) ),
      new_value : ConfigValue::String( "new".to_string() ),
      timestamp : std::time::Instant::now(),
    };

    sender.send_change( event.clone() ).expect( "Failed to send event" );

    let received = receiver.try_recv();
    assert!( received.is_some() );

    let received_event = received.unwrap();
    assert_eq!( received_event.key, event.key );
    assert_eq!( received_event.old_value, event.old_value );
    assert_eq!( received_event.new_value, event.new_value );
  }

  #[ tokio::test ]
  async fn test_config_change_update_helper()
  {
    let ( sender, mut receiver ) = DynamicConfigManager::create_change_watcher();

    let result = sender.send_update(
      "test_key".to_string(),
      Some( ConfigValue::String( "old".to_string() ) ),
      ConfigValue::String( "new".to_string() ),
    );

    assert!( result.is_ok() );

    let received = receiver.try_recv();
    assert!( received.is_some() );

    let event = received.unwrap();
    assert_eq!( event.key, "test_key" );
    assert_eq!( event.old_value, Some( ConfigValue::String( "old".to_string() ) ) );
    assert_eq!( event.new_value, ConfigValue::String( "new".to_string() ) );
  }

  #[ tokio::test ]
  async fn test_config_change_async_recv()
  {
    let ( sender, mut receiver ) = DynamicConfigManager::create_change_watcher();

    // Send event after a delay
    tokio ::spawn( async move
    {
      time ::sleep( Duration::from_millis( 25 ) ).await;
      let _ = sender.send_update(
        "async_key".to_string(),
        None,
        ConfigValue::Boolean( true ),
      );
    });

    // Receive asynchronously
    let event = receiver.recv().await;
    assert!( event.is_some() );

    let received_event = event.unwrap();
    assert_eq!( received_event.key, "async_key" );
    assert_eq!( received_event.new_value, ConfigValue::Boolean( true ) );
  }

  // ===== VALUE WATCHER TESTS =====

  #[ tokio::test ]
  async fn test_value_watcher()
  {
    let ( sender, mut receiver ) = DynamicConfigManager::create_value_watcher( "initial".to_string() );

    assert_eq!( *receiver.borrow(), "initial" );

    sender.send( "updated".to_string() ).expect( "Failed to send update" );

    // Wait for change
    receiver.changed().await.expect( "Failed to receive change" );
    assert_eq!( *receiver.borrow(), "updated" );
  }

  // ===== INTEGRATION TESTS =====

  #[ tokio::test ]
  async fn test_configuration_workflow()
  {
    let validator = ConfigValidator::new()
      .add_rule( "api_key".to_string(), ValidationRule::Required )
      .add_rule( "timeout".to_string(), ValidationRule::DurationRange( 1000, 30000 ) )
      .add_rule( "port".to_string(), ValidationRule::IntegerRange( 1, 65535 ) );

    let manager = ConfigManager::new( validator );
    let ( change_sender, mut change_receiver ) = DynamicConfigManager::create_change_watcher();

    // Apply initial configuration
    let initial_changes = vec![
      ( "api_key".to_string(), ConfigValue::String( "sk-test123".to_string() ) ),
      ( "timeout".to_string(), ConfigValue::Duration( 5000 ) ),
      ( "port".to_string(), ConfigValue::Integer( 8080 ) ),
    ];

    let result = DynamicConfigManager::apply_atomic_changes( &manager, initial_changes );
    assert!( result.is_ok() );

    // Send change notification
    change_sender.send_update(
      "timeout".to_string(),
      Some( ConfigValue::Duration( 5000 ) ),
      ConfigValue::Duration( 10000 ),
    ).expect( "Failed to send change" );

    // Update timeout
    let update_result = manager.update_value( "timeout".to_string(), ConfigValue::Duration( 10000 ) );
    assert!( update_result.is_ok() );

    // Verify final state
    let final_snapshot = manager.get_snapshot();
    assert_eq!( final_snapshot.get( "api_key" ), Some( &ConfigValue::String( "sk-test123".to_string() ) ) );
    assert_eq!( final_snapshot.get( "timeout" ), Some( &ConfigValue::Duration( 10000 ) ) );
    assert_eq!( final_snapshot.get( "port" ), Some( &ConfigValue::Integer( 8080 ) ) );

    // Verify change event was received
    let change_event = change_receiver.try_recv();
    assert!( change_event.is_some() );
  }

  #[ tokio::test ]
  async fn test_concurrent_updates()
  {
    let validator = ConfigValidator::new();
    let manager = std::sync::Arc::new( ConfigManager::new( validator ) );

    let mut handles = Vec::new();

    // Create multiple concurrent update tasks
    for i in 0..10
    {
      let manager_clone = manager.clone();
      let handle = tokio::spawn( async move
      {
        let result = manager_clone.update_value(
          format!( "key_{}", i ),
          ConfigValue::Integer( i64::from(i) ),
        );
        result.is_ok()
      });
      handles.push( handle );
    }

    // Wait for all updates to complete
    let mut results = Vec::new();
    for handle in handles
    {
      results.push( handle.await );
    }

    // All updates should succeed
    for result in results
    {
      assert!( result.unwrap() );
    }

    // Verify all values were set
    let final_snapshot = manager.get_snapshot();
    for i in 0..10
    {
      let key = format!( "key_{}", i );
      assert_eq!( final_snapshot.get( &key ), Some( &ConfigValue::Integer( i64::from(i) ) ) );
    }
  }

  #[ tokio::test ]
  async fn test_performance_many_updates()
  {
    let validator = ConfigValidator::new();
    let manager = ConfigManager::new( validator );

    let start = std::time::Instant::now();

    // Perform many sequential updates
    for i in 0..1000
    {
      let result = manager.update_value(
        format!( "key_{}", i % 10 ), // Reuse keys to test overwrites
        ConfigValue::Integer( i64::from(i) ),
      );
      assert!( result.is_ok() );
    }

    let duration = start.elapsed();
    assert!( duration < Duration::from_millis( 500 ) ); // Should be fast

    // Verify final state
    let snapshot = manager.get_snapshot();
    assert_eq!( snapshot.values.len(), 10 ); // Only 10 unique keys
  }
}