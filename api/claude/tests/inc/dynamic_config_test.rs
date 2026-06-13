//! Dynamic Configuration Tests
//!
//! Unit tests for the dynamic-config feature that provides hot-reloading
//! of runtime configuration from file system.

#[ allow( unused_imports ) ]
use super::*;

#[ cfg( feature = "dynamic-config" ) ]
mod dynamic_config_tests
{
  use super::*;
  use the_module::RuntimeConfig;
  use core::time::Duration;

  #[ test ]
  fn test_runtime_config_defaults()
  {
    let config = RuntimeConfig::new();
    assert_eq!( config.base_url, "https://api.anthropic.com" );
    assert_eq!( config.api_version, "2023-06-01" );
    assert_eq!( config.timeout_ms, 300_000 );
    assert!( config.enable_retry );
    assert_eq!( config.max_retries, 3 );
    assert!( config.enable_circuit_breaker );
    assert_eq!( config.circuit_breaker_threshold, 5 );
    assert!( !config.enable_rate_limiting );
    assert_eq!( config.rate_limit_rps, 10 );
  }

  #[ test ]
  fn test_runtime_config_default_trait()
  {
    let config = RuntimeConfig::default();
    assert_eq!( config.base_url, "https://api.anthropic.com" );
  }

  #[ test ]
  fn test_runtime_config_timeout()
  {
    let config = RuntimeConfig
    {
      base_url : "https://api.anthropic.com".to_string(),
      api_version : "2023-06-01".to_string(),
      timeout_ms : 60_000,
      enable_retry : true,
      max_retries : 3,
      enable_circuit_breaker : true,
      circuit_breaker_threshold : 5,
      enable_rate_limiting : false,
      rate_limit_rps : 10,
    };

    assert_eq!( config.timeout(), Duration::from_mins( 1 ) );
  }

  #[ test ]
  fn test_runtime_config_validate_success()
  {
    let config = RuntimeConfig::new();
    assert!( config.validate().is_ok() );
  }

  #[ test ]
  fn test_runtime_config_validate_empty_base_url()
  {
    let mut config = RuntimeConfig::new();
    config.base_url = String::new();

    let result = config.validate();
    assert!( result.is_err() );
    assert_eq!( result.unwrap_err(), "base_url cannot be empty" );
  }

  #[ test ]
  fn test_runtime_config_validate_empty_api_version()
  {
    let mut config = RuntimeConfig::new();
    config.api_version = String::new();

    let result = config.validate();
    assert!( result.is_err() );
    assert_eq!( result.unwrap_err(), "api_version cannot be empty" );
  }

  #[ test ]
  fn test_runtime_config_validate_zero_timeout()
  {
    let mut config = RuntimeConfig::new();
    config.timeout_ms = 0;

    let result = config.validate();
    assert!( result.is_err() );
    assert_eq!( result.unwrap_err(), "timeout_ms must be greater than 0" );
  }

  #[ test ]
  fn test_runtime_config_validate_max_retries_limit()
  {
    let mut config = RuntimeConfig::new();
    config.max_retries = 11;

    let result = config.validate();
    assert!( result.is_err() );
    assert_eq!( result.unwrap_err(), "max_retries cannot exceed 10" );
  }

  #[ test ]
  fn test_runtime_config_validate_zero_circuit_breaker_threshold()
  {
    let mut config = RuntimeConfig::new();
    config.circuit_breaker_threshold = 0;

    let result = config.validate();
    assert!( result.is_err() );
    assert_eq!( result.unwrap_err(), "circuit_breaker_threshold must be greater than 0" );
  }

  #[ test ]
  fn test_runtime_config_validate_zero_rate_limit()
  {
    let mut config = RuntimeConfig::new();
    config.rate_limit_rps = 0;

    let result = config.validate();
    assert!( result.is_err() );
    assert_eq!( result.unwrap_err(), "rate_limit_rps must be greater than 0" );
  }

  #[ test ]
  fn test_runtime_config_json_serialization()
  {
    let config = RuntimeConfig::new();
    let json = serde_json::to_string( &config ).unwrap();
    let deserialized : RuntimeConfig = serde_json::from_str( &json ).unwrap();

    assert_eq!( config, deserialized );
  }

  #[ test ]
  fn test_runtime_config_from_json_file()
  {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join( "config.json" );

    // Write test config
    let config = RuntimeConfig
    {
      base_url : "https://test.api.com".to_string(),
      api_version : "2024-01-01".to_string(),
      timeout_ms : 120_000,
      enable_retry : false,
      max_retries : 5,
      enable_circuit_breaker : false,
      circuit_breaker_threshold : 10,
      enable_rate_limiting : true,
      rate_limit_rps : 20,
    };

    let json = serde_json::to_string_pretty( &config ).unwrap();
    let mut file = File::create( &file_path ).unwrap();
    file.write_all( json.as_bytes() ).unwrap();
    drop( file );

    // Load from file
    let loaded = RuntimeConfig::from_json_file( &file_path ).unwrap();

    assert_eq!( loaded.base_url, "https://test.api.com" );
    assert_eq!( loaded.api_version, "2024-01-01" );
    assert_eq!( loaded.timeout_ms, 120_000 );
    assert!( !loaded.enable_retry );
    assert_eq!( loaded.max_retries, 5 );
    assert!( !loaded.enable_circuit_breaker );
    assert_eq!( loaded.circuit_breaker_threshold, 10 );
    assert!( loaded.enable_rate_limiting );
    assert_eq!( loaded.rate_limit_rps, 20 );
  }

  #[ test ]
  fn test_runtime_config_from_json_file_invalid()
  {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join( "invalid.json" );

    // Write invalid config (fails validation)
    let invalid_json = r#"
    {
      "base_url": "",
      "api_version": "2024-01-01",
      "timeout_ms": 120000,
      "enable_retry": true,
      "max_retries": 3,
      "enable_circuit_breaker": true,
      "circuit_breaker_threshold": 5,
      "enable_rate_limiting": false,
      "rate_limit_rps": 10
    }
    "#;

    let mut file = File::create( &file_path ).unwrap();
    file.write_all( invalid_json.as_bytes() ).unwrap();
    drop( file );

    // Should fail validation
    let result = RuntimeConfig::from_json_file( &file_path );
    assert!( result.is_err() );
  }

  #[ test ]
  fn test_runtime_config_partial_json()
  {
    // Test that missing fields use defaults via serde default attributes
    let partial_json = r#"
    {
      "base_url": "https://custom.api.com"
    }
    "#;

    let config : RuntimeConfig = serde_json::from_str( partial_json ).unwrap();

    assert_eq!( config.base_url, "https://custom.api.com" );
    assert_eq!( config.api_version, "2023-06-01" ); // default
    assert_eq!( config.timeout_ms, 300_000 ); // default
    assert!( config.enable_retry ); // default true
  }

  #[ test ]
  fn test_config_watcher_creation()
  {
    use the_module::ConfigWatcher;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join( "config.json" );

    let initial_config = RuntimeConfig::new();
    let watcher = ConfigWatcher::new( config_path, initial_config.clone() );

    assert!( watcher.is_ok() );
    let watcher = watcher.unwrap();

    // Should return initial config
    assert_eq!( watcher.config(), initial_config );
  }

  #[ test ]
  fn test_config_watcher_load_existing_file()
  {
    use the_module::ConfigWatcher;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join( "config.json" );

    // Create config file before watcher
    let file_config = RuntimeConfig
    {
      base_url : "https://from-file.com".to_string(),
      ..RuntimeConfig::new()
    };

    let json = serde_json::to_string_pretty( &file_config ).unwrap();
    let mut file = File::create( &config_path ).unwrap();
    file.write_all( json.as_bytes() ).unwrap();
    drop( file );

    // Create watcher with different initial config
    let initial_config = RuntimeConfig::new();
    let watcher = ConfigWatcher::new( config_path, initial_config ).unwrap();

    // Should load from file, not use initial config
    assert_eq!( watcher.config().base_url, "https://from-file.com" );
  }

  #[ test ]
  fn test_config_watcher_manual_reload()
  {
    use the_module::ConfigWatcher;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join( "config.json" );

    // Create initial file
    let config1 = RuntimeConfig::new();
    let json1 = serde_json::to_string_pretty( &config1 ).unwrap();
    let mut file = File::create( &config_path ).unwrap();
    file.write_all( json1.as_bytes() ).unwrap();
    drop( file );

    let watcher = ConfigWatcher::new( config_path.clone(), config1 ).unwrap();

    // Update file
    let config2 = RuntimeConfig
    {
      base_url : "https://updated.com".to_string(),
      ..RuntimeConfig::new()
    };
    let json2 = serde_json::to_string_pretty( &config2 ).unwrap();
    let mut file = File::create( &config_path ).unwrap();
    file.write_all( json2.as_bytes() ).unwrap();
    drop( file );

    // Manual reload
    let result = watcher.reload();
    assert!( result.is_ok() );

    // Should have new config
    assert_eq!( watcher.config().base_url, "https://updated.com" );
  }

  #[ test ]
  fn test_config_watcher_programmatic_update()
  {
    use the_module::ConfigWatcher;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join( "config.json" );

    let initial = RuntimeConfig::new();
    let watcher = ConfigWatcher::new( config_path, initial ).unwrap();

    // Programmatic update
    let new_config = RuntimeConfig
    {
      base_url : "https://programmatic.com".to_string(),
      ..RuntimeConfig::new()
    };

    let result = watcher.update( new_config );
    assert!( result.is_ok() );

    assert_eq!( watcher.config().base_url, "https://programmatic.com" );
  }

  #[ test ]
  fn test_config_watcher_update_invalid()
  {
    use the_module::ConfigWatcher;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join( "config.json" );

    let initial = RuntimeConfig::new();
    let watcher = ConfigWatcher::new( config_path, initial.clone() ).unwrap();

    // Try to update with invalid config
    let mut invalid = RuntimeConfig::new();
    invalid.base_url = String::new(); // Invalid

    let result = watcher.update( invalid );
    assert!( result.is_err() );

    // Should still have original config
    assert_eq!( watcher.config(), initial );
  }
}

#[ cfg( not( feature = "dynamic-config" ) ) ]
mod dynamic_config_feature_disabled
{
  #[ test ]
  fn test_dynamic_config_feature_disabled()
  {
    // When dynamic-config feature is disabled, this test verifies
    // that compilation succeeds without the feature
  }
}
