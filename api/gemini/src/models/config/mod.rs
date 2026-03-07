//! Dynamic configuration management for the Gemini API client.
//!
//! This module provides explicit runtime configuration updates following the "Thin Client, Rich API" principle.
//! All configuration changes are explicit operations triggered by user code, not automatic behaviors.

// Module declarations
pub mod sources;
pub mod rollback;
pub mod versioning;
pub mod hot_reload;
pub mod propagation;

// Re-exports from submodules
#[ cfg( feature = "dynamic_configuration" ) ]
pub use sources::*;
pub use rollback::*;
pub use versioning::*;
#[ cfg( feature = "dynamic_configuration" ) ]
pub use hot_reload::*;
pub use propagation::*;

mod private
{
  use serde::{ Deserialize, Serialize };
  use core::time::Duration;
  use std::path::Path;
  use std::collections::HashMap;
  use core::hash::{ Hash, Hasher };
  use std::collections::hash_map::DefaultHasher;
  use std::sync::{ Arc, RwLock, Mutex };
  use std::time::{ SystemTime, Instant };
  use tokio::sync::broadcast;

  pub use super::propagation::{ ConfigManagerOptions, ConfigMetrics, ConfigMetricsReport, ConfigHealthStatus, ConfigSyncContext, SyncStatus, ConfigChangeListener };
  pub use super::versioning::{ ConfigHistoryEntry, ConfigChangeType, ConfigChangeEvent, ConfigDelta };
  pub use super::rollback::RollbackAnalysis;

  /// Dynamic configuration that can be updated at runtime with optimization features
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub struct DynamicConfig
  {
    /// Request timeout duration
    pub timeout : Duration,
    /// Maximum retry attempts
    pub retry_attempts : u32,
    /// Base URL for API requests
    pub base_url : String,
    /// Whether to enable jitter in retry backoff
    pub enable_jitter : bool,
    /// Maximum delay between retries
    pub max_retry_delay : Duration,
    /// Base delay for retry backoff
    pub base_retry_delay : Duration,
    /// Backoff multiplier for retry delays
    pub backoff_multiplier : f64,
    /// Configuration source priority (higher = more important)
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub source_priority : Option< u8 >,
    /// Configuration tags for categorization and filtering
    #[ serde( default ) ]
    pub tags : HashMap<  String, String  >,
    /// Configuration validation cache key (for performance optimization)
    #[ serde( skip ) ]
    pub validation_hash : Option< u64 >,
  }

  impl Hash for DynamicConfig
  {
    fn hash< H: Hasher >( &self, state : &mut H )
    {
      self.timeout.hash( state );
      self.retry_attempts.hash( state );
      self.base_url.hash( state );
      self.enable_jitter.hash( state );
      self.max_retry_delay.hash( state );
      self.base_retry_delay.hash( state );
      self.backoff_multiplier.to_bits().hash( state );
      self.source_priority.hash( state );
      for ( k, v ) in &self.tags
      {
        k.hash( state );
        v.hash( state );
      }
    }
  }

  impl DynamicConfig
  {
    /// Compute a hash of this configuration for change detection
    #[ must_use ]
    #[ inline ]
    pub fn compute_hash( &self ) -> u64
    {
      let mut hasher = DefaultHasher::new();
      self.hash( &mut hasher );
      hasher.finish()
    }

    /// Check if this configuration has meaningful changes compared to another
    #[ must_use ]
    #[ inline ]
    pub fn has_changes( &self, other : &DynamicConfig ) -> bool
    {
      self.compute_hash() != other.compute_hash()
    }

    /// Get the validation cache key, computing it if necessary
    #[ must_use ]
    #[ inline ]
    pub fn get_validation_hash( &mut self ) -> u64
    {
      if let Some( hash ) = self.validation_hash
      {
        hash
      } else {
        let hash = self.compute_hash();
        self.validation_hash = Some( hash );
        hash
      }
    }

    /// Invalidate the validation cache (call when configuration changes)
    #[ inline ]
    pub fn invalidate_validation_cache( &mut self )
    {
      self.validation_hash = None;
    }

    /// Create a copy-on-write clone that shares validation cache
    #[ must_use ]
    #[ inline ]
    pub fn cow_clone( &self ) -> Self
    {
      let mut cloned = self.clone();
      cloned.validation_hash = self.validation_hash;
      cloned
    }

    /// Merge this configuration with another, using source priority
    #[ must_use ]
    #[ inline ]
    pub fn merge_with( &self, other : &DynamicConfig ) -> DynamicConfig
    {
      let self_priority = self.source_priority.unwrap_or( 0 );
      let other_priority = other.source_priority.unwrap_or( 0 );

      if other_priority > self_priority
      {
        let mut merged = other.clone();
        merged.tags.extend( self.tags.clone() );
        merged
      } else {
        let mut merged = self.clone();
        merged.tags.extend( other.tags.clone() );
        merged
      }
    }

    /// Create a new configuration builder
    pub fn builder() -> DynamicConfigBuilder
    {
      DynamicConfigBuilder::new()
    }

    /// Create default configuration
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Load configuration from JSON file
    pub async fn from_file< P: AsRef< Path > >( path : P ) -> Result< Self, crate::error::Error >
    {
      let content = tokio::fs::read_to_string( path ).await
        .map_err( | e | crate::error::Error::ConfigurationError(
          format!( "Failed to read configuration file : {}", e )
        ) )?;

      let file_config : FileConfig = serde_json::from_str( &content )
        .map_err( | e | crate::error::Error::ConfigurationError(
          format!( "Failed to parse configuration file : {}", e )
        ) )?;

      let config = Self {
        timeout : Duration::from_secs( file_config.timeout_seconds ),
        retry_attempts : file_config.retry_attempts,
        base_url : file_config.base_url,
        enable_jitter : file_config.enable_jitter.unwrap_or( true ),
        max_retry_delay : Duration::from_millis( file_config.max_retry_delay_ms.unwrap_or( 30000 ) ),
        base_retry_delay : Duration::from_millis( file_config.base_retry_delay_ms.unwrap_or( 100 ) ),
        backoff_multiplier : file_config.backoff_multiplier.unwrap_or( 2.0 ),
        source_priority : Some( 75 ),
        tags : HashMap::new(),
        validation_hash : None,
      };

      DynamicConfigBuilder::new().validate_config( &config )?;
      Ok( config )
    }
  }

  impl Default for DynamicConfig
  {
    fn default() -> Self
    {
      Self {
        timeout : Duration::from_secs( 30 ),
        retry_attempts : 3,
        base_url : "https://generativelanguage.googleapis.com".to_string(),
        enable_jitter : true,
        max_retry_delay : Duration::from_secs( 30 ),
        base_retry_delay : Duration::from_millis( 100 ),
        backoff_multiplier : 2.0,
        source_priority : Some( 50 ),
        tags : HashMap::new(),
        validation_hash : None,
      }
    }
  }

  /// File format for configuration serialization
  #[ derive( Debug, Deserialize ) ]
  struct FileConfig
  {
    timeout_seconds : u64,
    retry_attempts : u32,
    base_url : String,
    enable_jitter : Option< bool >,
    max_retry_delay_ms : Option< u64 >,
    base_retry_delay_ms : Option< u64 >,
    backoff_multiplier : Option< f64 >,
  }

  /// Builder for DynamicConfig with validation
  #[ derive( Debug, Clone ) ]
  pub struct DynamicConfigBuilder
  {
    config : DynamicConfig,
  }

  impl DynamicConfigBuilder
  {
    /// Create a new configuration builder with default values
    pub fn new() -> Self
    {
      Self {
        config : DynamicConfig::default(),
      }
    }

    /// Set request timeout duration
    pub fn timeout( mut self, timeout : Duration ) -> Self
    {
      self.config.timeout = timeout;
      self
    }

    /// Set maximum number of retry attempts
    pub fn retry_attempts( mut self, attempts : u32 ) -> Self
    {
      self.config.retry_attempts = attempts;
      self
    }

    /// Set base URL for API requests
    pub fn base_url( mut self, url : String ) -> Self
    {
      self.config.base_url = url;
      self
    }

    /// Enable or disable jitter in retry backoff
    pub fn enable_jitter( mut self, enable : bool ) -> Self
    {
      self.config.enable_jitter = enable;
      self
    }

    /// Set maximum delay between retries
    pub fn max_retry_delay( mut self, delay : Duration ) -> Self
    {
      self.config.max_retry_delay = delay;
      self
    }

    /// Set base delay for retry backoff
    pub fn base_retry_delay( mut self, delay : Duration ) -> Self
    {
      self.config.base_retry_delay = delay;
      self
    }

    /// Set backoff multiplier for retry delays
    pub fn backoff_multiplier( mut self, multiplier : f64 ) -> Self
    {
      self.config.backoff_multiplier = multiplier;
      self
    }

    /// Set configuration source priority (higher = more important, max 100)
    pub fn source_priority( mut self, priority : u8 ) -> Self
    {
      self.config.source_priority = Some( priority );
      self
    }

    /// Add a configuration tag for categorization and filtering
    pub fn tag( mut self, key : String, value : String ) -> Self
    {
      self.config.tags.insert( key, value );
      self
    }

    /// Build the configuration, validating all settings
    pub fn build( self ) -> Result< DynamicConfig, crate::error::Error >
    {
      self.validate_config( &self.config )?;
      Ok( self.config )
    }

    pub( crate ) fn validate_config( &self, config : &DynamicConfig ) -> Result< (), crate::error::Error >
    {
      if config.timeout.is_zero()
      {
        return Err( crate::error::Error::ConfigurationError(
          "Timeout cannot be zero".to_string()
        ) );
      }

      if config.timeout.as_secs() > 600
      {
        return Err( crate::error::Error::ConfigurationError(
          "Timeout cannot exceed 10 minutes".to_string()
        ) );
      }

      if config.retry_attempts > 50
      {
        return Err( crate::error::Error::ConfigurationError(
          "Retry attempts cannot exceed 50".to_string()
        ) );
      }

      if config.backoff_multiplier < 0.0 || config.backoff_multiplier > 10.0
      {
        return Err( crate::error::Error::ConfigurationError(
          "Backoff multiplier must be between 0 and 10".to_string()
        ) );
      }

      if config.base_url.is_empty()
      {
        return Err( crate::error::Error::ConfigurationError(
          "Base URL cannot be empty".to_string()
        ) );
      }

      if let Some( priority ) = config.source_priority
      {
        if priority > 100
        {
          return Err( crate::error::Error::ConfigurationError(
            "Source priority cannot exceed 100".to_string()
          ) );
        }
      }

      for ( key, value ) in &config.tags
      {
        if key.is_empty()
        {
          return Err( crate::error::Error::ConfigurationError(
            "Configuration tag keys cannot be empty".to_string()
          ) );
        }
        if key.len() > 255
        {
          return Err( crate::error::Error::ConfigurationError(
            format!( "Configuration tag key '{key}' exceeds maximum length of 255" )
          ) );
        }
        if value.len() > 1024
        {
          return Err( crate::error::Error::ConfigurationError(
            format!( "Configuration tag value for key '{key}' exceeds maximum length of 1024" )
          ) );
        }
      }

      if config.tags.len() > 50
      {
        return Err( crate::error::Error::ConfigurationError(
          "Configuration cannot have more than 50 tags".to_string()
        ) );
      }

      Ok( () )
    }
  }

  /// Configuration update operation in progress
  #[ derive( Debug ) ]
  pub struct ConfigUpdate
  {
    client : crate::client::Client,
    new_config : DynamicConfig,
    #[ allow( dead_code ) ]
    previous_config : DynamicConfig,
  }

  impl ConfigUpdate
  {
    /// Create a new configuration update operation
    pub fn new( client : crate::client::Client, new_config : DynamicConfig ) -> Self
    {
      let previous_config = client.current_config();
      Self {
        client,
        new_config,
        previous_config,
      }
    }

    /// Validate the new configuration without caching
    pub fn validate( &self ) -> Result< (), crate::error::Error >
    {
      DynamicConfigBuilder::new().validate_config( &self.new_config )
    }

    /// Validate the new configuration using cached validation results when available
    pub fn validate_with_cache( &mut self ) -> Result< (), crate::error::Error >
    {
      let current_hash = self.new_config.compute_hash();

      if let Some( cached_hash ) = self.new_config.validation_hash
      {
        if cached_hash == current_hash
        {
          return Ok( () );
        }
      }

      DynamicConfigBuilder::new().validate_config( &self.new_config )?;
      self.new_config.validation_hash = Some( current_hash );

      Ok( () )
    }

    /// Apply the configuration update and return a new client instance
    pub async fn apply( self ) -> Result< crate::client::Client, crate::error::Error >
    {
      self.validate()?;

      let mut new_client = self.client.clone();
      new_client.apply_config( self.new_config.clone() )?;

      Ok( new_client )
    }
  }

  /// Configuration management interface for the client with optimizations
  #[ allow( missing_debug_implementations ) ] // Cannot derive Debug due to function pointers
  pub struct ConfigManager
  {
    client : crate::client::Client,
    history : Arc< RwLock< Vec< ConfigHistoryEntry > > >, // RwLock for better read concurrency
    listeners : Arc< RwLock< Vec< Box< dyn Fn( ConfigChangeEvent ) + Send + Sync > > > >,
    options : ConfigManagerOptions,
    metrics : Arc< ConfigMetrics >,
    last_cleanup : Arc< Mutex< Instant > >, // Track when cleanup was last performed
    sync_context : Option< Arc< ConfigSyncContext > >, // Optional synchronization context for distributed systems
  }

  impl ConfigManager
  {
    /// Create a new configuration manager with default options
    pub fn new( client : crate::client::Client ) -> Self
    {
      Self::with_options( client, ConfigManagerOptions::default() )
    }

    /// Create a new configuration manager with custom options
    pub fn with_options( client : crate::client::Client, options : ConfigManagerOptions ) -> Self
    {
      Self::with_sync_context( client, options, None )
    }

    /// Create a new configuration manager with synchronization context for distributed systems
    pub fn with_sync_context( client : crate::client::Client, options : ConfigManagerOptions, sync_context : Option< Arc< ConfigSyncContext > > ) -> Self
    {
      let initial_config = client.current_config();
      let initial_entry = ConfigHistoryEntry::from_config(
        initial_config,
        ConfigChangeType::Update,
        "v0".to_string()
      );

      let metrics = Arc::new( ConfigMetrics::default() );
      metrics.update_history_stats( 1, initial_entry.size_bytes );

      Self {
        client,
        history : Arc::new( RwLock::new( vec![ initial_entry ] ) ),
        listeners : Arc::new( RwLock::new( Vec::new() ) ),
        options,
        metrics,
        last_cleanup : Arc::new( Mutex::new( Instant::now() ) ),
        sync_context,
      }
    }

    /// Get the current configuration
    pub fn current( &self ) -> DynamicConfig
    {
      self.client.current_config()
    }

    /// Start a configuration update operation with optimized validation
    pub fn update( &self, new_config : DynamicConfig ) -> ConfigUpdate
    {
      ConfigUpdate::new( self.client.clone(), new_config )
    }

    /// Apply configuration update with metrics and history management
    pub async fn apply_update( &self, mut config_update : ConfigUpdate ) -> Result< crate::client::Client, crate::error::Error >
    {
      let start_time = Instant::now();

      // Validate configuration with metrics tracking
      let validation_result = if self.options.enable_validation_caching
      {
        // Use cached validation if available
        config_update.validate_with_cache()
      } else {
        config_update.validate()
      };

      match validation_result
      {
        Ok( () ) => {
          // Extract config before consuming config_update
          let new_config = config_update.new_config.clone();

          // Apply the configuration
          let new_client = config_update.apply().await?;

          // Record successful update
          let duration_us = start_time.elapsed().as_micros() as u64;
          self.metrics.record_update( duration_us );

          // Add to history with bounds checking
          self.add_to_history( new_config.clone(), ConfigChangeType::Update )?;

          // Send change notifications if enabled
          if self.options.enable_change_notifications
          {
            self.send_change_notification( new_config, ConfigChangeType::Update ).await;
          }

          // Perform cleanup if needed
          self.cleanup_if_needed().await;

          Ok( new_client )
        },
        Err( e ) => {
          self.metrics.record_failed_update();
          Err( e )
        }
      }
    }

    /// Load configuration from file with optimized handling
    pub async fn load_from_file< P: AsRef< Path > >( &self, path : P ) -> Result< crate::client::Client, crate::error::Error >
    {
      let config = DynamicConfig::from_file( path ).await?;
      let config_update = self.update( config );
      self.apply_update( config_update ).await
    }

    /// Analyze the impact of rolling back to the previous configuration
    pub fn analyze_previous_rollback( &self ) -> Result< RollbackAnalysis, crate::error::Error >
    {
      let ( current_config, previous_config ) = {
        let history = self.history.read().unwrap();
        if history.len() < 2
        {
          return Err( crate::error::Error::ConfigurationError(
            "No previous configuration to rollback to".to_string()
          ) );
        }
        let current = self.client.current_config();
        let previous = history[ history.len() - 2 ].config.clone();
        ( current, previous )
      };

      Ok( RollbackAnalysis::analyze_rollback( &current_config, &previous_config ) )
    }

    /// Analyze the impact of rolling back to a specific version
    pub fn analyze_version_rollback( &self, version_id : &str ) -> Result< RollbackAnalysis, crate::error::Error >
    {
      let ( current_config, target_config ) = {
        let history = self.history.read().unwrap();
        let entry = history.iter().find( | e | e.version_id == version_id )
          .ok_or_else( || crate::error::Error::ConfigurationError(
            format!( "Configuration version '{}' not found", version_id )
          ) )?;
        let current = self.client.current_config();
        let target = entry.config.clone();
        ( current, target )
      };

      Ok( RollbackAnalysis::analyze_rollback( &current_config, &target_config ) )
    }

    /// Rollback to the previous configuration with safety analysis
    pub async fn rollback_with_analysis( &self, force : bool ) -> Result< crate::client::Client, crate::error::Error >
    {
      let analysis = self.analyze_previous_rollback()?;

      // Check safety unless forced
      if !force && !analysis.is_safe
      {
        return Err( crate::error::Error::ConfigurationError(
          format!( "Rollback not safe : {}", analysis.warnings.join( ", " ) )
        ) );
      }

      self.metrics.record_rollback();
      let config_update = ConfigUpdate::new( self.client.clone(), analysis.target_config );
      self.apply_rollback( config_update, ConfigChangeType::Rollback ).await
    }

    /// Rollback to the previous configuration with metrics (legacy method)
    pub async fn rollback( &self ) -> Result< crate::client::Client, crate::error::Error >
    {
      // Use the new analysis-based rollback with force=true for backward compatibility
      self.rollback_with_analysis( true ).await
    }

    /// Rollback to a specific configuration version with safety analysis
    pub async fn rollback_to_version_with_analysis( &self, version_id : String, force : bool ) -> Result< crate::client::Client, crate::error::Error >
    {
      let analysis = self.analyze_version_rollback( &version_id )?;

      // Check safety unless forced
      if !force && !analysis.is_safe
      {
        return Err( crate::error::Error::ConfigurationError(
          format!( "Rollback to version '{}' not safe : {}", version_id, analysis.warnings.join( ", " ) )
        ) );
      }

      self.metrics.record_rollback();
      let config_update = ConfigUpdate::new( self.client.clone(), analysis.target_config );
      self.apply_rollback( config_update, ConfigChangeType::VersionRestore ).await
    }

    /// Rollback to a specific configuration version with metrics (legacy method)
    pub async fn rollback_to_version( &self, version_id : String ) -> Result< crate::client::Client, crate::error::Error >
    {
      // Use the new analysis-based rollback with force=true for backward compatibility
      self.rollback_to_version_with_analysis( version_id, true ).await
    }

    /// Get optimized history with delta compression statistics
    pub fn history_with_compression_stats( &self ) -> ( Vec< ConfigHistoryEntry >, usize, usize )
    {
      let history = self.history.read().unwrap();
      let total_memory = history.iter().map( | e | e.size_bytes ).sum::< usize >();
      let compressed_memory = history.iter().map( | e | e.effective_memory_footprint() ).sum::< usize >();
      ( history.clone(), total_memory, compressed_memory )
    }

    /// Compress history using delta storage to save memory
    pub fn compress_history( &self ) -> Result< usize, crate::error::Error >
    {
      let mut history = self.history.write().unwrap();
      let mut memory_saved = 0;

      // Compress all entries except the first one (which serves as base)
      for i in 1..history.len()
      {
        if history[ i ].delta.is_none()
        {
          let previous_config = &history[ i - 1 ].config;
          let current_config = &history[ i ].config;

          let delta = ConfigDelta::create_delta( previous_config, current_config );
          let original_size = history[ i ].size_bytes;
          let compressed_size = delta.memory_footprint();

          if compressed_size < original_size
          {
            memory_saved += original_size - compressed_size;
            history[ i ].delta = Some( delta );
            // Keep the config for now - in a real implementation, we might store only the delta
          }
        }
      }

      // Update metrics
      let total_memory = history.iter().map( | e | e.effective_memory_footprint() ).sum::< usize >();
      self.metrics.update_history_stats( history.len(), total_memory );

      Ok( memory_saved )
    }

    /// Get configuration history with optimized read access
    pub fn history( &self ) -> Vec< ConfigHistoryEntry >
    {
      self.history.read().unwrap().clone()
    }

    /// Get configuration manager metrics
    pub fn metrics( &self ) -> Arc< ConfigMetrics >
    {
      self.metrics.clone()
    }

    /// Generate comprehensive metrics report
    pub fn generate_metrics_report( &self ) -> ConfigMetricsReport
    {
      self.metrics.generate_report()
    }

    /// Get current health status of configuration management
    pub fn health_status( &self ) -> ConfigHealthStatus
    {
      self.metrics.health_check()
    }

    /// Export metrics in Prometheus format for monitoring integration
    pub fn export_prometheus_metrics( &self, instance_name : &str ) -> String
    {
      self.metrics.to_prometheus_format( instance_name )
    }

    /// Reset metrics counters (useful for testing or periodic resets)
    pub fn reset_metrics( &self )
    {
      self.metrics.reset()
    }

    /// Get current configuration manager options
    pub fn options( &self ) -> &ConfigManagerOptions
    {
      &self.options
    }

    /// Get synchronization context if available
    pub fn sync_context( &self ) -> Option< Arc< ConfigSyncContext > >
    {
      self.sync_context.clone()
    }

    /// Get cached configuration by key (if synchronization context is available)
    pub async fn get_cached_config( &self, key : &str ) -> Option< DynamicConfig >
    {
      if let Some( sync_context ) = &self.sync_context
      {
        sync_context.get_cached_config( key ).await
      } else {
        None
      }
    }

    /// Cache a configuration with key (if synchronization context is available)
    pub async fn cache_config( &self, key : String, config : DynamicConfig )
    {
      if let Some( sync_context ) = &self.sync_context
      {
        sync_context.cache_config( key, config ).await;
      }
    }

    /// Subscribe to configuration change broadcasts (if synchronization context is available)
    pub fn subscribe_to_changes( &self ) -> Option< broadcast::Receiver< ConfigChangeEvent > >
    {
      self.sync_context.as_ref().map( | ctx | ctx.subscribe_to_changes() )
    }

    /// Get synchronization status (if synchronization context is available)
    pub async fn sync_status( &self ) -> Option< SyncStatus >
    {
      if let Some( sync_context ) = &self.sync_context
      {
        Some( sync_context.sync_status().await )
      } else {
        None
      }
    }

    /// Update synchronization status (if synchronization context is available)
    pub async fn update_sync_status( &self, status : SyncStatus )
    {
      if let Some( sync_context ) = &self.sync_context
      {
        sync_context.update_sync_status( status ).await;
      }
    }

    /// Merge configurations using registered strategies (if synchronization context is available)
    pub fn merge_configs( &self, base : &DynamicConfig, overlay : &DynamicConfig, source : &str ) -> DynamicConfig
    {
      if let Some( sync_context ) = &self.sync_context
      {
        sync_context.merge_configs( base, overlay, source )
      } else {
        // Fallback to default merge strategy
        base.merge_with( overlay )
      }
    }

    /// Get cache statistics (if synchronization context is available)
    pub async fn cache_stats( &self ) -> Option< ( usize, usize ) >
    {
      if let Some( sync_context ) = &self.sync_context
      {
        Some( sync_context.cache_stats().await )
      } else {
        None
      }
    }

    /// Clean up expired cache entries (if synchronization context is available)
    pub async fn cleanup_cache( &self ) -> usize
    {
      if let Some( sync_context ) = &self.sync_context
      {
        sync_context.cleanup_cache().await
      } else {
        0
      }
    }

    /// Register a listener for configuration changes with optimized access
    pub fn on_change< F >( &self, listener : F ) -> ConfigChangeListener
    where
      F: Fn( ConfigChangeEvent ) + Send + Sync + 'static,
    {
      let mut listeners = self.listeners.write().unwrap(); // Use write lock only when needed
      listeners.push( Box::new( listener ) );
      ConfigChangeListener {
        _handle : Arc::new( () ) // Placeholder for listener handle
      }
    }

    /// Helper method to apply rollback operations with metrics
    async fn apply_rollback( &self, config_update : ConfigUpdate, change_type : ConfigChangeType ) -> Result< crate::client::Client, crate::error::Error >
    {
      let start_time = Instant::now();

      // Extract config before consuming config_update
      let new_config = config_update.new_config.clone();

      // Apply the rollback configuration
      let new_client = config_update.apply().await?;

      // Record successful rollback
      let duration_us = start_time.elapsed().as_micros() as u64;
      self.metrics.record_update( duration_us );

      // Add to history
      self.add_to_history( new_config.clone(), change_type.clone() )?;

      // Send change notifications if enabled
      if self.options.enable_change_notifications
      {
        self.send_change_notification( new_config, change_type ).await;
      }

      // Perform cleanup if needed
      self.cleanup_if_needed().await;

      Ok( new_client )
    }

    /// Add configuration to history with bounds management
    fn add_to_history( &self, config : DynamicConfig, change_type : ConfigChangeType ) -> Result< (), crate::error::Error >
    {
      let mut history = self.history.write().unwrap();

      // Generate new version ID
      let version_id = format!( "v{}", history.len() );
      let new_entry = ConfigHistoryEntry::from_config( config, change_type, version_id );

      // Check memory limits before adding
      let current_memory = history.iter().map( | e | e.size_bytes ).sum::< usize >();
      let new_memory = current_memory + new_entry.size_bytes;

      // Remove oldest entries if we exceed limits
      while ( self.options.max_history_entries > 0 && history.len() >= self.options.max_history_entries ) ||
            ( self.options.max_history_memory_bytes > 0 && new_memory > self.options.max_history_memory_bytes )
      {
        if history.len() <= 1
        {
          break; // Always keep at least one entry
        }
        history.remove( 0 );
      }

      // Add new entry
      history.push( new_entry );

      // Update metrics
      let total_memory = history.iter().map( | e | e.size_bytes ).sum::< usize >();
      self.metrics.update_history_stats( history.len(), total_memory );

      Ok( () )
    }

    /// Send change notification to all listeners
    async fn send_change_notification( &self, new_config : DynamicConfig, change_type : ConfigChangeType )
    {
      let event = ConfigChangeEvent {
        version_id : format!( "v{}", self.history.read().unwrap().len() ),
        change_type,
        timestamp : SystemTime::now(),
        previous_config : self.history.read().unwrap().last().map( | e | e.config.clone() ),
        new_config,
      };

      // Notify listeners with read lock for better concurrency
      let listeners = self.listeners.read().unwrap();
      for listener in listeners.iter()
      {
        listener( event.clone() );
      }
      drop( listeners );

      // Also broadcast to sync context if available
      if let Some( sync_context ) = &self.sync_context
      {
        sync_context.broadcast_change( event );
      }

      self.metrics.record_change_event();
    }

    /// Perform cleanup of old history entries if needed
    async fn cleanup_if_needed( &self )
    {
      if let Some( interval ) = self.options.cleanup_interval
      {
        let mut last_cleanup = self.last_cleanup.lock().unwrap();
        if last_cleanup.elapsed() >= interval
        {
          *last_cleanup = Instant::now();
          drop( last_cleanup );

          // Perform cleanup in background
          self.cleanup_history();
        }
      }
    }

    /// Clean up old history entries based on retention policies
    fn cleanup_history( &self )
    {
      let mut history = self.history.write().unwrap();
      let initial_count = history.len();

      // Remove entries that exceed limits
      while ( self.options.max_history_entries > 0 && history.len() > self.options.max_history_entries ) ||
            ( self.options.max_history_memory_bytes > 0 &&
              history.iter().map( | e | e.size_bytes ).sum::< usize >() > self.options.max_history_memory_bytes )
      {
        if history.len() <= 1
        {
          break; // Always keep at least one entry
        }
        history.remove( 0 );
      }

      // Update metrics if anything was cleaned up
      if history.len() != initial_count
      {
        let total_memory = history.iter().map( | e | e.size_bytes ).sum::< usize >();
        self.metrics.update_history_stats( history.len(), total_memory );
      }
    }
  }
}

::mod_interface::mod_interface!
{
  exposed use private::DynamicConfig;
  exposed use private::DynamicConfigBuilder;
  exposed use private::ConfigUpdate;
  exposed use private::ConfigManager;
}
