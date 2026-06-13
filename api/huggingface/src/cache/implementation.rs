//! Cache Implementation
//!
//! Provides in-memory caching with TTL, size limits, and statistics.
//!
//! ## Features
//!
//! - TTL-based expiration
//! - Size-limited with LRU eviction
//! - Thread-safe concurrent access
//! - Hit/miss statistics
//!
//! ## Usage
//!
//! ```no_run
//! # use api_huggingface::cache::{Cache, CacheConfig};
//! # use std::time::Duration;
//! # async fn example( ) -> Result< ( ), Box< dyn std::error::Error > > {
//! let cache = Cache::new( CacheConfig {
//!   max_entries : 100,
//!   default_ttl : Some( Duration::from_secs( 60 )),
//! } );
//!
//! cache.insert( "key".to_string( ), "value".to_string( ), None ).await;
//! let value = cache.get( &"key".to_string( )).await;
//! # Ok( ( ))
//! # }
//! ```

use core::time::Duration;
use core::hash::Hash;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Cache entry with TTL
#[ derive( Debug, Clone ) ]
struct CacheEntry< V > 
{
  value : V,
  #[ allow( dead_code ) ] // Reserved for future entry age tracking
  inserted_at : Instant,
  expires_at : Option< Instant >,
  last_accessed : Instant,
}

impl< V > CacheEntry< V > 
{
  /// Create new cache entry
  #[ inline ]
  fn new( value : V, ttl : Option< Duration > ) -> Self 
  {
  let now = Instant::now( );
  Self {
      value,
      inserted_at : now,
      expires_at : ttl.map( |d| now + d ),
      last_accessed : now,
  }
  }

  /// Check if entry is expired
  #[ inline ]
  fn is_expired( &self ) -> bool 
  {
  self.expires_at.is_some_and( |exp| Instant::now( ) >= exp )
  }

  /// Update last accessed time
  #[ inline ]
  fn touch( &mut self ) 
  {
  self.last_accessed = Instant::now( );
  }
}

/// Cache configuration
#[ derive( Debug, Clone ) ]
pub struct CacheConfig 
{
  /// Maximum number of entries
  pub max_entries : usize,
  /// Default TTL for entries ( None = no expiration )
  pub default_ttl : Option< Duration >,
}

impl Default for CacheConfig 
{
  #[ inline ]
  fn default() -> Self 
  {
  Self {
      max_entries : 1000,
      default_ttl : Some( Duration::from_secs( 300 )), // 5 minutes
  }
  }
}

/// Cache statistics
#[ derive( Debug, Clone, Copy, Default ) ]
pub struct CacheStats 
{
  /// Number of cache hits
  pub hits : u64,
  /// Number of cache misses
  pub misses : u64,
  /// Number of entries evicted
  pub evictions : u64,
  /// Current number of entries
  pub entries : usize,
}

impl CacheStats 
{
  /// Calculate hit rate ( 0.0 - 1.0 )
  #[ inline ]
  #[ must_use ]
  pub fn hit_rate( &self ) -> f64 
  {
  let total = self.hits + self.misses;
  if total == 0
  {
      0.0
  } else {
      self.hits as f64 / total as f64
  }
  }

  /// Get total requests
  #[ inline ]
  #[ must_use ]
  pub fn total_requests( &self ) -> u64 
  {
  self.hits + self.misses
  }
}

/// Internal cache state
struct CacheState< K, V > 
{
  entries : HashMap< K, CacheEntry< V > >,
  config : CacheConfig,
  stats : CacheStats,
}

/// In-memory cache with TTL and size limits
#[ derive( Clone ) ]
pub struct Cache< K, V > 
{
  state : Arc< RwLock< CacheState< K, V > > >,
}

impl< K, V > Cache< K, V >
where
  K : Eq + Hash + Clone,
  V : Clone,
{
  /// Create new cache with given configuration
  #[ inline ]
  #[ must_use ]
  pub fn new( config : CacheConfig ) -> Self 
  {
  Self {
      state : Arc::new( RwLock::new( CacheState {
  entries : HashMap::new( ),
  config,
  stats : CacheStats::default( ),
      } )),
  }
  }

  /// Insert value into cache
  ///
  /// If `ttl` is None, uses default TTL from config.
  #[ inline ]
  pub async fn insert( &self, key : K, value : V, ttl : Option< Duration > ) 
  {
  let mut state = self.state.write( ).await;

  // Determine TTL to use
  let effective_ttl = ttl.or( state.config.default_ttl );

  // Check if we need to evict
  if state.entries.len( ) >= state.config.max_entries && !state.entries.contains_key( &key )
  {
      // Evict LRU entry
      let lru_key = state.entries.iter( )
  .min_by_key( |( _, entry )| entry.last_accessed )
  .map( |( k, _ )| k.clone( ));

      if let Some( lru_key ) = lru_key
      {
  state.entries.remove( &lru_key );
  state.stats.evictions += 1;
      }
  }

  // Insert new entry
  let entry = CacheEntry::new( value, effective_ttl );
  state.entries.insert( key, entry );
  state.stats.entries = state.entries.len( );
  }

  /// Get value from cache
  ///
  /// Returns None if key doesn't exist or entry is expired.
  #[ inline ]
  pub async fn get( &self, key : &K ) -> Option< V > 
  {
  let mut state = self.state.write( ).await;

  if let Some( entry ) = state.entries.get_mut( key )
  {
      if entry.is_expired( )
      {
  // Remove expired entry
  state.entries.remove( key );
  state.stats.entries = state.entries.len( );
  state.stats.misses += 1;
  None
      } else {
  // Update access time and clone value
  entry.touch( );
  let value = entry.value.clone( );
  state.stats.hits += 1;
  Some( value )
      }
  } else {
      state.stats.misses += 1;
      None
  }
  }

  /// Check if key exists in cache ( without updating access time )
  #[ inline ]
  pub async fn contains_key( &self, key : &K ) -> bool 
  {
  let state = self.state.read( ).await;
  state.entries.get( key )
      .is_some_and( |entry| !entry.is_expired( ))
  }

  /// Remove entry from cache
  #[ inline ]
  pub async fn remove( &self, key : &K ) -> Option< V > 
  {
  let mut state = self.state.write( ).await;
  let value = state.entries.remove( key ).map( |entry| entry.value );
  state.stats.entries = state.entries.len( );
  value
  }

  /// Clear all entries from cache
  #[ inline ]
  pub async fn clear( &self ) 
  {
  let mut state = self.state.write( ).await;
  state.entries.clear( );
  state.stats.entries = 0;
  }

  /// Remove expired entries
  #[ inline ]
  pub async fn cleanup_expired( &self ) -> usize 
  {
  let mut state = self.state.write( ).await;
  let before = state.entries.len( );

  state.entries.retain( |_, entry| !entry.is_expired( ));

  let removed = before - state.entries.len( );
  state.stats.entries = state.entries.len( );
  removed
  }

  /// Get cache statistics
  #[ inline ]
  pub async fn stats( &self ) -> CacheStats 
  {
  let state = self.state.read( ).await;
  state.stats
  }

  /// Reset statistics
  #[ inline ]
  pub async fn reset_stats( &self ) 
  {
  let mut state = self.state.write( ).await;
  state.stats = CacheStats {
      entries : state.entries.len( ),
      ..Default::default( )
  };
  }

  /// Get current cache size
  #[ inline ]
  pub async fn len( &self ) -> usize 
  {
  let state = self.state.read( ).await;
  state.entries.len( )
  }

  /// Check if cache is empty
  #[ inline ]
  pub async fn is_empty( &self ) -> bool 
  {
  let state = self.state.read( ).await;
  state.entries.is_empty( )
  }

  /// Get cache configuration
  #[ inline ]
  pub async fn config( &self ) -> CacheConfig 
  {
  let state = self.state.read( ).await;
  state.config.clone( )
  }
}

impl< K, V > core::fmt::Debug for Cache< K, V > 
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result 
  {
  f.debug_struct( "Cache" )
      .field( "state", &"< CacheState >" )
      .finish( )
  }
}

/// Cache errors
#[ derive( Debug ) ]
pub enum CacheError 
{
  /// Cache is full and cannot accept new entries
  CacheFull,
  /// Entry not found in cache
  NotFound,
}

impl core::fmt::Display for CacheError 
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result 
  {
  match self
  {
      Self::CacheFull => write!( f, "Cache is full" ),
      Self::NotFound => write!( f, "Entry not found in cache" ),
  }
  }
}

impl std::error::Error for CacheError {}
