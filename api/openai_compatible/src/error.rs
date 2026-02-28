//! Error type and result alias for OpenAI-compatible API operations.

mod private
{
  use error_tools::dependency::thiserror;

  /// Error variants for OpenAI-compatible API operations.
  ///
  /// Covers transport failures, API-level errors, and configuration problems.
  /// Convert from `reqwest::Error`, `serde_json::Error`, and
  /// `reqwest::header::InvalidHeaderValue` via `From` impls.
  #[ derive( Debug, Clone, PartialEq, thiserror::Error ) ]
  #[ non_exhaustive ]
  pub enum OpenAiCompatError
  {
    /// Error body returned by the API (e.g. invalid model name).
    #[ error( "API error : {0}" ) ]
    Api( String ),

    /// HTTP transport error (4xx / 5xx status code).
    #[ error( "HTTP error : {0}" ) ]
    Http( String ),

    /// Network connectivity error (DNS, TCP, etc.).
    #[ error( "Network error : {0}" ) ]
    Network( String ),

    /// Request exceeded the configured timeout.
    #[ error( "Timeout : {0}" ) ]
    Timeout( String ),

    /// Failed to deserialise the response body.
    #[ error( "Deserialisation error : {0}" ) ]
    Deserialise( String ),

    /// API key is absent or contains characters invalid in an HTTP header.
    #[ error( "Invalid API key : {0}" ) ]
    InvalidApiKey( String ),

    /// Environment is misconfigured (e.g. unparseable base URL).
    #[ error( "Environment error : {0}" ) ]
    Environment( String ),
  }

  /// Crate-level result type backed by a boxed dynamic error.
  pub type Result< T > = error_tools::untyped::Result< T >;

  impl From< reqwest::Error > for OpenAiCompatError
  {
    #[ inline ]
    fn from( e : reqwest::Error ) -> Self
    {
      if e.is_timeout()
      {
        Self::Timeout( e.to_string() )
      }
      else if e.is_connect()
      {
        Self::Network( e.to_string() )
      }
      else
      {
        Self::Http( e.to_string() )
      }
    }
  }

  impl From< serde_json::Error > for OpenAiCompatError
  {
    #[ inline ]
    fn from( e : serde_json::Error ) -> Self
    {
      Self::Deserialise( e.to_string() )
    }
  }

  impl From< reqwest::header::InvalidHeaderValue > for OpenAiCompatError
  {
    #[ inline ]
    fn from( e : reqwest::header::InvalidHeaderValue ) -> Self
    {
      Self::InvalidApiKey( e.to_string() )
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    OpenAiCompatError,
    Result,
  };
}
