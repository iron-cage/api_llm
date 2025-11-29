mod private
{
  use crate::error::{ XaiError, Result };
  use crate::environment::XaiEnvironment;
  use reqwest::Client as HttpClient;
  use serde::{ Serialize, de::DeserializeOwned };

  #[ cfg( feature = "streaming" ) ]
  use eventsource_stream::Eventsource;
  #[ cfg( feature = "streaming" ) ]
  use futures_util::{ Stream, StreamExt };
  #[ cfg( feature = "streaming" ) ]
  use std::pin::Pin;

  /// XAI API client.
  ///
  /// The main client for interacting with the XAI API. Handles HTTP communication,
  /// authentication, and error handling.
  ///
  /// # Generic Parameter
  ///
  /// * `E` - Environment type implementing `XaiEnvironment` trait
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use api_xai::{ Client, XaiEnvironmentImpl, Secret };
  ///
  /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
  /// let env = XaiEnvironmentImpl::new( secret )?;
  /// let client = Client::build( env )?;
  /// # Ok( () )
  /// # }
  /// ```
  #[ derive( Debug, Clone ) ]
  pub struct Client< E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    /// HTTP client for making requests.
    pub http_client : HttpClient,

    /// Environment configuration.
    pub environment : E,

    /// Failover manager for endpoint rotation (optional).
    #[ cfg( feature = "failover" ) ]
    pub failover_manager : Option< crate::failover::FailoverManager >,

    // Enterprise features will be added in Phase 4
    // #[ cfg( feature = "retry" ) ]
    // pub retry_config : Option< crate::enhanced_retry::EnhancedRetryConfig >,
    //
    // #[ cfg( feature = "circuit_breaker" ) ]
    // pub circuit_breaker : Option< std::sync::Arc< crate::circuit_breaker::CircuitBreaker > >,
  }

  impl< E > Client< E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    /// Builds a new client with the given environment configuration.
    ///
    /// # HTTP Client Configuration
    ///
    /// The client is configured with resilient network settings:
    ///
    /// - **Total timeout**: From environment (default 30s, tests use 120s)
    /// - **Connect timeout**: 15s for DNS resolution + TCP + TLS handshake
    /// - **Pool idle timeout**: 90s to reuse connections efficiently
    /// - **TCP keepalive**: 60s to detect dead connections
    ///
    /// The connect timeout is set to 15s to ensure sufficient time remains for
    /// the actual API response in production (30s total - 15s connect = 15s for API).
    /// For integration tests with 120s timeout, this leaves 105s for API processing.
    ///
    /// These settings prevent timeout issues on slow networks while still
    /// catching genuine failures within reasonable time.
    ///
    /// # Errors
    ///
    /// Returns `XaiError::Http` if the HTTP client cannot be created.
    pub fn build( environment : E ) -> Result< Self >
    {
      let http_client = HttpClient::builder()
        .timeout( environment.timeout() )
        .connect_timeout( std::time::Duration::from_secs( 15 ) )
        .pool_idle_timeout( std::time::Duration::from_secs( 90 ) )
        .tcp_keepalive( std::time::Duration::from_secs( 60 ) )
        .build()
        .map_err( |e| XaiError::Http( format!( "Failed to create HTTP client : {e}" ) ) )?;

      Ok( Self
      {
        http_client,
        environment,

        #[ cfg( feature = "failover" ) ]
        failover_manager : None,
      } )
    }

    /// Adds failover support with multiple endpoints (requires `failover` feature).
    ///
    /// # Panics
    ///
    /// Panics if endpoints list is empty.
    #[ cfg( feature = "failover" ) ]
    #[ must_use ]
    pub fn with_failover( mut self, endpoints : Vec< String > ) -> Self
    {
      let config = crate::failover::FailoverConfig::default();
      self.failover_manager = Some( crate::failover::FailoverManager::new( endpoints, config ) );
      self
    }

    /// Adds failover support with custom configuration (requires `failover` feature).
    ///
    /// # Panics
    ///
    /// Panics if endpoints list is empty.
    #[ cfg( feature = "failover" ) ]
    #[ must_use ]
    pub fn with_failover_config(
      mut self,
      endpoints : Vec< String >,
      config : crate::failover::FailoverConfig
    ) -> Self
    {
      self.failover_manager = Some( crate::failover::FailoverManager::new( endpoints, config ) );
      self
    }

    /// Gets the base URL for API requests.
    ///
    /// Returns the current endpoint from failover manager if configured,
    /// otherwise returns the environment base URL.
    fn get_base_url( &self ) -> Result< url::Url >
    {
      #[ cfg( feature = "failover" ) ]
      {
        if let Some( ref manager ) = self.failover_manager
        {
          let endpoint_str = manager.current_endpoint();
          return url::Url::parse( &endpoint_str )
            .map_err( | e | XaiError::UrlParse( format!( "Invalid failover endpoint : {e}" ) ).into() );
        }
      }

      Ok( self.environment.base_url().clone() )
    }

    /// Makes a POST request to the API.
    ///
    /// # Errors
    ///
    /// Returns errors for network failures, HTTP errors, or deserialization failures.
    pub async fn post< I, O >(
      &self,
      path : &str,
      body : &I
    ) -> Result< O >
    where
      I : Serialize,
      O : DeserializeOwned,
    {
      let base_url = self.get_base_url()?;
      let url = base_url.join( path )?;
      let headers = self.environment.headers()?;

      let response = self.http_client
        .post( url )
        .headers( headers )
        .json( body )
        .send()
        .await;

      match response
      {
        Ok( resp ) =>
        {
          let result = self.handle_response( resp ).await;

          // Record success/failure with failover manager
          #[ cfg( feature = "failover" ) ]
          {
            if let Some( ref manager ) = self.failover_manager
            {
              if result.is_ok()
              {
                manager.record_success();
              }
              else
              {
                manager.record_failure();
              }
            }
          }

          result
        }
        Err( e ) =>
        {
          // Record failure with failover manager
          #[ cfg( feature = "failover" ) ]
          {
            if let Some( ref manager ) = self.failover_manager
            {
              manager.record_failure();
            }
          }

          Err( e.into() )
        }
      }
    }

    /// Makes a GET request to the API.
    ///
    /// # Errors
    ///
    /// Returns errors for network failures, HTTP errors, or deserialization failures.
    pub async fn get< O >( &self, path : &str ) -> Result< O >
    where
      O : DeserializeOwned,
    {
      let base_url = self.get_base_url()?;
      let url = base_url.join( path )?;
      let headers = self.environment.headers()?;

      let response = self.http_client
        .get( url )
        .headers( headers )
        .send()
        .await;

      match response
      {
        Ok( resp ) =>
        {
          let result = self.handle_response( resp ).await;

          // Record success/failure with failover manager
          #[ cfg( feature = "failover" ) ]
          {
            if let Some( ref manager ) = self.failover_manager
            {
              if result.is_ok()
              {
                manager.record_success();
              }
              else
              {
                manager.record_failure();
              }
            }
          }

          result
        }
        Err( e ) =>
        {
          // Record failure with failover manager
          #[ cfg( feature = "failover" ) ]
          {
            if let Some( ref manager ) = self.failover_manager
            {
              manager.record_failure();
            }
          }

          Err( e.into() )
        }
      }
    }

    /// Handles HTTP response, checking status and deserializing body.
    ///
    /// # Errors
    ///
    /// Returns errors for non-success status codes or deserialization failures.
    async fn handle_response< O >(
      &self,
      response : reqwest::Response
    ) -> Result< O >
    where
      O : DeserializeOwned,
    {
      let status = response.status();

      if status.is_success()
      {
        let body = response.json::< O >().await?;
        Ok( body )
      }
      else
      {
        // Try to parse error response
        let error_body = response.text().await?;

        // Check if its a structured API error
        if let Ok( api_error ) = serde_json::from_str::< ApiErrorResponse >( &error_body )
        {
          return Err( XaiError::Api
          {
            message : api_error.error.message,
            code : api_error.error.code,
            error_type : api_error.error.error_type,
          }.into() );
        }

        // Fall back to HTTP error with body
        Err( XaiError::Http( format!(
          "HTTP {}: {}",
          status.as_u16(),
          error_body
        ) ).into() )
      }
    }

    /// Makes a streaming POST request to the API (requires `streaming` feature).
    ///
    /// # Errors
    ///
    /// Returns errors for network failures, HTTP errors, or SSE parsing failures.
    #[ cfg( feature = "streaming" ) ]
    pub async fn post_stream< I, O >(
      &self,
      path : &str,
      body : &I
    ) -> Result< Pin< Box< dyn Stream< Item = Result< O > > + Send + 'static > > >
    where
      I : Serialize,
      O : DeserializeOwned + Send + 'static,
    {
      let base_url = self.get_base_url()?;
      let url = base_url.join( path )?;
      let headers = self.environment.headers()?;

      let response = self.http_client
        .post( url )
        .headers( headers )
        .json( body )
        .send()
        .await;

      let response = match response
      {
        Ok( resp ) => resp,
        Err( e ) =>
        {
          // Record failure with failover manager
          #[ cfg( feature = "failover" ) ]
          {
            if let Some( ref manager ) = self.failover_manager
            {
              manager.record_failure();
            }
          }

          return Err( e.into() );
        }
      };

      let status = response.status();

      if !status.is_success()
      {
        // Record failure with failover manager
        #[ cfg( feature = "failover" ) ]
        {
          if let Some( ref manager ) = self.failover_manager
          {
            manager.record_failure();
          }
        }

        let error_body = response.text().await?;
        return Err( XaiError::Http( format!(
          "HTTP {}: {}",
          status.as_u16(),
          error_body
        ) ).into() );
      }

      // Record success with failover manager (initial connection successful)
      #[ cfg( feature = "failover" ) ]
      {
        if let Some( ref manager ) = self.failover_manager
        {
          manager.record_success();
        }
      }

      // Convert response to byte stream
      let byte_stream = response.bytes_stream();

      // Parse SSE events
      let event_stream = byte_stream
        .map( | result | result.map_err( std::io::Error::other ) )
        .eventsource();

      // Map events to deserialized objects
      let mapped_stream = event_stream.map( move | event_result | {
        match event_result {
          Ok( event ) => {
            // Skip [DONE] marker
            if event.data == "[DONE]" {
              return Err( XaiError::Stream( "Stream completed".to_string() ).into() );
            }

            // Parse JSON from event data
            serde_json::from_str::< O >( &event.data )
              .map_err( | e | XaiError::Serialization( e.to_string() ).into() )
          }
          Err( e ) => {
            Err( XaiError::Stream( format!( "SSE error : {e}" ) ).into() )
          }
        }
      });

      // Filter out the [DONE] marker errors
      let filtered_stream = mapped_stream.take_while( | result | {
        futures_util::future::ready( match result {
          Err( e ) => {
            // Check if this is the "Stream completed" sentinel error
            let error_str = format!( "{e:?}" );
            !error_str.contains( "Stream completed" )
          }
          Ok( _ ) => true,
        } )
      } );

      Ok( Box::pin( filtered_stream ) )
    }
  }

  /// API error response structure.
  ///
  /// Used for parsing structured error responses from the API.
  #[ derive( Debug, serde::Deserialize ) ]
  struct ApiErrorResponse
  {
    error : ApiErrorDetail,
  }

  /// API error detail.
  #[ derive( Debug, serde::Deserialize ) ]
  struct ApiErrorDetail
  {
    message : String,
    #[ serde( rename = "type" ) ]
    error_type : Option< String >,
    code : Option< String >,
  }
}

crate::mod_interface!
{
  exposed use
  {
    Client,
  };
}
