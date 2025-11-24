//! Enterprise features demonstration.
//!
//! Shows how to use circuit breaker, retry logic, and rate limiting
//! for production-grade applications.
//!
//! Run with:
//! ```bash
//! cargo run --example enterprise_features --features integration,circuit_breaker,retry,rate_limiting
//! ```

#[ cfg( all( feature = "circuit_breaker", feature = "retry", feature = "rate_limiting" ) ) ]
use api_xai::{
  Client, XaiEnvironmentImpl, Secret,
  ChatCompletionRequest, Message,
  CircuitBreaker, CircuitBreakerConfig,
  EnhancedRetryConfig,
  RateLimiter, RateLimiterConfig,
  ClientApiAccessors,
};
#[ cfg( all( feature = "circuit_breaker", feature = "retry", feature = "rate_limiting" ) ) ]
use core::time::Duration;

#[ cfg( all( feature = "circuit_breaker", feature = "retry", feature = "rate_limiting" ) ) ]
#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  println!( "🚀 XAI Grok API - Enterprise Features Example\n" );

  // Setup client
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
  let env = XaiEnvironmentImpl::new( secret )?;
  let client = Client::build( env )?;

  // === Circuit Breaker ===
  println!( "🔌 Circuit Breaker Configuration:" );
  let circuit_breaker = CircuitBreaker::new(
    CircuitBreakerConfig::default()
      .with_failure_threshold( 5 )      // Open after 5 failures
      .with_timeout( Duration::from_secs( 30 ) )  // Wait 30s before retry
      .with_success_threshold( 2 )      // Close after 2 successes
  );
  println!( "   - Failure threshold : 5" );
  println!( "   - Timeout : 30 seconds" );
  println!( "   - Success threshold : 2\n" );

  // === Retry Configuration ===
  println!( "🔄 Retry Configuration:" );
  let retry_config = EnhancedRetryConfig::default()
    .with_max_attempts( 3 )
    .with_base_delay( Duration::from_millis( 100 ) )
    .with_max_delay( Duration::from_secs( 10 ) )
    .with_jitter( true );
  println!( "   - Max attempts : 3" );
  println!( "   - Base delay : 100ms" );
  println!( "   - Max delay : 10 seconds" );
  println!( "   - Jitter : enabled\n" );

  // === Rate Limiting ===
  println!( "⏱️  Rate Limiter Configuration:" );
  let rate_limiter = RateLimiter::new(
    RateLimiterConfig::per_minute( 60 )  // 60 requests per minute
  );
  println!( "   - Rate : 60 requests/minute" );
  println!( "   - Available tokens : {}\n", rate_limiter.available_tokens() );

  // === Making Protected Requests ===
  println!( "📤 Making protected API requests...\n" );

  for i in 1..=3
  {
    println!( "Request #{i}" );

    // Acquire rate limit token
    rate_limiter.acquire().await;
    println!( "   ✓ Rate limit check passed" );

    // Check circuit breaker
    if !circuit_breaker.is_request_allowed()
    {
      println!( "   ✗ Circuit breaker is open, skipping request" );
      continue;
    }
    println!( "   ✓ Circuit breaker check passed" );

    // Execute with retry
    let request = ChatCompletionRequest::former()
      .model( "grok-2-1212".to_string() )
      .messages( vec![
        Message::user( format!( "Say hello in request {i}" ) )
      ] )
      .max_tokens( 50u32 )
      .form();

    let result = retry_config.call( || {
      let client = client.clone();
      let request = request.clone();
      async move {
        client.chat().create( request ).await
      }
    } ).await;

    match result
    {
      Ok( response ) =>
      {
        circuit_breaker.record_success();
        println!( "   ✓ Request succeeded" );

        if let Some( content ) = response.choices[ 0 ].message.content.as_ref()
        {
          println!( "   🤖 Response : {}", content.split( '\n' ).next().unwrap_or( "" ) );
        }
      }
      Err( e ) =>
      {
        circuit_breaker.record_failure();
        println!( "   ✗ Request failed : {e}" );
      }
    }

    println!();
  }

  println!( "📊 Final Statistics:" );
  println!( "   - Circuit breaker state : {:?}", circuit_breaker.state() );
  println!( "   - Available rate limit tokens : {}", rate_limiter.available_tokens() );

  Ok( () )
}

#[ cfg( not( all( feature = "circuit_breaker", feature = "retry", feature = "rate_limiting" ) ) ) ]
fn main()
{
  eprintln!( "This example requires 'circuit_breaker', 'retry', and 'rate_limiting' features." );
  eprintln!( "Run with:" );
  eprintln!( "cargo run --example enterprise_features --features integration,circuit_breaker,retry,rate_limiting" );
}
