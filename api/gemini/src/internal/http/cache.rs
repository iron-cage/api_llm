//! General HTTP request caching with LRU eviction

use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use std::time::{ Duration, Instant };
use std::hash::{ Hash, Hasher };
use std::collections::hash_map::DefaultHasher;
use reqwest::Method;
use serde::{ Serialize, Deserialize };

#[ cfg( feature = "logging" ) ]
use tracing::debug;

/// Configuration for HTTP request cache
#[ derive( Debug, Clone ) ]
pub struct CacheConfig
{
  /// Maximum number of cached responses
  pub max_size : usize,
  /// Time-to-live for cached entries
  pub ttl : Duration,
  /// Whether to collect cache metrics
  pub enable_metrics : bool,
}

impl Default for CacheConfig
{
  fn default() -> Self
  {
    Self {
      max_size : 1000,
      ttl : Duration::from_secs( 300 ), // 5 minutes
      enable_metrics : true,
    }
  }
}

/// Cache key for HTTP requests
#[ derive( Debug, Clone, PartialEq, Eq, Hash ) ]
struct CacheKey
{
  method : String,
  url : String,
  body_hash : u64,
}

impl CacheKey
{
  /// Create a new cache key from request components
  fn new< T : Serialize >( method : &Method, url : &str, body : Option< &T > ) -> Self
  {
    let body_hash = if let Some( body ) = body
    {
      // Hash the serialized body
      let json = serde_json::to_string( body ).unwrap_or_default();
      let mut hasher = DefaultHasher::new();
      json.hash( &mut hasher );
      hasher.finish()
    } else {
      0
    };

    Self {
      method : method.to_string(),
      url : url.to_string(),
      body_hash,
    }
  }
}

/// Cache entry with TTL and LRU tracking
#[ derive( Debug, Clone ) ]
struct CacheEntry
{
  /// Cached response data (JSON string)
  response_json : String,
  /// When this entry was created
  created_at : Instant,
  /// When this entry was last accessed (for LRU)
  last_accessed : Instant,
  /// Time-to-live for this entry
  ttl : Duration,
}

impl CacheEntry
{
  /// Create a new cache entry
  fn new( response_json : String, ttl : Duration ) -> Self
  {
    let now = Instant::now();
    Self {
      response_json,
      created_at : now,
      last_accessed : now,
      ttl,
    }
  }

  /// Check if this entry has expired
  fn is_expired( &self ) -> bool
  {
    self.created_at.elapsed() > self.ttl
  }

  /// Update last accessed time (for LRU tracking)
  fn touch( &mut self )
  {
    self.last_accessed = Instant::now();
  }
}

/// Cache metrics for monitoring
#[ derive( Debug, Clone, Default ) ]
pub struct CacheMetrics
{
  /// Total number of cache hits
  pub hits : u64,
  /// Total number of cache misses
  pub misses : u64,
  /// Total number of evictions due to size limit
  pub evictions : u64,
  /// Total number of expirations due to TTL
  pub expirations : u64,
  /// Current cache size
  pub current_size : usize,
  /// Total requests processed
  pub total_requests : u64,
}

impl CacheMetrics
{
  /// Calculate hit ratio as a percentage
  pub fn hit_ratio( &self ) -> f64
  {
    if self.total_requests == 0
    {
      0.0
    } else {
      ( self.hits as f64 / self.total_requests as f64 ) * 100.0
    }
  }
}

/// General HTTP request cache with LRU eviction
#[ derive( Debug, Clone ) ]
pub struct RequestCache
{
  config : CacheConfig,
  entries : Arc< Mutex< HashMap<  CacheKey, CacheEntry  > > >,
  metrics : Arc< Mutex< CacheMetrics > >,
}

impl RequestCache
{
  /// Create a new request cache with the given configuration
  pub fn new( config : CacheConfig ) -> Self
  {
    Self {
      config,
      entries : Arc::new( Mutex::new( HashMap::new() ) ),
      metrics : Arc::new( Mutex::new( CacheMetrics::default() ) ),
    }
  }

  /// Try to get a cached response for the given request
  pub fn get< T, R >( &self, method : &Method, url : &str, body : Option< &T > ) -> Option< R >
  where
    T: Serialize,
    R: for< 'de > Deserialize< 'de >,
  {
    let key = CacheKey::new( method, url, body );

    let mut entries = self.entries.lock().unwrap();
    let mut metrics = self.metrics.lock().unwrap();

    metrics.total_requests += 1;

    if let Some( entry ) = entries.get_mut( &key )
    {
      // Check if entry has expired
      if entry.is_expired()
      {
        #[ cfg( feature = "logging" ) ]
        debug!( "Cache entry expired for {} {}", method, url );

        entries.remove( &key );
        metrics.misses += 1;
        metrics.expirations += 1;
        metrics.current_size = entries.len();
        return None;
      }

      // Entry is valid, update access time and return
      entry.touch();
      metrics.hits += 1;

      #[ cfg( feature = "logging" ) ]
      debug!( "Cache hit for {} {}", method, url );

      // Deserialize and return
      serde_json ::from_str( &entry.response_json ).ok()
    } else {
      metrics.misses += 1;

      #[ cfg( feature = "logging" ) ]
      debug!( "Cache miss for {} {}", method, url );

      None
    }
  }

  /// Store a response in the cache
  pub fn put< T, R >( &self, method : &Method, url : &str, body : Option< &T >, response : &R )
  where
    T: Serialize,
    R: Serialize,
  {
    let key = CacheKey::new( method, url, body );

    // Serialize the response
    let response_json = match serde_json::to_string( response )
    {
      Ok( json ) => json,
      Err( _e ) => {
        #[ cfg( feature = "logging" ) ]
        debug!( "Failed to serialize response for caching : {}", _e );
        return;
      }
    };

    let mut entries = self.entries.lock().unwrap();
    let mut metrics = self.metrics.lock().unwrap();

    // Check if we need to evict entries (LRU eviction)
    if entries.len() >= self.config.max_size && !entries.contains_key( &key )
    {
      // Find the least recently used entry
      if let Some( lru_key ) = entries.iter()
        .min_by_key( |( _, entry )| entry.last_accessed )
        .map( |( k, _ )| k.clone() )
      {
        #[ cfg( feature = "logging" ) ]
        debug!( "Evicting LRU cache entry : {} {}", lru_key.method, lru_key.url );

        entries.remove( &lru_key );
        metrics.evictions += 1;
      }
    }

    // Insert the new entry
    let entry = CacheEntry::new( response_json, self.config.ttl );
    entries.insert( key, entry );
    metrics.current_size = entries.len();

    #[ cfg( feature = "logging" ) ]
    debug!( "Cached response for {} {} (cache size : {})", method, url, entries.len() );
  }

  /// Clear all cached entries
  pub fn clear( &self )
  {
    let mut entries = self.entries.lock().unwrap();
    let mut metrics = self.metrics.lock().unwrap();

    let _cleared_count = entries.len();
    entries.clear();
    metrics.current_size = 0;

    #[ cfg( feature = "logging" ) ]
    debug!( "Cleared {} cache entries", _cleared_count );
  }

  /// Get current cache metrics
  pub fn get_metrics( &self ) -> CacheMetrics
  {
    self.metrics.lock().unwrap().clone()
  }

  /// Remove expired entries (can be called periodically for cleanup)
  pub fn cleanup_expired( &self ) -> usize
  {
    let mut entries = self.entries.lock().unwrap();
    let mut metrics = self.metrics.lock().unwrap();

    let _initial_size = entries.len();

    // Collect expired keys
    let expired_keys : Vec< CacheKey > = entries
      .iter()
      .filter( |( _, entry )| entry.is_expired() )
      .map( |( key, _ )| key.clone() )
      .collect();

    // Remove expired entries
    for key in &expired_keys
    {
      entries.remove( key );
    }

    let expired_count = expired_keys.len();
    if expired_count > 0
    {
      metrics.expirations += expired_count as u64;
      metrics.current_size = entries.len();

      #[ cfg( feature = "logging" ) ]
      debug!( "Cleaned up {} expired cache entries ({} -> {})", expired_count, _initial_size, entries.len() );
    }

    expired_count
  }
}

/// Execute an HTTP request with caching support
pub async fn execute_with_cache< T, R >
(
  client : &reqwest::Client,
  method : reqwest::Method,
  url : &str,
  api_key : &str,
  body : Option< &T >,
  config : &super::HttpConfig,
  cache : Option< &RequestCache >,
)
-> Result< R, crate::error::Error >
where
  T: Serialize,
  R: Serialize + for< 'de > Deserialize< 'de >,
{
  // Only cache GET requests by default (safest approach)
  let should_cache = cache.is_some() && method == reqwest::Method::GET;

  if should_cache
  {
    if let Some( cache ) = cache
    {
      // Try to get from cache
      if let Some( cached_response ) = cache.get::< T, R >( &method, url, body )
      {
        return Ok( cached_response );
      }
    }
  }

  // Cache miss or caching disabled - execute request
  let response = super::execute( client, method.clone(), url, api_key, body, config ).await?;

  // Store in cache if caching is enabled
  if should_cache
  {
    if let Some( cache ) = cache
    {
      cache.put( &method, url, body, &response );
    }
  }

  Ok( response )
}
