//! Streaming chat completion example.
//!
//! Demonstrates real-time streaming responses using Server-Sent Events (SSE).
//!
//! Run with:
//! ```bash
//! cargo run --example streaming_chat --features integration,streaming
//! ```

#[ cfg( feature = "streaming" ) ]
use api_xai::{ Client, XaiEnvironmentImpl, Secret, ChatCompletionRequest, Message, ClientApiAccessors };
#[ cfg( feature = "streaming" ) ]
use futures_util::StreamExt;

#[ cfg( feature = "streaming" ) ]
#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  // Load API key
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
  let env = XaiEnvironmentImpl::new( secret )?;
  let client = Client::build( env )?;

  println!( "🚀 XAI Grok API - Streaming Chat Example\n" );

  // Create streaming request
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![
      Message::system( "You are a helpful assistant" ),
      Message::user( "Write a haiku about coding" ),
    ] )
    .max_tokens( 100u32 )
    .form();

  println!( "📤 Streaming response from Grok-3...\n" );
  print!( "🤖 Assistant : " );

  // Create streaming chat accessor
  let chat = client.chat();
  let mut stream = chat.create_stream( request ).await?;

  let mut chunks_received = 0;
  let mut content_parts = Vec::new();

  // Process stream chunks
  while let Some( chunk_result ) = stream.next().await
  {
    let chunk = chunk_result?;
    chunks_received += 1;

    // Extract and display content delta
    if let Some( delta ) = chunk.choices.first().map( | c | &c.delta )
    {
      if let Some( content ) = &delta.content
      {
        print!( "{content}" );
        content_parts.push( content.clone() );
      }
    }
  }

  let full_response = content_parts.join( "" );

  println!( "\n\n📊 Statistics:" );
  println!( "   - Chunks received : {chunks_received}" );
  println!( "   - Total characters : {}", full_response.len() );

  Ok( () )
}

#[ cfg( not( feature = "streaming" ) ) ]
fn main()
{
  eprintln!( "This example requires the 'streaming' feature." );
  eprintln!( "Run with : cargo run --example streaming_chat --features integration,streaming" );
}
