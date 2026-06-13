//! Metadata management, caching, and thumbnail generation

use super::*;
use std::time::{ SystemTime, Instant };
use std::sync::{ Arc, RwLock };
use core::sync::atomic::{ AtomicU64, AtomicUsize, Ordering };
use std::collections::HashMap;
use std::hash::{ Hash, Hasher };
use std::collections::hash_map::DefaultHasher;
use bytes::{ Bytes, BytesMut };

/// Optimized media cache with LRU eviction and compression
#[ derive( Debug ) ]
pub struct MediaCache
{
  /// Cache configuration
  config : MediaProcessingConfig,
  /// Cached media entries with access timestamps
  entries : Arc< RwLock< HashMap<  String, CachedMediaEntry  > > >,
  /// Total cache size in bytes
  total_size_bytes : AtomicUsize,
  /// Cache access statistics
  stats : Arc< MediaCacheStats >,
}

/// Cached media entry with metadata
#[ derive( Debug, Clone ) ]
struct CachedMediaEntry
{
  /// Original file data (possibly compressed)
  data : Bytes,
  /// Metadata about the cached file
  metadata : CachedMediaMetadata,
  /// Last access timestamp
  last_accessed : SystemTime,
  /// Size in bytes
  size_bytes : usize,
}

/// Metadata for cached media
#[ derive( Debug, Clone ) ]
pub struct CachedMediaMetadata
{
  /// Original MIME type
  pub mime_type : String,
  /// Original file size
  pub original_size : usize,
  /// Whether data is compressed
  pub is_compressed : bool,
  /// Compression ratio achieved
  pub compression_ratio : f64,
  /// File hash for integrity verification
  #[ allow(dead_code) ]
  content_hash : String,
}

/// Cache statistics for performance monitoring
#[ derive( Debug ) ]
pub struct MediaCacheStats
{
  /// Cache hits
  pub hits : AtomicU64,
  /// Cache misses
  pub misses : AtomicU64,
  /// Number of evictions performed
  pub evictions : AtomicU64,
  /// Total bytes compressed
  pub total_compressed_bytes : AtomicU64,
  /// Total compression time in microseconds
  pub total_compression_time_us : AtomicU64,
}

impl Default for MediaCacheStats
{
  fn default() -> Self
  {
    Self {
      hits : AtomicU64::new( 0 ),
      misses : AtomicU64::new( 0 ),
      evictions : AtomicU64::new( 0 ),
      total_compressed_bytes : AtomicU64::new( 0 ),
      total_compression_time_us : AtomicU64::new( 0 ),
    }
  }
}

impl MediaCache
{
  /// Create a new media cache
  #[ inline ]
  #[ must_use ]
  pub fn new( config : MediaProcessingConfig ) -> Self
  {
    Self {
      config,
      entries : Arc::new( RwLock::new( HashMap::new() ) ),
      total_size_bytes : AtomicUsize::new( 0 ),
      stats : Arc::new( MediaCacheStats::default() ),
    }
  }

  /// Get media from cache
  #[ inline ]
  pub fn get( &self, key : &str ) -> Option< ( Bytes, CachedMediaMetadata ) >
  {
    let mut entries = self.entries.write().unwrap();

    if let Some( entry ) = entries.get_mut( key )
    {
      // Check if entry has expired
      let age = SystemTime::now()
        .duration_since( entry.last_accessed )
        .unwrap_or_default()
        .as_secs();

      if age <= self.config.cache_ttl_seconds
      {
        // Update access time
        entry.last_accessed = SystemTime::now();
        self.stats.hits.fetch_add( 1, Ordering::Relaxed );
        return Some( ( entry.data.clone(), entry.metadata.clone() ) );
      }
      // Entry expired, remove it
      let entry = entries.remove( key ).unwrap();
      self.total_size_bytes.fetch_sub( entry.size_bytes, Ordering::Relaxed );
    }

    self.stats.misses.fetch_add( 1, Ordering::Relaxed );
    None
  }

  /// Put media in cache with optional compression
  #[ inline ]
  pub fn put( &self, key : String, data : Bytes, mime_type : String ) -> Result< (), crate::error::Error >
  {
    let original_size = data.len();
    let start_time = Instant::now();

    // Compress data if enabled and beneficial
    let ( final_data, is_compressed, compression_ratio ) = if self.config.enable_compression && self.should_compress( &mime_type )
    {
      match self.compress_data( &data )
      {
        Ok( compressed ) =>
        {
          let ratio = compressed.len() as f64 / original_size as f64;
          if ratio < 0.95  // Only use compression if it saves at least 5%
          {
            ( compressed, true, ratio )
          }
          else
          {
            ( data, false, 1.0 )
          }
        },
        Err( _ ) => ( data, false, 1.0 ),
      }
    } else {
      ( data, false, 1.0 )
    };

    let content_hash = self.calculate_hash( &final_data );
    let entry_size = final_data.len();

    // Check if cache would exceed size limit
    let current_size = self.total_size_bytes.load( Ordering::Relaxed );
    if current_size + entry_size > self.config.max_cache_size_bytes
    {
      self.evict_lru_entries( entry_size );
    }

    let metadata = CachedMediaMetadata {
      mime_type,
      original_size,
      is_compressed,
      compression_ratio,
      content_hash,
    };

    let entry = CachedMediaEntry {
      data : final_data,
      metadata,
      last_accessed : SystemTime::now(),
      size_bytes : entry_size,
    };

    // Insert into cache
    {
      let mut entries = self.entries.write().unwrap();
      entries.insert( key, entry );
    }

    self.total_size_bytes.fetch_add( entry_size, Ordering::Relaxed );

    // Update compression stats
    if is_compressed
    {
      self.stats.total_compressed_bytes.fetch_add( entry_size as u64, Ordering::Relaxed );
      let compression_time_us = start_time.elapsed().as_micros() as u64;
      self.stats.total_compression_time_us.fetch_add( compression_time_us, Ordering::Relaxed );
    }

    Ok( () )
  }

  /// Check if MIME type should be compressed
  #[ inline ]
  fn should_compress( &self, mime_type : &str ) -> bool
  {
    // Don't compress already compressed formats
    !matches!( mime_type,
      "image/jpeg" | "image/jpg" | "image/webp" |
      "video/mp4" | "video/webm" | "video/h264" |
      "audio/mp3" | "audio/aac" | "audio/ogg" |
      "application/zip" | "application/gzip" | "application/brotli"
    )
  }

  /// Compress data using deflate algorithm
  #[ inline ]
  fn compress_data( &self, data : &Bytes ) -> Result< Bytes, std::io::Error >
  {

    // Simple compression simulation (in production, use actual compression library)
    let mut compressed = BytesMut::new();

    // Simulate compression by removing duplicate bytes (very basic)
    let mut last_byte = None;
    let mut count = 0u8;

    for &byte in data.iter()
    {
      if last_byte == Some( byte ) && count < 255
      {
        count += 1;
      } else {
        if let Some( last ) = last_byte
        {
          if count > 0
          {
            compressed.extend_from_slice( &[ 0xFF, count, last ] ); // Compression marker
          } else {
            compressed.extend_from_slice( &[ last ] );
          }
        }
        last_byte = Some( byte );
        count = 0;
      }
    }

    // Write final byte
    if let Some( last ) = last_byte
    {
      if count > 0
      {
        compressed.extend_from_slice( &[ 0xFF, count, last ] );
      } else {
        compressed.extend_from_slice( &[ last ] );
      }
    }

    Ok( compressed.freeze() )
  }

  /// Calculate hash of data for integrity verification
  fn calculate_hash( &self, data : &Bytes ) -> String
  {
    let mut hasher = DefaultHasher::new();
    data.hash( &mut hasher );
    format!( "{:x}", hasher.finish() )
  }

  /// Evict LRU entries to make space
  fn evict_lru_entries( &self, required_space : usize )
  {
    let mut entries = self.entries.write().unwrap();
    let mut entries_by_access : Vec< _ > = entries.iter()
      .map( | ( key, entry ) | ( key.clone(), entry.last_accessed, entry.size_bytes ) )
      .collect();

    // Sort by access time (oldest first)
    entries_by_access.sort_by_key( | ( _, timestamp, _ ) | *timestamp );

    let mut freed_space = 0;
    let mut evicted_count = 0;

    for ( key, _, size ) in entries_by_access
    {
      if freed_space >= required_space
      {
        break;
      }

      entries.remove( &key );
      freed_space += size;
      evicted_count += 1;
    }

    self.total_size_bytes.fetch_sub( freed_space, Ordering::Relaxed );
    self.stats.evictions.fetch_add( evicted_count, Ordering::Relaxed );
  }

  /// Get cache statistics
  #[ inline ]
  #[ must_use ]
  pub fn get_stats( &self ) -> MediaCacheStatsReport
  {
    let hits = self.stats.hits.load( Ordering::Relaxed );
    let misses = self.stats.misses.load( Ordering::Relaxed );
    let total_requests = hits + misses;
    let hit_rate = if total_requests > 0 { hits as f64 / total_requests as f64 * 100.0 } else { 0.0 };

    MediaCacheStatsReport {
      hits,
      misses,
      hit_rate,
      evictions : self.stats.evictions.load( Ordering::Relaxed ),
      total_size_bytes : self.total_size_bytes.load( Ordering::Relaxed ),
      total_compressed_bytes : self.stats.total_compressed_bytes.load( Ordering::Relaxed ),
      avg_compression_time_us :
      {
        let total_time = self.stats.total_compression_time_us.load( Ordering::Relaxed );
        let total_compressed = self.stats.total_compressed_bytes.load( Ordering::Relaxed );
        total_time.checked_div( total_compressed ).unwrap_or( 0 )
      },
    }
  }

  /// Clear all cached entries
  #[ inline ]
  pub fn clear( &self )
  {
    let mut entries = self.entries.write().unwrap();
    entries.clear();
    self.total_size_bytes.store( 0, Ordering::Relaxed );
  }
}

/// Thumbnail generator for creating optimized previews
#[ derive( Debug ) ]
pub struct ThumbnailGenerator
{
  /// Configuration for thumbnail generation
  config : ThumbnailConfig,
}

impl ThumbnailGenerator
{
  /// Create a new thumbnail generator
  pub fn new( config : ThumbnailConfig ) -> Self
  {
    Self { config }
  }

  /// Generate thumbnail for image data
  pub async fn generate_thumbnail( &self, _image_data : &Bytes, mime_type : &str ) -> Result< Bytes, crate::error::Error >
  {
    if !self.config.enabled
    {
      return Err( crate::error::Error::ApiError( "Thumbnail generation disabled".to_string() ) );
    }

    // For this implementation, we'll create a simple placeholder thumbnail
    // In a real implementation, this would use an image processing library
    let thumbnail_data = self.create_placeholder_thumbnail( mime_type );
    Ok( thumbnail_data )
  }

  /// Create a placeholder thumbnail (simplified implementation)
  fn create_placeholder_thumbnail( &self, _mime_type : &str ) -> Bytes
  {
    // Create a simple byte pattern representing a thumbnail
    let mut thumbnail = BytesMut::new();

    // Write a simple header based on desired format
    match self.config.format
    {
      ThumbnailFormat::Jpeg => {
        thumbnail.extend_from_slice( &[ 0xFF, 0xD8, 0xFF, 0xE0 ] ); // JPEG header
      },
      ThumbnailFormat::Png => {
        thumbnail.extend_from_slice( &[ 0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A ] ); // PNG header
      },
      ThumbnailFormat::WebP => {
        thumbnail.extend_from_slice( b"RIFF" );
        thumbnail.extend_from_slice( &[ 0x00, 0x00, 0x00, 0x00 ] ); // Size placeholder
        thumbnail.extend_from_slice( b"WEBP" );
      },
    }

    // Add some data representing the thumbnail
    let data_size = ( self.config.width * self.config.height * 3 ) / 10; // Simplified size estimation
    thumbnail.resize( data_size as usize, 0x80 ); // Fill with gray value

    thumbnail.freeze()
  }
}
