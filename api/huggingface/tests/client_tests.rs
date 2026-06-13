//! Tests for the `HuggingFace` API client functionality

mod inc;

use api_huggingface::
{
  Client,
  environment::{ HuggingFaceEnvironmentImpl, HuggingFaceEnvironment, EnvironmentInterface },
  secret::Secret,
  error::HuggingFaceError,
};

#[ tokio::test ]
async fn client_build_with_valid_environment()
{
  let api_key = Secret::new( "test-key".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None ).expect( "Environment build should succeed" );
  let client = Client::build( env );
  
  assert!( client.is_ok(), "Client build should succeed with valid environment" );
}

#[ tokio::test ]
async fn client_build_with_custom_base_url()
{
  let api_key = Secret::new( "test-key".to_string() );
  let custom_url = Some( "https://custom-api.example.com".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( api_key, custom_url ).expect( "Environment build should succeed" );
  let client = Client::build( env );
  
  assert!( client.is_ok(), "Client build should succeed with custom base URL" );
  let client = client.expect( "[client_build_with_custom_base_url] Client should be Ok after is_ok() check - check Client::build() implementation" );
  assert_eq!( client.environment.base_url(), "https://custom-api.example.com" );
}

#[ tokio::test ]
async fn environment_from_env_missing_key()
{
  // Ensure the environment variable is not set
  std::env::remove_var( "HUGGINGFACE_API_KEY" );
  
  let result = HuggingFaceEnvironmentImpl::from_env();
  assert!( result.is_err(), "Should fail when API key is missing" );
  
  match result.unwrap_err()
  {
  HuggingFaceError::Authentication( _ ) => {}, // Expected
  other => panic!( "Expected Authentication error, got : {other:?}" ),
  }
}

#[ tokio::test ]
async fn environment_endpoint_url_construction()
{
  let api_key = Secret::new( "test-key".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None ).expect( "Environment build should succeed" );

  // Test with new Router API endpoint format (no leading slash)
  let url = env.endpoint_url( "chat/completions" );
  assert!( url.is_ok(), "URL construction should succeed" );

  let url = url.expect( "[environment_endpoint_url_construction] URL should be Ok after is_ok() check - check EnvironmentInterface::endpoint_url() implementation" );
  assert_eq!( url.as_str(), "https://router.huggingface.co/v1/chat/completions" );
}

#[ tokio::test ]
async fn environment_headers_generation()
{
  let api_key = Secret::new( "test-key".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None ).expect( "Environment build should succeed" );
  
  let headers = env.headers();
  assert!( headers.is_ok(), "Header generation should succeed" );

  let headers = headers.expect( "[environment_headers_generation] Headers should be Ok after is_ok() check - check EnvironmentInterface::headers() implementation" );
  assert!( headers.contains_key( "authorization" ), "Should contain authorization header" );
  assert!( headers.contains_key( "user-agent" ), "Should contain user-agent header" );
}

#[ tokio::test ]
async fn client_api_accessors()
{
  let api_key = Secret::new( "test-key".to_string() );
  let env = HuggingFaceEnvironmentImpl::build( api_key, None ).expect( "Environment build should succeed" );
  let client = Client::build( env ).expect( "Client build should succeed" );
  
  // Test that API group accessors work without panicking
  let _inference = client.inference();
  let _embeddings = client.embeddings();
  let _models = client.models();
}

#[ cfg( feature = "integration" ) ]
mod integration_tests
{
  use super::*;

  #[ tokio::test ]
  async fn integration_environment_from_env()
  {
  // Setup - Get API key (will panic if missing)
  let api_key_string = crate::inc::get_api_key_for_integration();
  
  // Create environment using workspace-loaded API key
  let api_key = Secret::new( api_key_string );
  let env_result = HuggingFaceEnvironmentImpl::build( api_key, None );
  assert!( env_result.is_ok(), "Should successfully create environment with workspace-loaded API key" );
  }
}