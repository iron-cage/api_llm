//! Configuration change propagation and distributed synchronization
//!
//! This module provides distributed configuration synchronization, caching,
//! metrics tracking, and change propagation across multiple instances.

use super::{ DynamicConfig, versioning::ConfigChangeEvent };
use core::time::Duration;
use std::time::{ SystemTime, Instant };
use core::sync::atomic::{ AtomicU64, AtomicUsize, Ordering };
use std::sync::{ Arc, Mutex, RwLock };
use std::collections::{ HashMap, BTreeMap };
use tokio::sync::{ RwLock as AsyncRwLock, broadcast };

/// Configuration management metrics for monitoring and optimization
#[ derive( Debug ) ]
pub struct ConfigMetrics
{
  /// Total number of configuration updates
  pub total_updates : AtomicU64,
  /// Number of configuration validation cache hits
  pub validation_cache_hits : AtomicU64,
  /// Number of configuration validation cache misses
  pub validation_cache_misses : AtomicU64,
  /// Total number of change events sent
  pub change_events_sent : AtomicU64,
  /// Number of rollback operations performed
  pub rollback_operations : AtomicU64,
  /// Number of history entries currently stored
  pub history_entries : AtomicUsize,
  /// Total memory used by configuration history (bytes)
  pub history_memory_bytes : AtomicUsize,
  /// Average configuration update processing time (microseconds)
  pub avg_update_time_us : AtomicU64,
  /// Number of failed configuration updates
  pub failed_updates : AtomicU64,
}

impl Default for ConfigMetrics
{
  fn default() -> Self
  {
    Self {
      total_updates : AtomicU64::new( 0 ),
      validation_cache_hits : AtomicU64::new( 0 ),
      validation_cache_misses : AtomicU64::new( 0 ),
      change_events_sent : AtomicU64::new( 0 ),
      rollback_operations : AtomicU64::new( 0 ),
      history_entries : AtomicUsize::new( 0 ),
      history_memory_bytes : AtomicUsize::new( 0 ),
      avg_update_time_us : AtomicU64::new( 0 ),
      failed_updates : AtomicU64::new( 0 ),
    }
  }
}

impl ConfigMetrics
{
  /// Record a successful configuration update with timing
  pub fn record_update( &self, duration_us : u64 )
  {
    self.total_updates.fetch_add( 1, Ordering::Relaxed );
    self.update_avg_time( duration_us );
  }

  /// Record a failed configuration update
  pub fn record_failed_update( &self )
  {
    self.failed_updates.fetch_add( 1, Ordering::Relaxed );
  }

  /// Record a validation cache hit
  pub fn record_cache_hit( &self )
  {
    self.validation_cache_hits.fetch_add( 1, Ordering::Relaxed );
  }

  /// Record a validation cache miss
  pub fn record_cache_miss( &self )
  {
    self.validation_cache_misses.fetch_add( 1, Ordering::Relaxed );
  }

  /// Record a change event being sent
  pub fn record_change_event( &self )
  {
    self.change_events_sent.fetch_add( 1, Ordering::Relaxed );
  }

  /// Record a rollback operation
  pub fn record_rollback( &self )
  {
    self.rollback_operations.fetch_add( 1, Ordering::Relaxed );
  }

  /// Update history entry count and memory usage
  pub fn update_history_stats( &self, entry_count : usize, total_bytes : usize )
  {
    self.history_entries.store( entry_count, Ordering::Relaxed );
    self.history_memory_bytes.store( total_bytes, Ordering::Relaxed );
  }

  /// Update the running average for update time
  fn update_avg_time( &self, new_time_us : u64 )
  {
    // Simple exponential moving average : new_avg = 0.9 * old_avg + 0.1 * new_value
    let current_avg = self.avg_update_time_us.load( Ordering::Relaxed );
    let new_avg = ( ( current_avg as f64 * 0.9 ) + ( new_time_us as f64 * 0.1 ) ) as u64;
    self.avg_update_time_us.store( new_avg, Ordering::Relaxed );
  }

  /// Get cache hit ratio as a percentage (0-100)
  pub fn cache_hit_ratio( &self ) -> f64
  {
    let hits = self.validation_cache_hits.load( Ordering::Relaxed );
    let misses = self.validation_cache_misses.load( Ordering::Relaxed );
    let total = hits + misses;

    if total == 0
    {
      0.0
    } else {
      ( hits as f64 / total as f64 ) * 100.0
    }
  }

  /// Get current error rate as a percentage (0-100)
  pub fn error_rate( &self ) -> f64
  {
    let total = self.total_updates.load( Ordering::Relaxed );
    let failed = self.failed_updates.load( Ordering::Relaxed );

    if total == 0
    {
      0.0
    } else {
      ( failed as f64 / total as f64 ) * 100.0
    }
  }

  /// Get memory efficiency ratio (compressed vs uncompressed)
  pub fn memory_efficiency( &self ) -> f64
  {
    let total_memory = self.history_memory_bytes.load( Ordering::Relaxed );
    if total_memory == 0
    {
      return 100.0; // Perfect efficiency when no memory is used
    }

    // In a real implementation, we'd track compressed vs uncompressed separately
    // For now, estimate based on entry count
    let entries = self.history_entries.load( Ordering::Relaxed );
    if entries <= 1
    {
      return 100.0;
    }

    // Estimate that delta compression saves about 60-80% for typical configs
    let estimated_uncompressed = ( total_memory as f64 * 1.7 ).round() as usize;
    ( total_memory as f64 / estimated_uncompressed as f64 ) * 100.0
  }

  /// Generate a comprehensive metrics report
  pub fn generate_report( &self ) -> ConfigMetricsReport
  {
    ConfigMetricsReport {
      total_updates : self.total_updates.load( Ordering::Relaxed ),
      failed_updates : self.failed_updates.load( Ordering::Relaxed ),
      error_rate : self.error_rate(),
      avg_update_time_us : self.avg_update_time_us.load( Ordering::Relaxed ),
      cache_hit_ratio : self.cache_hit_ratio(),
      validation_cache_hits : self.validation_cache_hits.load( Ordering::Relaxed ),
      validation_cache_misses : self.validation_cache_misses.load( Ordering::Relaxed ),
      change_events_sent : self.change_events_sent.load( Ordering::Relaxed ),
      rollback_operations : self.rollback_operations.load( Ordering::Relaxed ),
      history_entries : self.history_entries.load( Ordering::Relaxed ),
      history_memory_bytes : self.history_memory_bytes.load( Ordering::Relaxed ),
      memory_efficiency : self.memory_efficiency(),
      timestamp : SystemTime::now(),
    }
  }

  /// Reset all metrics (useful for testing or periodic resets)
  pub fn reset( &self )
  {
    self.total_updates.store( 0, Ordering::Relaxed );
    self.validation_cache_hits.store( 0, Ordering::Relaxed );
    self.validation_cache_misses.store( 0, Ordering::Relaxed );
    self.change_events_sent.store( 0, Ordering::Relaxed );
    self.rollback_operations.store( 0, Ordering::Relaxed );
    self.avg_update_time_us.store( 0, Ordering::Relaxed );
    self.failed_updates.store( 0, Ordering::Relaxed );
    // Note : history metrics are not reset as they represent current state
  }

  /// Export metrics in Prometheus format for monitoring integration
  pub fn to_prometheus_format( &self, instance_name : &str ) -> String
  {
    let report = self.generate_report();
    format!(
      "# HELP config_total_updates Total number of configuration updates\n\
       # TYPE config_total_updates counter\n\
       config_total_updates{{instance=\"{}\"}} {}\n\
       \n\
       # HELP config_failed_updates Total number of failed configuration updates\n\
       # TYPE config_failed_updates counter\n\
       config_failed_updates{{instance=\"{}\"}} {}\n\
       \n\
       # HELP config_error_rate Error rate as percentage\n\
       # TYPE config_error_rate gauge\n\
       config_error_rate{{instance=\"{}\"}} {:.2}\n\
       \n\
       # HELP config_avg_update_time_us Average update time in microseconds\n\
       # TYPE config_avg_update_time_us gauge\n\
       config_avg_update_time_us{{instance=\"{}\"}} {}\n\
       \n\
       # HELP config_cache_hit_ratio Cache hit ratio as percentage\n\
       # TYPE config_cache_hit_ratio gauge\n\
       config_cache_hit_ratio{{instance=\"{}\"}} {:.2}\n\
       \n\
       # HELP config_history_entries Number of history entries\n\
       # TYPE config_history_entries gauge\n\
       config_history_entries{{instance=\"{}\"}} {}\n\
       \n\
       # HELP config_history_memory_bytes Memory used by history in bytes\n\
       # TYPE config_history_memory_bytes gauge\n\
       config_history_memory_bytes{{instance=\"{}\"}} {}\n\
       \n\
       # HELP config_memory_efficiency Memory efficiency as percentage\n\
       # TYPE config_memory_efficiency gauge\n\
       config_memory_efficiency{{instance=\"{}\"}} {:.2}\n\
       \n\
       # HELP config_rollback_operations Total rollback operations\n\
       # TYPE config_rollback_operations counter\n\
       config_rollback_operations{{instance=\"{}\"}} {}\n",
      instance_name, report.total_updates,
      instance_name, report.failed_updates,
      instance_name, report.error_rate,
      instance_name, report.avg_update_time_us,
      instance_name, report.cache_hit_ratio,
      instance_name, report.history_entries,
      instance_name, report.history_memory_bytes,
      instance_name, report.memory_efficiency,
      instance_name, report.rollback_operations
    )
  }

  /// Check if metrics indicate any performance issues
  pub fn health_check( &self ) -> ConfigHealthStatus
  {
    let report = self.generate_report();
    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    // Check error rate
    if report.error_rate > 10.0
    {
      issues.push( format!( "High error rate : {:.1}%", report.error_rate ) );
    } else if report.error_rate > 5.0
    {
      warnings.push( format!( "Elevated error rate : {:.1}%", report.error_rate ) );
    }

    // Check cache performance
    if report.cache_hit_ratio < 50.0 && report.validation_cache_hits + report.validation_cache_misses > 10
    {
      issues.push( format!( "Low cache hit ratio : {:.1}%", report.cache_hit_ratio ) );
    } else if report.cache_hit_ratio < 80.0 && report.validation_cache_hits + report.validation_cache_misses > 10
    {
      warnings.push( format!( "Suboptimal cache hit ratio : {:.1}%", report.cache_hit_ratio ) );
    }

    // Check update performance
    if report.avg_update_time_us > 5000  // 5ms
    {
      issues.push( format!( "Slow updates : {}μs average", report.avg_update_time_us ) );
    }
    else if report.avg_update_time_us > 2000  // 2ms
    {
      warnings.push( format!( "Slow updates : {}μs average", report.avg_update_time_us ) );
    }

    // Check memory usage
    if report.history_memory_bytes > 10 * 1024 * 1024  // 10MB
    {
      warnings.push( format!( "High memory usage : {} bytes", report.history_memory_bytes ) );
    }

    // Check excessive rollbacks
    if report.rollback_operations > report.total_updates / 4
    {
      warnings.push( format!( "High rollback rate : {} rollbacks vs {} updates", report.rollback_operations, report.total_updates ) );
    }

    if !issues.is_empty()
    {
      ConfigHealthStatus::Unhealthy { issues, warnings }
    } else if !warnings.is_empty()
    {
      ConfigHealthStatus::Degraded { warnings }
    } else {
      ConfigHealthStatus::Healthy
    }
  }
}

/// Comprehensive metrics report for monitoring and analysis
#[ derive( Debug, Clone ) ]
pub struct ConfigMetricsReport
{
  /// Total number of configuration updates
  pub total_updates : u64,
  /// Number of failed configuration updates
  pub failed_updates : u64,
  /// Error rate as percentage (0-100)
  pub error_rate : f64,
  /// Average configuration update processing time (microseconds)
  pub avg_update_time_us : u64,
  /// Cache hit ratio as percentage (0-100)
  pub cache_hit_ratio : f64,
  /// Number of validation cache hits
  pub validation_cache_hits : u64,
  /// Number of validation cache misses
  pub validation_cache_misses : u64,
  /// Total number of change events sent
  pub change_events_sent : u64,
  /// Number of rollback operations performed
  pub rollback_operations : u64,
  /// Number of history entries currently stored
  pub history_entries : usize,
  /// Total memory used by configuration history (bytes)
  pub history_memory_bytes : usize,
  /// Memory efficiency percentage (0-100)
  pub memory_efficiency : f64,
  /// Timestamp when report was generated
  pub timestamp : SystemTime,
}

/// Health status of the configuration management system
#[ derive( Debug, Clone ) ]
pub enum ConfigHealthStatus
{
  /// System is healthy with no issues
  Healthy,
  /// System is functional but has performance warnings
  Degraded {
    /// Performance warnings that should be addressed
    warnings : Vec< String >
  },
  /// System has critical issues that need attention
  Unhealthy {
    /// Critical issues that must be addressed
    issues : Vec< String >,
    /// Additional performance warnings
    warnings : Vec< String >
  },
}

impl ConfigHealthStatus
{
  /// Check if the system is considered healthy
  pub fn is_healthy( &self ) -> bool
  {
    matches!( self, ConfigHealthStatus::Healthy )
  }

  /// Get all issues and warnings as a single list
  pub fn get_all_messages( &self ) -> Vec< String >
  {
    match self
    {
      ConfigHealthStatus::Healthy => Vec::new(),
      ConfigHealthStatus::Degraded { warnings } => warnings.clone(),
      ConfigHealthStatus::Unhealthy { issues, warnings } => {
        let mut messages = issues.clone();
        messages.extend( warnings.clone() );
        messages
      }
    }
  }
}

/// Configuration management options for tuning behavior in different deployment scenarios.
///
/// These options allow you to optimize the configuration manager for your specific use case.
#[ derive( Debug, Clone ) ]
pub struct ConfigManagerOptions
{
  /// Maximum number of history entries to keep (0 = unlimited)
  /// Recommended : 10-50 for constrained environments, 100-1000 for production
  pub max_history_entries : usize,

  /// Maximum memory usage for history in bytes (0 = unlimited)
  /// Recommended : 64KB-1MB for constrained, 10MB+ for production
  pub max_history_memory_bytes : usize,

  /// Enable configuration change notifications
  /// Set to false in memory-constrained environments to reduce overhead
  pub enable_change_notifications : bool,

  /// Enable validation caching for improved performance
  /// Generally recommended to be true unless debugging validation issues
  pub enable_validation_caching : bool,

  /// Automatic cleanup interval for old history entries
  /// More frequent cleanup in constrained environments, less frequent in production
  pub cleanup_interval : Option< Duration >,
}

impl Default for ConfigManagerOptions
{
  fn default() -> Self
  {
    Self {
      max_history_entries : 1000, // Keep last 1000 changes
      max_history_memory_bytes : 10 * 1024 * 1024, // 10MB limit
      enable_change_notifications : true,
      enable_validation_caching : true,
      cleanup_interval : Some( Duration::from_secs( 3600 ) ), // Cleanup every hour
    }
  }
}

/// Configuration synchronization status for distributed systems
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum SyncStatus
{
  /// Configuration is synchronized
  Synchronized,
  /// Configuration is pending synchronization
  Pending,
  /// Configuration synchronization failed
  Failed( String ),
  /// Configuration is being synchronized
  InProgress,
}

/// Configuration cache entry with metadata
#[ derive( Debug ) ]
pub struct ConfigCacheEntry
{
  /// Cached configuration
  pub config : DynamicConfig,
  /// When this entry was cached
  pub cached_at : Instant,
  /// Hash of the configuration for quick comparison
  pub config_hash : u64,
  /// Number of times this cache entry has been accessed
  pub access_count : AtomicUsize,
  /// Last time this cache entry was accessed
  pub last_accessed : Mutex< Instant >,
}

impl ConfigCacheEntry
{
  /// Create a new cache entry
  pub fn new( config : DynamicConfig ) -> Self
  {
    let config_hash = config.compute_hash();
    Self {
      config,
      cached_at : Instant::now(),
      config_hash,
      access_count : AtomicUsize::new( 0 ),
      last_accessed : Mutex::new( Instant::now() ),
    }
  }

  /// Mark this cache entry as accessed
  pub fn mark_accessed( &self )
  {
    self.access_count.fetch_add( 1, Ordering::Relaxed );
    *self.last_accessed.lock().unwrap() = Instant::now();
  }

  /// Check if this cache entry has expired based on TTL
  pub fn is_expired( &self, ttl : Duration ) -> bool
  {
    self.cached_at.elapsed() > ttl
  }
}

/// Map of configuration merge strategies keyed by source name
type MergeStrategyMap = HashMap< String, Box< dyn Fn( &DynamicConfig, &DynamicConfig ) -> DynamicConfig + Send + Sync > >;

/// Configuration synchronization context for distributed environments
#[ allow( missing_debug_implementations ) ]
pub struct ConfigSyncContext
{
  /// Configuration cache with keyed entries
  cache : AsyncRwLock< BTreeMap<  String, ConfigCacheEntry  > >,
  /// Broadcast channel for configuration change notifications
  change_broadcaster : broadcast::Sender< ConfigChangeEvent >,
  /// Current synchronization status
  sync_status : AsyncRwLock< SyncStatus >,
  /// Configuration merge strategies by source
  merge_strategies : RwLock< MergeStrategyMap >,
  /// Cache time-to-live settings
  cache_ttl : Duration,
  /// Maximum cache size (number of entries)
  max_cache_size : usize,
}

impl ConfigSyncContext
{
  /// Create a new synchronization context
  pub fn new( cache_ttl : Duration, max_cache_size : usize ) -> Self
  {
    let ( change_broadcaster, _ ) = broadcast::channel( 1000 );
    Self {
      cache : AsyncRwLock::new( BTreeMap::new() ),
      change_broadcaster,
      sync_status : AsyncRwLock::new( SyncStatus::Synchronized ),
      merge_strategies : RwLock::new( HashMap::new() ),
      cache_ttl,
      max_cache_size,
    }
  }

  /// Get configuration from cache if available and not expired
  pub async fn get_cached_config( &self, key : &str ) -> Option< DynamicConfig >
  {
    let cache = self.cache.read().await;
    if let Some( entry ) = cache.get( key )
    {
      if !entry.is_expired( self.cache_ttl )
      {
        entry.mark_accessed();
        return Some( entry.config.clone() );
      }
    }
    None
  }

  /// Cache a configuration with automatic eviction
  pub async fn cache_config( &self, key : String, config : DynamicConfig )
  {
    let mut cache = self.cache.write().await;

    // Remove expired entries first
    cache.retain( | _, entry | !entry.is_expired( self.cache_ttl ) );

    // If we're at capacity, remove least recently used entry
    if cache.len() >= self.max_cache_size
    {
      if let Some( lru_key ) = cache.iter()
        .min_by_key( | ( _, entry ) | *entry.last_accessed.lock().unwrap() )
        .map( | ( k, _ ) | k.clone() )
      {
        cache.remove( &lru_key );
      }
    }

    // Add new entry
    cache.insert( key, ConfigCacheEntry::new( config ) );
  }

  /// Subscribe to configuration change events
  pub fn subscribe_to_changes( &self ) -> broadcast::Receiver< ConfigChangeEvent >
  {
    self.change_broadcaster.subscribe()
  }

  /// Broadcast a configuration change event
  pub fn broadcast_change( &self, event : ConfigChangeEvent )
  {
    let _ = self.change_broadcaster.send( event );
  }

  /// Get current synchronization status
  pub async fn sync_status( &self ) -> SyncStatus
  {
    self.sync_status.read().await.clone()
  }

  /// Update synchronization status
  pub async fn update_sync_status( &self, status : SyncStatus )
  {
    *self.sync_status.write().await = status;
  }

  /// Register a merge strategy for a configuration source
  pub fn register_merge_strategy< F >( &self, source : String, strategy : F )
  where
    F: Fn( &DynamicConfig, &DynamicConfig ) -> DynamicConfig + Send + Sync + 'static,
  {
    let mut strategies = self.merge_strategies.write().unwrap();
    strategies.insert( source, Box::new( strategy ) );
  }

  /// Merge configurations using registered strategies
  pub fn merge_configs( &self, base : &DynamicConfig, overlay : &DynamicConfig, source : &str ) -> DynamicConfig
  {
    let strategies = self.merge_strategies.read().unwrap();
    if let Some( strategy ) = strategies.get( source )
    {
      strategy( base, overlay )
    } else {
      // Default merge strategy using source priority
      base.merge_with( overlay )
    }
  }

  /// Get cache statistics
  pub async fn cache_stats( &self ) -> ( usize, usize )
  {
    let cache = self.cache.read().await;
    let total_entries = cache.len();
    let expired_entries = cache.values().filter( | e | e.is_expired( self.cache_ttl ) ).count();
    ( total_entries, expired_entries )
  }

  /// Clean up expired cache entries
  pub async fn cleanup_cache( &self ) -> usize
  {
    let mut cache = self.cache.write().await;
    let initial_size = cache.len();
    cache.retain( | _, entry | !entry.is_expired( self.cache_ttl ) );
    initial_size - cache.len()
  }
}

/// Handle for a configuration change listener
#[ derive( Debug ) ]
pub struct ConfigChangeListener
{
  /// Internal reference-counted handle for listener lifecycle management
  pub _handle : Arc< () >,
}

impl ConfigChangeListener
{
  /// Create a new configuration change listener handle
  pub fn new() -> Self
  {
    Self {
      _handle : Arc::new( () ),
    }
  }
}

impl Default for ConfigChangeListener
{
  fn default() -> Self
  {
    Self::new()
  }
}
