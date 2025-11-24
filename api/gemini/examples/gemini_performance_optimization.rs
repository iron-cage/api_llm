//! Performance optimization patterns for `api_gemini`
//!
//! This example demonstrates various techniques to optimize performance
//! when using the Gemini API client in production applications.
//! Note : No benchmarking - focuses on usage patterns and best practices.

use api_gemini::{ client::Client, models::*, error::Error };
use std::time::Instant;
use tokio::time::{ Duration, sleep };

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  println!( "🚀 API Gemini Performance Optimization Examples\n" );

  // Pattern 1: Reuse client instances
  demonstrate_client_reuse().await?;

  // Pattern 2: Optimize request structure
  demonstrate_request_optimization().await?;

  // Pattern 3: Batch processing with rate limiting
  demonstrate_batch_processing().await?;

  // Pattern 4: Error-resilient patterns
  demonstrate_error_resilience().await?;

  println!( "\n✅ All performance optimization examples completed!" );
  Ok( () )
}

/// Demonstrate the importance of reusing client instances
async fn demonstrate_client_reuse() -> Result< (), Box< dyn core::error::Error > >
{
  println!( "📊 Pattern 1: Client Reuse vs Recreation\n" );

  let prompts = vec![ "Hello", "How are you?", "What's the weather?" ];

  // ❌ INEFFICIENT: Creating new client for each request
  let start = Instant::now();
  for prompt in &prompts
  {
    let client = Client::new()?; // Expensive operation
    let _ = make_simple_request( &client, prompt ).await;
  }
  let inefficient_time = start.elapsed();
println!( "❌ Multiple clients : {inefficient_time:?}" );

  // ✅ EFFICIENT: Reusing single client instance
  let start = Instant::now();
  let client = Client::new()?; // Create once
  for prompt in &prompts
  {
    let _ = make_simple_request( &client, prompt ).await;
  }
  let efficient_time = start.elapsed();
println!( "✅ Single client : {efficient_time:?}" );

println!( "💡 Performance improvement : ~{:.1}x faster\n",
  inefficient_time.as_secs_f64() / efficient_time.as_secs_f64() );

  Ok( () )
}

/// Demonstrate optimized request structure patterns
async fn demonstrate_request_optimization() -> Result< (), Box< dyn core::error::Error > >
{
  println!( "📊 Pattern 2: Request Structure Optimization\n" );

  let client = Client::new()?;

  // ❌ INEFFICIENT: Creating new request structure each time
  let start = Instant::now();
  for i in 0..3
  {
    let request = GenerateContentRequest {
      contents : vec![ Content {
        parts : vec![ Part {
        text : Some( format!( "Request {i}" ) ),
          ..Default::default()
        } ],
        role: "user".to_string(),
      } ],
      generation_config : Some( GenerationConfig {
        temperature: Some( 0.7 ),
        top_k: Some( 40 ),
        max_output_tokens: Some( 100 ),
        ..Default::default()
      } ),
      ..Default::default()
    };
    let _ = client.models().by_name( "gemini-2.5-flash" ).generate_content( &request ).await;
  }
  let inefficient_time = start.elapsed();
println!( "❌ Recreating request structure : {inefficient_time:?}" );

  // ✅ EFFICIENT: Reusing template and modifying only necessary parts
  let start = Instant::now();
  let mut request_template = GenerateContentRequest {
    contents : vec![ Content {
      parts : vec![ Part {
        text: Some( String::new() ), // Will be updated
        ..Default::default()
      } ],
      role: "user".to_string(),
    } ],
    generation_config : Some( GenerationConfig {
      temperature: Some( 0.7 ),
      top_k: Some( 40 ),
      max_output_tokens: Some( 100 ),
      ..Default::default()
    } ),
    ..Default::default()
  };

  for i in 0..3
  {
    // Only update the text field
  request_template.contents[ 0 ].parts[ 0 ].text = Some( format!( "Request {i}" ) );
    let _ = client.models().by_name( "gemini-2.5-flash" ).generate_content( &request_template ).await;
  }
  let efficient_time = start.elapsed();
println!( "✅ Reusing request template : {efficient_time:?}" );

  println!( "💡 Serialization overhead reduced\n" );

  Ok( () )
}

/// Demonstrate efficient batch processing with rate limiting
async fn demonstrate_batch_processing() -> Result< (), Box< dyn core::error::Error > >
{
  println!( "📊 Pattern 3: Batch Processing with Rate Limiting\n" );

  let client = Client::new()?;
  let prompts = vec![ "Task 1", "Task 2", "Task 3", "Task 4", "Task 5" ];

  // ❌ INEFFICIENT: Sequential processing without rate limiting
  let start = Instant::now();
  for prompt in &prompts
  {
    let _ = make_simple_request( &client, prompt ).await;
    // No rate limiting - may hit API limits
  }
  let uncontrolled_time = start.elapsed();
println!( "❌ Uncontrolled sequential : {uncontrolled_time:?}" );

  // ✅ EFFICIENT: Controlled batch processing
  let start = Instant::now();
  let batch_size = 3;
  let delay_between_batches = Duration::from_millis( 500 );

  for batch in prompts.chunks( batch_size )
  {
    // Process batch concurrently
    let batch_futures: Vec< _ > = batch.iter()
    .map( |prompt| make_simple_request( &client, prompt ) )
    .collect();

    let _batch_results: Vec< _ > = futures::future::join_all( batch_futures ).await;

    // Rate limiting between batches
    sleep( delay_between_batches ).await;
  }
  let controlled_time = start.elapsed();
println!( "✅ Controlled batch processing : {controlled_time:?}" );

  println!( "💡 Better rate limit compliance and predictable load\n" );

  Ok( () )
}

/// Demonstrate error-resilient patterns for better performance
async fn demonstrate_error_resilience() -> Result< (), Box< dyn core::error::Error > >
{
  println!( "📊 Pattern 4: Error-Resilient Performance Patterns\n" );

  let client = Client::new()?;

  // ✅ RESILIENT: Fail fast with informative errors
  match make_request_with_timeout( &client, "Test prompt", Duration::from_secs( 5 ) ).await
  {
    Ok( _response ) => println!( "✅ Request succeeded" ),
  Err( Error::TimeoutError( msg ) ) => println!( "⏱️ Timeout handled gracefully : {msg}" ),
    Err( Error::RateLimitError( msg ) ) => {
    println!( "🚦 Rate limit detected : {msg}" );
      println!( "💡 Application can implement exponential backoff" );
    },
  Err( e ) => println!( "❌ Other error : {e}" ),
  }

  // ✅ RESILIENT: Circuit breaker pattern (simulation)
  let mut consecutive_failures = 0;
  let max_failures = 3;

  for i in 0..5
  {
    if consecutive_failures >= max_failures
    {
      println!( "🔌 Circuit breaker opened - preventing requests" );
      break;
    }

    if (make_simple_request( &client, &format!( "Request {i}" ) ).await).is_ok()
    {
      consecutive_failures = 0; // Reset on success
      println!( "✅ Request {i} succeeded" );
    }
    else
    {
      consecutive_failures += 1;
      println!( "❌ Request {i} failed ({consecutive_failures}/{max_failures})" );
    }
  }

  println!( "💡 Circuit breaker prevents cascade failures\n" );

  Ok( () )
}

/// Helper function to make a simple request
async fn make_simple_request( client: &Client, prompt: &str ) -> Result< GenerateContentResponse, Error >
{
  let request = GenerateContentRequest {
    contents : vec![ Content {
      parts : vec![ Part {
        text: Some( prompt.to_string() ),
        ..Default::default()
      } ],
      role: "user".to_string(),
    } ],
    generation_config : Some( GenerationConfig {
      max_output_tokens: Some( 50 ), // Keep responses small for performance
      ..Default::default()
    } ),
    ..Default::default()
  };

  client
  .models()
  .by_name( "gemini-2.5-flash" )
  .generate_content( &request )
  .await
}

/// Helper function to make a request with timeout
async fn make_request_with_timeout(
client: &Client,
prompt: &str,
timeout: Duration
) -> Result< GenerateContentResponse, Error >
{
  let request_future = make_simple_request( client, prompt );

  match tokio::time::timeout( timeout, request_future ).await
  {
    Ok( result ) => result,
    Err( _ ) => Err( Error::TimeoutError(
  format!( "Request timed out after {timeout:?}" )
    ) ),
  }
}