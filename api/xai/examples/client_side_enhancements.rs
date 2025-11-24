//! Demonstrates client-side enhancement features.
//!
//! This example shows how to use:
//! - Token counting
//! - Response caching
//! - Input validation
//! - Batch operations
//! - Performance metrics
//! - CURL diagnostics

use api_xai::
{
  Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message,
};

#[ cfg( feature = "count_tokens" ) ]
use api_xai::count_tokens::{ count_tokens, count_tokens_for_request, validate_request_size };

#[ cfg( feature = "caching" ) ]
use api_xai::CachedClient;

#[ cfg( feature = "input_validation" ) ]
use api_xai::validate_request;

#[ cfg( feature = "batch_operations" ) ]
use api_xai::BatchProcessor;

#[ cfg( feature = "performance_metrics" ) ]
use api_xai::MetricsCollector;

#[ cfg( feature = "curl_diagnostics" ) ]
use api_xai::to_curl;

#[ allow( clippy::too_many_lines ) ]  // Demo file showcasing multiple features
#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  println!( "🚀 XAI Client-Side Enhancements Demo\n" );

  // Load API key
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
  let env = XaiEnvironmentImpl::new( secret )?;
  let client = Client::build( env )?;

  // 1. Token Counting
  #[ cfg( feature = "count_tokens" ) ]
  {
    println!( "📊 1. Token Counting" );
    println!( "===================" );

    let text = "Hello, how are you today?";
    let token_count = count_tokens( text, "grok-2-1212" )?;
    println!( "   Text : \"{text}\"" );
    println!( "   Tokens : {token_count}\n" );

    let request = ChatCompletionRequest::former()
      .model( "grok-2-1212".to_string() )
      .messages( vec![ Message::user( "What is the meaning of life?" ) ] )
      .form();

    let request_tokens = count_tokens_for_request( &request )?;
    println!( "   Request total tokens : {request_tokens}" );

    // Validate request fits in context window (Grok-3 has 131K tokens)
    validate_request_size( &request, 131_072 )?;
    println!( "   ✓ Request fits in context window\n" );
  }

  // 2. Input Validation
  #[ cfg( feature = "input_validation" ) ]
  {
    println!( "✅ 2. Input Validation" );
    println!( "======================" );

    let request = ChatCompletionRequest::former()
      .model( "grok-2-1212".to_string() )
      .messages( vec![ Message::user( "Hello!" ) ] )
      .temperature( 0.7 )
      .max_tokens( 100_u32 )
      .form();

    match validate_request( &request )
    {
      Ok( () ) => println!( "   ✓ Request validation passed" ),
      Err( e ) => println!( "   ✗ Validation failed : {e}" ),
    }

    // Try invalid temperature
    let bad_request = ChatCompletionRequest::former()
      .model( "grok-2-1212".to_string() )
      .messages( vec![ Message::user( "Hello!" ) ] )
      .temperature( 3.0 )  // Invalid!
      .form();

    match validate_request( &bad_request )
    {
      Ok( () ) => println!( "   ✗ Should have failed validation" ),
      Err( e ) => println!( "   ✓ Correctly rejected invalid request : {e}\n" ),
    }
  }

  // 3. CURL Diagnostics
  #[ cfg( feature = "curl_diagnostics" ) ]
  {
    println!( "🔍 3. CURL Diagnostics" );
    println!( "======================" );

    let request = ChatCompletionRequest::former()
      .model( "grok-2-1212".to_string() )
      .messages( vec![ Message::user( "Debug this!" ) ] )
      .temperature( 0.5 )
      .form();

    let curl_command = to_curl( &request );
    println!( "   Generated CURL command:" );
    println!( "   {}\n", curl_command.lines().take( 3 ).collect::< Vec< _ > >().join( "\n   " ) );
  }

  // 4. Response Caching
  #[ cfg( feature = "caching" ) ]
  {
    println!( "💾 4. Response Caching" );
    println!( "======================" );

    let cached_client = CachedClient::new( client.clone(), 10 );

    let request = ChatCompletionRequest::former()
      .model( "grok-2-1212".to_string() )
      .messages( vec![ Message::user( "What is 2+2?" ) ] )
      .max_tokens( 10_u32 )
      .form();

    println!( "   First request (cache miss)..." );
    let start = std::time::Instant::now();
    let _response1 = cached_client.cached_create( request.clone() ).await?;
    let duration1 = start.elapsed();
    println!( "   ✓ Response received in {duration1:?}" );

    println!( "   Second request (cache hit)..." );
    let start = std::time::Instant::now();
    let _response2 = cached_client.cached_create( request.clone() ).await?;
    let duration2 = start.elapsed();
    println!( "   ✓ Response received in {duration2:?}" );
    println!( "   ⚡ Speedup : {:.0}x faster\n", duration1.as_secs_f64() / duration2.as_secs_f64().max( 0.000_001 ) );
  }

  // 5. Batch Operations
  #[ cfg( feature = "batch_operations" ) ]
  {
    println!( "📦 5. Batch Operations" );
    println!( "======================" );

    let processor = BatchProcessor::new( client.clone(), 3 );

    let requests = vec!
    [
      ChatCompletionRequest::former()
        .model( "grok-2-1212".to_string() )
        .messages( vec![ Message::user( "What is 1+1?" ) ] )
        .max_tokens( 5_u32 )
        .form(),
      ChatCompletionRequest::former()
        .model( "grok-2-1212".to_string() )
        .messages( vec![ Message::user( "What is 2+2?" ) ] )
        .max_tokens( 5_u32 )
        .form(),
      ChatCompletionRequest::former()
        .model( "grok-2-1212".to_string() )
        .messages( vec![ Message::user( "What is 3+3?" ) ] )
        .max_tokens( 5_u32 )
        .form(),
    ];

    println!( "   Processing {} requests in parallel (max 3 concurrent)...", requests.len() );
    let start = std::time::Instant::now();
    let results = processor.process_batch( requests ).await;
    let duration = start.elapsed();

    let successes = results.iter().filter( | r | r.is_ok() ).count();
    println!( "   ✓ Completed {}/{} requests in {:?}", successes, results.len(), duration );
    #[ allow( clippy::cast_possible_truncation ) ]  // Small batch size in demo
    let avg = duration / results.len() as u32;
    println!( "   Average : {avg:?} per request\n" );
  }

  // 6. Performance Metrics
  #[ cfg( feature = "performance_metrics" ) ]
  {
    println!( "📈 6. Performance Metrics" );
    println!( "=========================" );

    let metrics = MetricsCollector::new();

    // Simulate some requests
    metrics.record_request( core::time::Duration::from_millis( 250 ), 150, true );
    metrics.record_request( core::time::Duration::from_millis( 180 ), 75, true );
    metrics.record_request( core::time::Duration::from_millis( 320 ), 200, false );

    let prometheus_text = metrics.export();
    let lines : Vec< &str > = prometheus_text.lines().take( 10 ).collect();
    println!( "   Prometheus metrics (first 10 lines):" );
    for line in lines
    {
      println!( "   {line}" );
    }
    println!();
  }

  println!( "✨ Client-side enhancements demo completed!\n" );

  Ok( () )
}
