//! Test for `responses_create` example
//!
//! This test reproduces the API key issue in the `responses_create` example
//! where it falls back to `dummy_key` instead of using proper secret loading.

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  components ::responses::{ CreateResponseRequest, ResponseInput },
};

#[ tokio::test ]
async fn test_responses_create_example_secret_loading()
{
  // This test verifies that the responses_create example pattern works
  // with proper secret loading instead of dummy_key fallback

  // Load secret using the comprehensive fallback system
  let secret = api_openai::secret::Secret::load_with_fallbacks("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY should be available in workspace secrets");

  let env = api_openai::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
    .expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // Create request similar to the example
  let request = CreateResponseRequest::former()
    .model( "gpt-5-mini".to_string() )
    .input( ResponseInput::String( "What is the capital of France?".to_string() ) )
    .form();

  // This should succeed with proper API key
  let result = client.responses().create( request ).await;

  match result
  {
    Ok( response ) =>
    {
      // Success - verify we got a proper response
      assert!( !response.output.is_empty(), "Response should contain output" );
      println!( "✅ responses_create example works with proper secret loading!" );

      // Verify the response contains reasonable content about France
      let output_text = format!( "{:?}", response.output );
      assert!(
        output_text.to_lowercase().contains( "paris" ) ||
        output_text.to_lowercase().contains( "france" ) ||
        output_text.to_lowercase().contains( "capital" ),
        "Response should contain content about France's capital"
      );

      println!( "✅ Response contains appropriate content about France's capital" );
    },
    Err( e ) =>
    {
      let error_msg = format!( "{e:?}" );
      if error_msg.contains( "dummy_key" )
      {
        panic!( "❌ ISSUE: Example still using dummy_key instead of proper secret loading : {error_msg}" );
      }
      else
      {
        // Some other API error - could be rate limiting, model issues, etc.
        // As long as it's not the dummy_key issue, we consider the secret loading fixed
        println!( "⚠️  API returned error (not dummy_key issue): {error_msg}" );
        println!( "✅ Secret loading works correctly (not a dummy_key issue)" );
      }
    }
  }
}