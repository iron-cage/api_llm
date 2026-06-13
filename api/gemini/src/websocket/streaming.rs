//! WebSocket streaming operations and builders

use super::protocol::*;
use super::connection::*;
use crate::error::Error;
use crate::models::websocket_streaming::*;
use crate::models::websocket_streaming_optimized::*;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{ Arc, RwLock };
use std::time::Instant;

/// Stream builder for creating WebSocket streams with fluent API
#[ derive( Debug, Clone ) ]
pub struct WebSocketStreamBuilder
{
  /// Target endpoint URL
  endpoint : Option< String >,
  /// Stream direction
  direction : StreamDirection,
  /// Configuration
  config : WebSocketConfig,
  /// Authentication token
  auth_token : Option< String >,
  /// Initial metadata
  metadata : HashMap<  String, String  >,
  /// Auto-reconnect setting
  auto_reconnect : bool,
}

impl WebSocketStreamBuilder
{
  /// Create a new stream builder
  pub fn new() -> Self
  {
    Self {
      endpoint : None,
      direction : StreamDirection::Bidirectional,
      config : WebSocketConfig::default(),
      auth_token : None,
      metadata : HashMap::new(),
      auto_reconnect : true,
    }
  }

  /// Set the endpoint URL
  pub fn endpoint( mut self, endpoint : &str ) -> Self
  {
    self.endpoint = Some( endpoint.to_string() );
    self
  }

  /// Set the stream direction
  pub fn direction( mut self, direction : StreamDirection ) -> Self
  {
    self.direction = direction;
    self
  }

  /// Set the configuration
  pub fn config( mut self, config : WebSocketConfig ) -> Self
  {
    self.config = config;
    self
  }

  /// Set authentication token
  pub fn auth_token( mut self, token : &str ) -> Self
  {
    self.auth_token = Some( token.to_string() );
    self
  }

  /// Add metadata
  pub fn metadata( mut self, key : &str, value : &str ) -> Self
  {
    self.metadata.insert( key.to_string(), value.to_string() );
    self
  }

  /// Set auto-reconnect behavior
  pub fn auto_reconnect( mut self, enabled : bool ) -> Self
  {
    self.auto_reconnect = enabled;
    self
  }

  /// Build and start the WebSocket stream
  pub async fn build( self, manager : &WebSocketConnectionManager ) -> Result< String, Error >
  {
    let endpoint = self.endpoint.ok_or_else( || Error::InvalidArgument( "Endpoint is required".to_string() ) )?;

    let session_id = manager.create_session( &endpoint, self.config ).await?;

    // Send authentication if provided
    if let Some( token ) = self.auth_token
    {
      if let Some( session ) = manager.get_session( &session_id )
      {
        let auth_message = WebSocketStreamMessage::Auth {
          token,
          scope : None,
        };
        session.send_message( auth_message ).await?;
      }
    }

    Ok( session_id )
  }
}

impl Default for WebSocketStreamBuilder
{
  fn default() -> Self
  {
    Self::new()
  }
}

/// Enhanced WebSocket streaming API for Gemini integration with optimizations
#[ derive( Debug ) ]
pub struct WebSocketStreamingApi< 'a >
{
  /// Reference to the Gemini client
  client : &'a crate::client::Client,
  /// Basic connection manager
  manager : Arc< WebSocketConnectionManager >,
  /// Optimized connection pool for high-performance scenarios
  optimized_pool : Arc< OptimizedConnectionPool >,
  /// Optimization configuration
  optimization_config : OptimizedWebSocketConfig,
  /// Performance metrics tracker
  performance_metrics : Arc< RwLock< EnhancedStreamingMetrics > >,
}

impl< 'a > WebSocketStreamingApi< 'a >
{
  /// Create a new enhanced WebSocket streaming API with basic configuration
  pub fn new( client : &'a crate::client::Client ) -> Self
  {
    Self::with_optimization_config( client, OptimizedWebSocketConfig::default() )
  }

  /// Create WebSocket streaming API with custom optimization configuration
  pub fn with_optimization_config( client : &'a crate::client::Client, config : OptimizedWebSocketConfig ) -> Self
  {
    let pool_config = WebSocketPoolConfig::default();
    let manager = Arc::new( WebSocketConnectionManager::new( pool_config ) );
    let optimized_pool = Arc::new( OptimizedConnectionPool::new( config.pool_config.clone() ) );

    Self {
      client,
      manager,
      optimized_pool,
      optimization_config : config,
      performance_metrics : Arc::new( RwLock::new( EnhancedStreamingMetrics {
        basic_metrics : WebSocketMetrics::default(),
        streaming_metrics : StreamingMetrics::default(),
        pool_stats : ConnectionPoolStats {
          total_connections : 0,
          active_connections : 0,
          idle_connections : 0,
          connections_created : 0,
          connections_reused : 0,
          hit_ratio : 0.0,
          avg_connection_age_seconds : 0.0,
        },
        performance_benchmarks : PerformanceBenchmarks::default(),
      } ) ),
    }
  }

  /// Create a new streaming session with automatic optimization selection
  pub async fn create_stream( &self, endpoint : &str ) -> Result< String, Error >
  {
    // Measure connection time for performance benchmarks
    let start_time = Instant::now();

    let config = WebSocketConfig::default();
    let result = self.manager.create_session( endpoint, config ).await;

    // Update performance metrics
    if result.is_ok()
    {
      self.update_connection_benchmark( start_time.elapsed().as_millis() as f64 ).await;
    }

    result
  }

  /// Create a stream with custom configuration and optimization features
  pub async fn create_stream_with_config( &self, endpoint : &str, config : WebSocketConfig ) -> Result< String, Error >
  {
    let start_time = Instant::now();
    let result = self.manager.create_session( endpoint, config ).await;

    if result.is_ok()
    {
      self.update_connection_benchmark( start_time.elapsed().as_millis() as f64 ).await;
    }

    result
  }

  /// Create an optimized streaming connection using the connection pool
  pub async fn create_optimized_stream( &self, endpoint : &str ) -> Result< Arc< OptimizedWebSocketConnection >, Error >
  {
    let start_time = Instant::now();
    let connection = self.optimized_pool.get_connection( endpoint ).await?;

    self.update_connection_benchmark( start_time.elapsed().as_millis() as f64 ).await;

    // Update pool statistics
    self.update_pool_metrics().await;

    Ok( connection )
  }

  /// Get a stream builder for fluent API
  pub fn stream_builder( &self ) -> WebSocketStreamBuilder
  {
    WebSocketStreamBuilder::new()
  }

  /// Get an enhanced stream builder with optimization features
  pub fn enhanced_stream_builder( &self ) -> EnhancedWebSocketStreamBuilder< '_ >
  {
    EnhancedWebSocketStreamBuilder::new( self )
  }

  /// Get a session by ID
  pub fn get_session( &self, session_id : &str ) -> Option< Arc< WebSocketStreamSession > >
  {
    self.manager.get_session( session_id )
  }

  /// Get a stream controller for a session
  pub fn get_controller( &self, session_id : &str ) -> Option< StreamController >
  {
    self.manager.get_session( session_id )
      .map( StreamController::new )
  }

  /// Close a streaming session
  pub async fn close_stream( &self, session_id : &str ) -> Result< (), Error >
  {
    self.manager.remove_session( session_id ).await
  }

  /// Return an optimized connection to the pool
  pub async fn return_optimized_connection( &self, connection : Arc< OptimizedWebSocketConnection > ) -> Result< (), Error >
  {
    let result = self.optimized_pool.return_connection( connection ).await;
    self.update_pool_metrics().await;
    result
  }

  /// List all active streaming sessions
  pub fn list_active_streams( &self ) -> Vec< String >
  {
    self.manager.list_sessions()
  }

  /// Get enhanced streaming metrics with performance benchmarks
  pub async fn get_enhanced_metrics( &self ) -> EnhancedStreamingMetrics
  {
    let mut metrics = self.performance_metrics.read()
      .unwrap_or_else( | poisoned | poisoned.into_inner() ).clone();

    // Update with real-time data
    metrics.basic_metrics = self.manager.get_metrics();
    metrics.pool_stats = self.optimized_pool.get_stats();

    metrics
  }

  /// Get global streaming metrics (basic compatibility method)
  pub fn get_streaming_metrics( &self ) -> WebSocketMetrics
  {
    self.manager.get_metrics()
  }

  /// Get connection pool statistics
  pub fn get_pool_statistics( &self ) -> ConnectionPoolStats
  {
    self.optimized_pool.get_stats()
  }

  /// Perform connection pool cleanup and return number of connections cleaned
  pub async fn cleanup_pool( &self ) -> Result< usize, Error >
  {
    let cleaned = self.optimized_pool.cleanup().await?;
    self.update_pool_metrics().await;
    Ok( cleaned )
  }

  /// Start the streaming manager and optimized pool
  pub async fn start_manager( &self ) -> Result< (), Error >
  {
    self.manager.start().await?;
    self.optimized_pool.start_cleanup_task().await;
    Ok( () )
  }

  /// Stop the streaming manager
  pub async fn stop_manager( &self ) -> Result< (), Error >
  {
    self.manager.stop().await
  }

  /// Get reference to the underlying client
  pub fn get_client( &self ) -> &crate::client::Client
  {
    self.client
  }

  /// Get optimization configuration
  pub fn get_optimization_config( &self ) -> &OptimizedWebSocketConfig
  {
    &self.optimization_config
  }

  /// Update optimization configuration
  pub fn update_optimization_config( &mut self, config : OptimizedWebSocketConfig )
  {
    self.optimization_config = config;
  }

  /// Private helper to update connection benchmark metrics
  async fn update_connection_benchmark( &self, connection_time_ms : f64 )
  {
    if let Ok( mut metrics ) = self.performance_metrics.write()
    {
      let current_avg = metrics.performance_benchmarks.avg_connection_time_ms;
      // Simple exponential moving average
      metrics.performance_benchmarks.avg_connection_time_ms =
        if current_avg == 0.0
        {
          connection_time_ms
        } else {
          current_avg * 0.7 + connection_time_ms * 0.3
        };
    }
  }

  /// Private helper to update pool metrics
  async fn update_pool_metrics( &self )
  {
    if let Ok( mut metrics ) = self.performance_metrics.write()
    {
      metrics.pool_stats = self.optimized_pool.get_stats();
    }
  }
}

/// Enhanced WebSocket stream builder with optimization features
#[ derive( Debug ) ]
pub struct EnhancedWebSocketStreamBuilder< 'a >
{
  /// Reference to the enhanced streaming API
  api : &'a WebSocketStreamingApi< 'a >,
  /// Target endpoint URL
  endpoint : Option< String >,
  /// Stream direction
  direction : StreamDirection,
  /// Basic WebSocket configuration
  basic_config : WebSocketConfig,
  /// Optimization configuration
  optimization_config : OptimizedWebSocketConfig,
  /// Authentication token
  auth_token : Option< String >,
  /// Initial metadata
  metadata : HashMap<  String, String  >,
  /// Auto-reconnect setting
  auto_reconnect : bool,
  /// Use optimized connection pool
  use_optimization : bool,
  /// Message serialization format
  serialization_format : SerializationFormat,
}

impl< 'a > EnhancedWebSocketStreamBuilder< 'a >
{
  /// Create a new enhanced stream builder
  pub fn new( api : &'a WebSocketStreamingApi< 'a > ) -> Self
  {
    Self {
      api,
      endpoint : None,
      direction : StreamDirection::Bidirectional,
      basic_config : WebSocketConfig::default(),
      optimization_config : api.optimization_config.clone(),
      auth_token : None,
      metadata : HashMap::new(),
      auto_reconnect : true,
      use_optimization : true,
      serialization_format : SerializationFormat::BinaryJson,
    }
  }

  /// Set the endpoint URL
  pub fn endpoint( mut self, endpoint : &str ) -> Self
  {
    self.endpoint = Some( endpoint.to_string() );
    self
  }

  /// Set the stream direction
  pub fn direction( mut self, direction : StreamDirection ) -> Self
  {
    self.direction = direction;
    self
  }

  /// Set basic WebSocket configuration
  pub fn basic_config( mut self, config : WebSocketConfig ) -> Self
  {
    self.basic_config = config;
    self
  }

  /// Set optimization configuration
  pub fn optimization_config( mut self, config : OptimizedWebSocketConfig ) -> Self
  {
    self.optimization_config = config;
    self
  }

  /// Set authentication token
  pub fn auth_token( mut self, token : &str ) -> Self
  {
    self.auth_token = Some( token.to_string() );
    self
  }

  /// Add metadata
  pub fn metadata( mut self, key : &str, value : &str ) -> Self
  {
    self.metadata.insert( key.to_string(), value.to_string() );
    self
  }

  /// Set auto-reconnect behavior
  pub fn auto_reconnect( mut self, enabled : bool ) -> Self
  {
    self.auto_reconnect = enabled;
    self
  }

  /// Enable or disable optimization features
  pub fn use_optimization( mut self, enabled : bool ) -> Self
  {
    self.use_optimization = enabled;
    self
  }

  /// Set message serialization format for optimized connections
  pub fn serialization_format( mut self, format : SerializationFormat ) -> Self
  {
    self.serialization_format = format.clone();
    self.optimization_config.message_config.serialization_format = format;
    self
  }

  /// Enable message compression for optimized connections
  pub fn enable_compression( mut self, enabled : bool ) -> Self
  {
    self.optimization_config.message_config.enable_compression = enabled;
    self
  }

  /// Set compression level (1-9, higher = better compression)
  pub fn compression_level( mut self, level : u8 ) -> Self
  {
    self.optimization_config.message_config.compression_level = level.clamp( 1, 9 );
    self
  }

  /// Enable message batching for optimized connections
  pub fn enable_batching( mut self, enabled : bool ) -> Self
  {
    self.optimization_config.message_config.enable_batching = enabled;
    self
  }

  /// Set maximum batch size
  pub fn max_batch_size( mut self, size : usize ) -> Self
  {
    self.optimization_config.message_config.max_batch_size = size;
    self
  }

  /// Set connection pool size
  pub fn pool_size( mut self, max_connections : usize ) -> Self
  {
    self.optimization_config.pool_config.max_connections_per_endpoint = max_connections;
    self
  }

  /// Enable performance monitoring
  pub fn enable_metrics( mut self, enabled : bool ) -> Self
  {
    self.optimization_config.monitoring_config.enable_metrics = enabled;
    self
  }

  /// Build and create the connection
  pub async fn build( self ) -> Result< EnhancedConnectionResult< 'a >, Error >
  {
    let endpoint = self.endpoint.ok_or_else( || Error::InvalidArgument( "Endpoint is required".to_string() ) )?;

    if self.use_optimization
    {
      // Create optimized connection
      let connection = self.api.create_optimized_stream( &endpoint ).await?;

      Ok( EnhancedConnectionResult::Optimized {
        connection,
        api : self.api,
      } )
    } else {
      // Create basic connection
      let session_id = self.api.create_stream_with_config( &endpoint, self.basic_config ).await?;

      Ok( EnhancedConnectionResult::Basic {
        session_id,
        api : self.api,
      } )
    }
  }
}

/// Result of building an enhanced WebSocket connection
#[ derive( Debug ) ]
pub enum EnhancedConnectionResult< 'a >
{
  /// Optimized connection with advanced features
  Optimized {
    /// The optimized connection
    connection : Arc< OptimizedWebSocketConnection >,
    /// Reference to the API for returning the connection to pool
    api : &'a WebSocketStreamingApi< 'a >,
  },
  /// Basic connection for compatibility
  Basic {
    /// Session ID for the basic connection
    session_id : String,
    /// Reference to the API for managing the session
    api : &'a WebSocketStreamingApi< 'a >,
  },
}

impl< 'a > EnhancedConnectionResult< 'a >
{
  /// Send a message through the connection
  pub async fn send_message< T >( &self, message : &T ) -> Result< (), Error >
  where
    T: Serialize,
  {
    match self
    {
      EnhancedConnectionResult::Optimized { connection, .. } => {
        connection.send_message( message ).await
      },
      EnhancedConnectionResult::Basic { session_id, api } => {
        if let Some( session ) = api.get_session( session_id )
        {
          // For basic connections, serialize to WebSocketStreamMessage format
          let json_data = serde_json::to_string( message )
            .map_err( | e | Error::SerializationError( e.to_string() ) )?;
          let ws_message = WebSocketStreamMessage::Data {
            content : json_data,
            message_type : "json".to_string(),
            correlation_id : None,
          };
          session.send_message( ws_message ).await
        } else {
          Err( Error::ServerError( "Session not found".to_string() ) )
        }
      }
    }
  }

  /// Get connection metrics
  pub fn get_metrics( &self ) -> Result< serde_json::Value, Error >
  {
    match self
    {
      EnhancedConnectionResult::Optimized { connection, .. } => {
        let metrics = connection.get_metrics();
        serde_json ::to_value( metrics ).map_err( | e | Error::SerializationError( e.to_string() ) )
      },
      EnhancedConnectionResult::Basic { session_id, api } => {
        if let Some( session ) = api.get_session( session_id )
        {
          let metrics = session.get_metrics();
          serde_json ::to_value( metrics ).map_err( | e | Error::SerializationError( e.to_string() ) )
        } else {
          Err( Error::ServerError( "Session not found".to_string() ) )
        }
      }
    }
  }

  /// Close the connection and return resources to pool if applicable
  pub async fn close( self ) -> Result< (), Error >
  {
    match self
    {
      EnhancedConnectionResult::Optimized { connection, api } => {
        api.return_optimized_connection( connection ).await
      },
      EnhancedConnectionResult::Basic { session_id, api } => {
        api.close_stream( &session_id ).await
      }
    }
  }

  /// Check if this is an optimized connection
  pub fn is_optimized( &self ) -> bool
  {
    matches!( self, EnhancedConnectionResult::Optimized { .. } )
  }
}
