//! Auto-scaling logic and intelligent resource management

use std::sync::{ Arc, Mutex };
use std::sync::atomic::Ordering;
use std::time::{ Duration, SystemTime };

use super::health::DeploymentMetrics;

/// Auto-scaling configuration
#[ derive( Debug, Clone ) ]
pub struct ScalingConfig
{
  /// Minimum number of instances
  pub min_instances : usize,
  /// Maximum number of instances
  pub max_instances : usize,
  /// Target CPU utilization percentage
  pub target_cpu_utilization : f64,
  /// Target memory utilization percentage
  pub target_memory_utilization : f64,
  /// Cooldown period before scaling up
  pub scale_up_cooldown : Duration,
  /// Cooldown period before scaling down
  pub scale_down_cooldown : Duration,
}

impl Default for ScalingConfig
{
  fn default() -> Self
  {
    Self {
      min_instances : 1,
      max_instances : 3,
      target_cpu_utilization : 70.0,
      target_memory_utilization : 80.0,
      scale_up_cooldown : Duration::from_secs( 300 ),   // 5 minutes
      scale_down_cooldown : Duration::from_secs( 600 ), // 10 minutes
    }
  }
}

/// Builder for scaling configuration
#[ derive( Debug, Clone ) ]
pub struct ScalingConfigBuilder
{
  config : ScalingConfig,
}

impl ScalingConfigBuilder
{
  /// Create a new scaling config builder
  pub fn new() -> Self
  {
    Self {
      config : ScalingConfig::default(),
    }
  }

  /// Set minimum instances
  pub fn min_instances( mut self, min : usize ) -> Self
  {
    self.config.min_instances = min;
    self
  }

  /// Set maximum instances
  pub fn max_instances( mut self, max : usize ) -> Self
  {
    self.config.max_instances = max;
    self
  }

  /// Set target CPU utilization
  pub fn target_cpu_utilization( mut self, cpu : f64 ) -> Self
  {
    self.config.target_cpu_utilization = cpu;
    self
  }

  /// Set target memory utilization
  pub fn target_memory_utilization( mut self, memory : f64 ) -> Self
  {
    self.config.target_memory_utilization = memory;
    self
  }

  /// Set scale up cooldown
  pub fn scale_up_cooldown( mut self, cooldown : Duration ) -> Self
  {
    self.config.scale_up_cooldown = cooldown;
    self
  }

  /// Set scale down cooldown
  pub fn scale_down_cooldown( mut self, cooldown : Duration ) -> Self
  {
    self.config.scale_down_cooldown = cooldown;
    self
  }

  /// Build the scaling configuration with validation
  pub fn build( self ) -> Result< ScalingConfig, crate::error::Error >
  {
    if self.config.min_instances == 0
    {
      return Err( crate::error::Error::ConfigurationError(
        "Minimum instances must be greater than 0".to_string()
      ) );
    }

    if self.config.max_instances < self.config.min_instances
    {
      return Err( crate::error::Error::ConfigurationError(
        "Maximum instances must be greater than or equal to minimum instances".to_string()
      ) );
    }

    if self.config.target_cpu_utilization <= 0.0 || self.config.target_cpu_utilization > 100.0
    {
      return Err( crate::error::Error::ConfigurationError(
        "Target CPU utilization must be between 0 and 100".to_string()
      ) );
    }

    if self.config.target_memory_utilization <= 0.0 || self.config.target_memory_utilization > 100.0
    {
      return Err( crate::error::Error::ConfigurationError(
        "Target memory utilization must be between 0 and 100".to_string()
      ) );
    }

    Ok( self.config )
  }
}

impl Default for ScalingConfigBuilder
{
  fn default() -> Self
  {
    Self::new()
  }
}

impl ScalingConfig
{
  /// Create a new scaling config builder
  pub fn builder() -> ScalingConfigBuilder
  {
    ScalingConfigBuilder::new()
  }
}

/// Resource allocation configuration
#[ derive( Debug, Clone ) ]
pub struct ResourceConfig
{
  /// Number of CPU cores
  pub cpu_cores : f64,
  /// Memory in GB
  pub memory_gb : f64,
  /// Number of GPUs
  pub gpu_count : usize,
  /// GPU memory in GB
  pub gpu_memory_gb : f64,
  /// Storage in GB
  pub storage_gb : f64,
}

impl Default for ResourceConfig
{
  fn default() -> Self
  {
    Self {
      cpu_cores : 1.0,
      memory_gb : 2.0,
      gpu_count : 0,
      gpu_memory_gb : 0.0,
      storage_gb : 10.0,
    }
  }
}

/// Builder for resource configuration
#[ derive( Debug, Clone ) ]
pub struct ResourceConfigBuilder
{
  config : ResourceConfig,
}

impl ResourceConfigBuilder
{
  /// Create a new resource config builder
  pub fn new() -> Self
  {
    Self {
      config : ResourceConfig::default(),
    }
  }

  /// Set CPU cores
  pub fn cpu_cores( mut self, cores : f64 ) -> Self
  {
    self.config.cpu_cores = cores;
    self
  }

  /// Set memory in GB
  pub fn memory_gb( mut self, memory : f64 ) -> Self
  {
    self.config.memory_gb = memory;
    self
  }

  /// Set GPU count
  pub fn gpu_count( mut self, count : usize ) -> Self
  {
    self.config.gpu_count = count;
    self
  }

  /// Set GPU memory in GB
  pub fn gpu_memory_gb( mut self, memory : f64 ) -> Self
  {
    self.config.gpu_memory_gb = memory;
    self
  }

  /// Set storage in GB
  pub fn storage_gb( mut self, storage : f64 ) -> Self
  {
    self.config.storage_gb = storage;
    self
  }

  /// Build the resource configuration with validation
  pub fn build( self ) -> Result< ResourceConfig, crate::error::Error >
  {
    if self.config.cpu_cores <= 0.0
    {
      return Err( crate::error::Error::ConfigurationError(
        "CPU cores must be greater than 0".to_string()
      ) );
    }

    if self.config.memory_gb <= 0.0
    {
      return Err( crate::error::Error::ConfigurationError(
        "Memory must be greater than 0".to_string()
      ) );
    }

    Ok( self.config )
  }
}

impl Default for ResourceConfigBuilder
{
  fn default() -> Self
  {
    Self::new()
  }
}

impl ResourceConfig
{
  /// Create a new resource config builder
  pub fn builder() -> ResourceConfigBuilder
  {
    ResourceConfigBuilder::new()
  }
}

/// Intelligent scaling algorithms for resource optimization
#[ derive( Debug, Clone ) ]
pub struct IntelligentScaler
{
  /// Scaling configuration
  config : ScalingConfig,
  /// Historical metrics for prediction
  metrics_history : Arc< Mutex< Vec< ( SystemTime, DeploymentMetrics ) > > >,
  /// Last scaling action timestamp
  last_scaling_action : Arc< Mutex< Option< SystemTime > > >,
  /// Prediction model parameters
  prediction_window_minutes : u64,
}

impl IntelligentScaler
{
  /// Create a new intelligent scaler
  pub fn new( config : ScalingConfig ) -> Self
  {
    Self {
      config,
      metrics_history : Arc::new( Mutex::new( Vec::new() ) ),
      last_scaling_action : Arc::new( Mutex::new( None ) ),
      prediction_window_minutes : 15, // 15-minute prediction window
    }
  }

  /// Record metrics for scaling decisions
  pub fn record_metrics( &self, metrics : &DeploymentMetrics )
  {
    let mut history = self.metrics_history.lock().unwrap();
    let now = SystemTime::now();

    // Clone metrics for storage (simplified for now)
    let cloned_metrics = DeploymentMetrics::new();
    cloned_metrics.set_cpu_utilization( metrics.cpu_utilization() );
    cloned_metrics.set_memory_utilization( metrics.memory_utilization() );
    cloned_metrics.set_request_rate( metrics.request_rate() );

    history.push( ( now, cloned_metrics ) );

    // Keep only recent history (last 24 hours)
    let cutoff = now - Duration::from_secs( 24 * 60 * 60 );
    history.retain( | ( timestamp, _ ) | *timestamp > cutoff );
  }

  /// Get prediction window in minutes
  pub fn prediction_window_minutes( &self ) -> u64
  {
    self.prediction_window_minutes
  }

  /// Make scaling decision based on current metrics and prediction
  pub fn should_scale( &self, current_metrics : &DeploymentMetrics ) -> Option< ScalingDecision >
  {
    let cpu_util = current_metrics.cpu_utilization();
    let memory_util = current_metrics.memory_utilization();
    let current_instances = current_metrics.instance_count.load( Ordering::Relaxed );

    // Check if we're in cooldown period
    if let Some( last_action ) = *self.last_scaling_action.lock().unwrap()
    {
      let time_since_last = SystemTime::now()
        .duration_since( last_action )
        .unwrap_or_default();

      let required_cooldown = if cpu_util > self.config.target_cpu_utilization
      {
        self.config.scale_up_cooldown
      } else {
        self.config.scale_down_cooldown
      };

      if time_since_last < required_cooldown
      {
        return None; // Still in cooldown
      }
    }

    // Scaling up conditions
    if ( cpu_util > self.config.target_cpu_utilization || memory_util > self.config.target_memory_utilization )
      && current_instances < self.config.max_instances
    {
      let predicted_load = self.predict_future_load();
      let recommended_instances = self.calculate_optimal_instances( predicted_load );

      if recommended_instances > current_instances
      {
        return Some( ScalingDecision::ScaleUp {
          target_instances : recommended_instances.min( self.config.max_instances ),
          reason : format!( "CPU: {:.1}%, Memory : {:.1}%, Target CPU: {:.1}%",
            cpu_util, memory_util, self.config.target_cpu_utilization ),
        } );
      }
    }

    // Scaling down conditions
    if cpu_util < self.config.target_cpu_utilization * 0.5
      && memory_util < self.config.target_memory_utilization * 0.5
      && current_instances > self.config.min_instances
    {
      let predicted_load = self.predict_future_load();
      let recommended_instances = self.calculate_optimal_instances( predicted_load );

      if recommended_instances < current_instances
      {
        return Some( ScalingDecision::ScaleDown {
          target_instances : recommended_instances.max( self.config.min_instances ),
          reason : format!( "CPU: {:.1}%, Memory : {:.1}%, low utilization detected",
            cpu_util, memory_util ),
        } );
      }
    }

    None
  }

  /// Predict future load based on historical data
  fn predict_future_load( &self ) -> f64
  {
    let history = self.metrics_history.lock().unwrap();

    if history.is_empty()
    {
      return 1.0; // Default load if no history
    }

    // Simple linear trend prediction
    let recent_metrics : Vec< _ > = history
      .iter()
      .rev()
      .take( 10 ) // Last 10 data points
      .collect();

    if recent_metrics.len() < 2
    {
      return 1.0;
    }

    let avg_cpu : f64 = recent_metrics
      .iter()
      .map( | ( _, metrics ) | metrics.cpu_utilization() )
      .sum::< f64 >() / recent_metrics.len() as f64;

    // Normalize load factor (simplified)
    ( avg_cpu / 100.0 ).clamp( 0.1, 2.0 )
  }

  /// Calculate optimal instance count based on predicted load
  fn calculate_optimal_instances( &self, predicted_load : f64 ) -> usize
  {
    // Simple calculation : scale instances proportionally to load
    let base_instances = self.config.min_instances;
    let load_factor = predicted_load;
    let target_instances = ( base_instances as f64 * load_factor ).ceil() as usize;

    target_instances
      .max( self.config.min_instances )
      .min( self.config.max_instances )
  }

  /// Record a scaling action
  pub fn record_scaling_action( &self )
  {
    *self.last_scaling_action.lock().unwrap() = Some( SystemTime::now() );
  }
}

/// Scaling decision recommendation
#[ derive( Debug, Clone ) ]
pub enum ScalingDecision
{
  /// Scale up to target instance count
  ScaleUp {
    /// Target number of instances
    target_instances : usize,
    /// Reason for scaling up
    reason : String,
  },
  /// Scale down to target instance count
  ScaleDown {
    /// Target number of instances
    target_instances : usize,
    /// Reason for scaling down
    reason : String,
  },
}
