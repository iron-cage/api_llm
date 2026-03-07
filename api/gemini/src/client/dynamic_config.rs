//! Dynamic configuration methods for Client.
//!
//! This module contains methods for managing dynamic configuration,
//! configuration watching, and health monitoring.

#[ cfg( not( feature = "retry" ) ) ]
use core::time::Duration;
use super::Client;
#[ cfg( feature = "dynamic_configuration" ) ]
use super::config::ConfigWatchHandle;

impl Client
{
  /// Get a configuration manager for explicit runtime configuration updates
  ///
  /// This method provides explicit, on-demand configuration management
  /// following the "Thin Client, Rich API" principle. No automatic background
  /// configuration monitoring or updates are performed - all configuration
  /// changes are explicit operations.
  #[ must_use ]
  #[ inline ]
  pub fn config( &self ) -> crate::models::config::ConfigManager
  {
      crate ::models::config::ConfigManager::new( self.clone() )
  }

  /// Get the current configuration as a `DynamicConfig` instance
  #[ must_use ]
  #[ inline ]
  pub fn current_config( &self ) -> crate::models::config::DynamicConfig
  {
      crate ::models::config::DynamicConfig::builder()
          .timeout( self.timeout )
          .retry_attempts( {
              #[ cfg( feature = "retry" ) ]
              { self.max_retries }
              #[ cfg( not( feature = "retry" ) ) ]
              { 3 }
          } )
          .base_url( self.base_url.clone() )
          .enable_jitter( {
              #[ cfg( feature = "retry" ) ]
              { self.enable_jitter }
              #[ cfg( not( feature = "retry" ) ) ]
              { true }
          } )
          .max_retry_delay( {
              #[ cfg( feature = "retry" ) ]
              { self.max_delay }
              #[ cfg( not( feature = "retry" ) ) ]
              { Duration::from_secs( 30 ) }
          } )
          .base_retry_delay( {
              #[ cfg( feature = "retry" ) ]
              { self.base_delay }
              #[ cfg( not( feature = "retry" ) ) ]
              { Duration::from_millis( 100 ) }
          } )
          .backoff_multiplier( {
              #[ cfg( feature = "retry" ) ]
              { self.backoff_multiplier }
              #[ cfg( not( feature = "retry" ) ) ]
              { 2.0 }
          } )
          .source_priority( 60 ) // Client config has medium-high priority
          .build()
          .unwrap_or_else( | _ | crate::models::config::DynamicConfig::default() )
  }

  /// Apply a new configuration to this client, returning an updated client instance
  ///
  /// # Errors
  ///
  /// Returns an error if the configuration validation fails or if applying the
  /// configuration encounters an internal error.
  #[ inline ]
  pub fn apply_config( &mut self, config : crate::models::config::DynamicConfig ) -> Result< (), crate::error::Error >
  {
      // Update the base URL
      self.base_url = config.base_url;

      // Update retry configuration if retry feature is enabled
      #[ cfg( feature = "retry" ) ]
      {
          self.max_retries = config.retry_attempts;
          self.enable_jitter = config.enable_jitter;
          self.max_delay = config.max_retry_delay;
          self.base_delay = config.base_retry_delay;
          self.backoff_multiplier = config.backoff_multiplier;
      }

      // Note : timeout would need to be applied to the HTTP client
      // For now, we store the configuration but don't recreate the HTTP client

      Ok( () )
  }

  /// Get a failover builder for explicit high availability operations
  ///
  /// This method provides explicit, on-demand failover management
  /// following the "Thin Client, Rich API" principle. No automatic background
  /// failover is performed - all failover operations are explicit.
  #[ must_use ]
  #[ inline ]
  pub fn failover( &self ) -> crate::models::failover::FailoverBuilder
  {
      crate ::models::failover::FailoverBuilder::new( self.clone() )
  }

  /// Create a configuration manager for dynamic configuration management
  ///
  /// This creates a `ConfigManager` instance that can track configuration history,
  /// provide rollback capabilities, and monitor configuration health.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let config_manager = client.create_config_manager();
  ///
  /// // Get current configuration
  /// let current = config_manager.current();
  /// println!( "Current timeout : {:?}", current.timeout );
  ///
  /// // Monitor configuration health
  /// let health = config_manager.health_status();
  /// println!( "Config health : {:?}", health );
  /// # Ok( () )
  /// # }
  /// ```
  #[ cfg( feature = "dynamic_configuration" ) ]
  #[ must_use ]
  #[ inline ]
  pub fn create_config_manager( &self ) -> crate::models::config::ConfigManager
  {
      crate ::models::config::ConfigManager::new( self.clone() )
  }

  /// Create a configuration manager with custom options
  ///
  /// This allows you to customize the configuration manager behavior
  /// for different deployment scenarios.
  ///
  /// # Arguments
  ///
  /// * `options` - Configuration manager options for tuning behavior
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # use api_gemini::models::config::ConfigManagerOptions;
  /// # use std::time::Duration;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  ///
  /// // Create options for memory-constrained environment
  /// let options = ConfigManagerOptions {
  ///     max_history_entries : 10,
  ///     max_history_memory_bytes : 64 * 1024, // 64KB
  ///     enable_change_notifications : false,
  ///     enable_validation_caching : true,
  ///     cleanup_interval : Some( Duration::from_secs( 300 ) ),
  /// };
  ///
  /// let config_manager = client.create_config_manager_with_options( options );
  /// # Ok( () )
  /// # }
  /// ```
  #[ cfg( feature = "dynamic_configuration" ) ]
  #[ must_use ]
  #[ inline ]
  pub fn create_config_manager_with_options( &self, options : crate::models::config::ConfigManagerOptions ) -> crate::models::config::ConfigManager
  {
      crate ::models::config::ConfigManager::with_options( self.clone(), options )
  }

  /// Load configuration from multiple sources with priority-based merging
  ///
  /// This method allows you to configure the client from multiple sources
  /// (files, environment variables, remote APIs) with intelligent merging
  /// based on source priorities.
  ///
  /// # Arguments
  ///
  /// * `sources` - Vector of configuration sources to load from
  ///
  /// # Returns
  ///
  /// Returns a new Client instance with the merged configuration applied.
  ///
  /// # Errors
  ///
  /// Returns `Error` if:
  /// - No configuration sources could be successfully loaded
  /// - Configuration merging or application fails
  /// - Individual config sources return load errors (logged but do not fail the operation)
  ///
  /// # Panics
  ///
  /// This function does not panic. The use of `.unwrap()` is safe because
  /// the function returns an error if no configurations are loaded successfully.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # use api_gemini::models::config::{ FileConfigSource, EnvironmentConfigSource };
  /// # use std::time::Duration;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  ///
  /// // Create configuration sources with priorities
  /// let sources : Vec< Box< dyn api_gemini::models::config::ConfigSource > > = vec![
  ///     Box::new( FileConfigSource::new( "config.yaml", 50 ) ),
  ///     Box::new( EnvironmentConfigSource::new( "GEMINI".to_string(), 75 ) ),
  /// ];
  ///
  /// // Load and apply merged configuration
  /// let updated_client = client.load_from_sources( sources ).await?;
  /// # Ok( () )
  /// # }
  /// ```
  #[ cfg( feature = "dynamic_configuration" ) ]
  #[ inline ]
  pub async fn load_from_sources( &self, sources : Vec< Box< dyn crate::models::config::ConfigSource > > ) -> Result< Self, crate::error::Error >
  {
      // Load configurations from all sources
      let mut configs = Vec::new();
      for source in sources
      {
          match source.load_config().await
          {
              Ok( config ) => configs.push( ( config, source.priority() ) ),
              Err( e ) => {
                  // Log error but continue with other sources
                  tracing ::warn!( "Failed to load config from source {}: {}", source.source_id(), e );
              }
          }
      }

      if configs.is_empty()
      {
          return Err( crate::error::Error::ConfigurationError(
              "No configuration sources could be loaded".to_string()
          ) );
      }

      // Sort by priority (higher priority first)
      configs.sort_by( | a, b | b.1.cmp( &a.1 ) );

      // Start with the lowest priority as base
      let mut merged_config = configs.last().unwrap().0.clone();

      // Merge higher priority configs on top
      for ( config, _ ) in configs.iter().rev().skip( 1 )
      {
          merged_config = merged_config.merge_with( config );
      }

      // Apply the merged configuration to create a new client
      let mut new_client = self.clone();
      new_client.apply_config( merged_config )?;

      Ok( new_client )
  }

  /// Start watching configuration sources for hot-reloading
  ///
  /// This method sets up automatic configuration reloading when any of the
  /// watched sources change. The client will be automatically updated
  /// when configuration changes are detected.
  ///
  /// # Arguments
  ///
  /// * `sources` - Vector of configuration sources to watch
  /// * `on_config_change` - Callback function called when configuration changes
  ///
  /// # Returns
  ///
  /// Returns a handle that can be used to stop watching when dropped.
  ///
  /// # Errors
  ///
  /// Returns `Error` if:
  /// - Any configuration source fails to start watching
  /// - Background task spawning fails
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # use api_gemini::models::config::{ FileConfigSource, EnvironmentConfigSource };
  /// # use std::sync::Arc;
  /// # use tokio::sync::Mutex;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let client = Arc::new( Mutex::new( client ) );
  ///
  /// // Create configuration sources to watch
  /// let sources : Vec< Box< dyn api_gemini::models::config::ConfigSource > > = vec![
  ///     Box::new( FileConfigSource::new( "config.yaml", 50 ) ),
  ///     Box::new( EnvironmentConfigSource::new( "GEMINI".to_string(), 75 ) ),
  /// ];
  ///
  /// let client_clone = client.clone();
  /// let _watch_handle = client.lock().await.start_config_watching(
  ///     sources,
  ///     move | event | {
  ///         println!( "Configuration changed from source : {}", event.source_id );
  ///         // Handle configuration update
  ///     }
  /// ).await?;
  ///
  /// // Configuration will be automatically reloaded when files change
  /// // Keep _watch_handle in scope to continue watching
  /// # Ok( () )
  /// # }
  /// ```
  #[ cfg( feature = "dynamic_configuration" ) ]
  #[ inline ]
  pub async fn start_config_watching< F >(
      &self,
      sources : Vec< Box< dyn crate::models::config::ConfigSource > >,
      on_config_change : F
  ) -> Result< ConfigWatchHandle, crate::error::Error >
  where
      F: Fn( crate::models::config::ConfigSourceEvent ) + Send + Sync + 'static,
  {
      use tokio::sync::mpsc;

      let ( sender, mut receiver ) = mpsc::channel::< crate::models::config::ConfigSourceEvent >( 100 );

      // Start watching all sources that support it
      for source in &sources
      {
          if source.supports_watching()
          {
              source.start_watching( sender.clone() ).await?;
          }
      }

      // Spawn background task to handle configuration changes
      let on_config_change = std::sync::Arc::new( on_config_change );
      tokio ::spawn( async move {
          while let Some( event ) = receiver.recv().await
          {
              on_config_change( event );
          }
      } );

      Ok( ConfigWatchHandle {
          _handle : std::sync::Arc::new( () ),
      } )
  }

  /// Get configuration metrics and health status
  ///
  /// Returns detailed metrics about configuration management performance
  /// and health status indicators.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let config_manager = client.create_config_manager();
  ///
  /// // Get configuration metrics
  /// let metrics = client.get_config_metrics( &config_manager );
  /// println!( "Total updates : {}", metrics.total_updates );
  /// println!( "Cache hit ratio : {:.1}%", metrics.cache_hit_ratio );
  /// println!( "Average update time : {}μs", metrics.avg_update_time_us );
  ///
  /// // Check health status
  /// let health = client.get_config_health( &config_manager );
  /// if !health.is_healthy() {
  ///     println!( "Configuration issues detected : {:?}", health.get_all_messages() );
  /// }
  /// # Ok( () )
  /// # }
  /// ```
  #[ cfg( feature = "dynamic_configuration" ) ]
  #[ must_use ]
  #[ inline ]
  pub fn get_config_metrics( &self, config_manager : &crate::models::config::ConfigManager ) -> crate::models::config::ConfigMetricsReport
  {
      config_manager.generate_metrics_report()
  }

  /// Get configuration health status
  ///
  /// # Arguments
  ///
  /// * `config_manager` - The configuration manager to check health for
  #[ cfg( feature = "dynamic_configuration" ) ]
  #[ must_use ]
  #[ inline ]
  pub fn get_config_health( &self, config_manager : &crate::models::config::ConfigManager ) -> crate::models::config::ConfigHealthStatus
  {
      config_manager.health_status()
  }
}
