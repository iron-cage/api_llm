//! Streaming control for fine-grained management of streaming operations.
//!
//! This module provides explicit streaming control following the "Thin Client, Rich API" principle.
//! All streaming control operations are explicit and user-triggered, not automatic behaviors.
//!
//! ## Performance Optimizations
//!
//! This implementation includes several optimizations for production use:
//! - Atomic operations for simple state management to reduce lock contention
//! - Event-driven pause timeout handling instead of polling
//! - Efficient circular buffer for paused data management
//! - Lock-free metrics updates where possible
//! - Optimized resource cleanup with structured cancellation

mod buffer;
mod operations;

use serde::{ Deserialize, Serialize };
use core::time::Duration;
use core::sync::atomic::{ AtomicU64, AtomicUsize, Ordering };

// Re-export buffer types
pub use buffer::BufferStrategy;

// Re-export operation types
pub use operations::
{
  ControllableStream,
  ControllableStreamBuilder,
  StreamingControlApi,
  StreamControlStreamBuilder,
};

#[ cfg( all( feature = "websocket_streaming", feature = "streaming_control" ) ) ]
pub use operations::ControllableWebSocketStream;

/// State of a controllable stream
#[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum StreamState
{
  /// Stream is actively running
  Running,
  /// Stream is paused
  Paused,
  /// Stream has been cancelled
  Cancelled,
  /// Stream completed normally
  Completed,
  /// Stream timed out during pause
  TimedOut,
  /// Stream encountered an error
  Error,
}

impl StreamState
{
  /// Convert StreamState to u8 for atomic operations
  #[ inline ]
  pub( crate ) fn to_u8( &self ) -> u8
  {
    match self
    {
      Self::Running => 0,
      Self::Paused => 1,
      Self::Cancelled => 2,
      Self::Completed => 3,
      Self::TimedOut => 4,
      Self::Error => 5,
    }
  }

  /// Convert u8 back to StreamState from atomic operations
  #[ inline ]
  pub( crate ) fn from_u8( value : u8 ) -> Self
  {
    match value
    {
      0 => Self::Running,
      1 => Self::Paused,
      2 => Self::Cancelled,
      3 => Self::Completed,
      4 => Self::TimedOut,
      5 => Self::Error,
      _ => Self::Error, // Default to error for invalid values
    }
  }
}

/// Level of metrics collection (affects performance)
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum MetricsLevel
{
  /// No metrics collection (fastest)
  None,
  /// Basic metrics only (balanced)
  Basic,
  /// Full metrics collection (most detailed)
  Detailed,
}

/// Configuration for streaming control behavior with performance tuning options
#[ derive( Debug, Clone ) ]
pub struct StreamControlConfig
{
  /// Maximum buffer size for paused streams (in bytes)
  pub buffer_size : usize,
  /// Timeout for paused streams before auto-cancellation
  pub pause_timeout : Duration,
  /// Whether to automatically cleanup resources on completion
  pub auto_cleanup : bool,
  /// Maximum number of chunks to buffer during pause
  pub max_buffered_chunks : usize,
  /// Control operation timeout (how long to wait for control commands to be processed)
  pub control_operation_timeout : Duration,
  /// Buffer management strategy for better memory usage
  pub buffer_strategy : BufferStrategy,
  /// Metrics collection level (affects performance vs observability trade-off)
  pub metrics_level : MetricsLevel,
  /// Whether to use event-driven timeout handling (more efficient)
  pub event_driven_timeouts : bool,
}

impl Default for StreamControlConfig
{
  #[ inline ]
  fn default() -> Self
  {
    Self {
      buffer_size : 1024 * 1024, // 1MB default buffer
      pause_timeout : Duration::from_secs( 300 ), // 5 minutes default
      auto_cleanup : true,
      max_buffered_chunks : 100,
      control_operation_timeout : Duration::from_millis( 100 ), // Fast control response
      buffer_strategy : BufferStrategy::Circular, // More memory efficient
      metrics_level : MetricsLevel::Basic, // Balanced performance/observability
      event_driven_timeouts : true, // More efficient timeout handling
    }
  }
}

/// Builder for creating streaming control configuration
#[ derive( Debug, Clone ) ]
pub struct StreamControlConfigBuilder
{
  config : StreamControlConfig,
}

impl StreamControlConfigBuilder
{
  /// Create a new configuration builder
  #[ inline ]
  #[ must_use ]
  pub fn new() -> Self
  {
    Self {
      config : StreamControlConfig::default(),
    }
  }

  /// Set the buffer size for paused streams
  #[ inline ]
  #[ must_use ]
  pub fn buffer_size( mut self, size : usize ) -> Self
  {
    self.config.buffer_size = size;
    self
  }

  /// Set the pause timeout duration
  #[ inline ]
  #[ must_use ]
  pub fn pause_timeout( mut self, timeout : Duration ) -> Self
  {
    self.config.pause_timeout = timeout;
    self
  }

  /// Enable or disable automatic cleanup
  #[ inline ]
  #[ must_use ]
  pub fn auto_cleanup( mut self, enable : bool ) -> Self
  {
    self.config.auto_cleanup = enable;
    self
  }

  /// Set maximum number of chunks to buffer
  #[ inline ]
  #[ must_use ]
  pub fn max_buffered_chunks( mut self, count : usize ) -> Self
  {
    self.config.max_buffered_chunks = count;
    self
  }

  /// Set control operation timeout
  #[ inline ]
  #[ must_use ]
  pub fn control_operation_timeout( mut self, timeout : Duration ) -> Self
  {
    self.config.control_operation_timeout = timeout;
    self
  }

  /// Set buffer management strategy
  #[ inline ]
  #[ must_use ]
  pub fn buffer_strategy( mut self, strategy : BufferStrategy ) -> Self
  {
    self.config.buffer_strategy = strategy;
    self
  }

  /// Set metrics collection level
  #[ inline ]
  #[ must_use ]
  pub fn metrics_level( mut self, level : MetricsLevel ) -> Self
  {
    self.config.metrics_level = level;
    self
  }

  /// Enable or disable event-driven timeout handling
  #[ inline ]
  #[ must_use ]
  pub fn event_driven_timeouts( mut self, enable : bool ) -> Self
  {
    self.config.event_driven_timeouts = enable;
    self
  }

  /// Build the configuration with validation
  ///
  /// # Errors
  ///
  /// Returns `Error` if:
  /// - Buffer size is 0
  /// - Pause timeout is 0
  /// - Max buffered chunks is 0
  #[ inline ]
  pub fn build( self ) -> Result< StreamControlConfig, crate::error::Error >
  {
    if self.config.buffer_size == 0
    {
      return Err( crate::error::Error::ConfigurationError(
        "Buffer size must be greater than 0".to_string()
      ) );
    }

    if self.config.pause_timeout.is_zero()
    {
      return Err( crate::error::Error::ConfigurationError(
        "Pause timeout must be greater than 0".to_string()
      ) );
    }

    if self.config.max_buffered_chunks == 0
    {
      return Err( crate::error::Error::ConfigurationError(
        "Max buffered chunks must be greater than 0".to_string()
      ) );
    }

    if self.config.control_operation_timeout.is_zero()
    {
      return Err( crate::error::Error::ConfigurationError(
        "Control operation timeout must be greater than 0".to_string()
      ) );
    }

    // Validate chunked buffer strategy has reasonable chunk size
    if let BufferStrategy::Chunked { chunk_size } = self.config.buffer_strategy
    {
      if chunk_size == 0
      {
        return Err( crate::error::Error::ConfigurationError(
          "Chunked buffer strategy chunk size must be greater than 0".to_string()
        ) );
      }
      if chunk_size > self.config.buffer_size
      {
        return Err( crate::error::Error::ConfigurationError(
          "Chunked buffer strategy chunk size cannot exceed total buffer size".to_string()
        ) );
      }
    }

    Ok( self.config )
  }
}

impl Default for StreamControlConfigBuilder
{
  fn default() -> Self
  {
    Self::new()
  }
}

impl StreamControlConfig
{
  /// Create a new configuration builder
  #[ inline ]
  #[ must_use ]
  pub fn builder() -> StreamControlConfigBuilder
  {
    StreamControlConfigBuilder::new()
  }
}

/// Metrics for streaming operations with atomic updates for better performance
#[ derive( Debug ) ]
pub struct StreamMetrics
{
  /// Total number of chunks received
  pub total_chunks : AtomicU64,
  /// Current buffer size in bytes
  pub buffer_size : AtomicUsize,
  /// Total bytes received
  pub bytes_received : AtomicU64,
  /// Number of times stream was paused
  pub pause_count : AtomicU64,
  /// Number of times stream was resumed
  pub resume_count : AtomicU64,
  /// Total number of state changes
  pub state_changes : AtomicU64,
  /// Peak buffer size reached during stream lifetime
  pub peak_buffer_size : AtomicUsize,
  /// Average response time for control operations in microseconds
  pub avg_control_response_time_us : AtomicU64,
  /// Number of control operations performed
  pub control_operations : AtomicU64,
  /// Number of buffer overflows (when pause buffer reached max capacity)
  pub buffer_overflows : AtomicU64,
  /// Number of items sent through the stream
  pub items_sent : AtomicU64,
}

impl StreamMetrics
{
  /// Create new StreamMetrics with all counters initialized to zero
  pub fn new() -> Self
  {
    Self
    {
      total_chunks : AtomicU64::new( 0 ),
      buffer_size : AtomicUsize::new( 0 ),
      bytes_received : AtomicU64::new( 0 ),
      pause_count : AtomicU64::new( 0 ),
      resume_count : AtomicU64::new( 0 ),
      state_changes : AtomicU64::new( 0 ),
      peak_buffer_size : AtomicUsize::new( 0 ),
      avg_control_response_time_us : AtomicU64::new( 0 ),
      control_operations : AtomicU64::new( 0 ),
      buffer_overflows : AtomicU64::new( 0 ),
      items_sent : AtomicU64::new( 0 ),
    }
  }
}

/// Snapshot of metrics for external consumers (non-atomic version)
#[ derive( Debug, Clone ) ]
pub struct StreamMetricsSnapshot
{
  /// Total number of chunks received
  pub total_chunks : u64,
  /// Current buffer size in bytes
  pub buffer_size : usize,
  /// Total bytes received
  pub bytes_received : u64,
  /// Number of times stream was paused
  pub pause_count : u64,
  /// Number of times stream was resumed
  pub resume_count : u64,
  /// Total number of state changes
  pub state_changes : u64,
  /// Peak buffer size reached during stream lifetime
  pub peak_buffer_size : usize,
  /// Average response time for control operations in microseconds
  pub avg_control_response_time_us : u64,
  /// Number of control operations performed
  pub control_operations : u64,
  /// Number of buffer overflows
  pub buffer_overflows : u64,
  /// Number of items sent through the stream
  pub items_sent : u64,
}

impl Default for StreamMetrics
{
  fn default() -> Self
  {
    Self {
      total_chunks : AtomicU64::new( 0 ),
      buffer_size : AtomicUsize::new( 0 ),
      bytes_received : AtomicU64::new( 0 ),
      pause_count : AtomicU64::new( 0 ),
      resume_count : AtomicU64::new( 0 ),
      state_changes : AtomicU64::new( 0 ),
      peak_buffer_size : AtomicUsize::new( 0 ),
      avg_control_response_time_us : AtomicU64::new( 0 ),
      control_operations : AtomicU64::new( 0 ),
      buffer_overflows : AtomicU64::new( 0 ),
      items_sent : AtomicU64::new( 0 ),
    }
  }
}

impl StreamMetrics
{
  /// Create a snapshot of the current metrics
  pub fn snapshot( &self ) -> StreamMetricsSnapshot
  {
    StreamMetricsSnapshot {
      total_chunks : self.total_chunks.load( Ordering::Relaxed ),
      buffer_size : self.buffer_size.load( Ordering::Relaxed ),
      bytes_received : self.bytes_received.load( Ordering::Relaxed ),
      pause_count : self.pause_count.load( Ordering::Relaxed ),
      resume_count : self.resume_count.load( Ordering::Relaxed ),
      state_changes : self.state_changes.load( Ordering::Relaxed ),
      peak_buffer_size : self.peak_buffer_size.load( Ordering::Relaxed ),
      avg_control_response_time_us : self.avg_control_response_time_us.load( Ordering::Relaxed ),
      control_operations : self.control_operations.load( Ordering::Relaxed ),
      buffer_overflows : self.buffer_overflows.load( Ordering::Relaxed ),
      items_sent : self.items_sent.load( Ordering::Relaxed ),
    }
  }
}
