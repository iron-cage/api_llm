//! Async HTTP client for OpenAI-compatible REST APIs.
//!
//! The [`Client`] is generic over an environment `E` so callers can supply
//! different credential sources and base URLs without changing client code.

mod private
{
  use crate::error::{ OpenAiCompatError, Result };
  use crate::environment::OpenAiCompatEnvironment;
  use core::time::Duration;
  use reqwest::Client as HttpClient;

  /// Async HTTP client for OpenAI-compatible REST APIs.
  ///
  /// Generic over an environment `E` so callers can supply different
  /// credential sources and base URLs without changing client code.
  ///
  /// Construct via [`build`][Client::build]. All request methods are `async`.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # #[ cfg( feature = "enabled" ) ]
  /// # {
  /// use api_openai_compatible::{ Client, OpenAiCompatEnvironmentImpl };
  ///
  /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let env    = OpenAiCompatEnvironmentImpl::new( "sk-..." )?;
  /// let client = Client::build( env )?;
  /// # Ok( () ) }
  /// # }
  /// ```
  #[ derive( Debug ) ]
  pub struct Client< E >
  where
    E : OpenAiCompatEnvironment,
  {
    /// Underlying reqwest HTTP client, pre-configured with timeout settings.
    http_client : HttpClient,
    /// Environment supplying credentials and base URL.
    environment : E,
  }

  impl< E > Client< E >
  where
    E : OpenAiCompatEnvironment,
  {
    /// Builds an HTTP client configured from the given environment.
    ///
    /// Configures connection pooling, timeout, and authentication headers.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `reqwest::Client` cannot be built
    /// (e.g. invalid TLS configuration on the current platform).
    #[ inline ]
    pub fn build( env : E ) -> Result< Self >
    {
      let http_client = HttpClient::builder()
        .timeout( env.timeout() )
        .connect_timeout( Duration::from_secs( 15 ) )
        .build()
        .map_err( | e | OpenAiCompatError::Environment( e.to_string() ) )?;
      Ok( Self { http_client, environment : env } )
    }

    /// Sends a POST request and deserialises the JSON response.
    ///
    /// # Errors
    ///
    /// Returns network, timeout, or deserialisation errors.
    #[ inline ]
    pub async fn post< I, O >( &self, path : &str, body : &I ) -> Result< O >
    where
      I : serde::Serialize,
      O : serde::de::DeserializeOwned,
    {
      let url = format!( "{}{}", self.environment.base_url(), path );
      let headers = self.environment.headers()?;
      let response = self.http_client
        .post( &url )
        .headers( headers )
        .json( body )
        .send()
        .await
        .map_err( OpenAiCompatError::from )?;
      Self::handle_response( response ).await
    }

    /// Sends a GET request and deserialises the JSON response.
    ///
    /// # Errors
    ///
    /// Returns network, timeout, or deserialisation errors.
    #[ inline ]
    pub async fn get< O >( &self, path : &str ) -> Result< O >
    where
      O : serde::de::DeserializeOwned,
    {
      let url = format!( "{}{}", self.environment.base_url(), path );
      let headers = self.environment.headers()?;
      let response = self.http_client
        .get( &url )
        .headers( headers )
        .send()
        .await
        .map_err( OpenAiCompatError::from )?;
      Self::handle_response( response ).await
    }

    /// Interprets a completed HTTP response, deserialising success bodies or
    /// returning an `Api` error for non-2xx status codes.
    async fn handle_response< O >( response : reqwest::Response ) -> Result< O >
    where
      O : serde::de::DeserializeOwned,
    {
      let status = response.status();
      if status.is_success()
      {
        response
          .json::< O >()
          .await
          .map_err( OpenAiCompatError::from )
          .map_err( Into::into )
      }
      else
      {
        let body = response
          .text()
          .await
          .unwrap_or_else( | _ | status.to_string() );
        Err( OpenAiCompatError::Api( body ).into() )
      }
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    Client,
  };
}
