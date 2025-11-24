//! Failover demonstration example.
//!
//! Shows how to configure automatic failover between multiple
//! XAI API endpoints with health tracking and rotation.
//!
//! Run with:
//! ```bash
//! cargo run --example failover_demo --features "integration,failover"
//! ```
//!
//! **Note**: This example uses simulated backup endpoints for demonstration.
//! In production, you would use actual backup endpoint URLs provided by XAI.

use api_xai::{
  Client, XaiEnvironmentImpl, Secret, ChatCompletionRequest,
  Message, FailoverConfig, ClientApiAccessors
};
use core::time::Duration;

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  println!( "🔄 XAI Grok API - Failover Demo\n" );
  println!( "================================\n" );

  // Load API key
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
  let env = XaiEnvironmentImpl::new( secret )?;

  // Configure failover with custom settings
  let failover_config = FailoverConfig::default()
    .with_max_failures( 3 )  // Mark unhealthy after 3 consecutive failures
    .with_retry_after( Duration::from_secs( 60 ) )  // Retry unhealthy endpoints after 60s
    .with_auto_rotate( true );  // Automatically rotate on unhealthy

  // Create client with multiple endpoints
  // Note : Using the same endpoint for demo purposes
  // In production, use distinct backup endpoints
  let client = Client::build( env )?
    .with_failover_config(
      vec![
        "https://api.x.ai/v1/".to_string(),
        // In production, add actual backup endpoints here:
        // "https://api-backup.x.ai/v1/".to_string(),
        // "https://api-backup2.x.ai/v1/".to_string(),
      ],
      failover_config
    );

  println!( "📊 Failover Configuration:" );
  println!( "   - Max failures before unhealthy : 3" );
  println!( "   - Retry delay : 60 seconds" );
  println!( "   - Auto-rotation : enabled\n" );

  // Display endpoint health
  if let Some( ref manager ) = client.failover_manager
  {
    let health = manager.endpoint_health();
    println!( "🏥 Endpoint Health Status:" );
    for ( i, ( endpoint, status ) ) in health.iter().enumerate()
    {
      println!( "   {}. {} - {:?}", i + 1, endpoint, status );
    }
    println!();

    println!( "📍 Current endpoint : {}\n", manager.current_endpoint() );
  }

  // Make a test request
  println!( "📤 Sending test request..." );

  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![
      Message::user( "Explain failover in one sentence." )
    ] )
    .max_tokens( 100u32 )
    .form();

  match client.chat().create( request ).await
  {
    Ok( response ) =>
    {
      println!( "✅ Request successful!\n" );

      if let Some( choice ) = response.choices.first()
      {
        if let Some( ref content ) = choice.message.content
        {
          println!( "🤖 Response : {content}\n" );
        }
      }

      // Display updated health after successful request
      if let Some( ref manager ) = client.failover_manager
      {
        let health = manager.endpoint_health();
        println!( "🏥 Updated Health Status:" );
        for ( i, ( endpoint, status ) ) in health.iter().enumerate()
        {
          println!( "   {}. {} - {:?}", i + 1, endpoint, status );
        }
      }
    }
    Err( e ) =>
    {
      println!( "❌ Request failed : {e}\n" );

      // Display health after failure
      if let Some( ref manager ) = client.failover_manager
      {
        let health = manager.endpoint_health();
        println!( "🏥 Health Status After Failure:" );
        for ( i, ( endpoint, status ) ) in health.iter().enumerate()
        {
          println!( "   {}. {} - {:?}", i + 1, endpoint, status );
        }
      }
    }
  }

  println!( "\n💡 In Production:" );
  println!( "   - Configure multiple distinct backup endpoints" );
  println!( "   - Monitor endpoint health metrics" );
  println!( "   - Set appropriate failure thresholds" );
  println!( "   - Adjust retry delays based on SLAs" );

  Ok( () )
}
