//! This module defines the `Client` structure for interacting with the `HuggingFace` API.
//! It provides methods for making various API requests, handling authentication,
//! and managing HTTP communication.
//!
//! ## Design Philosophy : "Thin Client, Rich API"
//!
//! This client provides comprehensive opt-in features with explicit developer control:
//!
//! ### Opt-In Enterprise Features
//!
//! All features require explicit cargo feature flags AND explicit developer configuration:
//!
//! 1. **Explicit Retry**: Configurable exponential backoff (developer calls `Client::with_retry()`)
//!    - NO automatic retries - developer must explicitly configure retry behavior
//!    - Configurable via `ExplicitRetryConfig` - developer controls all retry parameters
//!
//! 2. **Circuit Breakers**: Opt-in failure detection (developer enables and configures)
//!    - NO automatic circuit breaking - developer must explicitly enable
//!    - Developer controls failure thresholds and recovery behavior
//!
//! 3. **Rate Limiting**: Token bucket rate limiting (developer calls `rate_limiter.acquire()`)
//!    - NO automatic rate limiting - developer must explicitly acquire permits
//!    - Developer configures per-second, per-minute, per-hour limits
//!
//! 4. **Failover**: Multi-endpoint failover (developer configures strategy)
//!    - NO automatic failover - developer must explicitly configure endpoints
//!    - Developer chooses failover strategy (priority, round-robin, random, sticky)
//!
//! 5. **Health Checks**: Opt-in endpoint monitoring (developer enables checks)
//!    - NO automatic health checks - developer must explicitly enable monitoring
//!    - Developer configures check intervals and health criteria
//!
//! 6. **Caching**: LRU caching with TTL (developer uses cache methods)
//!    - NO automatic caching - developer must explicitly use `cache.get()` / `cache.insert()`
//!    - Developer configures TTL, max entries, and eviction policy
//!
//! 7. **Performance Metrics**: Request tracking (developer enables metrics)
//!    - NO automatic metrics - developer must explicitly enable metric collection
//!    - Developer queries metrics when needed
//!
//! 8. **Dynamic Configuration**: Runtime updates (developer sets up watchers)
//!    - NO automatic config changes - developer must explicitly watch for changes
//!    - Developer controls when and how to apply configuration updates
//!
//! ### Why Explicit Control
//!
//! - **Zero Magic**: No automatic behaviors or hidden decision-making
//! - **Predictable**: Features only work when you explicitly call them
//! - **Transparent**: You see exactly what the library does
//! - **Flexible**: Use only what you need
//!
//! ### Historical Context
//!
//! **2024-10-19**: Architecture decision to add opt-in enterprise reliability features.
//! All features require explicit cargo feature flags and developer configuration
//! to maintain the "Thin Client, Rich API" governing principle.

/// Define a private namespace for all its items.
mod private
{
  // Use crate root for base access
  use crate::
  {
  error::{ ApiErrorWrap, HuggingFaceError, Result, map_deserialization_error },
  };
  
  #[ cfg( feature = "env-config" ) ]
  use crate::environment::{ HuggingFaceEnvironment, EnvironmentInterface };

  // Grouped imports relative to crate root (feature-gated)
  #[ cfg( feature = "inference" ) ]
  use crate::inference::Inference;
  #[ cfg( feature = "embeddings" ) ]
  use crate::embeddings::Embeddings;
  #[ cfg( feature = "models" ) ]
  use crate::models::Models;
  #[ cfg( feature = "inference" ) ]
  use crate::providers::Providers;
  #[ cfg( feature = "vision" ) ]
  use crate::vision::Vision;
  #[ cfg( feature = "audio" ) ]
  use crate::audio::Audio;

  // External crates
  use reqwest::
  {
  Client as HttpClient,
  };
  use serde::
  {
  de::DeserializeOwned,
  Serialize,
  };
  #[ cfg( feature = "inference-streaming" ) ]
  use tokio::sync::mpsc;
  #[ cfg( feature = "inference-streaming" ) ]
  use eventsource_stream::Eventsource;

  /// The main client for interacting with the `HuggingFace` API.
  ///
  /// Provides methods for accessing different API categories like
  /// inference, embeddings, models, etc.
  ///
  /// # Example
  ///
  /// ```no_run
  /// use api_huggingface::{ Client, environment::HuggingFaceEnvironmentImpl, Secret };
  ///
  /// # async fn example() -> Result< (), Box< dyn core::error::Error > > {
  /// let secret = Secret::load_from_env( "HUGGINGFACE_API_KEY" )?;
  /// let env = HuggingFaceEnvironmentImpl::build( secret, None )?;
  /// let client = Client::build( env )?;
  ///
  /// // Use the client to access different APIs
  /// let response = client.inference().create( "What is the capital of France?", "gpt2" ).await?;
  /// # Ok(())
  /// # }
  /// ```
  #[ derive( Debug, Clone ) ]
  pub struct Client< E >
  where
  E : Clone,
  {
  /// The HTTP client used for making requests.
  pub http_client : HttpClient,
  /// The `HuggingFace` environment configuration.
  pub environment : E,
  // Automatic retry fields removed per governing principle - use explicit retry methods
  }

  // Automatic retry configuration structs removed per governing principle
  // Use explicit retry methods instead

  /// Configuration for explicit retry behavior
  ///
  /// This struct provides explicit configuration for retry operations,
  /// following the governing principle of transparent, developer-controlled behavior.
  #[ derive( Debug, Clone ) ]
  pub struct ExplicitRetryConfig
  {
  /// Maximum number of retry attempts
  pub max_retries : u32,
  /// Initial delay between retries in milliseconds
  pub initial_delay_ms : u64,
  /// Multiplier for exponential backoff
  pub multiplier : f64,
  /// Maximum delay between retries in milliseconds
  pub max_delay_ms : u64,
  /// Random jitter to add/subtract from delay in milliseconds
  pub jitter_ms : u64,
  }

  impl ExplicitRetryConfig
  {
  /// Creates a conservative retry configuration
  #[ inline ]
  #[ must_use ]
  pub fn conservative() -> Self
  {
      Self
      {
  max_retries : 3,
  initial_delay_ms : 1000,
  multiplier : 2.0,
  max_delay_ms : 30_000,
  jitter_ms : 100,
      }
  }

  /// Creates an aggressive retry configuration
  #[ inline ]
  #[ must_use ]
  pub fn aggressive() -> Self
  {
      Self
      {
  max_retries : 5,
  initial_delay_ms : 500,
  multiplier : 1.5,
  max_delay_ms : 10_000,
  jitter_ms : 50,
      }
  }
  }

  // is_retryable_error function removed per governing principle - use explicit retry methods

  #[ cfg( feature = "env-config" ) ]
  impl< E > Client< E >
  where
  E : HuggingFaceEnvironment + EnvironmentInterface + Send + Sync + 'static + Clone,
  {
  /// Creates a new `Client` instance with recommended configuration.
  ///
  /// # Governing Principle Compliance
  ///
  /// This provides HuggingFace-recommended retry configuration without making it implicit.
  /// For explicit control, use `with_explicit_config()`.
  ///
  /// # Arguments
  /// - `environment`: The `HuggingFace` environment configuration.
  ///
  /// # Errors
  /// Returns `HuggingFaceError::InvalidArgument` if the API key is invalid.
  #[ inline ]
  pub fn build( environment : E ) -> Result< Self >
  {
      let headers = environment.headers()?;
      let http_client = HttpClient::builder()
  .default_headers( headers )
  .build()
  .map_err( | e | HuggingFaceError::Http( e.to_string() ) )?;

      Ok( Self
      {
  http_client,
  environment,
  // retry_policy field removed per governing principle
      } )
  }

  // with_explicit_config method removed per governing principle - use explicit retry methods

  /// Returns the `Inference` API group for text generation operations.
  #[ cfg( feature = "inference" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn inference( &self ) -> Inference< E >
  {
      Inference::new( self )
  }

  /// Returns the `Embeddings` API group for feature extraction operations.
  #[ cfg( feature = "embeddings" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn embeddings( &self ) -> Embeddings< E >
  {
      Embeddings::new( self )
  }

  /// Returns the `Models` API group for model information operations.
  #[ cfg( feature = "models" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn models( &self ) -> Models< E >
  {
      Models::new( self )
  }

  /// Returns the `Providers` API group for Inference Providers operations.
  #[ cfg( feature = "inference" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn providers( &self ) -> Providers< E >
  {
      Providers::new( self )
  }

  /// Returns the `Vision` API group for computer vision operations.
  #[ cfg( feature = "vision" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn vision( &self ) -> Vision< E >
  {
      Vision::new( self )
  }

  /// Access audio processing APIs
  #[ cfg( feature = "audio" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn audio( &self ) -> Audio< E >
  {
      Audio::new( self )
  }

  // Automatic retry configuration methods removed per governing principle
  // Use explicit retry methods instead

  /// Makes a POST request to the specified URL with the given payload.
  ///
  /// # Arguments
  /// - `url`: The URL to send the request to.
  /// - `payload`: The request payload to serialize and send.
  ///
  /// # Errors
  /// Returns various `HuggingFaceError` types for different failure cases.
  #[ inline ]
  pub async fn post< T, R >( &self, url : &str, payload : &T ) -> Result< R >
  where
      T : Serialize + ?Sized,
      R : DeserializeOwned,
  {
      // Direct request without automatic retry per governing principle
      self.post_direct( url, payload ).await
  }

  /// Makes a POST request that returns raw bytes (for audio, images, etc.)
  ///
  /// # Arguments
  /// - `url`: The URL to send the request to.
  /// - `payload`: The request payload to serialize and send.
  ///
  /// # Errors
  /// Returns various `HuggingFaceError` types for different failure cases.
  #[ inline ]
  pub async fn post_bytes< T >( &self, url : &str, payload : &T ) -> Result< Vec< u8 > >
  where
      T : Serialize + ?Sized,
  {
      let response = self.http_client
  .post( url )
  .json( payload )
  .send()
  .await
  .map_err( | e | HuggingFaceError::Http( e.to_string() ) )?;

      let status = response.status();
      if !status.is_success()
      {
  let error_text = response.text().await
          .unwrap_or_else( | _ | "Failed to read error response".to_string() );
  return Err( HuggingFaceError::Api( ApiErrorWrap::new( error_text ).with_status_code( status.as_u16() ) ) );
      }

      response
  .bytes()
  .await
  .map( | b | b.to_vec() )
  .map_err( | e | HuggingFaceError::Http( e.to_string() ) )
  }

  /// Makes a direct POST request without retry logic
  #[ inline ]
  async fn post_direct< T, R >( &self, url : &str, payload : &T ) -> Result< R >
  where
      T : Serialize + ?Sized,
      R : DeserializeOwned,
  {
      let response = self.http_client
  .post( url )
  .json( payload )
  .send()
  .await
  .map_err( | e | HuggingFaceError::Http( e.to_string() ) )?;

      let status = response.status();

      if status.is_success()
      {
  response
          .json::< R >()
          .await
          .map_err( | e | HuggingFaceError::Serialization( e.to_string() ) )
      } else {
  let error_text = response.text().await
          .unwrap_or_else( | _ | "Failed to read error response".to_string() );
  Err( HuggingFaceError::Http( format!("HTTP {status} - {error_text}") ) )
      }
  }

  // post_with_retry method removed per governing principle - use explicit retry methods

  /// Makes a GET request to the specified URL.
  ///
  /// # Arguments
  /// - `url`: The URL to send the request to.
  ///
  /// # Errors
  /// Returns various `HuggingFaceError` types for different failure cases.
  #[ inline ]
  pub async fn get< R >( &self, url : &str ) -> Result< R >
  where
      R : DeserializeOwned,
  {
      // Direct request without automatic retry per governing principle
      self.get_direct( url ).await
  }

  /// Makes a direct GET request without retry logic
  #[ inline ]
  async fn get_direct< R >( &self, url : &str ) -> Result< R >
  where
      R : DeserializeOwned,
  {
      let response = self.http_client
  .get( url )
  .send()
  .await
  .map_err( | e | HuggingFaceError::Http( e.to_string() ) )?;

      let status = response.status();
      if !status.is_success()
      {
  let error_text = response.text().await
          .unwrap_or_else( | _ | "Failed to read error response".to_string() );
  return Err( HuggingFaceError::Api( ApiErrorWrap::new( error_text ).with_status_code( status.as_u16() ) ) );
      }

      response
  .json::< R >()
  .await
  .map_err( |e| map_deserialization_error( &e ) )
  }

  // get_with_retry method removed per governing principle - use explicit retry methods

  /// Makes a POST request with explicit retry logic
  ///
  /// This method provides explicit retry behavior that must be configured
  /// by the developer, following the governing principle of transparency.
  ///
  /// # Errors
  /// Returns `HuggingFaceError` if all retry attempts fail or if the error is non-retryable.
  #[ inline ]
  pub async fn post_with_explicit_retry< T, R >(
      &self,
      url : &str,
      payload : &T,
      retry_config : &ExplicitRetryConfig
  ) -> Result< R >
  where
      T : Serialize + ?Sized,
      R : DeserializeOwned,
  {
      let mut retry_count = 0;
      let mut delay = retry_config.initial_delay_ms;

      loop
      {
  let result = self.post_direct( url, payload ).await;

  match result
  {
          Ok( response ) => return Ok( response ),
          Err( error ) if retry_count >= retry_config.max_retries => return Err( error ),
          Err( error ) if Self::is_error_retryable( &error ) =>
          {
      retry_count += 1;

      // Add jitter to prevent thundering herd
      let jitter = ( rand::random::< u64 >() % ( retry_config.jitter_ms * 2 ) ).saturating_sub( retry_config.jitter_ms );
      let total_delay = delay.saturating_add( jitter ).min( retry_config.max_delay_ms );

      tokio::time::sleep( tokio::time::Duration::from_millis( total_delay ) ).await;

      // Update delay: multiply by f64 multiplier then clamp; truncation is intentional
      #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
      {
              delay = ( ( delay as f64 ) * retry_config.multiplier ) as u64;
      }
      delay = delay.min( retry_config.max_delay_ms );
          },
          Err( error ) => return Err( error ), // Non-retryable error
  }
      }
  }

  /// Makes a GET request with explicit retry logic
  ///
  /// This method provides explicit retry behavior that must be configured
  /// by the developer, following the governing principle of transparency.
  ///
  /// # Errors
  /// Returns `HuggingFaceError` if all retry attempts fail or if the error is non-retryable.
  #[ inline ]
  pub async fn get_with_explicit_retry< R >(
      &self,
      url : &str,
      retry_config : &ExplicitRetryConfig
  ) -> Result< R >
  where
      R : DeserializeOwned,
  {
      let mut retry_count = 0;
      let mut delay = retry_config.initial_delay_ms;

      loop
      {
  let result = self.get_direct( url ).await;

  match result
  {
          Ok( response ) => return Ok( response ),
          Err( error ) if retry_count >= retry_config.max_retries => return Err( error ),
          Err( error ) if Self::is_error_retryable( &error ) =>
          {
      retry_count += 1;

      // Add jitter to prevent thundering herd
      let jitter = ( rand::random::< u64 >() % ( retry_config.jitter_ms * 2 ) ).saturating_sub( retry_config.jitter_ms );
      let total_delay = delay.saturating_add( jitter ).min( retry_config.max_delay_ms );

      tokio::time::sleep( tokio::time::Duration::from_millis( total_delay ) ).await;

      // Update delay: multiply by f64 multiplier then clamp; truncation is intentional
      #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
      {
              delay = ( ( delay as f64 ) * retry_config.multiplier ) as u64;
      }
      delay = delay.min( retry_config.max_delay_ms );
          },
          Err( error ) => return Err( error ), // Non-retryable error
  }
      }
  }

  /// Determines if an error is suitable for retry attempts
  ///
  /// This is an explicit helper method for use with explicit retry operations.
  #[ inline ]
  fn is_error_retryable( error : &HuggingFaceError ) -> bool
  {
      match error
      {
  // Network and HTTP errors are generally retryable
  // Rate limiting should be retried
  // Model unavailable might be temporary
  // Stream errors could be network-related
  HuggingFaceError::Http( _ ) |
  HuggingFaceError::RateLimit( _ ) |
  HuggingFaceError::ModelUnavailable( _ ) |
  HuggingFaceError::Stream( _ ) => true,

  // API errors need more specific checking
  HuggingFaceError::Api( api_error ) =>
  {
          // Check if it's a 5xx server error (retryable) vs 4xx client error (not retryable)
          if let Some( status_code ) = api_error.status_code
          {
      ( 500..600 ).contains( &status_code )
          }
          else
          {
      // If no status code, check error message for common retryable patterns
      let msg = api_error.message.to_lowercase();
      msg.contains( "timeout" ) ||
      msg.contains( "unavailable" ) ||
      msg.contains( "overloaded" ) ||
      msg.contains( "rate limit" ) ||
      msg.contains( "service" )
          }
  },

  // These errors are generally not retryable
  HuggingFaceError::Authentication( _ ) |
  HuggingFaceError::Validation( _ ) |
  HuggingFaceError::Serialization( _ ) |
  HuggingFaceError::InvalidArgument( _ ) |
  HuggingFaceError::Generic( _ ) => false,
      }
  }

  /// Makes a streaming POST request to the specified URL.
  ///
  /// # Arguments
  /// - `url`: The URL to send the request to.
  /// - `payload`: The request payload to serialize and send.
  ///
  /// # Returns
  /// A receiver channel for streaming response chunks.
  ///
  /// # Errors
  /// Returns various `HuggingFaceError` types for different failure cases.
  #[ cfg( feature = "inference-streaming" ) ]
  #[ inline ]
  pub async fn post_stream< T >( &self, url : &str, payload : &T ) -> Result< mpsc::Receiver< Result< String > > >
  where
      T : Serialize + ?Sized,
  {
      let response = self.http_client
  .post( url )
  .header( "Accept", "text/event-stream" )
  .json( payload )
  .send()
  .await
  .map_err( | e | HuggingFaceError::Http( e.to_string() ) )?;

      let status = response.status();
      if !status.is_success()
      {
  let error_text = response.text().await
          .unwrap_or_else( | _ | "Failed to read error response".to_string() );
  return Err( HuggingFaceError::Api( ApiErrorWrap::new( error_text ).with_status_code( status.as_u16() ) ) );
      }

      let ( tx, rx ) = mpsc::channel( 100 );

      let byte_stream = response.bytes_stream();
      let event_stream = byte_stream.eventsource();

      tokio::spawn( async move
      {
  use futures_util::StreamExt;
  let mut stream = event_stream;
  while let Some( event ) = stream.next().await
  {
          match event
          {
      Ok( event ) =>
      {
              if (tx.send( Ok( event.data ) ).await).is_err()
              {
        break;
              }
      },
      Err( e ) =>
      {
              let _ = tx.send( Err( HuggingFaceError::Stream( e.to_string() ) ) ).await;
              break;
      }
          }
  }
      });

      Ok( rx )
  }
  }

  // Basic client implementation for when env-config is not available
  #[ cfg( not( feature = "env-config" ) ) ]
  impl< E > Client< E >
  where
  E : Clone,
  {
  /// Creates a new `Client` instance.
  ///
  /// # Arguments
  /// - `environment`: The `HuggingFace` environment configuration.
  ///
  /// # Errors
  /// Returns `HuggingFaceError::InvalidArgument` if the API key is invalid.
  #[ inline ]
  pub fn build( environment : E ) -> Result< Self >
  {
      let http_client = HttpClient::new();
      Ok( Self
      {
  http_client,
  environment,
  // retry_policy field removed per governing principle
      } )
  }

  /// Returns the `Inference` API group for text generation operations.
  #[ cfg( feature = "inference" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn inference( &self ) -> Inference< E >
  {
      Inference::new( self )
  }

  /// Returns the `Embeddings` API group for feature extraction operations.
  #[ cfg( feature = "embeddings" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn embeddings( &self ) -> Embeddings< E >
  {
      Embeddings::new( self )
  }

  /// Returns the `Models` API group for model information operations.
  #[ cfg( feature = "models" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn models( &self ) -> Models< E >
  {
      Models::new( self )
  }

  /// Returns the `Providers` API group for Inference Providers operations.
  #[ cfg( feature = "inference" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn providers( &self ) -> Providers< E >
  {
      Providers::new( self )
  }

  /// Returns the `Vision` API group for computer vision operations.
  #[ cfg( feature = "vision" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn vision( &self ) -> Vision< E >
  {
      Vision::new( self )
  }

  /// Access audio processing APIs
  #[ cfg( feature = "audio" ) ]
  #[ inline ]
  #[ must_use ]
  pub fn audio( &self ) -> Audio< E >
  {
      Audio::new( self )
  }
  }
}

crate::mod_interface!
{
  exposed use private::Client;
  exposed use private::ExplicitRetryConfig;
  // RetryPolicy export removed per governing principle - replaced with ExplicitRetryConfig
}