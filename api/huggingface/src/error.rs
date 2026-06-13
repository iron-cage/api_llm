//! Error types and handling for `HuggingFace` API interactions.

mod private
{
use error_tools::Error;
use std::fmt;

/// Result type alias for `HuggingFace` operations
pub type Result< T > = core::result::Result< T, HuggingFaceError >;

/// Comprehensive error types for `HuggingFace` API operations
#[ derive( Debug, Clone ) ]
pub enum HuggingFaceError
{
  /// API-related errors with detailed information
  Api( ApiErrorWrap ),
  
  /// HTTP communication errors
  Http( String ),
  
  /// Authentication errors
  Authentication( String ),
  
  /// Request validation errors
  Validation( String ),
  
  /// Rate limiting errors
  RateLimit( String ),
  
  /// Model loading/availability errors
  ModelUnavailable( String ),
  
  /// Streaming errors
  Stream( String ),
  
  /// JSON serialization/deserialization errors
  Serialization( String ),
  
  /// Invalid argument errors
  InvalidArgument( String ),
  
  /// Generic errors for unexpected cases
  Generic( String ),
}

impl fmt::Display for HuggingFaceError
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
  match self
  {
      HuggingFaceError::Api( e ) => write!( f, "API error : {e}" ),
      HuggingFaceError::Http( msg ) => write!( f, "HTTP error : {msg}" ),
      HuggingFaceError::Authentication( msg ) => write!( f, "Authentication error : {msg}" ),
      HuggingFaceError::Validation( msg ) => write!( f, "Validation error : {msg}" ),
      HuggingFaceError::RateLimit( msg ) => write!( f, "Rate limit error : {msg}" ),
      HuggingFaceError::ModelUnavailable( msg ) => write!( f, "Model unavailable : {msg}" ),
      HuggingFaceError::Stream( msg ) => write!( f, "Stream error : {msg}" ),
      HuggingFaceError::Serialization( msg ) => write!( f, "Serialization error : {msg}" ),
      HuggingFaceError::InvalidArgument( msg ) => write!( f, "Invalid argument : {msg}" ),
      HuggingFaceError::Generic( msg ) => write!( f, "Generic error : {msg}" ),
  }
  }
}

impl std::error::Error for HuggingFaceError
{}

/// Wrapper for API error responses
#[ derive( Debug, Clone ) ]
pub struct ApiErrorWrap
{
  /// Error message from the API
  pub message : String,
  
  /// Optional error type
  pub error_type : Option< String >,
  
  /// Optional HTTP status code
  pub status_code : Option< u16 >,
}

impl ApiErrorWrap
{
  /// Create a new API error wrapper
  #[ inline ]
  #[ must_use ]
  pub fn new( message : impl Into< String > ) -> Self
  {
  Self
  {
      message : message.into(),
      error_type : None,
      status_code : None,
  }
  }
  
  /// Set the error type
  #[ inline ]
  #[ must_use ]
  pub fn with_error_type( mut self, error_type : impl Into< String > ) -> Self
  {
  self.error_type = Some( error_type.into() );
  self
  }
  
  /// Set the status code
  #[ inline ]
  #[ must_use ]
  pub fn with_status_code( mut self, status_code : u16 ) -> Self
  {
  self.status_code = Some( status_code );
  self
  }
}

impl fmt::Display for ApiErrorWrap
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
  if let Some( error_type ) = &self.error_type
  {
      write!( f, "[{error_type}] {}", self.message )?;
  }
  else
  {
      write!( f, "{}", self.message )?;
  }
  
  if let Some( status_code ) = self.status_code
  {
      write!( f, " (HTTP {status_code})" )?;
  }
  
  Ok( () )
  }
}

/// Map JSON deserialization errors to `HuggingFace` errors
#[ cfg( feature = "client" ) ]
#[ inline ]
#[ must_use ]
pub fn map_deserialization_error( e : &reqwest::Error ) -> HuggingFaceError
{
  HuggingFaceError::Serialization( e.to_string() )
}

/// Convert from `error_tools::Error`
impl From< Error > for HuggingFaceError
{
  #[ inline ]
  fn from( e : Error ) -> Self
  {
  HuggingFaceError::Generic( e.to_string() )
  }
}

} // end mod private

crate::mod_interface!
{
  exposed use private::HuggingFaceError;
  exposed use private::ApiErrorWrap;
  exposed use private::Result;
  
  #[ cfg( feature = "client" ) ]
  exposed use private::map_deserialization_error;
}