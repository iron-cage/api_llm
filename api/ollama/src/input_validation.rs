//! Input validation framework for request types.
//!
//! Provides comprehensive validation to catch invalid requests before network calls,
//! improving error messages and preventing injection attacks.

#[ cfg( feature = "input_validation" ) ]
mod private
{
  use std::fmt;

  /// Validation error with detailed context
  #[ derive( Debug, Clone, PartialEq, Eq ) ]
  pub struct ValidationError
  {
    /// Field name that failed validation
    pub field : String,
    /// Human-readable error message
    pub message : String,
    /// The invalid value (may be truncated for large inputs)
    pub value : String,
    /// The constraint that was violated
    pub constraint : String,
  }

  impl fmt::Display for ValidationError
  {
    #[ inline ]
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      write!(
        f,
        "Validation error in field '{}': {}. Value : '{}', Constraint : '{}'",
        self.field, self.message, self.value, self.constraint
      )
    }
  }

  impl std::error::Error for ValidationError {}

  /// Result type for validation operations
  pub type ValidationResult = Result< (), Vec< ValidationError > >;

  /// Trait for types that can be validated
  pub trait Validate
  {
    /// Validate the instance, returning all validation errors
    ///
    /// # Errors
    ///
    /// Returns a vector of validation errors if any fields fail validation
    fn validate( &self ) -> ValidationResult;
  }

  /// Specific validators for common field types
  pub mod validators
  {

    /// Validate model name (non-empty, max 256 chars, alphanumeric+dash+underscore+colon)
    ///
    /// # Errors
    ///
    /// Returns error if model name is invalid
    #[ inline ]
    pub fn validate_model_name( name : &str ) -> Result< (), String >
    {
      if name.is_empty()
      {
        return Err( "Model name cannot be empty".to_string() );
      }

      if name.len() > 256
      {
        return Err( format!( "Model name too long : {} chars (max 256)", name.len() ) );
      }

      // Ollama model names : alphanumeric + dash + underscore + colon + slash + dot
      let valid_chars = name.chars().all( | c |
        c.is_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '/' || c == '.'
      );

      if !valid_chars
      {
        return Err( "Model name contains invalid characters (allowed : alphanumeric, -, _, :, /, .)".to_string() );
      }

      Ok( () )
    }

    /// Validate temperature (0.0 to 2.0)
    ///
    /// # Errors
    ///
    /// Returns error if temperature is out of range
    #[ inline ]
    pub fn validate_temperature( temp : f32 ) -> Result< (), String >
    {
      if !( 0.0..=2.0 ).contains( &temp )
      {
        return Err( format!( "Temperature {} out of range [0.0, 2.0]", temp ) );
      }

      if temp.is_nan() || temp.is_infinite()
      {
        return Err( format!( "Temperature must be a finite number, got {}", temp ) );
      }

      Ok( () )
    }

    /// Validate top_p (0.0 to 1.0)
    ///
    /// # Errors
    ///
    /// Returns error if top_p is out of range
    #[ inline ]
    pub fn validate_top_p( top_p : f32 ) -> Result< (), String >
    {
      if !( 0.0..=1.0 ).contains( &top_p )
      {
        return Err( format!( "top_p {} out of range [0.0, 1.0]", top_p ) );
      }

      if top_p.is_nan() || top_p.is_infinite()
      {
        return Err( format!( "top_p must be a finite number, got {}", top_p ) );
      }

      Ok( () )
    }

    /// Validate top_k (must be positive)
    ///
    /// # Errors
    ///
    /// Returns error if top_k is invalid
    #[ inline ]
    pub fn validate_top_k( top_k : i32 ) -> Result< (), String >
    {
      if top_k <= 0
      {
        return Err( format!( "top_k must be positive, got {}", top_k ) );
      }

      Ok( () )
    }

    /// Validate repeat penalty (typically 0.0 to 2.0)
    ///
    /// # Errors
    ///
    /// Returns error if repeat penalty is invalid
    #[ inline ]
    pub fn validate_repeat_penalty( penalty : f32 ) -> Result< (), String >
    {
      if penalty < 0.0
      {
        return Err( format!( "repeat_penalty must be non-negative, got {}", penalty ) );
      }

      if penalty.is_nan() || penalty.is_infinite()
      {
        return Err( format!( "repeat_penalty must be a finite number, got {}", penalty ) );
      }

      Ok( () )
    }

    /// Validate base64 image data (basic format check)
    ///
    /// # Errors
    ///
    /// Returns error if image data is invalid
    #[ inline ]
    #[ cfg( feature = "vision_support" ) ]
    pub fn validate_base64_image( data : &str ) -> Result< (), String >
    {
      if data.is_empty()
      {
        return Err( "Image data cannot be empty".to_string() );
      }

      // Check if it looks like base64 (only contains valid base64 chars)
      let valid_chars = data.chars().all( | c |
        c.is_alphanumeric() || c == '+' || c == '/' || c == '='
      );

      if !valid_chars
      {
        return Err( "Image data contains invalid base64 characters".to_string() );
      }

      // Base64 length must be multiple of 4
      if data.len() % 4 != 0
      {
        return Err( format!( "Invalid base64 length : {} (must be multiple of 4)", data.len() ) );
      }

      // Check reasonable size limits (max 10MB base64 ~ 13.3MB decoded)
      const MAX_BASE64_SIZE : usize = 13_333_333;
      if data.len() > MAX_BASE64_SIZE
      {
        return Err( format!( "Image data too large : {} bytes (max ~10MB)", data.len() ) );
      }

      Ok( () )
    }

    /// Validate max tokens (must be positive)
    ///
    /// # Errors
    ///
    /// Returns error if max tokens is invalid
    #[ inline ]
    pub fn validate_max_tokens( max_tokens : i32 ) -> Result< (), String >
    {
      if max_tokens <= 0
      {
        return Err( format!( "max_tokens must be positive, got {}", max_tokens ) );
      }

      // Reasonable upper limit (Ollama models typically have context windows up to 128k)
      const MAX_CONTEXT : i32 = 131_072;
      if max_tokens > MAX_CONTEXT
      {
        return Err( format!( "max_tokens {} exceeds reasonable limit ({})", max_tokens, MAX_CONTEXT ) );
      }

      Ok( () )
    }

    /// Validate messages list (must not be empty)
    ///
    /// # Errors
    ///
    /// Returns error if messages list is invalid
    #[ inline ]
    pub fn validate_messages< T >( messages : &[ T ] ) -> Result< (), String >
    {
      if messages.is_empty()
      {
        return Err( "Messages list cannot be empty".to_string() );
      }

      Ok( () )
    }

    /// Validate prompt (must not be empty)
    ///
    /// # Errors
    ///
    /// Returns error if prompt is invalid
    #[ inline ]
    pub fn validate_prompt( prompt : &str ) -> Result< (), String >
    {
      if prompt.is_empty()
      {
        return Err( "Prompt cannot be empty".to_string() );
      }

      // Reasonable length check (most models have context limits)
      const MAX_PROMPT_LENGTH : usize = 500_000; // ~500k chars
      if prompt.len() > MAX_PROMPT_LENGTH
      {
        return Err( format!( "Prompt too long : {} chars (max {})", prompt.len(), MAX_PROMPT_LENGTH ) );
      }

      Ok( () )
    }

    /// Validate audio format
    ///
    /// # Errors
    ///
    /// Returns error if audio format is invalid
    #[ inline ]
    #[ cfg( feature = "audio_processing" ) ]
    pub fn validate_audio_format( format : &str ) -> Result< (), String >
    {
      const VALID_FORMATS : &[ &str ] = &[ "wav", "mp3", "ogg", "flac", "m4a" ];

      if !VALID_FORMATS.contains( &format )
      {
        return Err( format!(
          "Invalid audio format '{}' (valid : {})",
          format,
          VALID_FORMATS.join( ", " )
        ));
      }

      Ok( () )
    }
  }

  // Implementation of Validate trait for request types
  impl Validate for crate::ChatRequest
  {
    #[ inline ]
    fn validate( &self ) -> ValidationResult
    {
      let mut errors = Vec::new();

      // Validate model name
      if let Err( e ) = validators::validate_model_name( &self.model )
      {
        errors.push( ValidationError
        {
          field : "model".to_string(),
          message : e,
          value : truncate_value( &self.model, 50 ),
          constraint : "non-empty, max 256 chars, alphanumeric+-_:/.".to_string(),
        });
      }

      // Validate messages
      if let Err( e ) = validators::validate_messages( &self.messages )
      {
        errors.push( ValidationError
        {
          field : "messages".to_string(),
          message : e,
          value : format!( "{} messages", self.messages.len() ),
          constraint : "at least 1 message".to_string(),
        });
      }

      // Validate options if present (check it's a valid object)
      if let Some( ref options ) = self.options
      {
        if !options.is_object() && !options.is_null()
        {
          errors.push( ValidationError
          {
            field : "options".to_string(),
            message : "Options must be a JSON object".to_string(),
            value : truncate_value( &options.to_string(), 50 ),
            constraint : "JSON object".to_string(),
          });
        }

        // Validate specific option fields if present
        if let Some( obj ) = options.as_object()
        {
          if let Some( temp ) = obj.get( "temperature" ).and_then( | v | v.as_f64() )
          {
            if let Err( e ) = validators::validate_temperature( temp as f32 )
            {
              errors.push( ValidationError
              {
                field : "options.temperature".to_string(),
                message : e,
                value : format!( "{}", temp ),
                constraint : "[0.0, 2.0]".to_string(),
              });
            }
          }

          if let Some( top_p ) = obj.get( "top_p" ).and_then( | v | v.as_f64() )
          {
            if let Err( e ) = validators::validate_top_p( top_p as f32 )
            {
              errors.push( ValidationError
              {
                field : "options.top_p".to_string(),
                message : e,
                value : format!( "{}", top_p ),
                constraint : "[0.0, 1.0]".to_string(),
              });
            }
          }

          if let Some( top_k ) = obj.get( "top_k" ).and_then( | v | v.as_i64() )
          {
            if let Err( e ) = validators::validate_top_k( top_k as i32 )
            {
              errors.push( ValidationError
              {
                field : "options.top_k".to_string(),
                message : e,
                value : format!( "{}", top_k ),
                constraint : "positive integer".to_string(),
              });
            }
          }

          if let Some( penalty ) = obj.get( "repeat_penalty" ).and_then( | v | v.as_f64() )
          {
            if let Err( e ) = validators::validate_repeat_penalty( penalty as f32 )
            {
              errors.push( ValidationError
              {
                field : "options.repeat_penalty".to_string(),
                message : e,
                value : format!( "{}", penalty ),
                constraint : "non-negative".to_string(),
              });
            }
          }
        }
      }

      if errors.is_empty() { Ok( () ) } else { Err( errors ) }
    }
  }

  impl Validate for crate::GenerateRequest
  {
    #[ inline ]
    fn validate( &self ) -> ValidationResult
    {
      let mut errors = Vec::new();

      // Validate model name
      if let Err( e ) = validators::validate_model_name( &self.model )
      {
        errors.push( ValidationError
        {
          field : "model".to_string(),
          message : e,
          value : truncate_value( &self.model, 50 ),
          constraint : "non-empty, max 256 chars, alphanumeric+-_:/.".to_string(),
        });
      }

      // Validate prompt
      if let Err( e ) = validators::validate_prompt( &self.prompt )
      {
        errors.push( ValidationError
        {
          field : "prompt".to_string(),
          message : e,
          value : truncate_value( &self.prompt, 100 ),
          constraint : "non-empty, max 500k chars".to_string(),
        });
      }

      // Validate options if present
      if let Some( ref options ) = self.options
      {
        if !options.is_object() && !options.is_null()
        {
          errors.push( ValidationError
          {
            field : "options".to_string(),
            message : "Options must be a JSON object".to_string(),
            value : truncate_value( &options.to_string(), 50 ),
            constraint : "JSON object".to_string(),
          });
        }

        // Validate specific option fields if present
        if let Some( obj ) = options.as_object()
        {
          if let Some( temp ) = obj.get( "temperature" ).and_then( | v | v.as_f64() )
          {
            if let Err( e ) = validators::validate_temperature( temp as f32 )
            {
              errors.push( ValidationError
              {
                field : "options.temperature".to_string(),
                message : e,
                value : format!( "{}", temp ),
                constraint : "[0.0, 2.0]".to_string(),
              });
            }
          }

          if let Some( top_p ) = obj.get( "top_p" ).and_then( | v | v.as_f64() )
          {
            if let Err( e ) = validators::validate_top_p( top_p as f32 )
            {
              errors.push( ValidationError
              {
                field : "options.top_p".to_string(),
                message : e,
                value : format!( "{}", top_p ),
                constraint : "[0.0, 1.0]".to_string(),
              });
            }
          }
        }
      }

      if errors.is_empty() { Ok( () ) } else { Err( errors ) }
    }
  }

  #[ cfg( feature = "embeddings" ) ]
  impl Validate for crate::EmbeddingsRequest
  {
    #[ inline ]
    fn validate( &self ) -> ValidationResult
    {
      let mut errors = Vec::new();

      // Validate model name
      if let Err( e ) = validators::validate_model_name( &self.model )
      {
        errors.push( ValidationError
        {
          field : "model".to_string(),
          message : e,
          value : truncate_value( &self.model, 50 ),
          constraint : "non-empty, max 256 chars, alphanumeric+-_:/.".to_string(),
        });
      }

      // Validate prompt
      if let Err( e ) = validators::validate_prompt( &self.prompt )
      {
        errors.push( ValidationError
        {
          field : "prompt".to_string(),
          message : e,
          value : truncate_value( &self.prompt, 100 ),
          constraint : "non-empty, max 500k chars".to_string(),
        });
      }

      if errors.is_empty() { Ok( () ) } else { Err( errors ) }
    }
  }

  /// Truncate value for display in error messages
  #[ inline ]
  fn truncate_value( s : &str, max_len : usize ) -> String
  {
    if s.len() <= max_len
    {
      s.to_string()
    }
    else
    {
      format!( "{}... ({} chars total)", &s[ ..max_len ], s.len() )
    }
  }
}

#[ cfg( feature = "input_validation" ) ]
crate ::mod_interface!
{
  exposed use private::ValidationError;
  exposed use private::ValidationResult;
  exposed use private::Validate;
  exposed use private::validators;
}
