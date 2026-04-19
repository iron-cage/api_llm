//! Response Cache Module
//!
//! This module provides intelligent response caching with TTL (Time To Live) functionality
//! for `OpenAI` API responses. Following the "Thin Client, Rich API" principle, this module
//! offers configurable caching strategies without automatic behaviors.
//!
//! This module is feature-gated behind the `caching` feature flag.

use mod_interface::mod_interface;

#[ cfg( feature = "caching" ) ]
mod private
{
  use crate::
  {
    environment ::{ EnvironmentInterface, OpenaiEnvironment },
    error ::{ OpenAIError, Result },
  };
  use std::
  {
    collections ::HashMap,
    sync ::Arc,
  };
  use core::
  {
    time ::Duration,
    hash ::Hash,
  };
  use std::time::Instant;
  use tokio::sync::RwLock;
  use serde::{ Serialize, Deserialize };
  use sha2::{ Sha256, Digest };

  /// Configuration for response caching behavior
  #[ derive( Debug, Clone ) ]
  pub struct CacheConfig
  {
    /// Maximum number of cached responses
    pub max_entries : usize,
    /// Default TTL for cached responses
    pub default_ttl : Duration,
    /// Maximum size of cached response data in bytes
    pub max_response_size : usize,
    /// Whether to enable cache compression
    pub enable_compression : bool,
    /// Whether to cache error responses
    pub cache_errors : bool,
    /// Cleanup interval for expired entries
    pub cleanup_interval : Duration,
  }

  impl Default for CacheConfig
  {
    #[ inline ]
    fn default() -> Self
    {
      Self
      {
        max_entries : 1000,
        default_ttl : Duration::from_secs( 300 ), // 5 minutes
        max_response_size : 1024 * 1024, // 1MB
        enable_compression : true,
        cache_errors : false,
        cleanup_interval : Duration::from_secs( 60 ), // 1 minute
      }
    }
  }

  /// Cached response entry with metadata
  #[ derive( Debug, Clone ) ]
  pub struct CacheEntry
  {
    /// Cached response data
    pub data : Vec< u8 >,
    /// When this entry was created
    pub created_at : Instant,
    /// TTL for this specific entry
    pub ttl : Duration,
    /// Size of the cached data in bytes
    pub size_bytes : usize,
    /// Number of times this entry has been accessed
    pub hit_count : u64,
    /// Request method that generated this cache entry
    pub method : String,
    /// Request path that generated this cache entry
    pub path : String,
  }

  impl CacheEntry
  {
    /// Check if this cache entry has expired
    #[ inline ]
    #[ must_use ]
    pub fn is_expired( &self ) -> bool
    {
      self.created_at.elapsed() > self.ttl
    }

    /// Record a cache hit
    #[ inline ]
    pub fn record_hit( &mut self )
    {
      self.hit_count += 1;
    }
  }

  /// Cache key generation and management
  #[ derive( Debug, Clone, PartialEq, Eq, Hash ) ]
  pub struct CacheKey
  {
    /// Request method (GET, POST, etc.)
    pub method : String,
    /// Request path
    pub path : String,
    /// Hash of request body (if any)
    pub body_hash : Option< String >,
    /// Hash of query parameters
    pub query_hash : Option< String >,
  }

  impl CacheKey
  {
    /// Create a new cache key from request components
    #[ inline ]
    #[ must_use ]
    pub fn new( method : &str, path : &str, body : Option< &[u8] >, query : Option< &str > ) -> Self
    {
      let body_hash = body.map( Self::hash_bytes );
      let query_hash = query.map( Self::hash_string );

      Self
      {
        method : method.to_uppercase(),
        path : path.to_string(),
        body_hash,
        query_hash,
      }
    }

    /// Generate a string representation for storage
    #[ inline ]
    #[ must_use ]
    pub fn to_cache_key( &self ) -> String
    {
      let mut hasher = Sha256::new();
      hasher.update( &self.method );
      hasher.update( &self.path );

      if let Some( ref body_hash ) = self.body_hash
      {
        hasher.update( body_hash );
      }

      if let Some( ref query_hash ) = self.query_hash
      {
        hasher.update( query_hash );
      }

      {
        use ::core::fmt::Write as _;
        hasher.finalize().iter().fold(
          String::with_capacity( 64 ),
          |mut s, b| { let _ = write!( s, "{b:02x}" ); s },
        )
      }
    }

    /// Hash bytes using SHA256
    fn hash_bytes( data : &[u8] ) -> String
    {
      let mut hasher = Sha256::new();
      hasher.update( data );
      {
        use ::core::fmt::Write as _;
        hasher.finalize().iter().fold(
          String::with_capacity( 64 ),
          |mut s, b| { let _ = write!( s, "{b:02x}" ); s },
        )
      }
    }

    /// Hash string using SHA256
    fn hash_string( data : &str ) -> String
    {
      Self::hash_bytes( data.as_bytes() )
    }
  }

  impl core::fmt::Display for CacheKey
  {
    #[ inline ]
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      write!( f, "{}", self.to_cache_key() )
    }
  }

  /// Cache statistics for monitoring and analysis
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct CacheStatistics
  {
    /// Total number of cache requests
    pub total_requests : u64,
    /// Number of cache hits
    pub cache_hits : u64,
    /// Number of cache misses
    pub cache_misses : u64,
    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio : f64,
    /// Current number of cached entries
    pub current_entries : usize,
    /// Total size of cached data in bytes
    pub total_cached_bytes : usize,
    /// Average TTL of cached entries
    pub average_ttl_seconds : f64,
    /// Number of expired entries cleaned up
    pub expired_entries_cleaned : u64,
    /// Average response size
    pub average_response_size : f64,
  }

  /// Advanced response cache with TTL and intelligent management
  #[ derive( Debug ) ]
  pub struct ResponseCache
  {
    /// Cache storage
    cache : Arc< RwLock< HashMap<  String, CacheEntry  > > >,
    /// Cache configuration
    config : CacheConfig,
    /// Cache statistics
    stats : Arc< RwLock< CacheStatistics > >,
    /// Background cleanup task handle
    cleanup_handle : Option< tokio::task::JoinHandle< () > >,
  }

  impl ResponseCache
  {
    /// Create a new response cache with default configuration
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::with_config( CacheConfig::default() )
    }

    /// Create a new response cache with custom configuration
    #[ inline ]
    #[ must_use ]
    pub fn with_config( config : CacheConfig ) -> Self
    {
      let cache = Arc::new( RwLock::new( HashMap::new() ) );
      let stats = Arc::new( RwLock::new( CacheStatistics
      {
        total_requests : 0,
        cache_hits : 0,
        cache_misses : 0,
        hit_ratio : 0.0,
        current_entries : 0,
        total_cached_bytes : 0,
        average_ttl_seconds : config.default_ttl.as_secs_f64(),
        expired_entries_cleaned : 0,
        average_response_size : 0.0,
      } ) );

      let mut instance = Self
      {
        cache,
        config,
        stats,
        cleanup_handle : None,
      };

      // Start background cleanup if cleanup interval is configured
      if instance.config.cleanup_interval > Duration::ZERO
      {
        instance.start_cleanup_task();
      }

      instance
    }

    /// Get cached response if available and not expired
    #[ inline ]
    pub async fn get( &self, key : &CacheKey ) -> Option< Vec< u8 > >
    {
      let key_str = key.to_cache_key();
      let mut cache = self.cache.write().await;
      let mut stats = self.stats.write().await;

      stats.total_requests += 1;

      if let Some( entry ) = cache.get_mut( &key_str )
      {
        if !entry.is_expired()
        {
          entry.record_hit();
          stats.cache_hits += 1;
          stats.hit_ratio = stats.cache_hits as f64 / stats.total_requests as f64;
          return Some( entry.data.clone() );
        }

        // Remove expired entry
        cache.remove( &key_str );
        stats.current_entries = cache.len();
        stats.expired_entries_cleaned += 1;
      }

      stats.cache_misses += 1;
      stats.hit_ratio = stats.cache_hits as f64 / stats.total_requests as f64;
      None
    }

    /// Store response in cache with TTL
    ///
    /// # Errors
    ///
    /// Returns an error if the response data is too large for caching.
    #[ inline ]
    pub async fn put( &self, key : &CacheKey, data : Vec< u8 >, ttl : Option< Duration > ) -> Result< () >
    {
      let data_size = data.len();

      // Check size limits
      if data_size > self.config.max_response_size
      {
        return Err( OpenAIError::Internal( format!(
          "Response too large for caching : {} bytes (max : {})",
          data_size,
          self.config.max_response_size
        ) ).into() );
      }

      let key_str = key.to_cache_key();
      let ttl = ttl.unwrap_or( self.config.default_ttl );

      let entry = CacheEntry
      {
        data,
        created_at : Instant::now(),
        ttl,
        size_bytes : data_size,
        hit_count : 0,
        method : key.method.clone(),
        path : key.path.clone(),
      };

      let mut cache = self.cache.write().await;
      let mut stats = self.stats.write().await;

      // Check if we need to evict entries
      if cache.len() >= self.config.max_entries
      {
        Self::evict_oldest_entry( &mut cache, &mut stats );
      }

      cache.insert( key_str, entry );
      stats.current_entries = cache.len();
      stats.total_cached_bytes += data_size;

      // Update average response size
      if stats.total_requests > 0
      {
        stats.average_response_size = stats.total_cached_bytes as f64 / stats.current_entries as f64;
      }

      Ok( () )
    }

    /// Clear all cached entries
    #[ inline ]
    pub async fn clear( &self )
    {
      let mut cache = self.cache.write().await;
      let mut stats = self.stats.write().await;

      cache.clear();
      stats.current_entries = 0;
      stats.total_cached_bytes = 0;
    }

    /// Get current cache statistics
    #[ inline ]
    pub async fn get_statistics( &self ) -> CacheStatistics
    {
      let stats = self.stats.read().await;
      stats.clone()
    }

    /// Manually trigger cleanup of expired entries
    #[ inline ]
    pub async fn cleanup_expired( &self ) -> usize
    {
      let mut cache = self.cache.write().await;
      let mut stats = self.stats.write().await;

      let initial_count = cache.len();
      cache.retain( | _, entry | !entry.is_expired() );
      let final_count = cache.len();
      let cleaned_count = initial_count - final_count;

      stats.current_entries = final_count;
      stats.expired_entries_cleaned += cleaned_count as u64;

      // Recalculate total cached bytes
      stats.total_cached_bytes = cache.values().map( | e | e.size_bytes ).sum();

      cleaned_count
    }

    /// Start background cleanup task
    fn start_cleanup_task( &mut self )
    {
      let cache = Arc::clone( &self.cache );
      let stats = Arc::clone( &self.stats );
      let cleanup_interval = self.config.cleanup_interval;

      let handle = tokio::spawn( async move
      {
        let mut interval = tokio::time::interval( cleanup_interval );
        loop
        {
          interval.tick().await;

          // Cleanup expired entries
          let mut cache_guard = cache.write().await;
          let mut stats_guard = stats.write().await;

          let initial_count = cache_guard.len();
          cache_guard.retain( | _, entry | !entry.is_expired() );
          let final_count = cache_guard.len();
          let cleaned_count = initial_count - final_count;

          if cleaned_count > 0
          {
            stats_guard.current_entries = final_count;
            stats_guard.expired_entries_cleaned += cleaned_count as u64;
            stats_guard.total_cached_bytes = cache_guard.values().map( | e | e.size_bytes ).sum();
          }
        }
      } );

      self.cleanup_handle = Some( handle );
    }

    /// Evict the oldest entry to make room for new ones
    fn evict_oldest_entry( cache : &mut HashMap<  String, CacheEntry  >, stats : &mut CacheStatistics )
    {
      if let Some( ( oldest_key, oldest_entry ) ) = cache.iter()
        .min_by_key( | ( _, entry ) | entry.created_at )
        .map( | ( k, v ) | ( k.clone(), v.clone() ) )
      {
        cache.remove( &oldest_key );
        stats.total_cached_bytes = stats.total_cached_bytes.saturating_sub( oldest_entry.size_bytes );
      }
    }
  }

  impl Drop for ResponseCache
  {
    #[ inline ]
    fn drop( &mut self )
    {
      if let Some( handle ) = self.cleanup_handle.take()
      {
        handle.abort();
      }
    }
  }

  /// Cache-aware HTTP client wrapper
  #[ derive( Debug ) ]
  pub struct CachedClient< E >
  where
    E: OpenaiEnvironment + EnvironmentInterface + Send + Sync + 'static,
  {
    /// Base client for actual HTTP requests
    client : crate::client::Client< E >,
    /// Response cache instance
    cache : ResponseCache,
    /// Cache configuration
    config : CacheConfig,
  }

  impl< E > CachedClient< E >
  where
    E: OpenaiEnvironment + EnvironmentInterface + Send + Sync + 'static,
  {
    /// Create a new cached client with default cache configuration
    #[ inline ]
    pub fn new( client : crate::client::Client< E > ) -> Self
    {
      Self::with_cache_config( client, CacheConfig::default() )
    }

    /// Create a new cached client with custom cache configuration
    #[ inline ]
    pub fn with_cache_config( client : crate::client::Client< E >, config : CacheConfig ) -> Self
    {
      let cache = ResponseCache::with_config( config.clone() );
      Self
      {
        client,
        cache,
        config,
      }
    }

    /// Execute GET request with caching
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or if response deserialization fails.
    #[ inline ]
    pub async fn get_cached< T >( &self, path : &str, ttl : Option< Duration > ) -> Result< T >
    where
      T: serde::de::DeserializeOwned + serde::Serialize,
    {
      let cache_key = CacheKey::new( "GET", path, None, None );

      // Try cache first
      if let Some( cached_data ) = self.cache.get( &cache_key ).await
      {
        let result : T = serde_json::from_slice( &cached_data )
          .map_err( | e | OpenAIError::Internal( format!( "Failed to deserialize cached response : {e}" ) ) )?;
        return Ok( result );
      }

      // Cache miss - make actual request
      let response : T = self.client.get( path ).await?;

      // Cache the response
      if let Ok( serialized ) = serde_json::to_vec( &response )
      {
        let _ = self.cache.put( &cache_key, serialized, ttl ).await;
      }

      Ok( response )
    }

    /// Execute POST request with optional caching
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails, the request fails, or response deserialization fails.
    #[ inline ]
    pub async fn post_cached< I, O >( &self, path : &str, body : &I, ttl : Option< Duration > ) -> Result< O >
    where
      I: serde::Serialize + Send + Sync,
      O: serde::de::DeserializeOwned + serde::Serialize,
    {
      let body_bytes = serde_json::to_vec( body )
        .map_err( | e | OpenAIError::Internal( format!( "Failed to serialize request body : {e}" ) ) )?;

      let cache_key = CacheKey::new( "POST", path, Some( &body_bytes ), None );

      // Try cache first (only for cacheable POST requests)
      if let Some( cached_data ) = self.cache.get( &cache_key ).await
      {
        let result : O = serde_json::from_slice( &cached_data )
          .map_err( | e | OpenAIError::Internal( format!( "Failed to deserialize cached response : {e}" ) ) )?;
        return Ok( result );
      }

      // Cache miss - make actual request
      let response : O = self.client.post( path, body ).await?;

      // Cache the response if TTL is specified
      if ttl.is_some()
      {
        if let Ok( serialized ) = serde_json::to_vec( &response )
        {
          let _ = self.cache.put( &cache_key, serialized, ttl ).await;
        }
      }

      Ok( response )
    }

    /// Get cache statistics
    #[ inline ]
    pub async fn get_cache_statistics( &self ) -> CacheStatistics
    {
      self.cache.get_statistics().await
    }

    /// Clear the cache
    #[ inline ]
    pub async fn clear_cache( &self )
    {
      self.cache.clear().await;
    }

    /// Access the underlying client
    #[ inline ]
    pub fn client( &self ) -> &crate::client::Client< E >
    {
      &self.client
    }

    /// Access the cache configuration
    #[ inline ]
    pub fn cache_config( &self ) -> &CacheConfig
    {
      &self.config
    }
  }

  impl Default for ResponseCache
  {
    #[ inline ]
    fn default() -> Self
    {
      Self::new()
    }
  }

  #[ cfg( test ) ]
  mod tests
  {
    use super::*;

    #[ test ]
    fn test_cache_key_generation()
    {
      let key1 = CacheKey::new( "GET", "/test", None, None );
      let key2 = CacheKey::new( "GET", "/test", None, None );
      let key3 = CacheKey::new( "POST", "/test", None, None );

      assert_eq!( key1.to_cache_key(), key2.to_cache_key() );
      assert_ne!( key1.to_cache_key(), key3.to_cache_key() );
    }

    #[ test ]
    fn test_cache_entry_expiration()
    {
      let mut entry = CacheEntry
      {
        data : vec![ 1, 2, 3 ],
        created_at : Instant::now().checked_sub( Duration::from_secs( 10 ) ).unwrap(),
        ttl : Duration::from_secs( 5 ),
        size_bytes : 3,
        hit_count : 0,
        method : "GET".to_string(),
        path : "/test".to_string(),
      };

      assert!( entry.is_expired() );

      entry.created_at = Instant::now();
      assert!( !entry.is_expired() );
    }

    #[ tokio::test ]
    async fn test_cache_basic_operations()
    {
      let cache = ResponseCache::new();
      let key = CacheKey::new( "GET", "/test", None, None );
      let data = vec![ 1, 2, 3, 4, 5 ];

      // Test cache miss
      assert!( cache.get( &key ).await.is_none() );

      // Test cache put and hit
      cache.put( &key, data.clone(), None ).await.unwrap();
      let cached = cache.get( &key ).await.unwrap();
      assert_eq!( cached, data );

      // Test statistics
      let stats = cache.get_statistics().await;
      assert_eq!( stats.cache_hits, 1 );
      assert_eq!( stats.cache_misses, 1 );
      assert_eq!( stats.current_entries, 1 );
    }

    #[ tokio::test ]
    async fn test_cache_ttl_expiration()
    {
      let cache = ResponseCache::new();
      let key = CacheKey::new( "GET", "/test", None, None );
      let data = vec![ 1, 2, 3 ];

      // Put with very short TTL
      cache.put( &key, data, Some( Duration::from_millis( 1 ) ) ).await.unwrap();

      // Wait for expiration
      tokio ::time::sleep( Duration::from_millis( 10 ) ).await;

      // Should be cache miss due to expiration
      assert!( cache.get( &key ).await.is_none() );
    }
  }
}

mod_interface!
{
  orphan use private::
  {
    CacheConfig,
    CacheEntry,
    CacheKey,
    CacheStatistics,
    ResponseCache,
    CachedClient,
  };
}