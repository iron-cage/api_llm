//! HTTP Request Cache Tests
//!
//! Unit-style tests for the internal request cache: LRU eviction, TTL expiry,
//! hit/miss metrics, and cleanup behaviour.

use std::time::Duration;

use api_gemini::internal::http::cache::{ CacheConfig, RequestCache };
use reqwest::Method;

#[ test ]
fn test_cache_key_distinguishes_method_and_url()
{
  let cache = RequestCache::new( CacheConfig::default() );

  // Same method + URL → cache hit
  cache.put( &Method::GET, "https://api.example.com/test", None::< &() >, &"v1" );
  let hit : Option< String > = cache.get( &Method::GET, "https://api.example.com/test", None::< &() > );
  assert_eq!( hit, Some( "v1".to_string() ) );

  // Different method → different key → cache miss
  let miss : Option< String > = cache.get( &Method::POST, "https://api.example.com/test", None::< &() > );
  assert!( miss.is_none(), "Different method must not hit the GET cache entry" );
}

#[ test ]
fn test_cache_basic_operations()
{
  let cache = RequestCache::new( CacheConfig::default() );

  // Miss before any put
  let miss : Option< String > = cache.get( &Method::GET, "https://api.example.com/test", None::< &() > );
  assert!( miss.is_none() );

  // Put then get
  cache.put( &Method::GET, "https://api.example.com/test", None::< &() >, &"cached_value" );
  let hit : Option< String > = cache.get( &Method::GET, "https://api.example.com/test", None::< &() > );
  assert_eq!( hit, Some( "cached_value".to_string() ) );

  // Metrics reflect one hit, one miss
  let metrics = cache.get_metrics();
  assert_eq!( metrics.hits, 1 );
  assert_eq!( metrics.misses, 1 );
  assert_eq!( metrics.total_requests, 2 );
}

#[ test ]
fn test_cache_lru_eviction()
{
  let config = CacheConfig { max_size : 2, ttl : Duration::from_secs( 300 ), enable_metrics : true };
  let cache = RequestCache::new( config );

  cache.put( &Method::GET, "https://api.example.com/1", None::< &() >, &"value1" );
  cache.put( &Method::GET, "https://api.example.com/2", None::< &() >, &"value2" );

  // Access entry 1 → it becomes most-recently-used
  let _ : Option< String > = cache.get( &Method::GET, "https://api.example.com/1", None::< &() > );

  // Insert entry 3 → evicts entry 2 (least recently used)
  cache.put( &Method::GET, "https://api.example.com/3", None::< &() >, &"value3" );

  let result1 : Option< String > = cache.get( &Method::GET, "https://api.example.com/1", None::< &() > );
  let result2 : Option< String > = cache.get( &Method::GET, "https://api.example.com/2", None::< &() > );
  let result3 : Option< String > = cache.get( &Method::GET, "https://api.example.com/3", None::< &() > );

  assert_eq!( result1, Some( "value1".to_string() ) );
  assert_eq!( result2, None, "Entry 2 should have been evicted" );
  assert_eq!( result3, Some( "value3".to_string() ) );

  assert_eq!( cache.get_metrics().evictions, 1 );
}

#[ test ]
fn test_cache_expiration()
{
  let config = CacheConfig { max_size : 100, ttl : Duration::from_millis( 100 ), enable_metrics : true };
  let cache = RequestCache::new( config );

  cache.put( &Method::GET, "https://api.example.com/ttl-test", None::< &() >, &"value" );

  let pre : Option< String > = cache.get( &Method::GET, "https://api.example.com/ttl-test", None::< &() > );
  assert_eq!( pre, Some( "value".to_string() ) );

  std::thread::sleep( Duration::from_millis( 150 ) );

  let post : Option< String > = cache.get( &Method::GET, "https://api.example.com/ttl-test", None::< &() > );
  assert_eq!( post, None, "Entry must expire after TTL" );

  assert_eq!( cache.get_metrics().expirations, 1 );
}

#[ test ]
fn test_cache_cleanup()
{
  let config = CacheConfig { max_size : 100, ttl : Duration::from_millis( 100 ), enable_metrics : true };
  let cache = RequestCache::new( config );

  cache.put( &Method::GET, "https://api.example.com/1", None::< &() >, &"v1" );
  cache.put( &Method::GET, "https://api.example.com/2", None::< &() >, &"v2" );
  cache.put( &Method::GET, "https://api.example.com/3", None::< &() >, &"v3" );

  std::thread::sleep( Duration::from_millis( 150 ) );

  let expired_count = cache.cleanup_expired();
  assert_eq!( expired_count, 3 );
  assert_eq!( cache.get_metrics().current_size, 0 );
}

#[ test ]
fn test_cache_clear()
{
  let cache = RequestCache::new( CacheConfig::default() );

  cache.put( &Method::GET, "https://api.example.com/1", None::< &() >, &"v1" );
  cache.put( &Method::GET, "https://api.example.com/2", None::< &() >, &"v2" );

  cache.clear();

  assert_eq!( cache.get_metrics().current_size, 0 );
  let result : Option< String > = cache.get( &Method::GET, "https://api.example.com/1", None::< &() > );
  assert_eq!( result, None );
}
