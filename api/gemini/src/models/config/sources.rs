//! Multi-source configuration loading and watching
//!
//! This module provides configuration sources that can load from files, environment
//! variables, and remote HTTP endpoints with automatic change watching.

#[ cfg( feature = "dynamic_configuration" ) ]
use super::DynamicConfig;
#[ cfg( feature = "dynamic_configuration" ) ]
use core::time::Duration;
#[ cfg( feature = "dynamic_configuration" ) ]
use std::collections::HashMap;
#[ cfg( feature = "dynamic_configuration" ) ]
use tokio::sync::mpsc::Sender;

#[ cfg( feature = "dynamic_configuration" ) ]
use notify::{ Watcher, RecursiveMode, Event };
#[ cfg( feature = "dynamic_configuration" ) ]
use std::sync::mpsc;
#[ cfg( feature = "dynamic_configuration" ) ]
use std::thread;
#[ cfg( feature = "dynamic_configuration" ) ]
use std::env;

/// Configuration source trait for multiple configuration sources
#[ cfg( feature = "dynamic_configuration" ) ]
#[ async_trait::async_trait ]
pub trait ConfigSource : Send + Sync
{
  /// Load configuration from this source
  async fn load_config( &self ) -> Result< DynamicConfig, crate::error::Error >;

  /// Get the source identifier for debugging/logging
  fn source_id( &self ) -> &str;

  /// Get the source priority (higher = more important)
  fn priority( &self ) -> u8;

  /// Check if this source supports watching for changes
  fn supports_watching( &self ) -> bool
  {
    false
  }

  /// Start watching for configuration changes (if supported)
  async fn start_watching( &self, _sender : Sender< ConfigSourceEvent > ) -> Result< (), crate::error::Error >
  {
    Err( crate::error::Error::NotImplemented( "Watching not supported for this source".to_string() ) )
  }
}

/// Event from a configuration source
#[ cfg( feature = "dynamic_configuration" ) ]
#[ derive( Debug, Clone ) ]
pub struct ConfigSourceEvent
{
  /// Source that generated the event
  pub source_id : String,
  /// Type of change
  pub event_type : ConfigSourceEventType,
  /// New configuration (if available)
  pub config : Option< DynamicConfig >,
  /// Error message (if event_type is Error)
  pub error : Option< String >,
}

/// Type of configuration source event
#[ cfg( feature = "dynamic_configuration" ) ]
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum ConfigSourceEventType
{
  /// Configuration changed and was successfully loaded
  ConfigChanged,
  /// Configuration file/source was modified but couldn't be loaded
  LoadError,
  /// Source became unavailable
  SourceUnavailable,
  /// Source became available again
  SourceAvailable,
}

/// File-based configuration source with watching capability
#[ cfg( feature = "dynamic_configuration" ) ]
#[ derive( Debug ) ]
pub struct FileConfigSource
{
  /// Path to the configuration file
  file_path : std::path::PathBuf,
  /// Source priority
  priority : u8,
  /// Source identifier
  source_id : String,
}

#[ cfg( feature = "dynamic_configuration" ) ]
impl FileConfigSource
{
  /// Create a new file configuration source
  pub fn new< P: AsRef< std::path::Path > >( file_path : P, priority : u8 ) -> Self
  {
    let file_path = file_path.as_ref().to_path_buf();
    let source_id = format!( "file:{}", file_path.display() );

    Self {
      file_path,
      priority,
      source_id,
    }
  }
}

#[ cfg( feature = "dynamic_configuration" ) ]
#[ async_trait::async_trait ]
impl ConfigSource for FileConfigSource
{
  async fn load_config( &self ) -> Result< DynamicConfig, crate::error::Error >
  {
    DynamicConfig::from_file( &self.file_path ).await
  }

  fn source_id( &self ) -> &str
  {
    &self.source_id
  }

  fn priority( &self ) -> u8
  {
    self.priority
  }

  fn supports_watching( &self ) -> bool
  {
    true
  }

  async fn start_watching( &self, sender : Sender< ConfigSourceEvent > ) -> Result< (), crate::error::Error >
  {
    let file_path = self.file_path.clone();
    let file_path_async = self.file_path.clone(); // Clone for async task
    let source_id = self.source_id.clone();

    // Use sync bridge for file watching
    let ( sync_tx, sync_rx ) = mpsc::channel();

    // Spawn sync thread for file watching
    thread ::spawn( move || {
      let tx = sync_tx;
      let watcher = notify::recommended_watcher( move | res | {
        let _ = tx.send( res );
      } );

      if let Ok( mut watcher ) = watcher
      {
        if let Some( parent ) = file_path.parent()
        {
          if watcher.watch( parent, RecursiveMode::NonRecursive ).is_ok()
          {
            // Keep the watcher alive by blocking the thread
            loop
            {
              std ::thread::sleep( std::time::Duration::from_secs( 1 ) );
            }
          }
        }
      }
    } );

    // Spawn async task to bridge sync -> async
    tokio ::spawn( async move {
      while let Ok( res ) = sync_rx.recv()
      {
        match res
        {
          Ok( Event { paths, .. } ) => {
            if paths.contains( &file_path_async )
            {
              let event = ConfigSourceEvent {
                source_id : source_id.clone(),
                event_type : ConfigSourceEventType::ConfigChanged,
                config : None,
                error : None,
              };

              if sender.send( event ).await.is_err()
              {
                break;
              }
            }
          }
          Err( e ) => {
            let event = ConfigSourceEvent {
              source_id : source_id.clone(),
              event_type : ConfigSourceEventType::LoadError,
              config : None,
              error : Some( format!( "File watch error : {}", e ) ),
            };

            if sender.send( event ).await.is_err()
            {
              break;
            }
          }
        }
      }
    } );

    Ok( () )
  }
}

/// Environment variable configuration source
#[ cfg( feature = "dynamic_configuration" ) ]
#[ derive( Debug ) ]
pub struct EnvironmentConfigSource
{
  /// Environment variable prefix
  prefix : String,
  /// Source priority
  priority : u8,
  /// Source identifier
  source_id : String,
}

#[ cfg( feature = "dynamic_configuration" ) ]
impl EnvironmentConfigSource
{
  /// Create a new environment configuration source
  pub fn new( prefix : String, priority : u8 ) -> Self
  {
    let source_id = format!( "env:{}", prefix );

    Self {
      prefix,
      priority,
      source_id,
    }
  }
}

#[ cfg( feature = "dynamic_configuration" ) ]
#[ async_trait::async_trait ]
impl ConfigSource for EnvironmentConfigSource
{
  async fn load_config( &self ) -> Result< DynamicConfig, crate::error::Error >
  {
    let mut builder = DynamicConfig::builder();

    // Map environment variables to configuration fields
    if let Ok( timeout ) = env::var( format!( "{}_TIMEOUT_SECONDS", self.prefix ) )
    {
      if let Ok( seconds ) = timeout.parse::< u64 >()
      {
        builder = builder.timeout( Duration::from_secs( seconds ) );
      }
    }

    if let Ok( retries ) = env::var( format!( "{}_RETRY_ATTEMPTS", self.prefix ) )
    {
      if let Ok( attempts ) = retries.parse::< u32 >()
      {
        builder = builder.retry_attempts( attempts );
      }
    }

    if let Ok( base_url ) = env::var( format!( "{}_BASE_URL", self.prefix ) )
    {
      builder = builder.base_url( base_url );
    }

    if let Ok( jitter ) = env::var( format!( "{}_ENABLE_JITTER", self.prefix ) )
    {
      if let Ok( enable ) = jitter.parse::< bool >()
      {
        builder = builder.enable_jitter( enable );
      }
    }

    // Set source priority and metadata
    let config = builder
      .source_priority( self.priority )
      .tag( "source".to_string(), "environment".to_string() )
      .tag( "prefix".to_string(), self.prefix.clone() )
      .build()?;

    Ok( config )
  }

  fn source_id( &self ) -> &str
  {
    &self.source_id
  }

  fn priority( &self ) -> u8
  {
    self.priority
  }
}

/// Remote HTTP-based configuration source with polling capability
#[ cfg( feature = "dynamic_configuration" ) ]
#[ derive( Debug ) ]
pub struct RemoteConfigSource
{
  /// Remote endpoint URL
  endpoint_url : String,
  /// Source priority
  priority : u8,
  /// Source identifier
  source_id : String,
  /// HTTP client for making requests
  http_client : reqwest::Client,
  /// Polling interval for checking updates
  poll_interval : Duration,
  /// Authentication headers (if needed)
  auth_headers : HashMap<  String, String  >,
}

#[ cfg( feature = "dynamic_configuration" ) ]
impl RemoteConfigSource
{
  /// Create a new remote configuration source
  pub fn new( endpoint_url : String, priority : u8 ) -> Self
  {
    let source_id = format!( "remote:{}", endpoint_url );
    let http_client = reqwest::Client::builder()
      .timeout( Duration::from_secs( 30 ) )
      .build()
      .unwrap_or_else( | _ | reqwest::Client::new() );

    Self {
      endpoint_url,
      priority,
      source_id,
      http_client,
      poll_interval : Duration::from_secs( 60 ), // Default 1 minute polling
      auth_headers : HashMap::new(),
    }
  }

  /// Create a new remote configuration source with custom polling interval
  pub fn with_poll_interval( endpoint_url : String, priority : u8, poll_interval : Duration ) -> Self
  {
    let mut source = Self::new( endpoint_url, priority );
    source.poll_interval = poll_interval;
    source
  }

  /// Add authentication header
  pub fn with_auth_header( mut self, key : String, value : String ) -> Self
  {
    self.auth_headers.insert( key, value );
    self
  }

  /// Add multiple authentication headers
  pub fn with_auth_headers( mut self, headers : HashMap<  String, String  > ) -> Self
  {
    self.auth_headers.extend( headers );
    self
  }

  /// Set polling interval for configuration updates
  pub fn set_poll_interval( &mut self, interval : Duration )
  {
    self.poll_interval = interval;
  }
}

#[ cfg( feature = "dynamic_configuration" ) ]
#[ async_trait::async_trait ]
impl ConfigSource for RemoteConfigSource
{
  async fn load_config( &self ) -> Result< DynamicConfig, crate::error::Error >
  {
    let mut request = self.http_client.get( &self.endpoint_url );

    // Add authentication headers
    for ( key, value ) in &self.auth_headers
    {
      request = request.header( key, value );
    }

    // Make the request
    let response = request.send().await
      .map_err( | e | crate::error::Error::NetworkError( format!( "Failed to fetch remote config : {}", e ) ) )?;

    if !response.status().is_success()
    {
      return Err( crate::error::Error::ServerError(
        format!( "Remote config request failed with status : {}", response.status() )
      ) );
    }

    // Parse the response as JSON
    let config_json = response.text().await
      .map_err( | e | crate::error::Error::NetworkError( format!( "Failed to read remote config response : {}", e ) ) )?;

    let mut config : DynamicConfig = serde_json::from_str( &config_json )
      .map_err( | e | crate::error::Error::DeserializationError( format!( "Failed to parse remote config JSON: {}", e ) ) )?;

    // Set source priority and metadata
    config.source_priority = Some( self.priority );
    config.tags.insert( "source".to_string(), "remote".to_string() );
    config.tags.insert( "endpoint".to_string(), self.endpoint_url.clone() );

    Ok( config )
  }

  fn source_id( &self ) -> &str
  {
    &self.source_id
  }

  fn priority( &self ) -> u8
  {
    self.priority
  }

  fn supports_watching( &self ) -> bool
  {
    true
  }

  async fn start_watching( &self, sender : Sender< ConfigSourceEvent > ) -> Result< (), crate::error::Error >
  {
    let endpoint_url = self.endpoint_url.clone();
    let source_id = self.source_id.clone();
    let http_client = self.http_client.clone();
    let auth_headers = self.auth_headers.clone();
    let poll_interval = self.poll_interval;

    tokio ::spawn( async move {
      let mut last_config_hash : Option< u64 > = None;
      let mut interval = tokio::time::interval( poll_interval );

      loop
      {
        interval.tick().await;

        // Try to fetch the current configuration
        match Self::fetch_config( &http_client, &endpoint_url, &auth_headers ).await
        {
          Ok( config ) => {
            let current_hash = config.compute_hash();

            // Check if configuration has changed
            if last_config_hash.map_or( true, | hash | hash != current_hash )
            {
              last_config_hash = Some( current_hash );

              let event = ConfigSourceEvent {
                source_id : source_id.clone(),
                event_type : ConfigSourceEventType::ConfigChanged,
                config : Some( config ),
                error : None,
              };

              if sender.send( event ).await.is_err()
              {
                break; // Channel closed, stop watching
              }
            }
          },
          Err( e ) => {
            let event = ConfigSourceEvent {
              source_id : source_id.clone(),
              event_type : ConfigSourceEventType::LoadError,
              config : None,
              error : Some( format!( "Failed to fetch remote config : {}", e ) ),
            };

            if sender.send( event ).await.is_err()
            {
              break; // Channel closed, stop watching
            }
          }
        }
      }
    } );

    Ok( () )
  }
}

#[ cfg( feature = "dynamic_configuration" ) ]
impl RemoteConfigSource
{
  /// Helper method to fetch configuration from remote endpoint
  async fn fetch_config(
    http_client : &reqwest::Client,
    endpoint_url : &str,
    auth_headers : &HashMap<  String, String  >
  ) -> Result< DynamicConfig, crate::error::Error >
  {
    let mut request = http_client.get( endpoint_url );

    // Add authentication headers
    for ( key, value ) in auth_headers
    {
      request = request.header( key, value );
    }

    // Make the request
    let response = request.send().await
      .map_err( | e | crate::error::Error::NetworkError( format!( "HTTP request failed : {}", e ) ) )?;

    if !response.status().is_success()
    {
      return Err( crate::error::Error::ServerError(
        format!( "HTTP request failed with status : {}", response.status() )
      ) );
    }

    // Parse the response as JSON
    let config_json = response.text().await
      .map_err( | e | crate::error::Error::NetworkError( format!( "Failed to read response : {}", e ) ) )?;

    let config : DynamicConfig = serde_json::from_str( &config_json )
      .map_err( | e | crate::error::Error::DeserializationError( format!( "Failed to parse JSON: {}", e ) ) )?;

    Ok( config )
  }
}
