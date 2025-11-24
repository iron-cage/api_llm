//! Example demonstrating how to list available Gemini models and their capabilities.
//!
//! This example shows:
//! - How to initialize the client with environment variables
//! - How to list all available models
//! - How to access model properties like token limits and supported features
//! - How to fetch details about a specific model

use api_gemini::client::Client;

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  // Initialize client from GEMINI_API_KEY environment variable
  // The client will automatically look for the API key in the environment
  let client = Client::new()?;

  println!( "=== Listing Available Gemini Models ===" );

  // List all available models
  let models_response = client.models().list().await?;

println!( "Found {} models:\n", models_response.models.len() );

  // Display information about each model
  for model in &models_response.models
  {
  println!( "Model : {}", model.name );

    // Display name is a human-friendly version of the model name
    if let Some( display_name ) = &model.display_name
    {
    println!( "  Display Name : {display_name}" );
    }

    // Description explains what the model is designed for
    if let Some( description ) = &model.description
    {
    println!( "  Description : {description}" );
    }

    // Token limits define the maximum input/output size
    // Important for planning your prompts and managing costs
    if let Some( input_limit ) = model.input_token_limit
    {
    println!( "  Max Input Tokens : {input_limit}" );
    }

    if let Some( output_limit ) = model.output_token_limit
    {
    println!( "  Max Output Tokens : {output_limit}" );
    }

    // Supported generation methods indicate what operations are available
    // Common methods : generateContent, embedContent, countTokens
    if let Some( methods ) = &model.supported_generation_methods
    {
    println!( "  Supported Methods : {}", methods.join( ", " ) );
    }

    println!();
  }

  // Example : Get detailed information about a specific model
  println!( "=== Getting Specific Model Details ===" );

  let model_name = "models/gemini-2.5-flash";
  let specific_model = client.models().get( model_name ).await?;

println!( "Detailed info for {}:", specific_model.name );

  // Model parameters affect generation behavior
  // These are the default values used if not specified in requests
  if let Some( temp ) = specific_model.temperature
  {
  println!( "  Default Temperature : {temp} (controls randomness)" );
  }

  if let Some( top_p ) = specific_model.top_p
  {
  println!( "  Default Top-P: {top_p} (nucleus sampling)" );
  }

  if let Some( top_k ) = specific_model.top_k
  {
  println!( "  Default Top-K: {top_k} (top-k sampling)" );
  }

  Ok( () )
}