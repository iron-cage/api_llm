//! Environment configuration trait and default implementation.
//!
//! The [`OpenAiCompatEnvironment`] trait abstracts over credential sources and
//! API endpoint configuration, enabling the same [`crate::Client`] to target
//! `OpenAI`, KIE.ai, xAI, or any other OpenAI-compatible endpoint.

mod private
{
  use crate::error::{ OpenAiCompatError, Result };
  use core::time::Duration;
  use reqwest::header;

  /// Configuration contract for an OpenAI-compatible API environment.
  ///
  /// Implementors supply the three required values (API key, base URL, timeout)
  /// and receive a default `headers()` implementation that builds the standard
  /// HTTP headers. Override `headers()` if custom header logic is required.
  ///
  /// # Trait Bounds
  ///
  /// `Send + Sync + 'static` are required for use across async task boundaries.
  pub trait OpenAiCompatEnvironment : Send + Sync + 'static
  {
    /// Returns the raw API key string.
    fn api_key( &self ) -> &str;

    /// Returns the base URL for this provider, including the trailing slash.
    ///
    /// Example: `"https://api.openai.com/v1/"`. The client appends endpoint
    /// paths (e.g. `"chat/completions"`) to produce the full request URL.
    fn base_url( &self ) -> &str;

    /// Returns the per-request timeout duration.
    fn timeout( &self ) -> Duration;

    /// Constructs the HTTP headers required for every request.
    ///
    /// Default implementation adds:
    /// - `Authorization: Bearer <api_key>`
    /// - `Content-Type: application/json`
    ///
    /// # Errors
    ///
    /// Returns an error if header value construction fails (e.g. key contains
    /// non-ASCII characters that reqwest rejects).
    #[ inline ]
    fn headers( &self ) -> Result< header::HeaderMap >
    {
      let mut map = header::HeaderMap::new();
      let auth_value = format!( "Bearer {}", self.api_key() )
        .parse::< header::HeaderValue >()
        .map_err( | e | OpenAiCompatError::InvalidApiKey( e.to_string() ) )?;
      map.insert( header::AUTHORIZATION, auth_value );
      map.insert
      (
        header::CONTENT_TYPE,
        header::HeaderValue::from_static( "application/json" ),
      );
      Ok( map )
    }
  }

  /// Default OpenAI-compatible environment backed by in-memory values.
  ///
  /// Construct with [`new()`][OpenAiCompatEnvironmentImpl::new], then chain
  /// builder methods to override defaults.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # #[ cfg( feature = "enabled" ) ]
  /// # {
  /// use api_openai_compatible::OpenAiCompatEnvironmentImpl;
  ///
  /// let env = OpenAiCompatEnvironmentImpl::new( "sk-..." ).unwrap();
  /// // override base URL for a KIE.ai model slug:
  /// let kie_env = env.with_base_url( "https://api.kie.ai/my-model/v1/" );
  /// # }
  /// ```
  #[ allow( dead_code ) ]
  #[ derive( Debug, Clone ) ]
  pub struct OpenAiCompatEnvironmentImpl
  {
    /// API authentication key.
    api_key  : String,
    /// Base URL including trailing slash.
    base_url : String,
    /// Per-request timeout.
    timeout  : Duration,
  }

  impl OpenAiCompatEnvironmentImpl
  {
    /// Default base URL for the `OpenAI` API.
    pub const DEFAULT_BASE_URL : &'static str = "https://api.openai.com/v1/";

    /// Default request timeout in seconds.
    pub const DEFAULT_TIMEOUT_SECS : u64 = 30;

    /// Creates a new environment with default base URL and 30-second timeout.
    ///
    /// # Arguments
    ///
    /// * `api_key` — API authentication key. May be any `Into<String>`.
    ///
    /// # Errors
    ///
    /// Returns an error if `api_key` is empty or otherwise invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "enabled" ) ]
    /// # {
    /// use api_openai_compatible::OpenAiCompatEnvironmentImpl;
    ///
    /// let env = OpenAiCompatEnvironmentImpl::new( "sk-mykey" ).unwrap();
    /// # }
    /// ```
    #[ inline ]
    pub fn new( api_key : impl Into< String > ) -> Result< Self >
    {
      let api_key = api_key.into();
      if api_key.is_empty()
      {
        return Err
        (
          OpenAiCompatError::InvalidApiKey( "API key must not be empty".to_owned() ).into()
        );
      }
      Ok( Self
      {
        api_key,
        base_url : Self::DEFAULT_BASE_URL.to_owned(),
        timeout  : Duration::from_secs( Self::DEFAULT_TIMEOUT_SECS ),
      })
    }

    /// Overrides the base URL, returning the modified environment.
    ///
    /// Use this to target non-standard endpoints such as KIE.ai model-slug
    /// URLs (`"https://api.kie.ai/{slug}/v1/"`) or local development proxies.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "enabled" ) ]
    /// # {
    /// use api_openai_compatible::OpenAiCompatEnvironmentImpl;
    ///
    /// let env = OpenAiCompatEnvironmentImpl::new( "sk-key" ).unwrap()
    ///   .with_base_url( "https://api.kie.ai/gpt-4o/v1/" );
    /// # }
    /// ```
    #[ must_use ]
    #[ inline ]
    pub fn with_base_url( mut self, base_url : impl Into< String > ) -> Self
    {
      self.base_url = base_url.into();
      self
    }

    /// Overrides the request timeout, returning the modified environment.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "enabled" ) ]
    /// # {
    /// use api_openai_compatible::OpenAiCompatEnvironmentImpl;
    /// use core::time::Duration;
    ///
    /// let env = OpenAiCompatEnvironmentImpl::new( "sk-key" ).unwrap()
    ///   .with_timeout( Duration::from_secs( 60 ) );
    /// # }
    /// ```
    #[ must_use ]
    #[ inline ]
    pub fn with_timeout( mut self, timeout : Duration ) -> Self
    {
      self.timeout = timeout;
      self
    }
  }

  impl OpenAiCompatEnvironment for OpenAiCompatEnvironmentImpl
  {
    #[ inline ]
    fn api_key( &self ) -> &str
    {
      &self.api_key
    }

    #[ inline ]
    fn base_url( &self ) -> &str
    {
      &self.base_url
    }

    #[ inline ]
    fn timeout( &self ) -> Duration
    {
      self.timeout
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    OpenAiCompatEnvironment,
    OpenAiCompatEnvironmentImpl,
  };
}
