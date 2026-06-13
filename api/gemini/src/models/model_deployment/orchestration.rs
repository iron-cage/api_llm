//! Container orchestration and model deployment management

use std::sync::{ Arc, atomic::{ AtomicU64, AtomicBool, Ordering } };
use std::time::{ Duration, SystemTime };
use std::hash::{ Hash, Hasher };
use std::collections::hash_map::DefaultHasher;
use tokio::sync::{ broadcast, RwLock as AsyncRwLock };

use super::{ DeploymentState, DeploymentEnvironment, DeploymentSummary };
use super::auto_scaling::{ ScalingConfig, ScalingDecision, ResourceConfig, IntelligentScaler };
use super::health::{ DeploymentHealthCheckConfig, MonitoringConfig, DeploymentMetrics, PerformanceOptimizer };
use super::strategies::DeploymentStrategy;

/// Container configuration
#[ derive( Debug, Clone ) ]
pub struct ContainerConfig
{
  /// Container image
  pub image : String,
  /// Exposed port
  pub port : u16,
  /// Environment variables
  pub environment_variables : Vec< ( String, String ) >,
  /// Volume mounts
  pub volumes : Vec< String >,
  /// Command to run
  pub command : Option< Vec< String > >,
  /// Working directory
  pub working_directory : Option< String >,
}

impl Default for ContainerConfig
{
  fn default() -> Self
  {
    Self {
      image : "gcr.io/project/model:latest".to_string(),
      port : 8080,
      environment_variables : Vec::new(),
      volumes : Vec::new(),
      command : None,
      working_directory : None,
    }
  }
}

/// Builder for container configuration
#[ derive( Debug, Clone ) ]
pub struct ContainerConfigBuilder
{
  config : ContainerConfig,
}

impl ContainerConfigBuilder
{
  /// Create a new container config builder
  pub fn new() -> Self
  {
    Self {
      config : ContainerConfig::default(),
    }
  }

  /// Set container image
  pub fn image( mut self, image : &str ) -> Self
  {
    self.config.image = image.to_string();
    self
  }

  /// Set exposed port
  pub fn port( mut self, port : u16 ) -> Self
  {
    self.config.port = port;
    self
  }

  /// Set environment variables
  pub fn environment_variables( mut self, vars : Vec< ( String, String ) > ) -> Self
  {
    self.config.environment_variables = vars;
    self
  }

  /// Set volume mounts
  pub fn volumes( mut self, volumes : Vec< String > ) -> Self
  {
    self.config.volumes = volumes;
    self
  }

  /// Set command
  pub fn command( mut self, command : Vec< String > ) -> Self
  {
    self.config.command = Some( command );
    self
  }

  /// Set working directory
  pub fn working_directory( mut self, dir : &str ) -> Self
  {
    self.config.working_directory = Some( dir.to_string() );
    self
  }

  /// Build the container configuration
  pub fn build( self ) -> Result< ContainerConfig, crate::error::Error >
  {
    if self.config.image.is_empty()
    {
      return Err( crate::error::Error::ConfigurationError(
        "Container image cannot be empty".to_string()
      ) );
    }

    Ok( self.config )
  }
}

impl Default for ContainerConfigBuilder
{
  fn default() -> Self
  {
    Self::new()
  }
}

impl ContainerConfig
{
  /// Create a new container config builder
  pub fn builder() -> ContainerConfigBuilder
  {
    ContainerConfigBuilder::new()
  }
}

/// Orchestration platform configuration
#[ derive( Debug, Clone ) ]
pub enum OrchestrationConfig
{
  /// Kubernetes configuration
  Kubernetes {
    /// Kubernetes namespace
    namespace : String,
    /// Cluster name
    cluster : String,
    /// Service account
    service_account : String,
  },
  /// Docker configuration
  Docker {
    /// Docker network
    network : String,
    /// Volume mappings
    volumes : Vec< String >,
  },
}

/// Optimized model deployment management with advanced features
pub struct ModelDeployment
{
  /// Deployment identifier
  pub deployment_id : String,
  /// Deployment name
  pub name : String,
  /// Model version
  pub version : String,
  /// Current deployment state with optimized concurrent access
  state : Arc< AsyncRwLock< DeploymentState > >,
  /// Deployment environment
  pub environment : DeploymentEnvironment,
  /// Optimized metrics with atomic operations
  metrics : Arc< DeploymentMetrics >,
  /// State change notifications
  state_tx : broadcast::Sender< DeploymentState >,
  /// Creation timestamp
  pub created_at : SystemTime,
  /// Intelligent scaler for automated scaling decisions
  scaler : Option< Arc< IntelligentScaler > >,
  /// Performance optimizer for recommendations
  optimizer : Arc< PerformanceOptimizer >,
  /// Deployment configuration hash for change detection
  config_hash : Arc< AtomicU64 >,
  /// Health status
  is_healthy : Arc< AtomicBool >,
}

impl ModelDeployment
{
  /// Create a new optimized model deployment
  pub fn new(
    deployment_id : String,
    name : String,
    version : String,
    environment : DeploymentEnvironment
  ) -> Self
  {
    let ( state_tx, _state_rx ) = broadcast::channel( 64 ); // Increased buffer size

    Self {
      deployment_id,
      name,
      version,
      state : Arc::new( AsyncRwLock::new( DeploymentState::Pending ) ),
      environment,
      metrics : Arc::new( DeploymentMetrics::new() ),
      state_tx,
      created_at : SystemTime::now(),
      scaler : None,
      optimizer : Arc::new( PerformanceOptimizer::new() ),
      config_hash : Arc::new( AtomicU64::new( 0 ) ),
      is_healthy : Arc::new( AtomicBool::new( true ) ),
    }
  }

  /// Create a deployment with intelligent scaling enabled
  pub fn with_intelligent_scaling(
    deployment_id : String,
    name : String,
    version : String,
    environment : DeploymentEnvironment,
    scaling_config : ScalingConfig
  ) -> Self
  {
    let mut deployment = Self::new( deployment_id, name, version, environment );
    deployment.scaler = Some( Arc::new( IntelligentScaler::new( scaling_config ) ) );
    deployment
  }

  /// Get current deployment state (async optimized)
  pub async fn state( &self ) -> DeploymentState
  {
    self.state.read().await.clone()
  }

  /// Update deployment state with notification
  pub async fn set_state( &self, new_state : DeploymentState ) -> Result< (), crate::error::Error >
  {
    {
      let mut state = self.state.write().await;
      *state = new_state.clone();
    }

    // Update health status based on state
    let is_healthy = matches!( new_state, DeploymentState::Active );
    self.is_healthy.store( is_healthy, Ordering::Relaxed );
    self.metrics.update_uptime( is_healthy );

    // Notify subscribers
    self.state_tx.send( new_state ).ok();

    Ok( () )
  }

  /// Get current deployment metrics (optimized with Arc)
  pub fn get_metrics( &self ) -> Arc< DeploymentMetrics >
  {
    self.metrics.clone()
  }

  /// Record a request for metrics tracking
  pub fn record_request( &self, response_time_us : u64, is_error : bool )
  {
    self.metrics.record_request( response_time_us, is_error );

    // Record metrics for intelligent scaling if enabled
    if let Some( scaler ) = &self.scaler
    {
      scaler.record_metrics( &self.metrics );
    }
  }

  /// Update resource utilization metrics
  pub fn update_resource_utilization( &self, cpu_percent : f64, memory_percent : f64 )
  {
    self.metrics.set_cpu_utilization( cpu_percent );
    self.metrics.set_memory_utilization( memory_percent );

    // Record for scaling decisions
    if let Some( scaler ) = &self.scaler
    {
      scaler.record_metrics( &self.metrics );
    }
  }

  /// Check if intelligent scaling recommends scaling action
  pub fn check_scaling_recommendation( &self ) -> Option< ScalingDecision >
  {
    self.scaler.as_ref()?.should_scale( &self.metrics )
  }

  /// Execute scaling decision with intelligent scaler
  pub async fn execute_scaling( &self, decision : ScalingDecision ) -> Result< (), crate::error::Error >
  {
    match decision
    {
      ScalingDecision::ScaleUp { target_instances, reason } => {
        tracing ::info!( "Scaling up deployment {} to {} instances : {}",
          self.deployment_id, target_instances, reason );

        self.set_state( DeploymentState::Scaling ).await?;
        self.metrics.instance_count.store( target_instances, Ordering::Relaxed );

        // Record scaling action
        if let Some( scaler ) = &self.scaler
        {
          scaler.record_scaling_action();
        }

        self.set_state( DeploymentState::Active ).await?;
      },
      ScalingDecision::ScaleDown { target_instances, reason } => {
        tracing ::info!( "Scaling down deployment {} to {} instances : {}",
          self.deployment_id, target_instances, reason );

        self.set_state( DeploymentState::Scaling ).await?;
        self.metrics.instance_count.store( target_instances, Ordering::Relaxed );

        // Record scaling action
        if let Some( scaler ) = &self.scaler
        {
          scaler.record_scaling_action();
        }

        self.set_state( DeploymentState::Active ).await?;
      }
    }

    Ok( () )
  }

  /// Get performance optimization recommendations
  pub fn get_optimization_recommendations( &self ) -> Vec< super::health::OptimizationRecommendation >
  {
    self.optimizer.analyze_deployment( self )
  }

  /// Subscribe to deployment state changes
  pub fn subscribe_state_changes( &self ) -> broadcast::Receiver< DeploymentState >
  {
    self.state_tx.subscribe()
  }

  /// Start the deployment with enhanced monitoring
  pub async fn start( &self ) -> Result< (), crate::error::Error >
  {
    self.set_state( DeploymentState::Active ).await?;

    // Initialize metrics
    self.metrics.instance_count.store( 1, Ordering::Relaxed );

    tracing ::info!( "Started deployment {} in {:?} environment",
      self.deployment_id, self.environment );

    Ok( () )
  }

  /// Stop the deployment gracefully
  pub async fn stop( &self ) -> Result< (), crate::error::Error >
  {
    self.set_state( DeploymentState::Terminated ).await?;

    // Clear metrics
    self.metrics.instance_count.store( 0, Ordering::Relaxed );

    tracing ::info!( "Stopped deployment {}", self.deployment_id );

    Ok( () )
  }

  /// Scale the deployment with validation
  pub async fn scale( &self, target_instances : usize ) -> Result< (), crate::error::Error >
  {
    let current_state = self.state().await;
    if !matches!( current_state, DeploymentState::Active )
    {
      return Err( crate::error::Error::ApiError(
        format!( "Cannot scale deployment in state : {:?}", current_state )
      ) );
    }

    self.set_state( DeploymentState::Scaling ).await?;

    // Simulate scaling operation
    tokio ::time::sleep( Duration::from_millis( 100 ) ).await;

    self.metrics.instance_count.store( target_instances, Ordering::Relaxed );

    self.set_state( DeploymentState::Active ).await?;

    tracing ::info!( "Scaled deployment {} to {} instances",
      self.deployment_id, target_instances );

    Ok( () )
  }

  /// Rollback the deployment
  pub async fn rollback( &self ) -> Result< (), crate::error::Error >
  {
    self.set_state( DeploymentState::RollingBack ).await?;

    tracing ::warn!( "Rolling back deployment {}", self.deployment_id );

    // Simulate rollback completion
    tokio ::time::sleep( Duration::from_millis( 500 ) ).await;

    self.set_state( DeploymentState::Active ).await?;

    Ok( () )
  }

  /// Get deployment health status
  pub fn is_healthy( &self ) -> bool
  {
    self.is_healthy.load( Ordering::Relaxed )
  }

  /// Update configuration hash for change detection
  pub fn update_config_hash( &self, config_data : &str )
  {
    let mut hasher = DefaultHasher::new();
    config_data.hash( &mut hasher );
    let hash = hasher.finish();
    self.config_hash.store( hash, Ordering::Relaxed );
  }

  /// Check if configuration has changed
  pub fn has_config_changed( &self, config_data : &str ) -> bool
  {
    let mut hasher = DefaultHasher::new();
    config_data.hash( &mut hasher );
    let new_hash = hasher.finish();
    let current_hash = self.config_hash.load( Ordering::Relaxed );

    new_hash != current_hash
  }

  /// Export metrics in Prometheus format
  pub fn export_prometheus_metrics( &self ) -> String
  {
    self.metrics.to_prometheus( &self.deployment_id )
  }

  /// Get deployment summary for monitoring dashboards
  pub async fn get_summary( &self ) -> DeploymentSummary
  {
    DeploymentSummary {
      deployment_id : self.deployment_id.clone(),
      name : self.name.clone(),
      version : self.version.clone(),
      state : self.state().await,
      environment : self.environment.clone(),
      instance_count : self.metrics.instance_count.load( Ordering::Relaxed ),
      cpu_utilization : self.metrics.cpu_utilization(),
      memory_utilization : self.metrics.memory_utilization(),
      error_rate : self.metrics.error_rate(),
      response_time_ms : self.metrics.response_time_ms(),
      uptime_percentage : self.metrics.uptime_percentage(),
      is_healthy : self.is_healthy(),
      created_at : self.created_at,
      total_requests : self.metrics.total_requests.load( Ordering::Relaxed ),
    }
  }
}

impl std::fmt::Debug for ModelDeployment
{
  fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    f.debug_struct( "ModelDeployment" )
      .field( "deployment_id", &self.deployment_id )
      .field( "name", &self.name )
      .field( "version", &self.version )
      .field( "environment", &self.environment )
      .field( "instance_count", &self.metrics.instance_count.load( Ordering::Relaxed ) )
      .field( "cpu_utilization", &self.metrics.cpu_utilization() )
      .field( "memory_utilization", &self.metrics.memory_utilization() )
      .field( "is_healthy", &self.is_healthy() )
      .field( "has_scaler", &self.scaler.is_some() )
      .field( "created_at", &self.created_at )
      .finish_non_exhaustive()
  }
}

/// Builder for model deployment operations
pub struct DeploymentBuilder< 'a >
{
  #[ allow( dead_code ) ]
  model : &'a crate::models::api::ModelApi< 'a >,
  name : Option< String >,
  version : Option< String >,
  environment : DeploymentEnvironment,
  strategy : Option< DeploymentStrategy >,
  scaling_config : Option< ScalingConfig >,
  resource_config : Option< ResourceConfig >,
  health_checks : Option< DeploymentHealthCheckConfig >,
  monitoring : Option< MonitoringConfig >,
  orchestration : Option< OrchestrationConfig >,
  container_config : Option< ContainerConfig >,
}

impl< 'a > DeploymentBuilder< 'a >
{
  /// Create a new deployment builder
  pub fn new( model : &'a crate::models::api::ModelApi< 'a > ) -> Self
  {
    Self {
      model,
      name : None,
      version : None,
      environment : DeploymentEnvironment::Development,
      strategy : None,
      scaling_config : None,
      resource_config : None,
      health_checks : None,
      monitoring : None,
      orchestration : None,
      container_config : None,
    }
  }

  /// Set deployment name
  pub fn with_name( mut self, name : &str ) -> Self
  {
    self.name = Some( name.to_string() );
    self
  }

  /// Set deployment version
  pub fn with_version( mut self, version : &str ) -> Self
  {
    self.version = Some( version.to_string() );
    self
  }

  /// Set deployment environment
  pub fn with_environment( mut self, environment : DeploymentEnvironment ) -> Self
  {
    self.environment = environment;
    self
  }

  /// Set deployment strategy
  pub fn with_strategy( mut self, strategy : DeploymentStrategy ) -> Self
  {
    self.strategy = Some( strategy );
    self
  }

  /// Set scaling configuration
  pub fn with_scaling_config( mut self, config : ScalingConfig ) -> Self
  {
    self.scaling_config = Some( config );
    self
  }

  /// Set resource configuration
  pub fn with_resource_config( mut self, config : ResourceConfig ) -> Self
  {
    self.resource_config = Some( config );
    self
  }

  /// Set health check configuration
  pub fn with_health_checks( mut self, config : DeploymentHealthCheckConfig ) -> Self
  {
    self.health_checks = Some( config );
    self
  }

  /// Set monitoring configuration
  pub fn with_monitoring( mut self, config : MonitoringConfig ) -> Self
  {
    self.monitoring = Some( config );
    self
  }

  /// Set orchestration configuration
  pub fn with_orchestration( mut self, config : OrchestrationConfig ) -> Self
  {
    self.orchestration = Some( config );
    self
  }

  /// Set container configuration
  pub fn with_container_config( mut self, config : ContainerConfig ) -> Self
  {
    self.container_config = Some( config );
    self
  }

  /// Create and start the deployment
  pub async fn deploy( self ) -> Result< ModelDeployment, crate::error::Error >
  {
    // Validate required fields
    let name = self.name.ok_or_else( ||
      crate ::error::Error::ApiError( "Deployment name is required".to_string() )
    )?;

    let version = self.version.unwrap_or_else( || "1.0.0".to_string() );

    // Create deployment
    let deployment_id = format!( "deploy-{}", "generated-id" ); // Simplified for now
    let deployment = ModelDeployment::new( deployment_id, name, version, self.environment );

    // Start the deployment
    deployment.start().await?;

    Ok( deployment )
  }
}

impl< 'a > std::fmt::Debug for DeploymentBuilder< 'a >
{
  fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    f.debug_struct( "DeploymentBuilder" )
      .field( "name", &self.name )
      .field( "version", &self.version )
      .field( "environment", &self.environment )
      .field( "strategy", &self.strategy )
      .field( "scaling_config", &self.scaling_config )
      .field( "resource_config", &self.resource_config )
      .field( "health_checks", &self.health_checks )
      .field( "monitoring", &self.monitoring )
      .field( "orchestration", &self.orchestration )
      .field( "container_config", &self.container_config )
      .finish_non_exhaustive()
  }
}
