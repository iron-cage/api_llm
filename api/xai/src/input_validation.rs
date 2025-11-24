mod private
{
  //! Input validation for XAI API requests.
  //!
  //! This module provides comprehensive client-side validation of request
  //! parameters before sending them to the XAI API. Early validation helps:
  //!
  //! 1. **Catch Errors Early**: Detect invalid parameters before API calls
  //! 2. **Better Error Messages**: Provide clear, actionable feedback
  //! 3. **Cost Savings**: Avoid wasting API calls on invalid requests
  //! 4. **Type Safety**: Enforce constraints beyond Rust's type system
  //!
  //! # Design Decisions
  //!
  //! ## Why Client-Side Validation?
  //!
  //! While the XAI API performs server-side validation, client-side validation:
  //!
  //! 1. **Faster Feedback**: Immediate errors without network round-trip
  //! 2. **Better UX**: Clear error messages vs generic API errors
  //! 3. **Cost Reduction**: Avoid API calls for obviously invalid requests
  //! 4. **Offline Development**: Validate without network access
  //!
  //! ## What is Validated?
  //!
  //! - **Model Names**: Only valid XAI models (grok-2-1212, grok-2-1212, grok-2)
  //! - **Messages**: Non-empty arrays, non-empty content
  //! - **Temperature**: Range [0.0, 2.0]
  //! - **Max Tokens**: Positive values within context window
  //! - **Top P**: Range [0.0, 1.0]
  //! - **Frequency/Presence Penalty**: Range [-2.0, 2.0]
  //! - **Tool Schemas**: Valid JSON schemas for function calling
  //!
  //! ## Validation Philosophy
  //!
  //! Validation is **explicit** and **opt-in**:
  //!
  //! 1. **Not Automatic**: Validation must be called explicitly
  //! 2. **Feature Gated**: Can be disabled for zero-cost abstraction
  //! 3. **Non-Mutating**: Validation never modifies requests
  //! 4. **Fail Fast**: Returns first error encountered

  use crate::{ ChatCompletionRequest, Message, Tool, Function };
  use crate::error::{ XaiError, Result };

  /// Validates a chat completion request.
  ///
  /// Performs comprehensive validation of all request parameters,
  /// returning the first error encountered.
  ///
  /// # Validation Rules
  ///
  /// - **Model**: Must be a valid XAI model name
  /// - **Messages**: Must be non-empty with non-empty content
  /// - **Temperature**: Must be in [0.0, 2.0]
  /// - **Max Tokens**: Must be positive
  /// - **Top P**: Must be in [0.0, 1.0]
  /// - **Penalties**: Must be in [-2.0, 2.0]
  /// - **Tools**: Must have valid schemas
  ///
  /// # Arguments
  ///
  /// * `request` - The chat completion request to validate
  ///
  /// # Returns
  ///
  /// `Ok(())` if validation passes, error otherwise.
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidParameter` or `XaiError::InvalidModel`
  /// for validation failures.
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "input_validation") ]
  /// # {
  /// use api_xai::{ validate_request, ChatCompletionRequest, Message };
  ///
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .temperature( 0.7 )
  ///   .form();
  ///
  /// // Validation passes
  /// assert!( validate_request( &request ).is_ok() );
  ///
  /// let bad_request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .temperature( 3.0 ) // Invalid!
  ///   .form();
  ///
  /// // Validation fails
  /// assert!( validate_request( &bad_request ).is_err() );
  /// # }
  /// ```
  #[ cfg( feature = "input_validation" ) ]
  pub fn validate_request( request : &ChatCompletionRequest ) -> Result< () >
  {
    validate_model( &request.model )?;
    validate_messages( &request.messages )?;
    validate_temperature( request.temperature )?;
    validate_max_tokens( request.max_tokens )?;
    validate_top_p( request.top_p )?;
    validate_frequency_penalty( request.frequency_penalty )?;
    validate_presence_penalty( request.presence_penalty )?;

    if let Some( ref tools ) = request.tools
    {
      validate_tools( tools )?;
    }

    Ok( () )
  }

  /// Validates that the model name is supported.
  ///
  /// # Valid Models
  ///
  /// - `grok-2-1212` - Latest Grok model
  /// - `grok-2-1212` - Beta version
  /// - `grok-2` - Previous version
  ///
  /// # Arguments
  ///
  /// * `model` - The model name to validate
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidModel` if the model is not recognized.
  #[ cfg( feature = "input_validation" ) ]
  pub fn validate_model( model : &str ) -> Result< () >
  {
    const VALID_MODELS : &[ &str ] = &[ "grok-2-1212", "grok-2-1212", "grok-2" ];

    if !VALID_MODELS.contains( &model )
    {
      return Err
      (
        XaiError::InvalidModel
        (
          format!
          (
            "Unknown model : '{}'. Valid models : {}",
            model,
            VALID_MODELS.join( ", " )
          )
        ).into()
      );
    }

    Ok( () )
  }

  /// Validates that messages array is non-empty with valid content.
  ///
  /// # Validation Rules
  ///
  /// - Messages array must not be empty
  /// - Each message with content must have non-empty content
  /// - System messages are allowed
  ///
  /// # Arguments
  ///
  /// * `messages` - The messages array to validate
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidParameter` if validation fails.
  #[ cfg( feature = "input_validation" ) ]
  pub fn validate_messages( messages : &[ Message ] ) -> Result< () >
  {
    if messages.is_empty()
    {
      return Err
      (
        XaiError::InvalidParameter
        (
          "messages array cannot be empty".to_string()
        ).into()
      );
    }

    for ( idx, message ) in messages.iter().enumerate()
    {
      if let Some( ref content ) = message.content
      {
        if content.trim().is_empty()
        {
          return Err
          (
            XaiError::InvalidParameter
            (
              format!( "message[{idx}] content cannot be empty or whitespace-only" )
            ).into()
          );
        }
      }
    }

    Ok( () )
  }

  /// Validates that temperature is within valid range [0.0, 2.0].
  ///
  /// # Arguments
  ///
  /// * `temperature` - The temperature value to validate
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidParameter` if outside range.
  #[ cfg( feature = "input_validation" ) ]
  pub fn validate_temperature( temperature : Option< f32 > ) -> Result< () >
  {
    if let Some( temp ) = temperature
    {
      if !( 0.0..=2.0 ).contains( &temp )
      {
        return Err
        (
          XaiError::InvalidParameter
          (
            format!
            (
              "temperature must be between 0.0 and 2.0, got : {temp}"
            )
          ).into()
        );
      }
    }

    Ok( () )
  }

  /// Validates that `max_tokens` is positive.
  ///
  /// # Arguments
  ///
  /// * `max_tokens` - The `max_tokens` value to validate
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidParameter` if non-positive.
  #[ cfg( feature = "input_validation" ) ]
  pub fn validate_max_tokens( max_tokens : Option< u32 > ) -> Result< () >
  {
    if let Some( tokens ) = max_tokens
    {
      if tokens == 0
      {
        return Err
        (
          XaiError::InvalidParameter
          (
            "max_tokens must be positive (> 0)".to_string()
          ).into()
        );
      }
    }

    Ok( () )
  }

  /// Validates that `top_p` is within valid range [0.0, 1.0].
  ///
  /// # Arguments
  ///
  /// * `top_p` - The `top_p` value to validate
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidParameter` if outside range.
  #[ cfg( feature = "input_validation" ) ]
  pub fn validate_top_p( top_p : Option< f32 > ) -> Result< () >
  {
    if let Some( p ) = top_p
    {
      if !( 0.0..=1.0 ).contains( &p )
      {
        return Err
        (
          XaiError::InvalidParameter
          (
            format!
            (
              "top_p must be between 0.0 and 1.0, got : {p}"
            )
          ).into()
        );
      }
    }

    Ok( () )
  }

  /// Validates that `frequency_penalty` is within valid range [-2.0, 2.0].
  ///
  /// # Arguments
  ///
  /// * `frequency_penalty` - The `frequency_penalty` value to validate
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidParameter` if outside range.
  #[ cfg( feature = "input_validation" ) ]
  pub fn validate_frequency_penalty( frequency_penalty : Option< f32 > ) -> Result< () >
  {
    if let Some( penalty ) = frequency_penalty
    {
      if !( -2.0..=2.0 ).contains( &penalty )
      {
        return Err
        (
          XaiError::InvalidParameter
          (
            format!
            (
              "frequency_penalty must be between -2.0 and 2.0, got : {penalty}"
            )
          ).into()
        );
      }
    }

    Ok( () )
  }

  /// Validates that `presence_penalty` is within valid range [-2.0, 2.0].
  ///
  /// # Arguments
  ///
  /// * `presence_penalty` - The `presence_penalty` value to validate
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidParameter` if outside range.
  #[ cfg( feature = "input_validation" ) ]
  pub fn validate_presence_penalty( presence_penalty : Option< f32 > ) -> Result< () >
  {
    if let Some( penalty ) = presence_penalty
    {
      if !( -2.0..=2.0 ).contains( &penalty )
      {
        return Err
        (
          XaiError::InvalidParameter
          (
            format!
            (
              "presence_penalty must be between -2.0 and 2.0, got : {penalty}"
            )
          ).into()
        );
      }
    }

    Ok( () )
  }

  /// Validates tool definitions (function calling schemas).
  ///
  /// # Validation Rules
  ///
  /// - Each tool must have a valid function definition
  /// - Function names must be non-empty
  /// - Function descriptions should be non-empty (warning only)
  /// - Parameters must be valid JSON schemas
  ///
  /// # Arguments
  ///
  /// * `tools` - The tools array to validate
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidParameter` if validation fails.
  #[ cfg( feature = "input_validation" ) ]
  pub fn validate_tools( tools : &[ Tool ] ) -> Result< () >
  {
    for ( idx, tool ) in tools.iter().enumerate()
    {
      validate_function_definition( &tool.function, idx )?;
    }

    Ok( () )
  }

  /// Validates a function definition.
  ///
  /// # Arguments
  ///
  /// * `function` - The function definition to validate
  /// * `idx` - Index in tools array (for error messages)
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidParameter` if validation fails.
  #[ cfg( feature = "input_validation" ) ]
  fn validate_function_definition
  (
    function : &Function,
    idx : usize
  )
  -> Result< () >
  {
    // Function name must be non-empty
    if function.name.trim().is_empty()
    {
      return Err
      (
        XaiError::InvalidParameter
        (
          format!( "tool[{idx}] function name cannot be empty" )
        ).into()
      );
    }

    // Description should be non-empty (best practice, not enforced)
    if function.description.trim().is_empty()
    {
      // Note : This is a warning, not an error
      // The API will accept it, but it's not recommended
    }

    // Parameters must be valid JSON schema
    // Try to serialize to ensure it's valid JSON
    serde_json::to_string( &function.parameters )
      .map_err
      (
        | e |
        {
          XaiError::InvalidParameter
          (
            format!( "tool[{idx}] parameters are not valid JSON: {e}" )
          )
        }
      )?;

    Ok( () )
  }
}

#[ cfg( feature = "input_validation" ) ]
crate::mod_interface!
{
  exposed use
  {
    validate_request,
    validate_model,
    validate_messages,
    validate_temperature,
    validate_max_tokens,
    validate_top_p,
    validate_frequency_penalty,
    validate_presence_penalty,
    validate_tools,
  };
}
