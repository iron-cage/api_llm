mod private
{
  //! Response caching functionality using LRU cache.
  //!
  //! This module provides application-level caching for XAI API responses,
  //! which helps reduce API calls, lower costs, and improve response times
  //! for repeated queries.
  //!
  //! # Design Decisions
  //!
  //! ## Why Application-Level Caching?
  //!
  //! The XAI Grok API does not provide server-side prompt caching like some
  //! other LLM providers. This makes client-side caching essential for:
  //!
  //! 1. **Cost Reduction**: Avoid redundant API calls for identical requests
  //! 2. **Performance**: Instant responses for cached queries
  //! 3. **Rate Limiting**: Reduce pressure on API rate limits
  //! 4. **Offline Capability**: Serve cached responses when API is unavailable
  //!
  //! ## Why LRU (Least Recently Used)?
  //!
  //! LRU eviction policy is ideal for chat applications because:
  //!
  //! 1. **Temporal Locality**: Recent queries are more likely to repeat
  //! 2. **Bounded Memory**: Automatic eviction prevents unbounded growth
  //! 3. **Simplicity**: No complex TTL management needed
  //! 4. **Performance**: O(1) get/put operations
  //!
  //! ## Alternatives Considered
  //!
  //! - **TTL-based cache**: Adds complexity, doesn't align with usage patterns
  //! - **Unlimited cache**: Memory leak risk for long-running applications
  //! - **Disk-backed cache**: Violates "Thin Client" principle (no persistence)
  //!
  //! ## Cache Key Strategy
  //!
  //! Cache keys are computed from the serialized JSON request. This ensures:
  //!
  //! 1. **Correctness**: Different requests never collide
  //! 2. **Completeness**: All request parameters affect caching
  //! 3. **Determinism**: Same request always produces same key
  //!
  //! **Note**: Streaming requests are NOT cached (responses are incremental).

  use crate::{ ChatCompletionRequest, ChatCompletionResponse, Client, XaiEnvironment, ClientApiAccessors };
  use crate::error::Result;
  use std::sync::{ Arc, Mutex };
  use std::num::NonZeroUsize;

  #[ cfg( feature = "caching" ) ]
  use lru::LruCache;

  #[ cfg( feature = "caching" ) ]
  use std::hash::{ Hash, Hasher };

  #[ cfg( feature = "caching" ) ]
  use std::collections::hash_map::DefaultHasher;

  /// A client wrapper that caches chat completion responses.
  ///
  /// Wraps a standard `Client` and adds LRU caching for non-streaming
  /// chat completion requests. Streaming requests bypass the cache.
  ///
  /// # Cache Key
  ///
  /// Cache keys are computed from the serialized JSON of the request.
  /// This ensures that any change to the request (model, messages,
  /// temperature, etc.) produces a different cache key.
  ///
  /// # Thread Safety
  ///
  /// The cache is protected by a `Mutex` and can be safely shared
  /// across threads via `Arc`.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # #[ cfg( feature = "caching") ]
  /// # {
  /// use api_xai::{ CachedClient, Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message };
  ///
  /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let secret = Secret::new( "xai-key".to_string() )?;
  /// let env = XaiEnvironmentImpl::new( secret )?;
  /// let client = Client::build( env )?;
  ///
  /// // Wrap with cache (capacity : 100 responses)
  /// let cached_client = CachedClient::new( client, 100 );
  ///
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "What is Rust?" ) ] )
  ///   .form();
  ///
  /// // First call : hits API
  /// let response1 = cached_client.cached_create( request.clone() ).await?;
  ///
  /// // Second call : hits cache (instant, no API call)
  /// let response2 = cached_client.cached_create( request ).await?;
  ///
  /// assert_eq!( response1.id, response2.id ); // Same response
  /// # Ok( () )
  /// # }
  /// # }
  /// ```
  #[ cfg( feature = "caching" ) ]
  pub struct CachedClient< E >
  where
    E : XaiEnvironment,
  {
    client : Client< E >,
    cache : Arc< Mutex< LruCache< String, ChatCompletionResponse > > >,
  }

  #[ cfg( feature = "caching" ) ]
  impl< E > std::fmt::Debug for CachedClient< E >
  where
    E : XaiEnvironment + std::fmt::Debug,
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      f.debug_struct( "CachedClient" )
        .field( "client", &self.client )
        .field( "cache", &"< LruCache >" )
        .finish()
    }
  }

  #[ cfg( feature = "caching" ) ]
  impl< E > CachedClient< E >
  where
    E : XaiEnvironment,
  {
    /// Creates a new cached client with specified capacity.
    ///
    /// # Arguments
    ///
    /// * `client` - The underlying XAI client
    /// * `capacity` - Maximum number of responses to cache
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "caching") ]
    /// # {
    /// use api_xai::{ CachedClient, Client, Secret, XaiEnvironmentImpl };
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    ///
    /// // Cache up to 100 responses
    /// let cached_client = CachedClient::new( client, 100 );
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if capacity is zero.
    pub fn new( client : Client< E >, capacity : usize ) -> Self
    {
      Self
      {
        client,
        cache : Arc::new
        (
          Mutex::new
          (
            LruCache::new
            (
              NonZeroUsize::new( capacity ).expect( "Capacity must be > 0" )
            )
          )
        ),
      }
    }

    /// Creates a chat completion request, using cache when possible.
    ///
    /// # Caching Behavior
    ///
    /// - **Cache hit**: Returns cached response instantly (no API call)
    /// - **Cache miss**: Makes API call, stores response, returns it
    /// - **Streaming requests**: Always bypass cache (no caching)
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
    /// Returns errors from the underlying API client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "caching") ]
    /// # {
    /// use api_xai::{ CachedClient, Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message };
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    /// let cached_client = CachedClient::new( client, 100 );
    ///
    /// let request = ChatCompletionRequest::former()
    ///   .model( "grok-2-1212".to_string() )
    ///   .messages( vec![ Message::user( "Hello!" ) ] )
    ///   .form();
    ///
    /// let response = cached_client.cached_create( request ).await?;
    /// println!( "Response : {:?}", response.choices[ 0 ].message.content );
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub async fn cached_create
    (
      &self,
      request : ChatCompletionRequest
    )
    -> Result< ChatCompletionResponse >
    {
      // Streaming requests cannot be cached (responses are incremental)
      if request.stream.unwrap_or( false )
      {
        return self.client.chat().create( request ).await;
      }

      let cache_key = Self::compute_cache_key( &request );

      // Check cache (with explicit scope to release lock before await)
      {
        let mut cache = self.cache.lock().unwrap();
        if let Some( cached ) = cache.get( &cache_key )
        {
          return Ok( cached.clone() );
        }
      }

      // Cache miss - make API request
      let response = self.client.chat().create( request ).await?;

      // Store in cache
      {
        let mut cache = self.cache.lock().unwrap();
        cache.put( cache_key, response.clone() );
      }

      Ok( response )
    }

    /// Clears all cached responses.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "caching") ]
    /// # {
    /// use api_xai::{ CachedClient, Client, Secret, XaiEnvironmentImpl };
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    /// let cached_client = CachedClient::new( client, 100 );
    ///
    /// // ... use cache ...
    ///
    /// // Clear cache
    /// cached_client.clear();
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn clear( &self )
    {
      let mut cache = self.cache.lock().unwrap();
      cache.clear();
    }

    /// Returns the number of cached responses.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "caching") ]
    /// # {
    /// use api_xai::{ CachedClient, Client, Secret, XaiEnvironmentImpl };
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    /// let cached_client = CachedClient::new( client, 100 );
    ///
    /// println!( "Cache size : {}", cached_client.len() );
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn len( &self ) -> usize
    {
      let cache = self.cache.lock().unwrap();
      cache.len()
    }

    /// Returns true if the cache is empty.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "caching") ]
    /// # {
    /// use api_xai::{ CachedClient, Client, Secret, XaiEnvironmentImpl };
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    /// let cached_client = CachedClient::new( client, 100 );
    ///
    /// assert!( cached_client.is_empty() );
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn is_empty( &self ) -> bool
    {
      let cache = self.cache.lock().unwrap();
      cache.is_empty()
    }

    /// Computes a cache key for a request.
    ///
    /// Uses the serialized JSON representation of the request as input
    /// to a hash function. This ensures that any change to the request
    /// produces a different cache key.
    fn compute_cache_key( request : &ChatCompletionRequest ) -> String
    {
      let json = serde_json::to_string( request ).unwrap_or_default();
      let mut hasher = DefaultHasher::new();
      json.hash( &mut hasher );
      format!( "{:x}", hasher.finish() )
    }
  }
}

#[ cfg( feature = "caching" ) ]
crate::mod_interface!
{
  exposed use
  {
    CachedClient,
  };
}
