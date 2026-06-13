//! WebSocket protocol types, messages, and state definitions

use serde::{ Deserialize, Serialize };
use std::collections::HashMap;

/// WebSocket stream direction for bidirectional communication
#[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum StreamDirection
{
  /// Data flowing from client to server
  Outbound,
  /// Data flowing from server to client
  Inbound,
  /// Bidirectional data flow
  Bidirectional,
}

/// WebSocket streaming operation control
#[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum StreamControl
{
  /// Start streaming
  Start,
  /// Pause streaming (buffer messages)
  Pause,
  /// Resume streaming from pause
  Resume,
  /// Stop streaming and close connection
  Stop,
  /// Reset streaming state
  Reset,
}

/// Message types for WebSocket communication
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub enum WebSocketStreamMessage
{
  /// Control message for stream management
  Control {
    /// Control command to execute
    command : StreamControl,
    /// Optional metadata for the control command
    metadata : Option< HashMap<  String, String  > >
  },
  /// Data message containing actual content
  Data {
    /// Message content
    content : String,
    /// Type of the message
    message_type : String,
    /// Optional correlation identifier
    correlation_id : Option< String >
  },
  /// Heartbeat/keepalive message
  Heartbeat {
    /// Timestamp when heartbeat was sent
    timestamp : u64
  },
  /// Error message from server or client
  Error {
    /// Error code identifier
    error_code : u32,
    /// Human-readable error message
    message : String,
    /// Optional additional error details
    details : Option< HashMap<  String, String  > >
  },
  /// Authentication message
  Auth {
    /// Authentication token
    token : String,
    /// Optional list of requested scopes
    scope : Option< Vec< String > >
  },
}

/// Stream session state tracking
#[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum StreamSessionState
{
  /// Session is being initialized
  Initializing,
  /// Session is active and streaming
  Active,
  /// Session is paused (buffering)
  Paused,
  /// Session is reconnecting
  Reconnecting,
  /// Session is terminated
  Terminated,
  /// Session encountered an error
  Error,
}

/// Session-specific metrics
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq, Default ) ]
pub struct SessionMetrics
{
  /// Total messages sent in this session
  pub messages_sent : u64,
  /// Total messages received in this session
  pub messages_received : u64,
  /// Total bytes transmitted
  pub bytes_sent : u64,
  /// Total bytes received
  pub bytes_received : u64,
  /// Session uptime in seconds
  pub uptime_seconds : u64,
  /// Number of reconnections in this session
  pub reconnection_count : u32,
  /// Last activity timestamp
  pub last_activity : Option< u64 >,
  /// Error count
  pub error_count : u32,
}

/// Enhanced streaming metrics that combine basic and optimized metrics
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct EnhancedStreamingMetrics
{
  /// Basic WebSocket metrics
  pub basic_metrics : crate::models::websocket_streaming::WebSocketMetrics,
  /// Advanced streaming metrics from optimized implementation
  pub streaming_metrics : crate::models::websocket_streaming_optimized::StreamingMetrics,
  /// Connection pool statistics
  pub pool_stats : crate::models::websocket_streaming_optimized::ConnectionPoolStats,
  /// Performance benchmarks
  pub performance_benchmarks : PerformanceBenchmarks,
}

/// Performance benchmarks for monitoring optimization effectiveness
#[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
pub struct PerformanceBenchmarks
{
  /// Average connection establishment time in milliseconds
  pub avg_connection_time_ms : f64,
  /// Average message serialization time in microseconds
  pub avg_serialization_time_us : f64,
  /// Memory usage optimization ratio (compared to baseline)
  pub memory_optimization_ratio : f64,
  /// CPU usage optimization ratio (compared to baseline)
  pub cpu_optimization_ratio : f64,
  /// Overall performance improvement percentage
  pub performance_improvement_percent : f64,
}

impl Default for EnhancedStreamingMetrics
{
  fn default() -> Self
  {
    use crate::models::websocket_streaming::WebSocketMetrics;
    use crate::models::websocket_streaming_optimized::{ StreamingMetrics, ConnectionPoolStats };

    Self {
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
    }
  }
}

impl Default for PerformanceBenchmarks
{
  fn default() -> Self
  {
    Self {
      avg_connection_time_ms : 0.0,
      avg_serialization_time_us : 0.0,
      memory_optimization_ratio : 1.0,
      cpu_optimization_ratio : 1.0,
      performance_improvement_percent : 0.0,
    }
  }
}
