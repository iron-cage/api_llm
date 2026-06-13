//! Input validation module for Claude API requests
//!
//! Provides pre-request parameter validation to catch errors before API calls.
//! Validation rules based on Anthropic Claude API documentation.

mod private
{
  use std::fmt;

  /// Validation error with detailed failure information
  #[ derive( Debug, Clone, PartialEq ) ]
  pub struct ValidationError
  {
    /// Field name that failed validation
    pub field : String,
    /// Human-readable error message
    pub message : String,
    /// Optional value that failed
    pub value : Option< String >,
    /// Optional constraint that was violated
    pub constraint : Option< String >,
  }

  impl fmt::Display for ValidationError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      write!( f, "Validation error for field '{}' : {}", self.field, self.message )?;
      if let Some( ref value ) = self.value
      {
        write!( f, " (value : {value})" )?;
      }
      if let Some( ref constraint ) = self.constraint
      {
        write!( f, " (constraint : {constraint})" )?;
      }
      Ok( () )
    }
  }

  impl std::error::Error for ValidationError {}

  impl ValidationError
  {
    /// Create new validation error
    pub fn new( field : impl Into< String >, message : impl Into< String > ) -> Self
    {
      Self
      {
        field : field.into(),
        message : message.into(),
        value : None,
        constraint : None,
      }
    }

    /// Set the value that failed
    #[ must_use ]
    pub fn with_value( mut self, value : impl Into< String > ) -> Self
    {
      self.value = Some( value.into() );
      self
    }

    /// Set the constraint that was violated
    #[ must_use ]
    pub fn with_constraint( mut self, constraint : impl Into< String > ) -> Self
    {
      self.constraint = Some( constraint.into() );
      self
    }
  }

  /// Trait for validating request parameters
  pub trait Validate
  {
    /// Validate request parameters
    ///
    /// # Errors
    ///
    /// Returns Vec of validation errors if validation fails
    fn validate( &self ) -> Result< (), Vec< ValidationError > >;
  }

  /// Reusable validation functions for Claude API parameters
  pub mod validators
  {
    use super::ValidationError;

    /// Validate model name (non-empty, reasonable length)
    ///
    /// # Errors
    ///
    /// Returns error if model name is empty or exceeds 256 characters
    pub fn validate_model( model : &str ) -> Result< (), ValidationError >
    {
      if model.is_empty()
      {
        return Err(
          ValidationError::new( "model", "Model name cannot be empty" )
            .with_constraint( "non-empty string" )
        );
      }

      if model.len() > 256
      {
        return Err(
          ValidationError::new( "model", "Model name exceeds maximum length" )
            .with_value( format!( "{} chars", model.len() ) )
            .with_constraint( "max 256 characters" )
        );
      }

      Ok( () )
    }

    /// Validate `max_tokens` (must be positive, Claude supports up to 4096)
    ///
    /// # Errors
    ///
    /// Returns error if `max_tokens` is non-positive or exceeds 4096
    pub fn validate_max_tokens( max_tokens : u32 ) -> Result< (), ValidationError >
    {
      if max_tokens == 0
      {
        return Err(
          ValidationError::new( "max_tokens", "max_tokens must be positive" )
            .with_value( max_tokens.to_string() )
            .with_constraint( "max_tokens > 0" )
        );
      }

      // Claude's max output tokens is typically 4096
      if max_tokens > 8192
      {
        return Err(
          ValidationError::new( "max_tokens", "max_tokens exceeds reasonable limit" )
            .with_value( max_tokens.to_string() )
            .with_constraint( "max_tokens <= 8192" )
        );
      }

      Ok( () )
    }

    /// Validate temperature (0.0 to 1.0 for Claude)
    ///
    /// # Errors
    ///
    /// Returns error if temperature is outside 0.0-1.0
    pub fn validate_temperature( temperature : f32 ) -> Result< (), ValidationError >
    {
      if !( 0.0..=1.0 ).contains( &temperature )
      {
        return Err(
          ValidationError::new( "temperature", "Temperature must be between 0.0 and 1.0" )
            .with_value( temperature.to_string() )
            .with_constraint( "0.0 <= temperature <= 1.0" )
        );
      }
      Ok( () )
    }

    /// Validate `top_p` (0.0 to 1.0)
    ///
    /// # Errors
    ///
    /// Returns error if `top_p` is outside 0.0-1.0
    pub fn validate_top_p( top_p : f32 ) -> Result< (), ValidationError >
    {
      if !( 0.0..=1.0 ).contains( &top_p )
      {
        return Err(
          ValidationError::new( "top_p", "top_p must be between 0.0 and 1.0" )
            .with_value( top_p.to_string() )
            .with_constraint( "0.0 <= top_p <= 1.0" )
        );
      }
      Ok( () )
    }

    /// Validate `top_k` (must be positive)
    ///
    /// # Errors
    ///
    /// Returns error if `top_k` is non-positive or exceeds reasonable limit
    pub fn validate_top_k( top_k : u32 ) -> Result< (), ValidationError >
    {
      if top_k == 0
      {
        return Err(
          ValidationError::new( "top_k", "top_k must be positive" )
            .with_value( top_k.to_string() )
            .with_constraint( "top_k > 0" )
        );
      }

      if top_k > 500
      {
        return Err(
          ValidationError::new( "top_k", "top_k exceeds reasonable limit" )
            .with_value( top_k.to_string() )
            .with_constraint( "top_k <= 500" )
        );
      }

      Ok( () )
    }

    /// Validate messages array is not empty
    ///
    /// # Errors
    ///
    /// Returns error if messages array is empty
    pub fn validate_messages_not_empty< T >( messages : &[ T ] ) -> Result< (), ValidationError >
    {
      if messages.is_empty()
      {
        return Err(
          ValidationError::new( "messages", "Messages array cannot be empty" )
            .with_constraint( "at least one message required" )
        );
      }
      Ok( () )
    }

    /// Validate system prompt length
    ///
    /// # Errors
    ///
    /// Returns error if system prompt exceeds reasonable length
    pub fn validate_system_prompt( system : &str ) -> Result< (), ValidationError >
    {
      if system.len() > 100_000
      {
        return Err(
          ValidationError::new( "system", "System prompt exceeds maximum length" )
            .with_value( format!( "{} chars", system.len() ) )
            .with_constraint( "max 100000 characters" )
        );
      }
      Ok( () )
    }
  }

}

crate::mod_interface!
{
  exposed use
  {
    ValidationError,
    Validate,
    validators,
  };
}
