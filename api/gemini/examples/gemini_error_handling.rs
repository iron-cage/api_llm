//! Example demonstrating comprehensive error handling strategies.
//!
//! This example shows:
//! - How to handle different types of errors
//! - Retry strategies for transient failures
//! - Rate limit handling
//! - Graceful degradation techniques
//! - Error logging and monitoring patterns


use api_gemini::{ client::Client, models::* };
use core::time::Duration;
use tokio::time::sleep;

/// Retry configuration for API calls
struct RetryConfig
{
  max_attempts: u32,
  initial_delay: Duration,
  max_delay: Duration,
  exponential_base: f32,
}

impl Default
for RetryConfig
{
  fn default() -> Self
  {
    RetryConfig
    {
      max_attempts: 3,
      initial_delay: Duration::from_secs( 1 ),
      max_delay: Duration::from_secs( 60 ),
      exponential_base: 2.0,
    }
  }
}

/// Execute an API call with retry logic
async fn call_with_retry< F, T >
(
operation: F,
config: &RetryConfig,
operation_name: &str,
)
->
Result< T, api_gemini::error::Error >
where
F: Fn() -> core::pin::Pin< Box< dyn core::future::Future< Output = Result< T, api_gemini::error::Error > > + Send > >,
{
  let mut attempt = 0;
  let mut delay = config.initial_delay;

  loop
  {
    attempt += 1;
println!( "Attempt {attempt} for {operation_name}" );

    match operation().await
    {
      Ok( result ) => return Ok( result ),
      Err( error ) =>
      {
    println!( "Error on attempt {attempt}: {error:?}" );

        // Check if error is retryable
        let should_retry = matches!( &error, api_gemini::error::Error::RateLimitError( _ ) | api_gemini::error::Error::NetworkError( _ ) | api_gemini::error::Error::ServerError( _ ) );

        if !should_retry || attempt >= config.max_attempts
        {
          return Err( error );
        }

        // Special handling for rate limits
        if let api_gemini::error::Error::RateLimitError( msg ) = &error
        {
        println!( "Rate limit hit : {msg}. Waiting longer..." );
          delay = config.max_delay; // Use max delay for rate limits
        }

      println!( "Retrying after {delay:?}..." );
        sleep( delay ).await;

        // Exponential backoff
        delay = core::cmp::min(
        Duration::from_secs_f32( delay.as_secs_f32() * config.exponential_base ),
        config.max_delay
        );
      }
    }
  }
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  println!( "=== Error Handling Example ===" );

  // Example 1: Handling missing API key
  println!( "\n1. Handling Missing API Key" );

  match Client::builder().build()
  {
    Ok( _ ) => println!( "Client created successfully" ),
    Err( e ) => match e
    {
      api_gemini ::error::Error::AuthenticationError( msg ) =>
      {
      println!( "Authentication error as expected : {msg}" );
        println!( "Solution: Set GEMINI_API_KEY environment variable or use .api_key()" );
      },
    _ => println!( "Unexpected error type : {e:?}" ),
    }
  }

  // For remaining examples, we'll use a client
  let client = if let Ok( c ) = Client::new()
  {
    c
  }
  else
  {
    println!( "\nNo API key found. Using demo client for demonstration." );
    // In real code, you might want to exit or use a different strategy
    Client::builder()
    .api_key( "demo-api-key-for-examples".to_string() )
    .build()?
  };

  // Example 2: Handling invalid model names
  println!( "\n2. Handling Invalid Model Names" );

  let invalid_request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "Hello".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: None,
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  match client
  .models()
  .by_name( "invalid-model-name" )
  .generate_content( &invalid_request )
  .await
  {
    Ok( _ ) => println!( "Unexpected success" ),
    Err( e ) => match e
    {
      api_gemini ::error::Error::ApiError( msg ) =>
      {
      println!( "API error (invalid model): {msg}" );
        println!( "Solution: Use client.models().list() to see available models" );
      },
    _ => println!( "Error type : {e:?}" ),
    }
  }

  // Example 3: Retry logic for transient failures
  println!( "\n3. Implementing Retry Logic" );

  let retry_config = RetryConfig::default();

  let valid_request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "What is 2+2?".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: Some( GenerationConfig
    {
      temperature: Some( 0.1 ),
      top_k: Some( 1 ),
      top_p: Some( 0.1 ),
      candidate_count: Some( 1 ),
      max_output_tokens: Some( 100 ),
      stop_sequences: None,
    }),
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  let result = call_with_retry
  (
  ||
  {
    let client = client.clone();
    let request = valid_request.clone();
    Box::pin( async move
    {
      client
      .models()
      .by_name( "gemini-2.5-flash" )
      .generate_content( &request )
      .await
    })
  },
  &retry_config,
  "generate_content"
  )
  .await;

  match result
  {
    Ok( response ) =>
    {
      println!( "Success after retry!" );
      if let Some( candidate ) = response.candidates.first()
      {
        if let Some( part ) = candidate.content.parts.first()
        {
          if let Some( text ) = &part.text
          {
          println!( "Response : {text}" );
          }
        }
      }
    },
Err( e ) => println!( "Failed after {} attempts : {:?}", retry_config.max_attempts, e ),
  }

  // Example 4: Handling malformed requests
  println!( "\n4. Handling Malformed Requests" );

  let malformed_request = GenerateContentRequest
  {
    contents: vec![], // Empty contents - invalid
    generation_config: None,
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  match client
  .models()
  .by_name( "gemini-2.5-flash" )
  .generate_content( &malformed_request )
  .await
  {
    Ok( _ ) => println!( "Unexpected success" ),
    Err( e ) => match e
    {
      api_gemini ::error::Error::InvalidArgument( msg ) =>
      {
      println!( "Invalid argument error : {msg}" );
        println!( "Solution: Ensure request has at least one content item" );
      },
      api_gemini ::error::Error::ApiError( msg ) =>
      {
      println!( "API rejected malformed request : {msg}" );
      },
    _ => println!( "Error : {e:?}" ),
    }
  }

  // Example 5: Graceful degradation strategies
  println!( "\n5. Graceful Degradation Example" );

  async fn generate_with_fallback
  (
  client: &Client,
  prompt: &str,
  )
  ->
  Result< String, Box< dyn core::error::Error > >
  {
    // Try with advanced model first
    let models = vec![ "gemini-2.5-flash", "gemini-2.5-flash", "gemini-2.5-flash" ];

    for model in models
    {
    println!( "Trying model : {model}" );

      let request = GenerateContentRequest
      {
        contents: vec!
        [
        Content
        {
          role: "user".to_string(),
          parts: vec!
          [
          Part
          {
            text: Some( prompt.to_string() ),
            inline_data: None,
            function_call: None,
            function_response: None,
            ..Default::default()
          }
          ],
        }
        ],
        generation_config: Some( GenerationConfig
        {
          temperature: Some( 0.7 ),
          top_k: Some( 40 ),
          top_p: Some( 0.95 ),
          candidate_count: Some( 1 ),
          max_output_tokens: Some( 512 ),
          stop_sequences: None,
        }),
        safety_settings: None,
        tools: None,
        tool_config: None,
        system_instruction: None,
        cached_content: None,
      };

      match client.models().by_name( model ).generate_content( &request ).await
      {
        Ok( response ) =>
        {
          if let Some( candidate ) = response.candidates.first()
          {
            if let Some( part ) = candidate.content.parts.first()
            {
              if let Some( text ) = &part.text
              {
              println!( "Success with model : {model}" );
                return Ok( text.clone() );
              }
            }
          }
        },
        Err( e ) =>
        {
      println!( "Failed with {model}: {e:?}" );
        }
      }
    }

    Err( "All models failed".into() )
  }

  match generate_with_fallback( &client, "Tell me a short joke" ).await
  {
  Ok( response ) => println!( "Fallback response : {response}" ),
  Err( e ) => println!( "All fallback attempts failed : {e}" ),
  }

  // Example 6: Error monitoring and logging patterns
  println!( "\n6. Error Monitoring Pattern" );

  #[ derive( Debug ) ]
  struct ErrorMetrics
  {
    total_requests: u64,
    successful_requests: u64,
    rate_limit_errors: u64,
    network_errors: u64,
    other_errors: u64,
  }

  impl ErrorMetrics
  {
    #[ allow( dead_code ) ]
    fn record_error
    (
    &mut self,
    error: &api_gemini::error::Error,
    )
    {
      self.total_requests += 1;

      match error
      {
        api_gemini ::error::Error::RateLimitError( _ ) => self.rate_limit_errors += 1,
        api_gemini ::error::Error::NetworkError( _ ) => self.network_errors += 1,
        _ => self.other_errors += 1,
      }
    }

    #[ allow( dead_code ) ]
    fn record_success( &mut self )
    {
      self.total_requests += 1;
      self.successful_requests += 1;
    }

    fn success_rate( &self ) -> f64
    {
      if self.total_requests == 0
      {
        0.0
      }
      else
      {
        self.successful_requests as f64 / self.total_requests as f64
      }
    }
  }

  let metrics = ErrorMetrics
  {
    total_requests: 10,
    successful_requests: 7,
    rate_limit_errors: 1,
    network_errors: 1,
    other_errors: 1,
  };

  println!( "Error Metrics:" );
println!( "  Total requests : {}", metrics.total_requests );
println!( "  Success rate : {:.1}%", metrics.success_rate() * 100.0 );
println!( "  Rate limit errors : {}", metrics.rate_limit_errors );
println!( "  Network errors : {}", metrics.network_errors );
println!( "  Other errors : {}", metrics.other_errors );

  println!( "\n=== Best Practices for Error Handling ===" );
  println!( "1. Always handle authentication errors at startup" );
  println!( "2. Implement retry logic with exponential backoff" );
  println!( "3. Use circuit breakers for cascading failure prevention" );
  println!( "4. Log errors with context for debugging" );
  println!( "5. Monitor error rates and types" );
  println!( "6. Provide meaningful error messages to users" );
  println!( "7. Consider fallback strategies for critical operations" );
  println!( "8. Respect rate limits with proper backoff" );

  Ok( () )
}