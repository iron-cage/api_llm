//! Interactive chat example with conversation history and response caching.
//!
//! Demonstrates multi-turn conversation with the XAI Grok API,
//! maintaining context across multiple exchanges while caching
//! responses for improved performance and cost reduction.
//!
//! Run with:
//! ```bash
//! cargo run --example cached_interactive_chat --features "integration,caching"
//! ```
//!
//! Commands:
//! - 'exit' or 'quit' - End the conversation
//! - 'clear' - Clear conversation history and cache
//! - 'cache' - Show cache statistics
//! - Ctrl+C - End the conversation

use api_xai::{ CachedClient, Client, XaiEnvironmentImpl, Secret, ChatCompletionRequest, Message };
use std::io::{ self, Write };

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  // Load API key from workspace or environment
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;

  // Create environment and client
  let env = XaiEnvironmentImpl::new( secret )?;
  let client = Client::build( env )?;

  // Wrap with caching (capacity: 100 responses)
  let cached_client = CachedClient::new( client, 100 );

  println!( "🚀 XAI Grok API - Interactive Chat (with caching)" );
  println!( "================================================" );
  println!( "Type 'exit' or 'quit' to end the conversation." );
  println!( "Type 'clear' to reset conversation and cache." );
  println!( "Type 'cache' to show cache statistics.\n" );

  // Maintain conversation history
  let mut messages : Vec< Message > = Vec::new();

  loop
  {
    // Get user input
    print!( "You : " );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line( &mut input )?;
    let input = input.trim();

    // Check for commands
    match input.to_lowercase().as_str()
    {
      "exit" | "quit" =>
      {
        println!( "\n👋 Goodbye!" );
        println!( "   Final cache size : {} entries", cached_client.len() );
        break;
      }
      "clear" =>
      {
        messages.clear();
        cached_client.clear();
        println!( "✨ Conversation history and cache cleared.\n" );
        continue;
      }
      "cache" =>
      {
        println!( "📊 Cache Statistics:" );
        println!( "   Size : {} entries", cached_client.len() );
        println!( "   Empty : {}\n", cached_client.is_empty() );
        continue;
      }
      "" => continue,
      _ => {}
    }

    // Add user message to history
    messages.push( Message::user( input ) );

    // Create request with full conversation history
    let request = ChatCompletionRequest::former()
      .model( "grok-2-1212".to_string() )
      .messages( messages.clone() )
      .max_tokens( 500u32 )
      .form();

    // Execute the request (with caching)
    print!( "🤖 Assistant : " );
    io::stdout().flush()?;

    match cached_client.cached_create( request ).await
    {
      Ok( response ) =>
      {
        if let Some( choice ) = response.choices.first()
        {
          if let Some( ref content ) = choice.message.content
          {
            println!( "{content}\n" );

            // Add assistant response to history
            messages.push( choice.message.clone() );

            // Display token usage and cache info
            println!( "   [Tokens : {} prompt + {} completion = {} total]",
              response.usage.prompt_tokens,
              response.usage.completion_tokens,
              response.usage.total_tokens
            );
            println!( "   [Cache : {} entries]", cached_client.len() );
            println!();
          }
          else
          {
            println!( "[No content in response]\n" );
          }
        }
        else
        {
          println!( "[No choices in response]\n" );
        }
      }
      Err( e ) =>
      {
        println!( "\n❌ Error : {e}\n" );
        // Remove the last user message since we got an error
        messages.pop();
      }
    }
  }

  Ok( () )
}
