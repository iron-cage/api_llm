//! Interactive chat example with conversation history.
//!
//! Demonstrates multi-turn conversation with the XAI Grok API,
//! maintaining context across multiple exchanges.
//!
//! Run with:
//! ```bash
//! cargo run --example interactive_chat --features integration
//! ```
//!
//! Type 'exit', 'quit', or press Ctrl+C to end the conversation.

use api_xai::{ Client, XaiEnvironmentImpl, Secret, ChatCompletionRequest, Message, ClientApiAccessors };
use std::io::{ self, Write };

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  // Load API key from workspace or environment
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;

  // Create environment and client
  let env = XaiEnvironmentImpl::new( secret )?;
  let client = Client::build( env )?;

  println!( "🚀 XAI Grok API - Interactive Chat" );
  println!( "=================================" );
  println!( "Type 'exit' or 'quit' to end the conversation.\n" );

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

    // Check for exit commands
    if input.eq_ignore_ascii_case( "exit" ) || input.eq_ignore_ascii_case( "quit" )
    {
      println!( "\n👋 Goodbye!" );
      break;
    }

    // Skip empty inputs
    if input.is_empty()
    {
      continue;
    }

    // Add user message to history
    messages.push( Message::user( input ) );

    // Create request with full conversation history
    let request = ChatCompletionRequest::former()
      .model( "grok-2-1212".to_string() )
      .messages( messages.clone() )
      .max_tokens( 500u32 )
      .form();

    // Execute the request
    print!( "🤖 Assistant : " );
    io::stdout().flush()?;

    match client.chat().create( request ).await
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

            // Display token usage
            println!( "   [Tokens : {} prompt + {} completion = {} total]",
              response.usage.prompt_tokens,
              response.usage.completion_tokens,
              response.usage.total_tokens
            );
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
