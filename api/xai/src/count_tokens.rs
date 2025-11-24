mod private
{
  //! Token counting functionality using tiktoken.
  //!
  //! This module provides local token counting for XAI Grok API requests,
  //! which is essential for context management since the XAI API does not
  //! provide a token counting endpoint.
  //!
  //! # Design Decisions
  //!
  //! ## Why tiktoken?
  //!
  //! The XAI Grok API is OpenAI-compatible and uses the same tokenization
  //! as GPT models. tiktoken provides:
  //!
  //! 1. **Accuracy**: Same tokenizer used by OpenAI/XAI APIs
  //! 2. **Performance**: Rust implementation is fast
  //! 3. **Offline**: No API calls needed for counting
  //! 4. **Standards**: Industry-standard BPE tokenization
  //!
  //! ## Alternatives Considered
  //!
  //! - **API-based counting**: XAI doesn't provide this endpoint
  //! - **Custom tokenizer**: Would diverge from actual API behavior
  //! - **Estimated counting**: Inaccurate, leads to context overflow errors
  //!
  //! ## Model Mapping
  //!
  //! XAI's Grok models use GPT-4 tokenization:
  //! - `grok-2-1212` → uses `cl100k_base` encoding (same as GPT-4)
  //! - `grok-2-1212` → uses `cl100k_base` encoding
  //!
  //! This mapping may need updates as XAI releases new models.

  use crate::{ ChatCompletionRequest, Message };
  use crate::error::{ XaiError, Result };

  #[ cfg( feature = "count_tokens" ) ]
  use tiktoken_rs::{ get_bpe_from_model, CoreBPE };

  /// Counts tokens in a text string for a specific model.
  ///
  ///  Uses the tiktoken library to accurately count tokens as the XAI API would.
  ///
  /// # Arguments
  ///
  /// * `text` - The text to count tokens for
  /// * `model` - The model name (e.g., "grok-2-1212", "grok-2-1212")
  ///
  /// # Returns
  ///
  /// Number of tokens in the text.
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidModel` if the model is not supported.
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "count_tokens") ]
  /// # {
  /// use api_xai::count_tokens::count_tokens;
  ///
  /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let count = count_tokens( "Hello, world!", "grok-2-1212" )?;
  /// println!( "Token count : {}", count );
  /// # Ok( () )
  /// # }
  /// # }
  /// ```
  #[ cfg( feature = "count_tokens" ) ]
  pub fn count_tokens( text : &str, model : &str ) -> Result< usize >
  {
    let bpe = get_tokenizer_for_model( model )?;
    Ok( bpe.encode_with_special_tokens( text ).len() )
  }

  /// Counts tokens in a chat completion request.
  ///
  /// Accurately estimates the total token count for a chat request,
  /// including system messages, user messages, assistant messages,
  /// and function calling overhead.
  ///
  /// # Arguments
  ///
  /// * `request` - The chat completion request
  ///
  /// # Returns
  ///
  /// Estimated total token count for the request.
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidModel` if the model is not supported.
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "count_tokens") ]
  /// # {
  /// use api_xai::{ ChatCompletionRequest, Message, count_tokens_for_request };
  ///
  /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .form();
  ///
  /// let count = count_tokens_for_request( &request )?;
  /// println!( "Total request tokens : {}", count );
  /// # Ok( () )
  /// # }
  /// # }
  /// ```
  #[ cfg( feature = "count_tokens" ) ]
  pub fn count_tokens_for_request( request : &ChatCompletionRequest ) -> Result< usize >
  {
    let bpe = get_tokenizer_for_model( &request.model )?;
    let mut total = 0;

    // Count tokens in all messages
    for message in &request.messages
    {
      total += count_tokens_for_message( message, &bpe )?;
    }

    // Add overhead for chat completion formatting
    // Based on OpenAI's token counting methodology
    total += 3; // Every reply is primed with <|start|>assistant<|message|>

    // Add tokens for function calling if tools are present
    if let Some( ref tools ) = request.tools
    {
      for tool in tools
      {
        let tool_json = serde_json::to_string( &tool.function )
          .map_err( | e | XaiError::Serialization( e.to_string() ) )?;
        total += bpe.encode_with_special_tokens( &tool_json ).len();
      }
    }

    Ok( total )
  }

  /// Counts tokens for a single message.
  ///
  /// # Arguments
  ///
  /// * `message` - The message to count
  /// * `bpe` - The tokenizer to use
  ///
  /// # Returns
  ///
  /// Token count for the message, including role and formatting overhead.
  #[ cfg( feature = "count_tokens" ) ]
  fn count_tokens_for_message( message : &Message, bpe : &CoreBPE ) -> Result< usize >
  {
    let mut tokens = 0;

    // Role tokens (convert enum to string)
    let role_str = serde_json::to_string( &message.role )
      .map_err( | e | XaiError::Serialization( e.to_string() ) )?;
    // Remove quotes from JSON string
    let role_str = role_str.trim_matches( '"' );
    tokens += bpe.encode_with_special_tokens( role_str ).len();

    // Content tokens
    if let Some( ref content ) = message.content
    {
      tokens += bpe.encode_with_special_tokens( content ).len();
    }

    // Tool calls tokens (if present)
    if let Some( ref tool_calls ) = message.tool_calls
    {
      for tool_call in tool_calls
      {
        let tool_json = serde_json::to_string( tool_call )
          .map_err( | e | XaiError::Serialization( e.to_string() ) )?;
        tokens += bpe.encode_with_special_tokens( &tool_json ).len();
      }
    }

    // Message formatting overhead
    tokens += 4; // Every message follows <|start|>{role/name}\n{content}<|end|>\n

    Ok( tokens )
  }

  /// Gets the appropriate tokenizer for a model.
  ///
  /// Maps XAI model names to tiktoken encodings.
  ///
  /// # Arguments
  ///
  /// * `model` - The model name
  ///
  /// # Returns
  ///
  /// The tokenizer for the model.
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidModel` if the model is not recognized.
  #[ cfg( feature = "count_tokens" ) ]
  fn get_tokenizer_for_model( model : &str ) -> Result< CoreBPE >
  {
    // XAI Grok models use GPT-4's tokenization (cl100k_base)
    match model
    {
      "grok-2-1212" | "grok-2" =>
      {
        // Map to gpt-4 for tiktoken
        get_bpe_from_model( "gpt-4" )
          .map_err( | e | XaiError::InvalidModel( format!( "Tokenizer error : {e}" ) ).into() )
      }
      _ =>
      {
        Err( XaiError::InvalidModel( format!( "Unknown model : {model}" ) ).into() )
      }
    }
  }

  /// Validates that a request fits within the model's context window.
  ///
  /// # Arguments
  ///
  /// * `request` - The chat completion request
  /// * `max_tokens` - The model's maximum context window size
  ///
  /// # Returns
  ///
  /// `Ok(())` if the request fits, error otherwise.
  ///
  /// # Errors
  ///
  /// Returns `XaiError::InvalidParameter` if the request exceeds the context window.
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "count_tokens") ]
  /// # {
  /// use api_xai::{ ChatCompletionRequest, Message, validate_request_size };
  ///
  /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .form();
  ///
  /// // Grok-3 has 131K context window
  /// validate_request_size( &request, 131072 )?;
  /// # Ok( () )
  /// # }
  /// # }
  /// ```
  #[ cfg( feature = "count_tokens" ) ]
  pub fn validate_request_size( request : &ChatCompletionRequest, max_tokens : usize ) -> Result< () >
  {
    let token_count = count_tokens_for_request( request )?;

    // Account for max_tokens parameter (response budget)
    let response_budget = request.max_tokens.unwrap_or( 1000 );
    let total_needed = token_count + response_budget as usize;

    if total_needed > max_tokens
    {
      return Err( XaiError::InvalidParameter(
        format!(
          "Request ({token_count} tokens) + response budget ({response_budget} tokens) \
           = {total_needed} tokens exceeds context window ({max_tokens} tokens)"
        )
      ).into() );
    }

    Ok( () )
  }
}

#[ cfg( feature = "count_tokens" ) ]
crate::mod_interface!
{
  exposed use
  {
    count_tokens,
    count_tokens_for_request,
    validate_request_size,
  };
}
