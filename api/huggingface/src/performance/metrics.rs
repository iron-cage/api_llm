//! Performance Metrics Core Implementation
//!
//! Provides request latency tracking, throughput metrics, and error rate monitoring.

use core::time::Duration;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Configuration for performance metrics
#[ derive( Debug, Clone ) ]
pub struct MetricsConfig
{
  /// Maximum number of latency samples to retain
  pub max_samples : usize,
  /// Window duration for recent metrics ( e.g., last 60 seconds )
  pub window_duration : Duration,
}

impl Default for MetricsConfig
{
  #[ inline ]
  fn default() -> Self
  {
  Self
  {
      max_samples : 10000,
      window_duration : Duration::from_secs( 60 ),
  }
  }
}

/// Individual request measurement
#[ derive( Debug, Clone ) ]
struct RequestMeasurement
{
  timestamp : Instant,
  latency : Duration,
  success : bool,
  bytes_transferred : u64,
}

/// Latency statistics
#[ derive( Debug, Clone, Copy, Default ) ]
pub struct LatencyStats
{
  /// Minimum latency
  pub min : Duration,
  /// Maximum latency
  pub max : Duration,
  /// Mean latency
  pub mean : Duration,
  /// Median latency ( p50 )
  pub p50 : Duration,
  /// 95th percentile latency
  pub p95 : Duration,
  /// 99th percentile latency
  pub p99 : Duration,
}

/// Snapshot of metrics at a point in time
#[ derive( Debug, Clone ) ]
pub struct MetricsSnapshot
{
  /// Total number of requests
  pub total_requests : u64,
  /// Number of successful requests
  pub successful_requests : u64,
  /// Number of failed requests
  pub failed_requests : u64,
  /// Total bytes transferred
  pub total_bytes : u64,
  /// Latency statistics
  pub latency : LatencyStats,
  /// Timestamp when snapshot was taken
  pub snapshot_time : Instant,
  /// Duration covered by this snapshot
  pub duration : Duration,
}

impl MetricsSnapshot
{
  /// Calculate error rate ( 0.0 - 1.0 )
  #[ inline ]
  #[ must_use ]
  pub fn error_rate( &self ) -> f64
  {
  if self.total_requests == 0
  {
      0.0
  }
  else
  {
      self.failed_requests as f64 / self.total_requests as f64
  }
  }

  /// Calculate success rate ( 0.0 - 1.0 )
  #[ inline ]
  #[ must_use ]
  pub fn success_rate( &self ) -> f64
  {
  if self.total_requests == 0
  {
      0.0
  }
  else
  {
      self.successful_requests as f64 / self.total_requests as f64
  }
  }

  /// Calculate requests per second
  #[ inline ]
  #[ must_use ]
  pub fn requests_per_second( &self ) -> f64
  {
  let duration_secs = self.duration.as_secs_f64( );
  if duration_secs == 0.0
  {
      0.0
  }
  else
  {
      self.total_requests as f64 / duration_secs
  }
  }

  /// Calculate bytes per second
  #[ inline ]
  #[ must_use ]
  pub fn bytes_per_second( &self ) -> f64
  {
  let duration_secs = self.duration.as_secs_f64( );
  if duration_secs == 0.0
  {
      0.0
  }
  else
  {
      self.total_bytes as f64 / duration_secs
  }
  }

  /// Get mean latency as Duration
  #[ inline ]
  #[ must_use ]
  pub fn mean_latency( &self ) -> Duration
  {
  self.latency.mean
  }

  /// Get median latency ( p50 ) as Duration
  #[ inline ]
  #[ must_use ]
  pub fn median_latency( &self ) -> Duration
  {
  self.latency.p50
  }

  /// Get p95 latency as Duration
  #[ inline ]
  #[ must_use ]
  pub fn p95_latency( &self ) -> Duration
  {
  self.latency.p95
  }

  /// Get p99 latency as Duration
  #[ inline ]
  #[ must_use ]
  pub fn p99_latency( &self ) -> Duration
  {
  self.latency.p99
  }

  /// Get minimum latency as Duration
  #[ inline ]
  #[ must_use ]
  pub fn min_latency( &self ) -> Duration
  {
  self.latency.min
  }

  /// Get maximum latency as Duration
  #[ inline ]
  #[ must_use ]
  pub fn max_latency( &self ) -> Duration
  {
  self.latency.max
  }
}

/// Internal metrics state
struct MetricsState
{
  config : MetricsConfig,
  measurements : Vec< RequestMeasurement >,
  start_time : Instant,
}

impl MetricsState
{
  fn new( config : MetricsConfig ) -> Self
  {
  Self
  {
      config,
      measurements : Vec::new( ),
      start_time : Instant::now( ),
  }
  }

  /// Remove measurements outside the time window
  fn cleanup_old_measurements( &mut self )
  {
  let now = Instant::now( );
  let cutoff = now.checked_sub( self.config.window_duration )
      .unwrap_or( Instant::now( ) );

  self.measurements.retain( | m | m.timestamp >= cutoff );

  // Also enforce max_samples limit
  if self.measurements.len( ) > self.config.max_samples
  {
      let excess = self.measurements.len( ) - self.config.max_samples;
      self.measurements.drain( 0..excess );
  }
  }

  /// Calculate latency statistics from measurements
  fn calculate_latency_stats( &self ) -> LatencyStats
  {
  if self.measurements.is_empty( )
  {
      return LatencyStats::default( );
  }

  let mut latencies : Vec< Duration > = self.measurements
      .iter( )
      .map( | m | m.latency )
      .collect( );

  latencies.sort( );

  let min = latencies.first( ).copied( ).unwrap_or_default( );
  let max = latencies.last( ).copied( ).unwrap_or_default( );

  let sum : Duration = latencies.iter( ).sum( );
  #[ allow( clippy::cast_possible_truncation ) ]
  let mean = sum / latencies.len( ) as u32;

  let p50 = Self::percentile( &latencies, 0.50 );
  let p95 = Self::percentile( &latencies, 0.95 );
  let p99 = Self::percentile( &latencies, 0.99 );

  LatencyStats
  {
      min,
      max,
      mean,
      p50,
      p95,
      p99,
  }
  }

  /// Calculate percentile from sorted latencies
  fn percentile( sorted_latencies : &[ Duration ], percentile : f64 ) -> Duration
  {
  if sorted_latencies.is_empty( )
  {
      return Duration::default( );
  }

  #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
  let index = ( sorted_latencies.len( ) as f64 * percentile ) as usize;
  let index = index.min( sorted_latencies.len( ) - 1 );
  sorted_latencies[ index ]
  }

  /// Generate metrics snapshot
  fn snapshot( &self ) -> MetricsSnapshot
  {
  let now = Instant::now( );
  let duration = now.duration_since( self.start_time );

  let total_requests = self.measurements.len( ) as u64;
  let successful_requests = self.measurements.iter( ).filter( | m | m.success ).count( ) as u64;
  let failed_requests = total_requests - successful_requests;
  let total_bytes = self.measurements.iter( ).map( | m | m.bytes_transferred ).sum( );

  let latency = self.calculate_latency_stats( );

  MetricsSnapshot
  {
      total_requests,
      successful_requests,
      failed_requests,
      total_bytes,
      latency,
      snapshot_time : now,
      duration,
  }
  }
}

/// Performance metrics tracker
#[ derive( Clone ) ]
pub struct PerformanceMetrics
{
  state : Arc< RwLock< MetricsState > >,
}

impl PerformanceMetrics
{
  /// Create new performance metrics tracker
  #[ inline ]
  #[ must_use ]
  pub fn new( config : MetricsConfig ) -> Self
  {
  Self
  {
      state : Arc::new( RwLock::new( MetricsState::new( config ) ) ),
  }
  }

  /// Record a request measurement
  ///
  /// # Arguments
  ///
  /// * `latency` - Request duration
  /// * `success` - Whether request succeeded
  /// * `bytes_transferred` - Number of bytes transferred
  #[ inline ]
  pub async fn record_request( &self, latency : Duration, success : bool, bytes_transferred : u64 )
  {
  let mut state = self.state.write( ).await;

  state.measurements.push( RequestMeasurement
  {
      timestamp : Instant::now( ),
      latency,
      success,
      bytes_transferred,
  } );

  // Periodically clean up old measurements
  if state.measurements.len( ) % 100 == 0
  {
      state.cleanup_old_measurements( );
  }
  }

  /// Get current metrics snapshot
  #[ inline ]
  pub async fn snapshot( &self ) -> MetricsSnapshot
  {
  let state = self.state.read( ).await;
  state.snapshot( )
  }

  /// Get metrics for a specific time window
  ///
  /// Returns snapshot containing only measurements within the specified duration.
  #[ inline ]
  pub async fn snapshot_window( &self, window : Duration ) -> MetricsSnapshot
  {
  let state = self.state.read( ).await;
  let now = Instant::now( );
  let cutoff = now.checked_sub( window )
      .unwrap_or( Instant::now( ) );

  // Create temporary state with filtered measurements
  let filtered : Vec< RequestMeasurement > = state.measurements
      .iter( )
      .filter( | m | m.timestamp >= cutoff )
      .cloned( )
      .collect( );

  let total_requests = filtered.len( ) as u64;
  let successful_requests = filtered.iter( ).filter( | m | m.success ).count( ) as u64;
  let failed_requests = total_requests - successful_requests;
  let total_bytes = filtered.iter( ).map( | m | m.bytes_transferred ).sum( );

  // Calculate latency stats from filtered measurements
  let mut latencies : Vec< Duration > = filtered.iter( ).map( | m | m.latency ).collect( );
  latencies.sort( );

  let latency = if latencies.is_empty( )
  {
      LatencyStats::default( )
  }
  else
  {
      let min = latencies.first( ).copied( ).unwrap_or_default( );
      let max = latencies.last( ).copied( ).unwrap_or_default( );
      let sum : Duration = latencies.iter( ).sum( );
      #[ allow( clippy::cast_possible_truncation ) ]
      let mean = sum / latencies.len( ) as u32;
      let p50 = MetricsState::percentile( &latencies, 0.50 );
      let p95 = MetricsState::percentile( &latencies, 0.95 );
      let p99 = MetricsState::percentile( &latencies, 0.99 );

      LatencyStats { min, max, mean, p50, p95, p99 }
  };

  MetricsSnapshot
  {
      total_requests,
      successful_requests,
      failed_requests,
      total_bytes,
      latency,
      snapshot_time : now,
      duration : window,
  }
  }

  /// Reset all metrics
  #[ inline ]
  pub async fn reset( &self )
  {
  let mut state = self.state.write( ).await;
  state.measurements.clear( );
  state.start_time = Instant::now( );
  }

  /// Clean up old measurements outside the time window
  #[ inline ]
  pub async fn cleanup( &self ) -> usize
  {
  let mut state = self.state.write( ).await;
  let before = state.measurements.len( );
  state.cleanup_old_measurements( );
  let after = state.measurements.len( );
  before - after
  }

  /// Get number of measurements currently tracked
  #[ inline ]
  pub async fn measurement_count( &self ) -> usize
  {
  let state = self.state.read( ).await;
  state.measurements.len( )
  }

  /// Get configuration
  #[ inline ]
  pub async fn config( &self ) -> MetricsConfig
  {
  let state = self.state.read( ).await;
  state.config.clone( )
  }
}

impl core::fmt::Debug for PerformanceMetrics
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
  f.debug_struct( "PerformanceMetrics" )
      .field( "state", &"< MetricsState >" )
      .finish( )
  }
}
