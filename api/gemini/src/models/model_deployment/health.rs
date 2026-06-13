//! Health monitoring, metrics tracking, and performance optimization

use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use std::sync::atomic::{ AtomicU64, AtomicUsize, Ordering };
use std::time::{ Duration, SystemTime };

/// Health check configuration for deployments
#[ derive( Debug, Clone ) ]
pub struct DeploymentHealthCheckConfig
{
  /// Health check endpoint
  pub endpoint : String,
  /// Check interval
  pub interval : Duration,
  /// Request timeout
  pub timeout : Duration,
  /// Number of consecutive failures before marking unhealthy
  pub failure_threshold : usize,
  /// Number of consecutive successes before marking healthy
  pub success_threshold : usize,
}

impl Default for DeploymentHealthCheckConfig
{
  fn default() -> Self
  {
    Self {
      endpoint : "/health".to_string(),
      interval : Duration::from_secs( 30 ),
      timeout : Duration::from_secs( 5 ),
      failure_threshold : 3,
      success_threshold : 1,
    }
  }
}

/// Builder for health check configuration
#[ derive( Debug, Clone ) ]
pub struct DeploymentHealthCheckConfigBuilder
{
  config : DeploymentHealthCheckConfig,
}

impl DeploymentHealthCheckConfigBuilder
{
  /// Create a new health check config builder
  pub fn new() -> Self
  {
    Self {
      config : DeploymentHealthCheckConfig::default(),
    }
  }

  /// Set health check endpoint
  pub fn endpoint( mut self, endpoint : &str ) -> Self
  {
    self.config.endpoint = endpoint.to_string();
    self
  }

  /// Set check interval
  pub fn interval( mut self, interval : Duration ) -> Self
  {
    self.config.interval = interval;
    self
  }

  /// Set request timeout
  pub fn timeout( mut self, timeout : Duration ) -> Self
  {
    self.config.timeout = timeout;
    self
  }

  /// Set failure threshold
  pub fn failure_threshold( mut self, threshold : usize ) -> Self
  {
    self.config.failure_threshold = threshold;
    self
  }

  /// Set success threshold
  pub fn success_threshold( mut self, threshold : usize ) -> Self
  {
    self.config.success_threshold = threshold;
    self
  }

  /// Build the health check configuration
  pub fn build( self ) -> Result< DeploymentHealthCheckConfig, crate::error::Error >
  {
    if self.config.endpoint.is_empty()
    {
      return Err( crate::error::Error::ConfigurationError(
        "Health check endpoint cannot be empty".to_string()
      ) );
    }

    Ok( self.config )
  }
}

impl Default for DeploymentHealthCheckConfigBuilder
{
  fn default() -> Self
  {
    Self::new()
  }
}

impl DeploymentHealthCheckConfig
{
  /// Create a new health check config builder
  pub fn builder() -> DeploymentHealthCheckConfigBuilder
  {
    DeploymentHealthCheckConfigBuilder::new()
  }
}

/// Monitoring configuration
#[ derive( Debug, Clone ) ]
pub struct MonitoringConfig
{
  /// Whether to enable metrics collection
  pub enable_metrics : bool,
  /// Metrics collection interval
  pub metrics_interval : Duration,
  /// Whether to enable logging
  pub enable_logging : bool,
  /// Log level
  pub log_level : String,
  /// Whether to alert on errors
  pub alert_on_errors : bool,
  /// Custom metric labels
  pub metric_labels : HashMap<  String, String  >,
}

impl Default for MonitoringConfig
{
  fn default() -> Self
  {
    Self {
      enable_metrics : true,
      metrics_interval : Duration::from_secs( 60 ),
      enable_logging : true,
      log_level : "INFO".to_string(),
      alert_on_errors : true,
      metric_labels : HashMap::new(),
    }
  }
}

/// Builder for monitoring configuration
#[ derive( Debug, Clone ) ]
pub struct MonitoringConfigBuilder
{
  config : MonitoringConfig,
}

impl MonitoringConfigBuilder
{
  /// Create a new monitoring config builder
  pub fn new() -> Self
  {
    Self {
      config : MonitoringConfig::default(),
    }
  }

  /// Enable or disable metrics
  pub fn enable_metrics( mut self, enable : bool ) -> Self
  {
    self.config.enable_metrics = enable;
    self
  }

  /// Set metrics interval
  pub fn metrics_interval( mut self, interval : Duration ) -> Self
  {
    self.config.metrics_interval = interval;
    self
  }

  /// Enable or disable logging
  pub fn enable_logging( mut self, enable : bool ) -> Self
  {
    self.config.enable_logging = enable;
    self
  }

  /// Set log level
  pub fn log_level( mut self, level : String ) -> Self
  {
    self.config.log_level = level;
    self
  }

  /// Enable or disable error alerting
  pub fn alert_on_errors( mut self, alert : bool ) -> Self
  {
    self.config.alert_on_errors = alert;
    self
  }

  /// Add metric labels
  pub fn metric_labels( mut self, labels : HashMap<  String, String  > ) -> Self
  {
    self.config.metric_labels = labels;
    self
  }

  /// Build the monitoring configuration
  pub fn build( self ) -> Result< MonitoringConfig, crate::error::Error >
  {
    Ok( self.config )
  }
}

impl Default for MonitoringConfigBuilder
{
  fn default() -> Self
  {
    Self::new()
  }
}

impl MonitoringConfig
{
  /// Create a new monitoring config builder
  pub fn builder() -> MonitoringConfigBuilder
  {
    MonitoringConfigBuilder::new()
  }
}

/// Optimized deployment metrics with atomic operations for high-performance tracking
#[ derive( Debug ) ]
pub struct DeploymentMetrics
{
  /// Number of active instances
  pub instance_count : AtomicUsize,
  /// Current CPU utilization percentage (scaled by 100 for precision)
  cpu_utilization_scaled : AtomicU64,
  /// Current memory utilization percentage (scaled by 100 for precision)
  memory_utilization_scaled : AtomicU64,
  /// Requests per second (scaled by 100 for precision)
  request_rate_scaled : AtomicU64,
  /// Error rate percentage (scaled by 100 for precision)
  error_rate_scaled : AtomicU64,
  /// Average response time in microseconds
  pub response_time_us : AtomicU64,
  /// Uptime percentage (scaled by 100 for precision)
  uptime_percentage_scaled : AtomicU64,
  /// Total requests processed
  pub total_requests : AtomicU64,
  /// Total errors encountered
  pub total_errors : AtomicU64,
  /// Last updated timestamp (microseconds since epoch)
  pub last_updated_us : AtomicU64,
  /// Deployment start time for uptime calculation
  deployment_start_us : AtomicU64,
}

impl DeploymentMetrics
{
  /// Create new deployment metrics
  pub fn new() -> Self
  {
    let now_us = SystemTime::now()
      .duration_since( SystemTime::UNIX_EPOCH )
      .unwrap_or_default()
      .as_micros() as u64;

    Self {
      instance_count : AtomicUsize::new( 0 ),
      cpu_utilization_scaled : AtomicU64::new( 0 ),
      memory_utilization_scaled : AtomicU64::new( 0 ),
      request_rate_scaled : AtomicU64::new( 0 ),
      error_rate_scaled : AtomicU64::new( 0 ),
      response_time_us : AtomicU64::new( 0 ),
      uptime_percentage_scaled : AtomicU64::new( 10000 ), // 100% * 100
      total_requests : AtomicU64::new( 0 ),
      total_errors : AtomicU64::new( 0 ),
      last_updated_us : AtomicU64::new( now_us ),
      deployment_start_us : AtomicU64::new( now_us ),
    }
  }

  /// Get CPU utilization as f64 percentage
  pub fn cpu_utilization( &self ) -> f64
  {
    self.cpu_utilization_scaled.load( Ordering::Relaxed ) as f64 / 100.0
  }

  /// Set CPU utilization
  pub fn set_cpu_utilization( &self, value : f64 )
  {
    let scaled = ( value * 100.0 ).round() as u64;
    self.cpu_utilization_scaled.store( scaled, Ordering::Relaxed );
    self.update_timestamp();
  }

  /// Get memory utilization as f64 percentage
  pub fn memory_utilization( &self ) -> f64
  {
    self.memory_utilization_scaled.load( Ordering::Relaxed ) as f64 / 100.0
  }

  /// Set memory utilization
  pub fn set_memory_utilization( &self, value : f64 )
  {
    let scaled = ( value * 100.0 ).round() as u64;
    self.memory_utilization_scaled.store( scaled, Ordering::Relaxed );
    self.update_timestamp();
  }

  /// Get request rate as f64
  pub fn request_rate( &self ) -> f64
  {
    self.request_rate_scaled.load( Ordering::Relaxed ) as f64 / 100.0
  }

  /// Set request rate
  pub fn set_request_rate( &self, value : f64 )
  {
    let scaled = ( value * 100.0 ).round() as u64;
    self.request_rate_scaled.store( scaled, Ordering::Relaxed );
    self.update_timestamp();
  }

  /// Get error rate as f64 percentage
  pub fn error_rate( &self ) -> f64
  {
    self.error_rate_scaled.load( Ordering::Relaxed ) as f64 / 100.0
  }

  /// Record a new request
  pub fn record_request( &self, response_time_us : u64, is_error : bool )
  {
    self.total_requests.fetch_add( 1, Ordering::Relaxed );
    if is_error
    {
      self.total_errors.fetch_add( 1, Ordering::Relaxed );
    }

    // Update average response time using exponential moving average
    let current_avg = self.response_time_us.load( Ordering::Relaxed );
    let new_avg = if current_avg == 0
    {
      response_time_us
    } else {
      // EMA with alpha = 0.1
      ( ( current_avg as f64 * 0.9 ) + ( response_time_us as f64 * 0.1 ) ).round() as u64
    };
    self.response_time_us.store( new_avg, Ordering::Relaxed );

    // Update error rate
    let total_requests = self.total_requests.load( Ordering::Relaxed );
    let total_errors = self.total_errors.load( Ordering::Relaxed );
    let error_rate = if total_requests > 0
    {
      ( total_errors as f64 / total_requests as f64 ) * 100.0
    } else {
      0.0
    };
    let error_rate_scaled = ( error_rate * 100.0 ).round() as u64;
    self.error_rate_scaled.store( error_rate_scaled, Ordering::Relaxed );

    self.update_timestamp();
  }

  /// Get uptime percentage
  pub fn uptime_percentage( &self ) -> f64
  {
    self.uptime_percentage_scaled.load( Ordering::Relaxed ) as f64 / 100.0
  }

  /// Update uptime based on current status
  pub fn update_uptime( &self, is_healthy : bool )
  {
    let now_us = SystemTime::now()
      .duration_since( SystemTime::UNIX_EPOCH )
      .unwrap_or_default()
      .as_micros() as u64;

    let start_us = self.deployment_start_us.load( Ordering::Relaxed );
    let total_time_us = now_us.saturating_sub( start_us );

    if total_time_us > 0
    {
      // For simplicity, assume uptime is based on current health status
      // In production, this would track actual downtime
      let uptime_percentage = if is_healthy { 100.0 } else { 95.0 };
      let uptime_scaled = ( uptime_percentage * 100.0_f64 ).round() as u64;
      self.uptime_percentage_scaled.store( uptime_scaled, Ordering::Relaxed );
    }

    self.update_timestamp();
  }

  /// Update the last updated timestamp
  fn update_timestamp( &self )
  {
    let now_us = SystemTime::now()
      .duration_since( SystemTime::UNIX_EPOCH )
      .unwrap_or_default()
      .as_micros() as u64;
    self.last_updated_us.store( now_us, Ordering::Relaxed );
  }

  /// Get response time in milliseconds
  pub fn response_time_ms( &self ) -> f64
  {
    self.response_time_us.load( Ordering::Relaxed ) as f64 / 1000.0
  }

  /// Export metrics for monitoring systems (Prometheus format)
  pub fn to_prometheus( &self, deployment_id : &str ) -> String
  {
    format!(
      "# HELP deployment_instance_count Number of active instances\n\
       # TYPE deployment_instance_count gauge\n\
       deployment_instance_count{{deployment_id=\"{}\"}} {}\n\
       \n\
       # HELP deployment_cpu_utilization CPU utilization percentage\n\
       # TYPE deployment_cpu_utilization gauge\n\
       deployment_cpu_utilization{{deployment_id=\"{}\"}} {:.2}\n\
       \n\
       # HELP deployment_memory_utilization Memory utilization percentage\n\
       # TYPE deployment_memory_utilization gauge\n\
       deployment_memory_utilization{{deployment_id=\"{}\"}} {:.2}\n\
       \n\
       # HELP deployment_request_rate Requests per second\n\
       # TYPE deployment_request_rate gauge\n\
       deployment_request_rate{{deployment_id=\"{}\"}} {:.2}\n\
       \n\
       # HELP deployment_error_rate Error rate percentage\n\
       # TYPE deployment_error_rate gauge\n\
       deployment_error_rate{{deployment_id=\"{}\"}} {:.2}\n\
       \n\
       # HELP deployment_response_time_ms Average response time in milliseconds\n\
       # TYPE deployment_response_time_ms gauge\n\
       deployment_response_time_ms{{deployment_id=\"{}\"}} {:.2}\n\
       \n\
       # HELP deployment_uptime_percentage Uptime percentage\n\
       # TYPE deployment_uptime_percentage gauge\n\
       deployment_uptime_percentage{{deployment_id=\"{}\"}} {:.2}\n\
       \n\
       # HELP deployment_total_requests Total requests processed\n\
       # TYPE deployment_total_requests counter\n\
       deployment_total_requests{{deployment_id=\"{}\"}} {}\n\
       \n\
       # HELP deployment_total_errors Total errors encountered\n\
       # TYPE deployment_total_errors counter\n\
       deployment_total_errors{{deployment_id=\"{}\"}} {}\n",
      deployment_id, self.instance_count.load( Ordering::Relaxed ),
      deployment_id, self.cpu_utilization(),
      deployment_id, self.memory_utilization(),
      deployment_id, self.request_rate(),
      deployment_id, self.error_rate(),
      deployment_id, self.response_time_ms(),
      deployment_id, self.uptime_percentage(),
      deployment_id, self.total_requests.load( Ordering::Relaxed ),
      deployment_id, self.total_errors.load( Ordering::Relaxed )
    )
  }
}

impl Default for DeploymentMetrics
{
  fn default() -> Self
  {
    Self::new()
  }
}

/// Deployment performance optimization recommendations
#[ derive( Debug, Clone ) ]
pub struct PerformanceOptimizer
{
  /// Optimization recommendations
  recommendations : Arc< Mutex< Vec< OptimizationRecommendation > > >,
  /// Analysis history
  analysis_history : Arc< Mutex< Vec< ( SystemTime, String ) > > >,
}

impl PerformanceOptimizer
{
  /// Create a new performance optimizer
  pub fn new() -> Self
  {
    Self {
      recommendations : Arc::new( Mutex::new( Vec::new() ) ),
      analysis_history : Arc::new( Mutex::new( Vec::new() ) ),
    }
  }

  /// Analyze deployment performance and generate recommendations
  pub fn analyze_deployment( &self, deployment : &super::orchestration::ModelDeployment ) -> Vec< OptimizationRecommendation >
  {
    let mut recommendations = Vec::new();
    let metrics = deployment.get_metrics();

    // CPU optimization recommendations
    if metrics.cpu_utilization() > 90.0
    {
      recommendations.push( OptimizationRecommendation {
        category : OptimizationCategory::ResourceAllocation,
        priority : OptimizationPriority::High,
        title : "High CPU Utilization Detected".to_string(),
        description : format!(
          "CPU utilization is at {:.1}%, consider scaling up or optimizing compute resources",
          metrics.cpu_utilization()
        ),
        estimated_impact : ImpactEstimate::High,
        implementation_effort : ImplementationEffort::Medium,
      } );
    }

    // Memory optimization recommendations
    if metrics.memory_utilization() > 85.0
    {
      recommendations.push( OptimizationRecommendation {
        category : OptimizationCategory::ResourceAllocation,
        priority : OptimizationPriority::High,
        title : "High Memory Utilization Detected".to_string(),
        description : format!(
          "Memory utilization is at {:.1}%, consider increasing memory allocation",
          metrics.memory_utilization()
        ),
        estimated_impact : ImpactEstimate::High,
        implementation_effort : ImplementationEffort::Low,
      } );
    }

    // Response time optimization
    if metrics.response_time_ms() > 1000.0
    {
      recommendations.push( OptimizationRecommendation {
        category : OptimizationCategory::Performance,
        priority : OptimizationPriority::Medium,
        title : "High Response Time Detected".to_string(),
        description : format!(
          "Average response time is {:.1}ms, consider optimizing model inference or adding caching",
          metrics.response_time_ms()
        ),
        estimated_impact : ImpactEstimate::Medium,
        implementation_effort : ImplementationEffort::High,
      } );
    }

    // Error rate optimization
    if metrics.error_rate() > 5.0
    {
      recommendations.push( OptimizationRecommendation {
        category : OptimizationCategory::Reliability,
        priority : OptimizationPriority::High,
        title : "High Error Rate Detected".to_string(),
        description : format!(
          "Error rate is {:.1}%, investigate error patterns and improve error handling",
          metrics.error_rate()
        ),
        estimated_impact : ImpactEstimate::High,
        implementation_effort : ImplementationEffort::Medium,
      } );
    }

    // Store recommendations and analysis
    {
      let mut stored_recommendations = self.recommendations.lock().unwrap();
      stored_recommendations.extend( recommendations.clone() );

      let mut history = self.analysis_history.lock().unwrap();
      history.push( (
        SystemTime::now(),
        format!( "Generated {} recommendations for deployment {}",
          recommendations.len(), deployment.deployment_id )
      ) );
    }

    recommendations
  }

  /// Get all recommendations
  pub fn get_recommendations( &self ) -> Vec< OptimizationRecommendation >
  {
    self.recommendations.lock().unwrap().clone()
  }

  /// Clear recommendations
  pub fn clear_recommendations( &self )
  {
    self.recommendations.lock().unwrap().clear();
  }
}

impl Default for PerformanceOptimizer
{
  fn default() -> Self
  {
    Self::new()
  }
}

/// Optimization recommendation
#[ derive( Debug, Clone ) ]
pub struct OptimizationRecommendation
{
  /// Category of optimization
  pub category : OptimizationCategory,
  /// Priority level
  pub priority : OptimizationPriority,
  /// Recommendation title
  pub title : String,
  /// Detailed description
  pub description : String,
  /// Estimated impact
  pub estimated_impact : ImpactEstimate,
  /// Implementation effort required
  pub implementation_effort : ImplementationEffort,
}

/// Categories of optimization recommendations
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum OptimizationCategory
{
  /// Resource allocation optimization
  ResourceAllocation,
  /// Performance optimization
  Performance,
  /// Reliability improvement
  Reliability,
  /// Cost optimization
  Cost,
  /// Security enhancement
  Security,
}

/// Priority levels for recommendations
#[ derive( Debug, Clone, PartialEq, Eq, PartialOrd, Ord ) ]
pub enum OptimizationPriority
{
  /// Low priority
  Low,
  /// Medium priority
  Medium,
  /// High priority
  High,
  /// Critical priority
  Critical,
}

/// Impact estimation for recommendations
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum ImpactEstimate
{
  /// Low impact
  Low,
  /// Medium impact
  Medium,
  /// High impact
  High,
}

/// Implementation effort estimation
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum ImplementationEffort
{
  /// Low effort
  Low,
  /// Medium effort
  Medium,
  /// High effort
  High,
}
