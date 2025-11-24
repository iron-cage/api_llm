//! Basic chat conversation example with the Gemini API.
//!
//! This example demonstrates:
//! - Simple client initialization from environment variables
//! - Creating a basic text generation request
//! - Sending a single-turn conversation to the AI
//! - Handling responses and extracting generated text
//! - Basic error handling patterns
//!
//! ## Usage
//!
//! ```bash
//! # Set your API key
//! export GEMINI_API_KEY="your-api-key-here"
//!
//! # Run the example
//! cargo run --example chat
//! ```
//!
//! ## What You'll Learn
//!
//! - How to initialize the Gemini client
//! - Basic request structure for text generation
//! - How to access generated content from responses
//! - Essential error handling for API calls
//!
//! This is perfect for beginners to understand the basic flow of using the Gemini API.

use api_gemini::{ client::Client, models::* };

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  // Initialize the client with API key from environment variable
  let client = Client::new()?;

  // Create a simple conversation request
  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      parts: vec!
      [
      Part
      {
        text: Some( "Hello! Can you explain what artificial intelligence is in simple terms?".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
      role: "user".to_string(),
    }
    ],
    generation_config: Some( GenerationConfig
    {
      temperature: Some( 0.7 ),
      top_k: Some( 40 ),
      top_p: Some( 0.95 ),
      candidate_count: Some( 1 ),
      max_output_tokens: Some( 1024 ),
      stop_sequences: None,
    }),
    safety_settings: Some( vec!
    [
    SafetySetting
    {
      category: "HARM_CATEGORY_HARASSMENT".to_string(),
      threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
    },
    SafetySetting
    {
      category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
      threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
    }
    ]),
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  // Show the exact request being sent for API transparency
  #[ cfg( feature = "diagnostics_curl" ) ]
  {
    use api_gemini::diagnostics::AsCurl;
    println!( "=== Exact Curl Command Being Executed ===" );
  println!( "{}", request.as_curl() );
    println!( "=== End Curl Command ===\n" );
  }

  // Always show the JSON payload for transparency
  println!( "=== Request JSON Payload ===" );
println!( "{}", serde_json::to_string_pretty( &request )? );
  println!( "=== End JSON Payload ===\n" );

  println!( "Sending request to Gemini API..." );

  // Generate content using the Gemini model
  let response = client
  .models()
  .by_name( "gemini-2.5-flash" )
  .generate_content( &request )
  .await?;

  // Process and display the response
  if let Some( candidate ) = response.candidates.first()
  {
    if let Some( part ) = candidate.content.parts.first()
    {
      if let Some( text ) = &part.text
      {
        println!( "\n=== Gemini Response ===" );
      println!( "{text}" );
      }
    }

    if let Some( finish_reason ) = &candidate.finish_reason
    {
    println!( "\nFinish reason : {finish_reason}" );
    }
  }
  else
  {
    println!( "No response received from the API." );
  }

  // Display usage metadata if available
  if let Some( usage ) = &response.usage_metadata
  {
    println!( "\n=== Token Usage ===" );
    if let Some( prompt_tokens ) = usage.prompt_token_count
    {
    println!( "Prompt tokens : {prompt_tokens}" );
    }
    if let Some( candidate_tokens ) = usage.candidates_token_count
    {
    println!( "Response tokens : {candidate_tokens}" );
    }
    if let Some( total_tokens ) = usage.total_token_count
    {
    println!( "Total tokens : {total_tokens}" );
    }
  }

  Ok( () )
}