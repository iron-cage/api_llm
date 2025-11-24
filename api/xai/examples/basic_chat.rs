//! Basic chat completion example.
//!
//! Demonstrates simple chat API usage with the XAI Grok API.
//!
//! Run with:
//! ```bash
//! cargo run --example basic_chat --features integration
//! ```

use api_xai::{ Client, XaiEnvironmentImpl, Secret, ChatCompletionRequest, Message, ClientApiAccessors };

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  // Load API key from workspace or environment
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;

  // Create environment and client
  let env = XaiEnvironmentImpl::new( secret )?;
  let client = Client::build( env )?;

  println!( "🚀 XAI Grok API - Basic Chat Example\n" );

  // Create a simple chat request
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![
      Message::user( "What is the capital of France?" )
    ] )
    .max_tokens( 100u32 )
    .form();

  println!( "📤 Sending request to Grok-3..." );

  // Execute the request
  let response = client.chat().create( request ).await?;

  // Display the response
  println!( "✅ Response received:\n" );
  for choice in response.choices
  {
    if let Some( content ) = choice.message.content
    {
      println!( "🤖 Assistant : {content}\n" );
    }
  }

  // Display usage statistics
  println!( "📊 Token Usage:" );
  println!( "   - Prompt : {} tokens", response.usage.prompt_tokens );
  println!( "   - Completion : {} tokens", response.usage.completion_tokens );
  println!( "   - Total : {} tokens", response.usage.total_tokens );

  Ok( () )
}
