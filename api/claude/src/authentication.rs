//! Advanced authentication functionality for Anthropic API

// Allow missing inline attributes for authentication module
#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  use crate::error::{ AnthropicError, AnthropicResult, AuthenticationError };

  /// Extension methods for Secret
  impl crate::secret::Secret
  {
    /// Create secret with explicit validation requirements
    ///
    /// # Errors
    ///
    /// Returns an error if API key fails explicit validation requirements
    pub fn new_with_validation
    (
      api_key : String,
      required_prefix : &str,
      min_length : Option< usize >,
      max_length : Option< usize >,
    )
    -> AnthropicResult< Self >
    {
      if !api_key.starts_with( required_prefix )
      {
        return Err( AnthropicError::Authentication(
          AuthenticationError::new( format!( "API key must start with '{required_prefix}'" ) )
        ));
      }

      if let Some( min ) = min_length
      {
        if api_key.len() < min
        {
          return Err( AnthropicError::Authentication(
            AuthenticationError::new( format!( "API key must be at least {min} characters long" ) )
          ));
        }
      }

      if let Some( max ) = max_length
      {
        if api_key.len() > max
        {
          return Err( AnthropicError::Authentication(
            AuthenticationError::new( format!( "API key must be at most {max} characters long" ) )
          ));
        }
      }

      Self::new( api_key ).map_err( | e | AnthropicError::InvalidArgument( e.to_string() ) )
    }

    /// Load secret trying each environment variable in order; return first valid one found
    ///
    /// # Errors
    ///
    /// Returns an error if no valid environment variable is found
    pub fn load_with_precedence( env_vars : &[ &str ] ) -> AnthropicResult< Self >
    {
      for env_var in env_vars
      {
        if let Ok( api_key ) = std::env::var( env_var )
        {
          if !api_key.trim().is_empty()
          {
            return Self::new( api_key ).map_err( | e | AnthropicError::InvalidArgument( e.to_string() ) );
          }
        }
      }

      let env_list = env_vars.join( ", " );
      Err( AnthropicError::MissingEnvironment(
        format!( "No API key found in environment variables : {env_list}" )
      ))
    }
  }
}

#[ cfg( feature = "authentication" ) ]
crate::mod_interface!
{
  // No public types — authentication feature provides extension methods on Secret
}

#[ cfg( not( feature = "authentication" ) ) ]
crate::mod_interface!
{
  // Empty when authentication feature is disabled
}
