//! Basic chat conversation example with the `OpenAI` API.
//!
//! This example demonstrates:
//! - Simple client initialization from environment variables
//! - Creating a basic text generation request
//! - Sending a single-turn conversation to the AI
//! - Handling responses and extracting generated text
//! - Basic error handling patterns
//! - Request transparency with JSON payload and cURL generation
//! - Token usage tracking and display
//!
//! ## Usage
//!
//! ```bash
//! # Set your API key
//! export OPENAI_API_KEY="your-api-key-here"
//!
//! # Run the example
//! cargo run --example chat
//! ```
//!
//! ## What You'll Learn
//!
//! - How to initialize the `OpenAI` client
//! - Basic request structure for text generation
//! - How to access generated content from responses
//! - Essential error handling for API calls
//! - Understanding request transparency and debugging
//! - Token usage monitoring and cost management
//!
//! **Complexity**: ⭐ (Basic)
//! **Target Audience**: Beginners learning `OpenAI` API fundamentals
//!
//! This is perfect for beginners to understand the basic flow of using the `OpenAI` API.

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput, ResponseObject },
    output ::{ OutputItem, OutputContentPart },
    common ::ModelIdsResponses,
  },
};

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  println!( "OpenAI Basic Chat Example" );
  println!( "{}", "=".repeat( 50 ) );
  println!( "Demonstrating single-turn conversation with comprehensive configuration\n" );

  // Initialize the client with API key from environment variable
  let secret = api_openai::secret::Secret::load_with_fallbacks( "OPENAI_API_KEY" )
    .expect( "Failed to load OPENAI_API_KEY. Please set environment variable or add to workspace secrets file." );

  let env = api_openai::environment::OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    api_openai ::environment::OpenAIRecommended::base_url().to_string(),
    api_openai ::environment::OpenAIRecommended::realtime_base_url().to_string()
  ).expect( "Failed to create environment" );

  let client = Client::build( env ).expect( "Failed to create client" );

  // Create a simple conversation request with comprehensive configuration
  // Using the same question as the Gemini reference for consistency
  let request = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5.1-chat-latest".to_string() ) )
    .input( ResponseInput::String( "Hello! Can you explain what artificial intelligence is in simple terms?".to_string() ) )
    .temperature( 0.7 )
    .max_output_tokens( 1024 )
    .top_p( 0.95 )
    .form();

  // Request transparency : Show JSON payload for educational purposes
  println!( "=== Request JSON Payload ===" );
  let json_payload = serde_json::to_string_pretty( &request )?;
  println!( "{json_payload}" );
  println!();

  // Request transparency : Show cURL command for debugging
  // Note : cURL generation available through CurlGeneration trait
  println!( "=== cURL Command Generation ===" );
  println!( "Note : cURL command generation available through response client interface" );
  println!();

  println!( "Sending request to OpenAI API..." );
  println!( "Model : gpt-4o" );
  println!( "Temperature : 0.7 (balanced creativity)" );
  println!( "Max tokens : 1024" );
  println!( "Top-p : 0.95 (nucleus sampling)" );
  println!();

  // Send the request and handle the response
  let response : ResponseObject = client.responses().create( request ).await?;

  println!( "✅ Response received successfully!\n" );

  // Display the AI's response
  println!( "=== AI Response ===" );

  // Use the correct pattern matching for OpenAI API response structure
  if let Some( OutputItem::Message( message_struct ) ) = response.output.first()
  {
    // Access the content field within the nested OutputMessage struct
    if let Some( OutputContentPart::Text { text, .. } ) = message_struct.content.first()
    {
      println!( "{text}" );
    }
    else
    {
      println!( "No text content found in response." );
    }
  }
  else
  {
    println!( "No message output received in response." );
  }

  println!();

  // Display token usage information for cost management
  if let Some( usage ) = response.usage
  {
    println!( "=== Token Usage Information ===" );
    println!( "Input tokens : {}", usage.prompt_tokens );
    if let Some( completion_tokens ) = usage.completion_tokens
    {
      println!( "Output tokens : {completion_tokens}" );
    }
    println!( "Total tokens : {}", usage.total_tokens );

    // Educational note about cost implications
    println!( "\n💡 Note : Token usage directly impacts API costs." );
    println!( "Monitor usage to optimize both performance and expenses." );
  }
  else
  {
    println!( "No token usage information available." );
  }

  println!( "\n✨ Single-turn conversation completed successfully!" );
  println!( "This demonstrates the fundamental OpenAI API request-response cycle." );

  Ok( () )
}