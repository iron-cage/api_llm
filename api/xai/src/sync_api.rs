mod private
{
  //! Synchronous (blocking) API wrappers.
  //!
  //! Provides blocking wrappers around the async XAI API client using `tokio::runtime::Runtime`.
  //!
  //! # ⚠️ Design Warning
  //!
  //! **This contradicts Rust async-first design.** Use cases : legacy integration, simple scripts, learning.
  //! Not recommended due to : performance overhead, thread blocking, poor composability with async code.
  //!
  //! ## Recommended Alternative
  //!
  //! Create application-level runtime and use `runtime.block_on()` for async calls:
  //!
  //! ```no_run
  //! use api_xai::{ Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message, ClientApiAccessors };
  //! use tokio::runtime::Runtime;
  //!
  //! # fn example() -> Result< (), Box< dyn std::error::Error > > {
  //! let rt = Runtime::new()?;
  //! let secret = Secret::new( "xai-key".to_string() )?;
  //! let env = XaiEnvironmentImpl::new( secret )?;
  //! let client = Client::build( env )?;
  //! let request = ChatCompletionRequest::former()
  //!   .model( "grok-2-1212".to_string() )
  //!   .messages( vec![ Message::user( "Hello!" ) ] )
  //!   .form();
  //! let response = rt.block_on( client.chat().create( request ) )?;
  //! # Ok( () )
  //! # }
  //! ```

  use crate::{ ChatCompletionRequest, ChatCompletionResponse, Client, XaiEnvironment, ClientApiAccessors };
  use crate::error::Result;
  #[ cfg( feature = "streaming" ) ]
  use crate::ChatCompletionChunk;
  #[ cfg( feature = "streaming" ) ]
  use futures_core::Stream;
  #[ cfg( feature = "streaming" ) ]
  use std::pin::Pin;

  #[ cfg( feature = "sync_api" ) ]
  use tokio::runtime::Runtime;

  /// A synchronous (blocking) wrapper around the async XAI client.
  ///
  /// **⚠️ WARNING**: This contradicts Rust async-first design principles.
  /// Use the async `Client` instead when possible.
  ///
  /// # Performance Note
  ///
  /// Each `SyncClient` owns a `tokio::runtime::Runtime`, which has
  /// non-trivial overhead. Do not create many `SyncClient` instances.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # #[ cfg( feature = "sync_api") ]
  /// # {
  /// use api_xai::{ SyncClient, Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message };
  ///
  /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let secret = Secret::new( "xai-key".to_string() )?;
  /// let env = XaiEnvironmentImpl::new( secret )?;
  /// let client = Client::build( env )?;
  ///
  /// // Wrap in sync client
  /// let sync_client = SyncClient::new( client )?;
  ///
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .form();
  ///
  /// // Blocking call
  /// let response = sync_client.create( request )?;
  /// println!( "Response : {:?}", response.choices[ 0 ].message.content );
  /// # Ok( () )
  /// # }
  /// # }
  /// ```
  #[ cfg( feature = "sync_api" ) ]
  #[ derive( Debug ) ]
  pub struct SyncClient< E >
  where
    E : XaiEnvironment,
  {
    runtime : Runtime,
    client : Client< E >,
  }

  #[ cfg( feature = "sync_api" ) ]
  impl< E > SyncClient< E >
  where
    E : XaiEnvironment,
  {
    /// Creates a new synchronous client.
    ///
    /// # Arguments
    ///
    /// * `client` - The async client to wrap
    ///
    /// # Errors
    ///
    /// Returns error if the tokio runtime cannot be created.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "sync_api") ]
    /// # {
    /// use api_xai::{ SyncClient, Client, Secret, XaiEnvironmentImpl };
    ///
    /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    ///
    /// let sync_client = SyncClient::new( client )?;
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    pub fn new( client : Client< E > ) -> Result< Self >
    {
      let runtime = Runtime::new()
        .map_err( | e | crate::error::XaiError::ApiError( format!( "Runtime error : {e}" ) ) )?;

      Ok
      (
        Self
        {
          runtime,
          client,
        }
      )
    }

    /// Creates a chat completion request (blocking).
    ///
    /// Blocks the current thread until the API request completes.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat completion request
    ///
    /// # Returns
    ///
    /// The chat completion response.
    ///
    /// # Errors
    ///
    /// Returns errors from the underlying API client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "sync_api") ]
    /// # {
    /// use api_xai::{ SyncClient, Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message };
    ///
    /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    /// let sync_client = SyncClient::new( client )?;
    ///
    /// let request = ChatCompletionRequest::former()
    ///   .model( "grok-2-1212".to_string() )
    ///   .messages( vec![ Message::user( "Hello!" ) ] )
    ///   .form();
    ///
    /// let response = sync_client.create( request )?;
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    pub fn create( &self, request : ChatCompletionRequest ) -> Result< ChatCompletionResponse >
    {
      self.runtime.block_on( self.client.chat().create( request ) )
    }

    /// Creates a streaming chat completion request (blocking iterator).
    ///
    /// Returns a blocking iterator over streaming chunks.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat completion request
    ///
    /// # Returns
    ///
    /// A blocking iterator that yields `ChatCompletionChunk` items.
    ///
    /// # Errors
    ///
    /// Returns errors from the underlying API client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( all( feature = "sync_api", feature = "streaming" ) ) ]
    /// # {
    /// use api_xai::{ SyncClient, Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message };
    ///
    /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    /// let sync_client = SyncClient::new( client )?;
    ///
    /// let request = ChatCompletionRequest::former()
    ///   .model( "grok-2-1212".to_string() )
    ///   .messages( vec![ Message::user( "Hello!" ) ] )
    ///   .form();
    ///
    /// let mut stream = sync_client.create_stream( request )?;
    /// for chunk in stream
    /// {
    ///   let chunk = chunk?;
    ///   if let Some( choice ) = chunk.choices.first()
    ///   {
    ///     if let Some( ref content ) = choice.delta.content
    ///     {
    ///       print!( "{}", content );
    ///     }
    ///   }
    /// }
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    #[ cfg( feature = "streaming" ) ]
    pub fn create_stream( &self, request : ChatCompletionRequest ) -> Result< SyncStreamIterator< E > >
    {
      let stream = self.runtime.block_on( self.client.chat().create_stream( request ) )?;

      Ok
      (
        SyncStreamIterator
        {
          stream,
          runtime : Runtime::new()
            .map_err( | e | crate::error::XaiError::ApiError( format!( "Runtime error : {e}" ) ) )?,
          _phantom : core::marker::PhantomData,
        }
      )
    }

    /// Lists available models (blocking).
    ///
    /// # Returns
    ///
    /// List models response.
    ///
    /// # Errors
    ///
    /// Returns errors from the underlying API client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "sync_api") ]
    /// # {
    /// use api_xai::{ SyncClient, Client, Secret, XaiEnvironmentImpl };
    ///
    /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    /// let sync_client = SyncClient::new( client )?;
    ///
    /// let response = sync_client.list_models()?;
    /// for model in response.data
    /// {
    ///   println!( "Model : {}", model.id );
    /// }
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    pub fn list_models( &self ) -> Result< crate::components::ListModelsResponse >
    {
      self.runtime.block_on( self.client.models().list() )
    }

    /// Gets model information (blocking).
    ///
    /// # Arguments
    ///
    /// * `model_id` - The model ID to retrieve
    ///
    /// # Returns
    ///
    /// Model information.
    ///
    /// # Errors
    ///
    /// Returns errors from the underlying API client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "sync_api") ]
    /// # {
    /// use api_xai::{ SyncClient, Client, Secret, XaiEnvironmentImpl };
    ///
    /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    /// let sync_client = SyncClient::new( client )?;
    ///
    /// let model = sync_client.get_model( "grok-2-1212" )?;
    /// println!( "Model : {}", model.id );
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    pub fn get_model( &self, model_id : &str ) -> Result< crate::components::Model >
    {
      self.runtime.block_on( self.client.models().get( model_id ) )
    }
  }

  /// Synchronous iterator wrapper around async streaming.
  ///
  /// Provides a blocking iterator over `ChatCompletionChunk` items
  /// by wrapping the async stream in a dedicated tokio runtime.
  #[ cfg( feature = "streaming" ) ]
  pub struct SyncStreamIterator< E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    stream : Pin< Box< dyn Stream< Item = Result< ChatCompletionChunk > > + Send + 'static > >,
    runtime : Runtime,
    _phantom : core::marker::PhantomData< E >,
  }

  #[ cfg( feature = "streaming" ) ]
  impl< E > core::fmt::Debug for SyncStreamIterator< E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      f.debug_struct( "SyncStreamIterator" )
        .field( "runtime", &self.runtime )
        .finish_non_exhaustive()
    }
  }

  #[ cfg( feature = "streaming" ) ]
  impl< E > Iterator for SyncStreamIterator< E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    type Item = Result< ChatCompletionChunk >;

    fn next( &mut self ) -> Option< Self::Item >
    {
      use futures_util::StreamExt;

      self.runtime.block_on( self.stream.next() )
    }
  }

  /// Synchronous wrapper for `count_tokens` (requires `count_tokens` feature).
  ///
  /// Counts tokens in a text string for a specific model.
  ///
  /// # Arguments
  ///
  /// * `text` - The text to count tokens for
  /// * `model` - The model name
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
  /// ```no_run
  /// # #[ cfg( all(feature = "sync_api", feature = "count_tokens")) ]
  /// # {
  /// use api_xai::sync_count_tokens;
  ///
  /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let count = sync_count_tokens( "Hello, world!", "grok-2-1212" )?;
  /// println!( "Token count : {}", count );
  /// # Ok( () )
  /// # }
  /// # }
  /// ```
  #[ cfg( all( feature = "sync_api", feature = "count_tokens" ) ) ]
  pub fn sync_count_tokens( text : &str, model : &str ) -> Result< usize >
  {
    crate::count_tokens( text, model )
  }

  /// Synchronous wrapper for `count_tokens_for_request` (requires `count_tokens` feature).
  ///
  /// Counts tokens in a chat completion request.
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
  /// ```no_run
  /// # #[ cfg( all(feature = "sync_api", feature = "count_tokens")) ]
  /// # {
  /// use api_xai::{ sync_count_tokens_for_request, ChatCompletionRequest, Message };
  ///
  /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .form();
  ///
  /// let count = sync_count_tokens_for_request( &request )?;
  /// println!( "Total request tokens : {}", count );
  /// # Ok( () )
  /// # }
  /// # }
  /// ```
  #[ cfg( all( feature = "sync_api", feature = "count_tokens" ) ) ]
  pub fn sync_count_tokens_for_request( request : &ChatCompletionRequest ) -> Result< usize >
  {
    crate::count_tokens_for_request( request )
  }

  /// Synchronous wrapper for `validate_request_size` (requires `count_tokens` feature).
  ///
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
  /// ```no_run
  /// # #[ cfg( all(feature = "sync_api", feature = "count_tokens")) ]
  /// # {
  /// use api_xai::{ sync_validate_request_size, ChatCompletionRequest, Message };
  ///
  /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .form();
  ///
  /// // Grok-3 has 131K context window
  /// sync_validate_request_size( &request, 131072 )?;
  /// # Ok( () )
  /// # }
  /// # }
  /// ```
  #[ cfg( all( feature = "sync_api", feature = "count_tokens" ) ) ]
  pub fn sync_validate_request_size
  (
    request : &ChatCompletionRequest,
    max_tokens : usize
  )
  -> Result< () >
  {
    crate::validate_request_size( request, max_tokens )
  }

  /// Synchronous wrapper for `cached_create` (requires `caching` feature).
  ///
  /// **Note**: This is NOT recommended. Caching works better with async
  /// because the cache can be shared across concurrent requests.
  ///
  /// For sync usage, prefer using `SyncClient` with application-level caching.
  #[ cfg( all( feature = "sync_api", feature = "caching" ) ) ]
  #[ derive( Debug ) ]
  pub struct SyncCachedClient< E >
  where
    E : XaiEnvironment,
  {
    runtime : Runtime,
    cached_client : crate::CachedClient< E >,
  }

  #[ cfg( all( feature = "sync_api", feature = "caching" ) ) ]
  impl< E > SyncCachedClient< E >
  where
    E : XaiEnvironment,
  {
    /// Creates a new synchronous cached client.
    ///
    /// # Arguments
    ///
    /// * `client` - The async client to wrap
    /// * `capacity` - Maximum number of responses to cache
    ///
    /// # Errors
    ///
    /// Returns error if the tokio runtime cannot be created.
    pub fn new( client : Client< E >, capacity : usize ) -> Result< Self >
    {
      let runtime = Runtime::new()
        .map_err( | e | crate::error::XaiError::ApiError( format!( "Runtime error : {e}" ) ) )?;

      let cached_client = crate::CachedClient::new( client, capacity );

      Ok
      (
        Self
        {
          runtime,
          cached_client,
        }
      )
    }

    /// Creates a chat completion request with caching (blocking).
    ///
    /// # Arguments
    ///
    /// * `request` - The chat completion request
    ///
    /// # Returns
    ///
    /// The chat completion response (cached or fresh).
    ///
    /// # Errors
    ///
    /// Returns errors from the underlying API client, including network errors,
    /// API errors, authentication failures, and serialization errors.
    pub fn create( &self, request : ChatCompletionRequest ) -> Result< ChatCompletionResponse >
    {
      self.runtime.block_on( self.cached_client.cached_create( request ) )
    }

    /// Clears all cached responses.
    pub fn clear( &self )
    {
      self.cached_client.clear();
    }

    /// Returns the number of cached responses.
    pub fn len( &self ) -> usize
    {
      self.cached_client.len()
    }

    /// Returns true if the cache is empty.
    pub fn is_empty( &self ) -> bool
    {
      self.cached_client.is_empty()
    }
  }
}

#[ cfg( feature = "sync_api" ) ]
crate::mod_interface!
{
  exposed use
  {
    SyncClient,
  };

  #[ cfg( all( feature = "sync_api", feature = "streaming" ) ) ]
  exposed use
  {
    SyncStreamIterator,
  };

  #[ cfg( all( feature = "sync_api", feature = "count_tokens" ) ) ]
  exposed use
  {
    sync_count_tokens,
    sync_count_tokens_for_request,
    sync_validate_request_size,
  };

  #[ cfg( all( feature = "sync_api", feature = "caching" ) ) ]
  exposed use
  {
    SyncCachedClient,
  };
}
