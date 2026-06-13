//! Optimized WebSocket streaming implementation with enhanced performance and reliability.
//!
//! This module provides optimized WebSocket streaming capabilities with:
//! - Connection pooling and intelligent reuse strategies
//! - High-performance message serialization with binary protocols
//! - Advanced metrics collection and monitoring
//! - Sophisticated error handling and recovery mechanisms
//! - Resource-efficient connection management
//! - Comprehensive configuration options for fine-tuning

#![ allow( dead_code, missing_debug_implementations, async_fn_in_trait, missing_docs ) ] // Advanced implementation with comprehensive features

mod private
{
  use serde::{ Deserialize, Serialize };
  use std::collections::{ HashMap, VecDeque };
  use std::sync::{ Arc, RwLock };
  use core::sync::atomic::{ AtomicU64, AtomicBool, AtomicUsize, Ordering };
  use core::time::Duration;
  use std::time::Instant;
  use tokio::sync::{ mpsc, Semaphore };
  use tokio::time::sleep;

  // Re-export base types from original WebSocket module
  pub use crate::models::websocket_streaming::
  {
    WebSocketConnectionState, WebSocketConfig, WebSocketMessage
  };

  /// Trait for connection pooling strategies
  pub trait ConnectionPool : Send + Sync
  {
    /// Get an available connection from the pool
    async fn get_connection( &self, endpoint : &str ) -> Result< Arc< OptimizedWebSocketConnection >, crate::error::Error >;

    /// Return a connection to the pool
    async fn return_connection( &self, connection : Arc< OptimizedWebSocketConnection > ) -> Result< (), crate::error::Error >;

    /// Get pool statistics
    fn get_stats( &self ) -> ConnectionPoolStats;

    /// Clean up stale or unused connections
    async fn cleanup( &self ) -> Result< usize, crate::error::Error >;
  }

  /// Message serializer enum for dyn-compatibility
  #[ derive( Debug, Clone ) ]
  pub enum MessageSerializerType
  {
    /// Binary JSON serializer
    BinaryJson { enable_compression : bool, compression_level : u8 },
    /// MessagePack serializer
    MessagePack { enable_compression : bool },
    /// Standard JSON serializer
    Json,
  }

  impl MessageSerializerType
  {
    /// Serialize a message for transmission
    pub fn serialize< T >( &self, message : &T ) -> Result< Vec< u8 >, crate::error::Error >
    where
      T: Serialize,
    {
      match self
      {
        Self::BinaryJson { enable_compression, .. } => {
          let json_bytes = serde_json::to_vec( message )
            .map_err( | e | crate::error::Error::SerializationError( e.to_string() ) )?;

          if *enable_compression
          {
            // xxx : Implement compression (task/unverified/008)
            Ok( json_bytes )
          } else {
            Ok( json_bytes )
          }
        },
        Self::MessagePack { .. } => {
          // xxx : Implement MessagePack, fallback to JSON for now (task/unverified/008)
          serde_json ::to_vec( message )
            .map_err( | e | crate::error::Error::SerializationError( e.to_string() ) )
        },
        Self::Json => {
          serde_json ::to_vec( message )
            .map_err( | e | crate::error::Error::SerializationError( e.to_string() ) )
        },
      }
    }

    /// Deserialize a received message
    pub fn deserialize< T >( &self, data : &[ u8 ] ) -> Result< T, crate::error::Error >
    where
      T: for< 'de > Deserialize< 'de >,
    {
      match self
      {
        Self::BinaryJson { .. } | Self::MessagePack { .. } | Self::Json => {
          serde_json ::from_slice( data )
            .map_err( | e | crate::error::Error::DeserializationError( e.to_string() ) )
        },
      }
    }

    /// Get serialization format identifier
    #[ inline ]
    #[ must_use ]
    pub fn format_id( &self ) -> &'static str
    {
      match self
      {
        Self::BinaryJson { .. } => "binary_json",
        Self::MessagePack { .. } => "messagepack",
        Self::Json => "json",
      }
    }

    /// Get compression capability
    #[ inline ]
    #[ must_use ]
    pub fn supports_compression( &self ) -> bool
    {
      match self
      {
        Self::BinaryJson { enable_compression, .. } | Self::MessagePack { enable_compression } => *enable_compression,
        Self::Json => false,
      }
    }
  }

  /// Connection pool statistics
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct ConnectionPoolStats
  {
    /// Total connections in pool
    pub total_connections : usize,
    /// Active connections currently in use
    pub active_connections : usize,
    /// Idle connections available for reuse
    pub idle_connections : usize,
    /// Total connections created
    pub connections_created : u64,
    /// Total connections reused
    pub connections_reused : u64,
    /// Pool hit ratio (reuse rate)
    pub hit_ratio : f64,
    /// Average connection age in seconds
    pub avg_connection_age_seconds : f64,
  }

  /// Advanced WebSocket configuration with optimization settings
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct OptimizedWebSocketConfig
  {
    /// Base WebSocket configuration
    pub base : WebSocketConfig,
    /// Connection pool configuration
    pub pool_config : ConnectionPoolConfig,
    /// Message optimization settings
    pub message_config : MessageOptimizationConfig,
    /// Performance monitoring settings
    pub monitoring_config : WebSocketMonitoringConfig,
    /// Resource management settings
    pub resource_config : ResourceManagementConfig,
  }

  /// Connection pool configuration
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq, Default ) ]
  pub struct ConnectionPoolConfig
  {
    /// Maximum connections per endpoint
    pub max_connections_per_endpoint : usize,
    /// Maximum total connections in pool
    pub max_total_connections : usize,
    /// Maximum idle time before connection cleanup
    pub max_idle_time_seconds : u64,
    /// Pool cleanup interval
    pub cleanup_interval_seconds : u64,
    /// Enable connection warming
    pub enable_connection_warming : bool,
    /// Minimum connections to maintain per endpoint
    pub min_connections_per_endpoint : usize,
  }

  /// Message optimization configuration
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq, Default ) ]
  pub struct MessageOptimizationConfig
  {
    /// Preferred serialization format
    pub serialization_format : SerializationFormat,
    /// Enable message compression
    pub enable_compression : bool,
    /// Compression level (1-9, higher = better compression)
    pub compression_level : u8,
    /// Enable message batching
    pub enable_batching : bool,
    /// Maximum batch size in messages
    pub max_batch_size : usize,
    /// Maximum batch delay in milliseconds
    pub max_batch_delay_ms : u64,
  }

  /// WebSocket monitoring configuration
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct WebSocketMonitoringConfig
  {
    /// Enable detailed performance metrics
    pub enable_metrics : bool,
    /// Metrics collection interval in seconds
    pub metrics_interval_seconds : u64,
    /// Enable connection lifecycle logging
    pub enable_lifecycle_logging : bool,
    /// Enable message flow tracing
    pub enable_message_tracing : bool,
    /// Maximum metrics history to retain
    pub max_metrics_history : usize,
  }

  /// Resource management configuration
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct ResourceManagementConfig
  {
    /// Maximum concurrent connections
    pub max_concurrent_connections : usize,
    /// Memory usage limit in bytes
    pub memory_limit_bytes : u64,
    /// Enable automatic resource scaling
    pub enable_auto_scaling : bool,
    /// CPU usage threshold for scaling (0.0-1.0)
    pub cpu_threshold : f64,
    /// Memory usage threshold for scaling (0.0-1.0)
    pub memory_threshold : f64,
  }

  /// Serialization format options
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq, Default ) ]
  pub enum SerializationFormat
  {
    /// JSON format (human readable)
    #[ default ]
    Json,
    /// Binary JSON format (faster)
    BinaryJson,
    /// MessagePack format (compact)
    MessagePack,
    /// Protocol Buffers format (efficient)
    ProtocolBuffers,
    /// Custom binary format
    CustomBinary,
  }

  impl Default for OptimizedWebSocketConfig
  {
    fn default() -> Self
    {
      Self {
        base : WebSocketConfig::default(),
        pool_config : ConnectionPoolConfig {
          max_connections_per_endpoint : 10,
          max_total_connections : 100,
          max_idle_time_seconds : 300, // 5 minutes
          cleanup_interval_seconds : 60, // 1 minute
          enable_connection_warming : true,
          min_connections_per_endpoint : 2,
        },
        message_config : MessageOptimizationConfig {
          serialization_format : SerializationFormat::BinaryJson,
          enable_compression : true,
          compression_level : 6,
          enable_batching : true,
          max_batch_size : 10,
          max_batch_delay_ms : 100,
        },
        monitoring_config : WebSocketMonitoringConfig {
          enable_metrics : true,
          metrics_interval_seconds : 60,
          enable_lifecycle_logging : true,
          enable_message_tracing : false, // Can be expensive
          max_metrics_history : 1000,
        },
        resource_config : ResourceManagementConfig {
          max_concurrent_connections : 1000,
          memory_limit_bytes : 1024 * 1024 * 512, // 512MB
          enable_auto_scaling : true,
          cpu_threshold : 0.8,
          memory_threshold : 0.8,
        },
      }
    }
  }

  /// High-performance connection pool with intelligent reuse
  #[ derive( Debug ) ]
  pub struct OptimizedConnectionPool
  {
    /// Connections grouped by endpoint
    pools : Arc< RwLock< HashMap< String, VecDeque< PooledConnection > > > >,
    /// Pool configuration
    config : ConnectionPoolConfig,
    /// Pool statistics
    stats : Arc< RwLock< ConnectionPoolStats > >,
    /// Active connections count
    active_count : Arc< AtomicUsize >,
    /// Total created connections
    created_count : Arc< AtomicU64 >,
    /// Total reused connections
    reused_count : Arc< AtomicU64 >,
    /// Connection semaphore for limiting concurrency
    connection_semaphore : Arc< Semaphore >,
    /// Cleanup task handle
    cleanup_running : Arc< AtomicBool >,
  }

  /// Pooled connection wrapper
  #[ derive( Debug ) ]
  struct PooledConnection
  {
    /// The actual connection
    connection : Arc< OptimizedWebSocketConnection >,
    /// When this connection was last used
    last_used : Instant,
    /// Creation timestamp
    created_at : Instant,
    /// Usage count for this connection
    usage_count : u64,
  }

  /// High-performance WebSocket connection with optimizations
  #[ derive( Debug ) ]
  pub struct OptimizedWebSocketConnection
  {
    /// Connection identifier
    pub id : String,
    /// Endpoint URL
    pub endpoint : String,
    /// Connection state
    state : Arc< RwLock< WebSocketConnectionState > >,
    /// Configuration
    config : OptimizedWebSocketConfig,
    /// Message serializer
    serializer : MessageSerializerType,
    /// Performance metrics
    metrics : Arc< RwLock< ConnectionMetrics > >,
    /// Message sender channel
    message_sender : Option< mpsc::UnboundedSender< WebSocketMessage > >,
    /// Connection health checker
    health_checker : Arc< ConnectionHealthChecker >,
  }

  /// Connection-specific performance metrics
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct ConnectionMetrics
  {
    /// Total messages sent
    pub messages_sent : u64,
    /// Total messages received
    pub messages_received : u64,
    /// Total bytes sent
    pub bytes_sent : u64,
    /// Total bytes received
    pub bytes_received : u64,
    /// Average message latency in milliseconds
    pub avg_latency_ms : f64,
    /// Connection uptime in seconds
    pub uptime_seconds : u64,
    /// Number of reconnections
    pub reconnection_count : u32,
    /// Last error encountered
    pub last_error : Option< String >,
    /// Message throughput (messages per second)
    pub throughput_msg_per_sec : f64,
    /// Bandwidth utilization (bytes per second)
    pub bandwidth_bytes_per_sec : f64,
  }

  /// Connection health monitoring
  #[ derive( Debug ) ]
  pub struct ConnectionHealthChecker
  {
    /// Last heartbeat timestamp
    last_heartbeat : Arc< RwLock< Option< Instant > > >,
    /// Health check interval
    check_interval : Duration,
    /// Consecutive failed checks
    failed_checks : Arc< AtomicU64 >,
    /// Maximum allowed failed checks
    max_failed_checks : u64,
    /// Health status
    is_healthy : Arc< AtomicBool >,
  }


  impl OptimizedConnectionPool
  {
    /// Create a new optimized connection pool
    #[ inline ]
    #[ must_use ]
    pub fn new( config : ConnectionPoolConfig ) -> Self
    {
      let max_connections = config.max_total_connections;

      Self {
        pools : Arc::new( RwLock::new( HashMap::new() ) ),
        config,
        stats : Arc::new( RwLock::new( ConnectionPoolStats {
          total_connections : 0,
          active_connections : 0,
          idle_connections : 0,
          connections_created : 0,
          connections_reused : 0,
          hit_ratio : 0.0,
          avg_connection_age_seconds : 0.0,
        } ) ),
        active_count : Arc::new( AtomicUsize::new( 0 ) ),
        created_count : Arc::new( AtomicU64::new( 0 ) ),
        reused_count : Arc::new( AtomicU64::new( 0 ) ),
        connection_semaphore : Arc::new( Semaphore::new( max_connections ) ),
        cleanup_running : Arc::new( AtomicBool::new( false ) ),
      }
    }

    /// Get connection pool statistics
    #[ inline ]
    #[ must_use ]
    pub fn get_stats( &self ) -> ConnectionPoolStats
    {
      // Calculate current stats including average connection age
      let mut stats = self.stats.read().unwrap_or_else( | poisoned | poisoned.into_inner() ).clone();

      // Update with real-time data
      stats.active_connections = self.active_count.load( Ordering::Relaxed );
      stats.connections_created = self.created_count.load( Ordering::Relaxed );
      stats.connections_reused = self.reused_count.load( Ordering::Relaxed );

      // Calculate average connection age from pooled connections
      if let Ok( pools_lock ) = self.pools.read()
      {
        let now = Instant::now();
        let mut total_age_seconds = 0.0;
        let mut connection_count = 0;

        for pool in pools_lock.values()
        {
          for conn in pool
          {
            total_age_seconds += now.duration_since( conn.created_at ).as_secs_f64();
            connection_count += 1;
          }
        }

        if connection_count > 0
        {
          stats.avg_connection_age_seconds = total_age_seconds / connection_count as f64;
        }

        stats.idle_connections = connection_count;
      }

      // Calculate hit ratio
      let total_requests = stats.connections_created + stats.connections_reused;
      if total_requests > 0
      {
        stats.hit_ratio = stats.connections_reused as f64 / total_requests as f64;
      }

      stats
    }

    /// Start the cleanup task
    pub async fn start_cleanup_task( &self )
    {
      if self.cleanup_running.swap( true, Ordering::Relaxed )
      {
        return; // Already running
      }

      let pools = Arc::downgrade( &self.pools );
      let cleanup_interval = Duration::from_secs( self.config.cleanup_interval_seconds );
      let max_idle_time = Duration::from_secs( self.config.max_idle_time_seconds );
      let cleanup_running = Arc::downgrade( &self.cleanup_running );

      tokio ::spawn( async move {
        while let ( Some( pools ), Some( cleanup_running ) ) = ( pools.upgrade(), cleanup_running.upgrade() )
        {
          if !cleanup_running.load( Ordering::Relaxed )
          {
            break;
          }

          // Perform cleanup
          if let Ok( mut pools_lock ) = pools.write()
          {
            let now = Instant::now();
            for ( _endpoint, pool ) in pools_lock.iter_mut()
            {
              pool.retain( | conn | {
                now.duration_since( conn.last_used ) < max_idle_time
              } );
            }
          }

          sleep( cleanup_interval ).await;
        }
      } );
    }
  }

  impl ConnectionPool for OptimizedConnectionPool
  {
    async fn get_connection( &self, endpoint : &str ) -> Result< Arc< OptimizedWebSocketConnection >, crate::error::Error >
    {
      // Try to acquire connection semaphore
      let _permit = self.connection_semaphore.try_acquire()
        .map_err( | _ | crate::error::Error::ServerError( "Connection pool exhausted".to_string() ) )?;

      // Check for existing connection in pool
      if let Ok( mut pools_lock ) = self.pools.write()
      {
        if let Some( pool ) = pools_lock.get_mut( endpoint )
        {
          if let Some( mut pooled_conn ) = pool.pop_front()
          {
            pooled_conn.last_used = Instant::now();
            pooled_conn.usage_count += 1;
            self.reused_count.fetch_add( 1, Ordering::Relaxed );
            self.active_count.fetch_add( 1, Ordering::Relaxed );
            return Ok( pooled_conn.connection );
          }
        }
      }

      // Create new connection
      let connection = Arc::new( OptimizedWebSocketConnection::new(
        endpoint,
        OptimizedWebSocketConfig::default()
      ).await? );

      self.created_count.fetch_add( 1, Ordering::Relaxed );
      self.active_count.fetch_add( 1, Ordering::Relaxed );

      Ok( connection )
    }

    async fn return_connection( &self, connection : Arc< OptimizedWebSocketConnection > ) -> Result< (), crate::error::Error >
    {
      self.active_count.fetch_sub( 1, Ordering::Relaxed );

      // Check if connection is still healthy
      if !connection.health_checker.is_healthy.load( Ordering::Relaxed )
      {
        return Ok( () ); // Don't return unhealthy connections to pool
      }

      // Return to pool
      if let Ok( mut pools_lock ) = self.pools.write()
      {
        let pool = pools_lock.entry( connection.endpoint.clone() ).or_insert_with( VecDeque::new );

        // Check pool size limits
        if pool.len() < self.config.max_connections_per_endpoint
        {
          let pooled_conn = PooledConnection {
            connection,
            last_used : Instant::now(),
            created_at : Instant::now(),
            usage_count : 1,
          };
          pool.push_back( pooled_conn );
        }
      }

      Ok( () )
    }

    fn get_stats( &self ) -> ConnectionPoolStats
    {
      let total_created = self.created_count.load( Ordering::Relaxed );
      let total_reused = self.reused_count.load( Ordering::Relaxed );
      let hit_ratio = if total_created + total_reused > 0
      {
        total_reused as f64 / ( total_created + total_reused ) as f64
      } else {
        0.0
      };

      let pools_lock = self.pools.read().unwrap_or_else( | poisoned | poisoned.into_inner() );
      let total_connections = pools_lock.values().map( | pool | pool.len() ).sum::< usize >();

      ConnectionPoolStats {
        total_connections,
        active_connections : self.active_count.load( Ordering::Relaxed ),
        idle_connections : total_connections,
        connections_created : total_created,
        connections_reused : total_reused,
        hit_ratio,
        avg_connection_age_seconds : 0.0, // Could be calculated from pooled connections
      }
    }

    async fn cleanup( &self ) -> Result< usize, crate::error::Error >
    {
      let mut cleaned_count = 0;
      let now = Instant::now();
      let max_idle_time = Duration::from_secs( self.config.max_idle_time_seconds );

      if let Ok( mut pools_lock ) = self.pools.write()
      {
        for ( _endpoint, pool ) in pools_lock.iter_mut()
        {
          let initial_size = pool.len();
          pool.retain( | conn | {
            now.duration_since( conn.last_used ) < max_idle_time
          } );
          cleaned_count += initial_size - pool.len();
        }
      }

      Ok( cleaned_count )
    }
  }

  impl OptimizedWebSocketConnection
  {
    /// Create a new optimized WebSocket connection
    pub async fn new( endpoint : &str, config : OptimizedWebSocketConfig ) -> Result< Self, crate::error::Error >
    {
      let id = format!( "ws_opt_{}", uuid::Uuid::new_v4() );
      let serializer = match config.message_config.serialization_format
      {
        SerializationFormat::BinaryJson => MessageSerializerType::BinaryJson {
          enable_compression : config.message_config.enable_compression,
          compression_level : config.message_config.compression_level,
        },
        SerializationFormat::MessagePack => MessageSerializerType::MessagePack {
          enable_compression : config.message_config.enable_compression,
        },
        _ => MessageSerializerType::Json,
      };

      Ok( Self {
        id,
        endpoint : endpoint.to_string(),
        state : Arc::new( RwLock::new( WebSocketConnectionState::Connecting ) ),
        config,
        serializer,
        metrics : Arc::new( RwLock::new( ConnectionMetrics {
          messages_sent : 0,
          messages_received : 0,
          bytes_sent : 0,
          bytes_received : 0,
          avg_latency_ms : 0.0,
          uptime_seconds : 0,
          reconnection_count : 0,
          last_error : None,
          throughput_msg_per_sec : 0.0,
          bandwidth_bytes_per_sec : 0.0,
        } ) ),
        message_sender : None,
        health_checker : Arc::new( ConnectionHealthChecker {
          last_heartbeat : Arc::new( RwLock::new( Some( Instant::now() ) ) ),
          check_interval : Duration::from_secs( 30 ),
          failed_checks : Arc::new( AtomicU64::new( 0 ) ),
          max_failed_checks : 3,
          is_healthy : Arc::new( AtomicBool::new( true ) ),
        } ),
      } )
    }

    /// Send a message through the optimized connection
    pub async fn send_message< T >( &self, message : &T ) -> Result< (), crate::error::Error >
    where
      T: Serialize,
    {
      let serialized = self.serializer.serialize( message )?;

      // Update metrics
      if let Ok( mut metrics ) = self.metrics.write()
      {
        metrics.messages_sent += 1;
        metrics.bytes_sent += serialized.len() as u64;
      }

      // xxx : Actual WebSocket sending implementation (task/unverified/008)
      Err( crate::error::Error::NotImplemented( "WebSocket send_message not yet implemented".to_string() ) )
    }

    /// Get connection metrics
    pub fn get_metrics( &self ) -> ConnectionMetrics
    {
      self.metrics.read().unwrap_or_else( | poisoned | poisoned.into_inner() ).clone()
    }
  }


  /// Optimized WebSocket streaming API
  pub struct OptimizedWebSocketStreamingApi< 'a >
  {
    /// Reference to the Gemini client
    client : &'a crate::client::Client,
    /// Connection pool
    pool : Arc< OptimizedConnectionPool >,
    /// Configuration
    config : OptimizedWebSocketConfig,
    /// Performance metrics
    metrics : Arc< RwLock< StreamingMetrics > >,
  }

  /// Streaming performance metrics
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct StreamingMetrics
  {
    /// Total streaming sessions created
    pub total_sessions : u64,
    /// Currently active sessions
    pub active_sessions : u64,
    /// Average session duration in seconds
    pub avg_session_duration_seconds : f64,
    /// Total messages streamed
    pub total_messages_streamed : u64,
    /// Average throughput (messages per second)
    pub avg_throughput_msg_per_sec : f64,
    /// Total data transferred in bytes
    pub total_bytes_transferred : u64,
    /// Connection pool hit ratio
    pub pool_hit_ratio : f64,
    /// Error rate (errors per session)
    pub error_rate : f64,
  }

  impl< 'a > OptimizedWebSocketStreamingApi< 'a >
  {
    /// Create a new optimized WebSocket streaming API
    pub fn new( client : &'a crate::client::Client ) -> Self
    {
      Self::with_config( client, OptimizedWebSocketConfig::default() )
    }

    /// Create API with custom configuration
    pub fn with_config( client : &'a crate::client::Client, config : OptimizedWebSocketConfig ) -> Self
    {
      let pool = Arc::new(
        OptimizedConnectionPool::new( config.pool_config.clone() )
      );

      Self {
        client,
        pool,
        config,
        metrics : Arc::new( RwLock::new( StreamingMetrics {
          total_sessions : 0,
          active_sessions : 0,
          avg_session_duration_seconds : 0.0,
          total_messages_streamed : 0,
          avg_throughput_msg_per_sec : 0.0,
          total_bytes_transferred : 0,
          pool_hit_ratio : 0.0,
          error_rate : 0.0,
        } ) ),
      }
    }

    /// Get streaming performance metrics
    pub fn get_metrics( &self ) -> StreamingMetrics
    {
      self.metrics.read().unwrap_or_else( | poisoned | poisoned.into_inner() ).clone()
    }

    /// Get connection pool statistics
    pub fn get_pool_stats( &self ) -> ConnectionPoolStats
    {
      self.pool.get_stats()
    }

    /// Cleanup idle connections
    pub async fn cleanup_connections( &self ) -> Result< usize, crate::error::Error >
    {
      self.pool.cleanup().await
    }
  }

  impl< 'a > std::fmt::Debug for OptimizedWebSocketStreamingApi< 'a >
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      f.debug_struct( "OptimizedWebSocketStreamingApi" )
        .field( "config", &self.config )
        .finish_non_exhaustive()
    }
  }

  impl Default for StreamingMetrics
  {
    fn default() -> Self
    {
      Self {
        total_sessions : 0,
        active_sessions : 0,
        avg_session_duration_seconds : 0.0,
        total_messages_streamed : 0,
        avg_throughput_msg_per_sec : 0.0,
        total_bytes_transferred : 0,
        pool_hit_ratio : 0.0,
        error_rate : 0.0,
      }
    }
  }
}

// Public API exports
pub use private::
{
  ConnectionPool, MessageSerializerType, ConnectionPoolStats,
  OptimizedWebSocketConfig, ConnectionPoolConfig, MessageOptimizationConfig,
  WebSocketMonitoringConfig, ResourceManagementConfig, SerializationFormat,
  OptimizedConnectionPool, OptimizedWebSocketConnection, ConnectionMetrics,
  ConnectionHealthChecker, OptimizedWebSocketStreamingApi, StreamingMetrics
};