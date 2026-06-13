//! System prompt construction types
//!
//! `CacheControl`, `SystemPrompt`, `SystemContent`, and `SystemInstructions`
//! for building structured system prompts with optional prompt caching.

#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  use serde::{ Serialize, Deserialize };

  /// Cache control configuration for prompt caching
  ///
  /// Anthropic Prompt Caching allows caching of large context (system prompts, documents, etc.)
  /// to reduce costs (~90% savings on cached tokens) and improve latency.
  #[ derive( Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
  pub struct CacheControl
  {
    /// Cache type - currently only "ephemeral" is supported (5-minute TTL)
    #[ serde( rename = "type" ) ]
    pub cache_type : String,
  }

  impl CacheControl
  {
    /// Create an ephemeral cache control (5-minute TTL)
    pub fn ephemeral() -> Self
    {
      Self { cache_type : "ephemeral".to_string() }
    }
  }

  impl From< String > for SystemPrompt
  {
    fn from( text : String ) -> Self
    {
      Self { text, cache_control : None }
    }
  }

  impl From< &str > for SystemPrompt
  {
    fn from( text : &str ) -> Self
    {
      Self { text : text.to_string(), cache_control : None }
    }
  }

  /// System prompt with optional cache control
  ///
  /// Replaces simple String system prompts to support caching large system contexts.
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub struct SystemPrompt
  {
    /// System prompt text
    pub text : String,
    /// Optional cache control for this system prompt
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub cache_control : Option< CacheControl >,
  }

  /// System content block for count tokens endpoint
  ///
  /// The count tokens endpoint expects system as an array of content blocks
  #[ derive( Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
  pub struct SystemContent
  {
    /// Type - always "text"
    #[ serde( rename = "type" ) ]
    pub r#type : String,
    /// Text content
    pub text : String,
    /// Optional cache control
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub cache_control : Option< CacheControl >,
  }

  impl SystemContent
  {
    /// Create a new system content block from text
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::SystemContent;
    ///
    /// let content = SystemContent::text( "You are a helpful assistant" );
    /// assert_eq!( content.text, "You are a helpful assistant" );
    /// ```
    pub fn text< S : Into< String > >( text : S ) -> Self
    {
      Self
      {
        r#type : "text".to_string(),
        text : text.into(),
        cache_control : None,
      }
    }

    /// Set cache control for this system content (builder pattern)
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::{ SystemContent, CacheControl };
    ///
    /// let content = SystemContent::text( "Knowledge base" )
    ///   .with_cache_control( CacheControl::ephemeral() );
    ///
    /// assert!( content.cache_control.is_some() );
    /// ```
    #[ must_use ]
    pub fn with_cache_control( mut self, cache_control : CacheControl ) -> Self
    {
      self.cache_control = Some( cache_control );
      self
    }

    /// Validate the system content
    ///
    /// Checks that:
    /// - Text is not empty
    /// - Type is set correctly
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::SystemContent;
    ///
    /// let content = SystemContent::text( "Valid content" );
    /// assert!( content.validate().is_ok() );
    ///
    /// let empty = SystemContent
    /// {
    ///   r#type : "text".to_string(),
    ///   text : "".to_string(),
    ///   cache_control : None,
    /// };
    /// assert!( empty.validate().is_err() );
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the text is empty or the type is not "text"
    pub fn validate( &self ) -> Result< (), String >
    {
      if self.text.is_empty()
      {
        return Err( "System content text cannot be empty".to_string() );
      }

      if self.r#type != "text"
      {
        return Err( format!( "Invalid system content type : {}", self.r#type ) );
      }

      Ok( () )
    }

    /// Check if this system content has cache control enabled
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::{ SystemContent, CacheControl };
    ///
    /// let cached = SystemContent::text( "Cached" )
    ///   .with_cache_control( CacheControl::ephemeral() );
    /// assert!( cached.has_cache_control() );
    ///
    /// let not_cached = SystemContent::text( "Not cached" );
    /// assert!( !not_cached.has_cache_control() );
    /// ```
    pub fn has_cache_control( &self ) -> bool
    {
      self.cache_control.is_some()
    }
  }

  impl From< &str > for SystemContent
  {
    fn from( text : &str ) -> Self
    {
      Self::text( text )
    }
  }

  impl From< String > for SystemContent
  {
    fn from( text : String ) -> Self
    {
      Self::text( text )
    }
  }

  /// Builder for composing multi-part system instructions
  ///
  /// Provides a convenient API for building structured system prompts
  /// with multiple content blocks, optional caching, and validation.
  ///
  /// # Examples
  ///
  /// ```
  /// use api_claude::{ SystemInstructions, CacheControl };
  ///
  /// let instructions = SystemInstructions::new()
  ///   .add_text( "You are a helpful assistant." )
  ///   .add_cached_text( "Knowledge base : Large corpus of information..." )
  ///   .add_text( "Help the user with their questions." )
  ///   .build();
  ///
  /// assert_eq!( instructions.len(), 3 );
  /// ```
  #[ derive( Debug, Clone, Default ) ]
  pub struct SystemInstructions
  {
    parts : Vec< SystemContent >,
  }

  impl SystemInstructions
  {
    /// Create a new empty system instructions builder
    pub fn new() -> Self
    {
      Self
      {
        parts : Vec::new(),
      }
    }

    /// Add a text instruction
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::SystemInstructions;
    ///
    /// let instructions = SystemInstructions::new()
    ///   .add_text( "You are a helpful assistant" )
    ///   .build();
    ///
    /// assert_eq!( instructions.len(), 1 );
    /// ```
    #[ must_use ]
    pub fn add_text< S : Into< String > >( mut self, text : S ) -> Self
    {
      self.parts.push( SystemContent::text( text ) );
      self
    }

    /// Add a cached text instruction
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::SystemInstructions;
    ///
    /// let instructions = SystemInstructions::new()
    ///   .add_cached_text( "Large knowledge base" )
    ///   .build();
    ///
    /// assert!( instructions[ 0 ].has_cache_control() );
    /// ```
    #[ must_use ]
    pub fn add_cached_text< S : Into< String > >( mut self, text : S ) -> Self
    {
      let content = SystemContent::text( text )
        .with_cache_control( CacheControl::ephemeral() );
      self.parts.push( content );
      self
    }

    /// Add a custom system content block
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::{ SystemInstructions, SystemContent, CacheControl };
    ///
    /// let custom = SystemContent::text( "Custom instruction" )
    ///   .with_cache_control( CacheControl::ephemeral() );
    ///
    /// let instructions = SystemInstructions::new()
    ///   .add( custom )
    ///   .build();
    ///
    /// assert_eq!( instructions.len(), 1 );
    /// ```
    #[ must_use ]
    #[ allow( clippy::should_implement_trait ) ]
    pub fn add( mut self, content : SystemContent ) -> Self
    {
      self.parts.push( content );
      self
    }

    /// Build the final vector of system content blocks
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::SystemInstructions;
    ///
    /// let instructions = SystemInstructions::new()
    ///   .add_text( "Part 1" )
    ///   .add_text( "Part 2" )
    ///   .build();
    ///
    /// assert_eq!( instructions.len(), 2 );
    /// ```
    #[ must_use ]
    pub fn build( self ) -> Vec< SystemContent >
    {
      self.parts
    }

    /// Validate all system content blocks
    ///
    /// # Examples
    ///
    /// ```
    /// use api_claude::SystemInstructions;
    ///
    /// let instructions = SystemInstructions::new()
    ///   .add_text( "Valid instruction" );
    ///
    /// assert!( instructions.validate().is_ok() );
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if any content block fails validation
    pub fn validate( &self ) -> Result< (), String >
    {
      if self.parts.is_empty()
      {
        return Err( "System instructions cannot be empty".to_string() );
      }

      for ( idx, content ) in self.parts.iter().enumerate()
      {
        content.validate()
          .map_err( |e| format!( "Invalid content at index {idx}: {e}" ) )?;
      }

      Ok( () )
    }

    /// Get the number of content blocks
    pub fn len( &self ) -> usize
    {
      self.parts.len()
    }

    /// Check if there are no content blocks
    pub fn is_empty( &self ) -> bool
    {
      self.parts.is_empty()
    }
  }
}

crate::mod_interface!
{
  exposed use CacheControl;
  exposed use SystemPrompt;
  exposed use SystemContent;
  exposed use SystemInstructions;
}
