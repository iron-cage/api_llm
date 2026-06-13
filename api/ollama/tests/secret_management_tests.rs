//! Secret management tests for `api_ollama`
//! 
//! These tests verify secure credential storage, retrieval, and handling
//! functionality when the `secret_management` feature is enabled.

#![ cfg( feature = "secret_management" ) ]

use api_ollama::{ OllamaClient, SecretStore, SecretConfig };
use std::env;
use std::collections::HashMap;

#[ tokio::test ]
async fn test_secret_store_creation()
{
  // Test creating a new secret store
  let secret_store = SecretStore::new();
  
  // Should start empty
  assert!(secret_store.is_empty());
  assert_eq!(secret_store.len(), 0);
}

#[ tokio::test ] 
async fn test_secret_store_basic_operations()
{
  let mut secret_store = SecretStore::new();
  
  // Test storing a secret
  secret_store.set("api_key", "secret-key-123").expect("Failed to store secret");
  
  // Test retrieving a secret
  let retrieved = secret_store.get("api_key").expect("Failed to retrieve secret");
  assert_eq!(retrieved.unwrap(), "secret-key-123");
  
  // Test checking existence
  assert!(secret_store.contains("api_key"));
  assert!(!secret_store.contains("non_existent"));
  
  // Test store is no longer empty
  assert!(!secret_store.is_empty());
  assert_eq!(secret_store.len(), 1);
}

#[ tokio::test ]
async fn test_secret_store_secure_debug()
{
  let mut secret_store = SecretStore::new();
  secret_store.set("password", "super-secret-password").expect("Failed to store secret");
  
  // Debug output should not contain actual secrets
  let debug_output = format!( "{secret_store:?}" );
  assert!(!debug_output.contains("super-secret-password"));
  assert!(debug_output.contains("***")); // Should contain masked representation
}

#[ tokio::test ]
async fn test_secret_config_from_env()
{
  // Set environment variables for testing
  env ::set_var("OLLAMA_API_KEY", "env-api-key-456");
  env ::set_var("OLLAMA_SECRET_TOKEN", "env-secret-789");
  
  // Create secret config from environment
  let secret_config = SecretConfig::from_env().expect("Failed to create config from environment");
  
  // Verify secrets are loaded from environment
  assert_eq!(secret_config.api_key().unwrap(), "env-api-key-456");
  assert_eq!(secret_config.secret_token().unwrap(), "env-secret-789");
  
  // Cleanup
  env ::remove_var("OLLAMA_API_KEY");
  env ::remove_var("OLLAMA_SECRET_TOKEN");
}

#[ tokio::test ] 
async fn test_secret_config_from_map()
{
  // Create secrets from a HashMap
  let mut secrets = HashMap::new();
  secrets.insert("api_key".to_string(), "map-api-key-789".to_string());
  secrets.insert("auth_token".to_string(), "map-auth-token-123".to_string());
  
  let secret_config = SecretConfig::from_map(secrets).expect("Failed to create config from map");
  
  // Verify secrets are accessible
  assert_eq!(secret_config.api_key().unwrap(), "map-api-key-789");
  assert_eq!(secret_config.get("auth_token").unwrap(), "map-auth-token-123");
}

#[ tokio::test ]
async fn test_client_with_secret_store()
{
  let mut secret_store = SecretStore::new();
  secret_store.set("api_key", "client-secret-key").expect("Failed to store secret");
  
  // Create client with secret store
  let mut client = OllamaClient::new( "http://localhost:11434".to_string(), OllamaClient::recommended_timeout_fast() )
    .with_secret_store(secret_store);
  
  // Verify client has secrets configured
  assert!(client.has_secrets());
  
  // Verify client can access secrets securely
  let api_key = client.get_secret("api_key").expect("Failed to get secret from client");
  assert_eq!(api_key.unwrap(), "client-secret-key");
}

#[ tokio::test ]
async fn test_client_secret_masking()
{
  let mut secret_store = SecretStore::new();
  secret_store.set("sensitive_token", "very-sensitive-data").expect("Failed to store secret");
  
  let client = OllamaClient::new( "http://localhost:11434".to_string(), OllamaClient::recommended_timeout_fast() )
    .with_secret_store(secret_store);
  
  // Debug output should not reveal secrets
  let debug_output = format!( "{client:?}" );
  assert!(!debug_output.contains("very-sensitive-data"));
  // Should contain indication of secrets without revealing them
  assert!(debug_output.contains("secrets") || debug_output.contains("***"));
}

#[ tokio::test ]
async fn test_secret_rotation()
{
  let mut secret_store = SecretStore::new();
  
  // Store initial secret
  secret_store.set("rotatable_key", "old-secret").expect("Failed to store initial secret");
  assert_eq!(secret_store.get("rotatable_key").unwrap().unwrap(), "old-secret");
  
  // Rotate secret
  secret_store.rotate("rotatable_key", "new-secret").expect("Failed to rotate secret");
  assert_eq!(secret_store.get("rotatable_key").unwrap().unwrap(), "new-secret");
  
  // Old secret should not be accessible
  assert_ne!(secret_store.get("rotatable_key").unwrap().unwrap(), "old-secret");
}

#[ tokio::test ]
async fn test_secret_expiration()
{
  let mut secret_store = SecretStore::new();
  
  // Store secret with expiration
  let expires_at = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() + 1; // 1 second from now
  secret_store.set_with_expiration("temp_secret", "expires-soon", expires_at)
    .expect("Failed to store secret with expiration");
  
  // Secret should be available immediately
  assert!(secret_store.get("temp_secret").unwrap().is_some());
  
  // Wait for expiration
  tokio ::time::sleep(tokio::time::Duration::from_secs(2)).await;
  
  // Secret should be expired and unavailable
  assert!(secret_store.get("temp_secret").unwrap().is_none());
}

#[ tokio::test ]
async fn test_secret_store_clear()
{
  let mut secret_store = SecretStore::new();
  
  // Add multiple secrets
  secret_store.set("key1", "value1").expect("Failed to store key1");
  secret_store.set("key2", "value2").expect("Failed to store key2");
  secret_store.set("key3", "value3").expect("Failed to store key3");
  
  assert_eq!(secret_store.len(), 3);
  
  // Clear all secrets
  secret_store.clear();
  
  assert!(secret_store.is_empty());
  assert_eq!(secret_store.len(), 0);
  assert!(secret_store.get("key1").unwrap().is_none());
}

#[ tokio::test ]
async fn test_secret_validation()
{
  let secret_store = SecretStore::new();
  
  // Test invalid secret names
  let result = secret_store.validate_secret_name("");
  assert!(result.is_err()); // Empty name should be invalid
  
  let result = secret_store.validate_secret_name("valid_name");
  assert!(result.is_ok()); // Valid name should be accepted
  
  // Test secret value validation
  let result = secret_store.validate_secret_value("");
  assert!(result.is_err()); // Empty value should be invalid
  
  let result = secret_store.validate_secret_value("valid_secret_value");
  assert!(result.is_ok()); // Valid value should be accepted
}
