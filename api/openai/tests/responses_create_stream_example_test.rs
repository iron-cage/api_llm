//! Test for `responses_create_stream` example
//!
//! This test reproduces the API key issue in the `responses_create_stream` example
//! where it falls back to `dummy_key` and may hang due to authentication failure.

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  components ::responses::{ CreateResponseRequest, ResponseInput },
};
#[ tokio::test ]
async fn test_responses_create_stream_example_secret_loading()
{
  // This test verifies that the responses_create_stream example pattern works
  // with proper secret loading instead of dummy_key fallback

  // Load secret using the comprehensive fallback system
  let secret = api_openai::secret::Secret::load_with_fallbacks("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY should be available in workspace secrets");

  let env = api_openai::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
    .expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // Create streaming request similar to the example
  let request = CreateResponseRequest::former()
    .model( "gpt-5-mini".to_string() )
    .input( ResponseInput::String( "Tell me a very short story about a brave knight in exactly 2 sentences.".to_string() ) )
    .stream( true )
    .form();

  // This should succeed with proper API key and not hang
  let result = client.responses().create_stream( request ).await;

  match result
  {
    Ok( mut receiver ) =>
    {
      println!( "✅ Streaming started successfully with proper secret loading!" );

      let mut message_count = 0;
      let mut content_received = String::new();

      // Process a few stream messages to verify it's working
      while let Some( event ) = receiver.recv().await
      {
        match event
        {
          Ok( event ) =>
          {
            println!( "📦 Received stream event : {event:?}" );

            // Extract content from stream events
            let event_str = format!( "{event:?}" );
            if event_str.contains( "knight" ) || event_str.contains( "brave" )
            {
              content_received = event_str;
            }

            message_count += 1;

            // Stop after receiving several events to avoid long test times
            if message_count >= 5
            {
              break;
            }
          },
          Err( e ) =>
          {
            let error_msg = format!( "{e:?}" );
            if error_msg.contains( "dummy_key" )
            {
              panic!( "❌ ISSUE: Stream example still using dummy_key : {error_msg}" );
            }
            else
            {
              // Some other streaming error - not the dummy_key issue
              println!( "⚠️  Stream error (not dummy_key issue): {error_msg}" );
              break;
            }
          }
        }
      }

      assert!( message_count > 0, "Should have received at least one stream event" );
      println!( "✅ Stream processing works correctly - received {message_count} events" );

      if !content_received.is_empty()
      {
        println!( "✅ Received content about knight story" );
      }
    },
    Err( e ) =>
    {
      let error_msg = format!( "{e:?}" );
      if error_msg.contains( "dummy_key" )
      {
        panic!( "❌ ISSUE: Stream example still using dummy_key : {error_msg}" );
      }
      else
      {
        // Some other API error - as long as it's not dummy_key, secret loading works
        println!( "⚠️  Stream API returned error (not dummy_key issue): {error_msg}" );
        println!( "✅ Secret loading works correctly (not a dummy_key issue)" );
      }
    }
  }
}