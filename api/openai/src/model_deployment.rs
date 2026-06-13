//! Model Deployment Module
//!
//! This module provides stateless model deployment utilities for `OpenAI` API models.
//! Following the "Thin Client, Rich API" principle, this module offers deployment management
//! patterns and orchestration tools without automatic behaviors or persistent state management.

#![ allow( clippy::missing_inline_in_public_items ) ]

mod private
{
  use std::
  {
    collections ::HashMap,
    time ::SystemTime,
  };
  use core::time::Duration;
  use serde::{ Deserialize, Serialize };
  use tokio::sync::mpsc;

  /// Model deployment status
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub enum DeploymentStatus
  {
    /// Deployment is being prepared
    Preparing,
    /// Deployment is in progress
    Deploying,
    /// Deployment is active and healthy
    Active,
    /// Deployment is scaling (up or down)
    Scaling,
    /// Deployment is being updated
    Updating,
    /// Deployment is being rolled back
    RollingBack,
    /// Deployment is paused
    Paused,
    /// Deployment has failed
    Failed( String ),
    /// Deployment is being terminated
    Terminating,
    /// Deployment has been terminated
    Terminated,
  }

  /// Deployment strategy type
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub enum DeploymentStrategy
  {
    /// Rolling deployment with gradual replacement
    Rolling
    {
      /// Maximum number of instances to update at once
      max_surge : u32,
      /// Maximum number of instances that can be unavailable
      max_unavailable : u32,
    },
    /// Blue-green deployment with environment switching
    BlueGreen
    {
      /// Traffic split percentage for new version
      traffic_split : u8,
    },
    /// Canary deployment with gradual traffic shifting
    Canary
    {
      /// Initial traffic percentage for canary
      initial_traffic : u8,
      /// Final traffic percentage for canary
      final_traffic : u8,
      /// Duration for canary evaluation
      evaluation_duration_ms : u64,
    },
    /// Immediate replacement of all instances
    Recreate,
  }

  /// Model deployment configuration
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct DeploymentConfig
  {
    /// Deployment name/identifier
    pub name : String,
    /// Model identifier to deploy
    pub model_id : String,
    /// Model version/tag
    pub model_version : String,
    /// Target environment
    pub environment : String,
    /// Number of desired instances
    pub replicas : u32,
    /// Resource requirements per instance
    pub resources : ResourceRequirements,
    /// Auto-scaling configuration
    pub auto_scaling : Option< AutoScalingConfig >,
    /// Deployment strategy
    pub strategy : DeploymentStrategy,
    /// Health check configuration
    pub health_check : HealthCheckConfig,
    /// Environment variables
    pub env_vars : HashMap<  String, String  >,
    /// Deployment timeout in milliseconds
    pub timeout_ms : u64,
  }

  /// Resource requirements for deployment
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ResourceRequirements
  {
    /// CPU requirements (in CPU units)
    pub cpu : f64,
    /// Memory requirements in MB
    pub memory_mb : u64,
    /// GPU requirements (if any)
    pub gpu : Option< u32 >,
    /// Storage requirements in GB
    pub storage_gb : u64,
  }

  impl Default for ResourceRequirements
  {
    fn default() -> Self
    {
      Self
      {
        cpu : 1.0,
        memory_mb : 2048,
        gpu : None,
        storage_gb : 10,
      }
    }
  }

  /// Auto-scaling configuration
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct AutoScalingConfig
  {
    /// Minimum number of replicas
    pub min_replicas : u32,
    /// Maximum number of replicas
    pub max_replicas : u32,
    /// Target CPU utilization percentage
    pub target_cpu_percent : u8,
    /// Target memory utilization percentage
    pub target_memory_percent : u8,
    /// Scale up cooldown in seconds
    pub scale_up_cooldown_s : u64,
    /// Scale down cooldown in seconds
    pub scale_down_cooldown_s : u64,
  }

  impl Default for AutoScalingConfig
  {
    fn default() -> Self
    {
      Self
      {
        min_replicas : 1,
        max_replicas : 10,
        target_cpu_percent : 70,
        target_memory_percent : 80,
        scale_up_cooldown_s : 300,
        scale_down_cooldown_s : 600,
      }
    }
  }

  /// Health check configuration for deployments
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct HealthCheckConfig
  {
    /// Health check endpoint path
    pub path : String,
    /// Health check port
    pub port : u16,
    /// Interval between health checks in seconds
    pub interval_s : u64,
    /// Timeout for each health check in seconds
    pub timeout_s : u64,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold : u32,
    /// Number of consecutive successes before marking healthy
    pub success_threshold : u32,
  }

  impl Default for HealthCheckConfig
  {
    fn default() -> Self
    {
      Self
      {
        path : "/health".to_string(),
        port : 8080,
        interval_s : 30,
        timeout_s : 5,
        failure_threshold : 3,
        success_threshold : 1,
      }
    }
  }

  /// Model deployment instance
  #[ derive( Debug, Clone ) ]
  pub struct ModelDeployment
  {
    /// Deployment configuration
    pub config : DeploymentConfig,
    /// Current deployment status
    pub status : DeploymentStatus,
    /// Deployment creation timestamp
    pub created_at : SystemTime,
    /// Last status update timestamp
    pub updated_at : SystemTime,
    /// Current number of running instances
    pub running_replicas : u32,
    /// Current number of healthy instances
    pub healthy_replicas : u32,
    /// Deployment history
    deployment_history : Vec< DeploymentEvent >,
  }

  impl ModelDeployment
  {
    /// Create a new model deployment
    #[ must_use ]
    pub fn new( config : DeploymentConfig ) -> Self
    {
      Self
      {
        config,
        status : DeploymentStatus::Preparing,
        created_at : SystemTime::now(),
        updated_at : SystemTime::now(),
        running_replicas : 0,
        healthy_replicas : 0,
        deployment_history : Vec::new(),
      }
    }

    /// Update deployment status
    pub fn update_status( &mut self, status : DeploymentStatus )
    {
      let event = DeploymentEvent
      {
        timestamp : SystemTime::now(),
        event_type : DeploymentEventType::StatusChanged
        {
          from : self.status.clone(),
          to : status.clone(),
        },
        message : format!( "Status changed from {:?} to {:?}", self.status, status ),
      };

      self.status = status;
      self.updated_at = SystemTime::now();
      self.deployment_history.push( event );
    }

    /// Update replica counts
    pub fn update_replicas( &mut self, running : u32, healthy : u32 )
    {
      let event = DeploymentEvent
      {
        timestamp : SystemTime::now(),
        event_type : DeploymentEventType::ReplicasChanged
        {
          running,
          healthy,
        },
        message : format!( "Replicas updated : {running} running, {healthy} healthy" ),
      };

      self.running_replicas = running;
      self.healthy_replicas = healthy;
      self.updated_at = SystemTime::now();
      self.deployment_history.push( event );
    }

    /// Check if deployment is healthy
    #[ must_use ]
    pub fn is_healthy( &self ) -> bool
    {
      matches!( self.status, DeploymentStatus::Active ) &&
      self.healthy_replicas >= self.config.replicas
    }

    /// Check if deployment is ready
    #[ must_use ]
    pub fn is_ready( &self ) -> bool
    {
      matches!( self.status, DeploymentStatus::Active ) &&
      self.running_replicas >= self.config.replicas
    }

    /// Get deployment age
    #[ must_use ]
    pub fn age( &self ) -> Duration
    {
      self.created_at.elapsed().unwrap_or( Duration::from_secs( 0 ) )
    }

    /// Get deployment history
    #[ must_use ]
    pub fn history( &self ) -> &Vec< DeploymentEvent >
    {
      &self.deployment_history
    }

    /// Add deployment event
    pub fn add_event( &mut self, event_type : DeploymentEventType, message : String )
    {
      let event = DeploymentEvent
      {
        timestamp : SystemTime::now(),
        event_type,
        message,
      };

      self.deployment_history.push( event );
      self.updated_at = SystemTime::now();
    }
  }

  /// Deployment event for history tracking
  #[ derive( Debug, Clone ) ]
  pub struct DeploymentEvent
  {
    /// Event timestamp
    pub timestamp : SystemTime,
    /// Type of deployment event
    pub event_type : DeploymentEventType,
    /// Event message
    pub message : String,
  }

  /// Types of deployment events
  #[ derive( Debug, Clone ) ]
  pub enum DeploymentEventType
  {
    /// Status change event
    StatusChanged
    {
      /// Previous status
      from : DeploymentStatus,
      /// New status
      to : DeploymentStatus,
    },
    /// Replica count change
    ReplicasChanged
    {
      /// Current running replicas
      running : u32,
      /// Current healthy replicas
      healthy : u32,
    },
    /// Scaling event
    ScalingTriggered
    {
      /// Previous replica count
      from : u32,
      /// Target replica count
      to : u32,
      /// Scaling reason
      reason : String,
    },
    /// Deployment rollback
    RollbackTriggered
    {
      /// Target version to rollback to
      target_version : String,
      /// Rollback reason
      reason : String,
    },
    /// Health check failure
    HealthCheckFailed
    {
      /// Number of consecutive failures
      failures : u32,
    },
    /// Configuration update
    ConfigurationUpdated
    {
      /// Updated fields
      fields : Vec< String >,
    },
  }

  /// Deployment manager for orchestrating model deployments
  #[ derive( Debug ) ]
  pub struct DeploymentManager
  {
    /// Active deployments
    deployments : HashMap<  String, ModelDeployment  >,
    /// Manager configuration
    config : DeploymentManagerConfig,
  }

  /// Configuration for deployment manager
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct DeploymentManagerConfig
  {
    /// Maximum number of concurrent deployments
    pub max_deployments : usize,
    /// Default deployment timeout in milliseconds
    pub default_timeout_ms : u64,
    /// Health check retry interval in seconds
    pub health_check_retry_s : u64,
    /// Maximum deployment history size
    pub max_history_size : usize,
  }

  impl Default for DeploymentManagerConfig
  {
    fn default() -> Self
    {
      Self
      {
        max_deployments : 100,
        default_timeout_ms : 600_000, // 10 minutes
        health_check_retry_s : 30,
        max_history_size : 1000,
      }
    }
  }

  impl DeploymentManager
  {
    /// Create a new deployment manager
    #[ must_use ]
    pub fn new( config : DeploymentManagerConfig ) -> Self
    {
      Self
      {
        deployments : HashMap::new(),
        config,
      }
    }

    /// Add a deployment to the manager
    ///
    /// # Errors
    /// Returns an error if the maximum number of deployments is reached or if a deployment with the same name already exists.
    pub fn add_deployment( &mut self, deployment : ModelDeployment ) -> Result< (), String >
    {
      if self.deployments.len() >= self.config.max_deployments
      {
        return Err( "Maximum number of deployments reached".to_string() );
      }

      let deployment_name = deployment.config.name.clone();
      if self.deployments.contains_key( &deployment_name )
      {
        return Err( format!( "Deployment '{deployment_name}' already exists" ) );
      }

      self.deployments.insert( deployment_name, deployment );
      Ok( () )
    }

    /// Get a deployment by name
    #[ must_use ]
    pub fn get_deployment( &self, name : &str ) -> Option< &ModelDeployment >
    {
      self.deployments.get( name )
    }

    /// Get a mutable deployment by name
    pub fn get_deployment_mut( &mut self, name : &str ) -> Option< &mut ModelDeployment >
    {
      self.deployments.get_mut( name )
    }

    /// Remove a deployment
    pub fn remove_deployment( &mut self, name : &str ) -> Option< ModelDeployment >
    {
      self.deployments.remove( name )
    }

    /// List all deployment names
    #[ must_use ]
    pub fn deployment_names( &self ) -> Vec< String >
    {
      self.deployments.keys().cloned().collect()
    }

    /// Get deployment statistics
    #[ must_use ]
    pub fn deployment_stats( &self ) -> DeploymentStats
    {
      let mut stats = DeploymentStats
      {
        total : self.deployments.len(),
        active : 0,
        failed : 0,
        preparing : 0,
        deploying : 0,
        scaling : 0,
        total_replicas : 0,
        healthy_replicas : 0,
      };

      for deployment in self.deployments.values()
      {
        match deployment.status
        {
          DeploymentStatus::Active => stats.active += 1,
          DeploymentStatus::Failed( _ ) => stats.failed += 1,
          DeploymentStatus::Preparing => stats.preparing += 1,
          DeploymentStatus::Deploying => stats.deploying += 1,
          DeploymentStatus::Scaling => stats.scaling += 1,
          _ => {},
        }

        stats.total_replicas += deployment.running_replicas;
        stats.healthy_replicas += deployment.healthy_replicas;
      }

      stats
    }

    /// Cleanup old deployment history
    pub fn cleanup_history( &mut self )
    {
      for deployment in self.deployments.values_mut()
      {
        if deployment.deployment_history.len() > self.config.max_history_size
        {
          let excess = deployment.deployment_history.len() - self.config.max_history_size;
          deployment.deployment_history.drain( 0..excess );
        }
      }
    }
  }

  /// Deployment statistics
  #[ derive( Debug, Clone ) ]
  pub struct DeploymentStats
  {
    /// Total number of deployments
    pub total : usize,
    /// Number of active deployments
    pub active : usize,
    /// Number of failed deployments
    pub failed : usize,
    /// Number of preparing deployments
    pub preparing : usize,
    /// Number of deploying deployments
    pub deploying : usize,
    /// Number of scaling deployments
    pub scaling : usize,
    /// Total number of replicas across all deployments
    pub total_replicas : u32,
    /// Total number of healthy replicas
    pub healthy_replicas : u32,
  }

  /// Model deployment utilities
  #[ derive( Debug ) ]
  pub struct ModelDeploymentUtils;

  impl ModelDeploymentUtils
  {
    /// Create a deployment event notifier
    #[ must_use ]
    pub fn create_event_notifier() -> ( DeploymentEventSender, DeploymentEventReceiver )
    {
      let ( tx, rx ) = mpsc::unbounded_channel();
      ( DeploymentEventSender { sender : tx }, DeploymentEventReceiver { receiver : rx } )
    }

    /// Validate deployment configuration
    ///
    /// # Errors
    /// Returns a vector of validation error messages if the configuration is invalid.
    #[ must_use = "validation errors should be handled" ]
    pub fn validate_config( config : &DeploymentConfig ) -> Result< (), Vec< String > >
    {
      let mut errors = Vec::new();

      if config.name.is_empty()
      {
        errors.push( "Deployment name cannot be empty".to_string() );
      }

      if config.model_id.is_empty()
      {
        errors.push( "Model ID cannot be empty".to_string() );
      }

      if config.model_version.is_empty()
      {
        errors.push( "Model version cannot be empty".to_string() );
      }

      if config.replicas == 0
      {
        errors.push( "Replicas must be greater than 0".to_string() );
      }

      if config.resources.cpu <= 0.0
      {
        errors.push( "CPU requirements must be greater than 0".to_string() );
      }

      if config.resources.memory_mb == 0
      {
        errors.push( "Memory requirements must be greater than 0".to_string() );
      }

      if config.timeout_ms == 0
      {
        errors.push( "Timeout must be greater than 0".to_string() );
      }

      // Validate auto-scaling configuration
      if let Some( ref auto_scaling ) = config.auto_scaling
      {
        if auto_scaling.min_replicas > auto_scaling.max_replicas
        {
          errors.push( "Min replicas cannot be greater than max replicas".to_string() );
        }

        if auto_scaling.target_cpu_percent == 0 || auto_scaling.target_cpu_percent > 100
        {
          errors.push( "Target CPU percent must be between 1 and 100".to_string() );
        }

        if auto_scaling.target_memory_percent == 0 || auto_scaling.target_memory_percent > 100
        {
          errors.push( "Target memory percent must be between 1 and 100".to_string() );
        }
      }

      // Validate deployment strategy
      match &config.strategy
      {
        DeploymentStrategy::BlueGreen { traffic_split } if *traffic_split > 100 =>
        {
          errors.push( "Traffic split cannot be greater than 100%".to_string() );
        }
        DeploymentStrategy::Canary { initial_traffic, final_traffic, .. } =>
        {
          if *initial_traffic > 100 || *final_traffic > 100
          {
            errors.push( "Traffic percentages cannot be greater than 100%".to_string() );
          }
          if initial_traffic >= final_traffic
          {
            errors.push( "Initial traffic must be less than final traffic for canary deployment".to_string() );
          }
        }
        _ => {},
      }

      if errors.is_empty()
      {
        Ok( () )
      }
      else
      {
        Err( errors )
      }
    }

    /// Calculate resource requirements for deployment
    #[ must_use ]
    pub fn calculate_total_resources( config : &DeploymentConfig ) -> ResourceRequirements
    {
      ResourceRequirements
      {
        cpu : config.resources.cpu * f64::from( config.replicas ),
        memory_mb : config.resources.memory_mb * u64::from( config.replicas ),
        gpu : config.resources.gpu.map( | gpu | gpu * config.replicas ),
        storage_gb : config.resources.storage_gb * u64::from( config.replicas ),
      }
    }

    /// Estimate deployment time based on strategy and configuration
    #[ must_use ]
    pub fn estimate_deployment_time( config : &DeploymentConfig ) -> Duration
    {
      let base_time = Duration::from_secs( 60 ); // Base deployment time per replica

      let strategy_multiplier = match &config.strategy
      {
        DeploymentStrategy::Rolling { .. } => 1.5,
        DeploymentStrategy::BlueGreen { .. } => 2.0,
        DeploymentStrategy::Canary { evaluation_duration_ms, .. } =>
        {
          2.0 + ( (*evaluation_duration_ms as f64) / 60000.0 ) // Add evaluation time
        }
        DeploymentStrategy::Recreate => 0.5,
      };

      let total_seconds_f64 = ( (base_time.as_secs() as f64) * f64::from( config.replicas ) * strategy_multiplier )
        .max( 0.0 )
        .min( u64::MAX as f64 );
      let total_seconds = if total_seconds_f64.is_finite() && total_seconds_f64 >= 0.0
      {
        #[ allow(clippy::cast_possible_truncation, clippy::cast_sign_loss) ]
        let result = total_seconds_f64 as u64;
        result
      }
      else
      {
        0u64
      };
      Duration::from_secs( total_seconds )
    }

    /// Generate deployment rollback plan
    #[ must_use ]
    pub fn create_rollback_plan( current_config : &DeploymentConfig, target_version : String ) -> DeploymentConfig
    {
      let mut rollback_config = current_config.clone();
      rollback_config.model_version = target_version;
      rollback_config.strategy = DeploymentStrategy::Rolling
      {
        max_surge : 1,
        max_unavailable : 0,
      };
      rollback_config
    }

    /// Check if scaling is needed based on current metrics
    #[ must_use ]
    pub fn should_scale(
      current_replicas : u32,
      auto_scaling : &AutoScalingConfig,
      cpu_utilization : u8,
      memory_utilization : u8
    ) -> Option< u32 >
    {
      if cpu_utilization > auto_scaling.target_cpu_percent || memory_utilization > auto_scaling.target_memory_percent
      {
        // Scale up
        Some( core::cmp::min( current_replicas + 1, auto_scaling.max_replicas ) )
      }
      else if cpu_utilization < auto_scaling.target_cpu_percent / 2 && memory_utilization < auto_scaling.target_memory_percent / 2
      {
        // Scale down
        Some( core::cmp::max( current_replicas.saturating_sub( 1 ), auto_scaling.min_replicas ) )
      }
      else
      {
        None
      }
    }
  }

  /// Sender for deployment events
  #[ derive( Debug, Clone ) ]
  pub struct DeploymentEventSender
  {
    sender : mpsc::UnboundedSender< DeploymentNotification >,
  }

  impl DeploymentEventSender
  {
    /// Send a deployment event
    ///
    /// # Errors
    /// Returns an error if the event channel is closed.
    pub fn send_event( &self, event : DeploymentNotification ) -> Result< (), &'static str >
    {
      self.sender.send( event ).map_err( | _ | "Failed to send deployment event" )
    }

    /// Send deployment started event
    ///
    /// # Errors
    /// Returns an error if the event channel is closed.
    pub fn send_deployment_started( &self, name : String, config : DeploymentConfig ) -> Result< (), &'static str >
    {
      self.send_event( DeploymentNotification::DeploymentStarted { name, config } )
    }

    /// Send deployment completed event
    ///
    /// # Errors
    /// Returns an error if the event channel is closed.
    pub fn send_deployment_completed( &self, name : String ) -> Result< (), &'static str >
    {
      self.send_event( DeploymentNotification::DeploymentCompleted { name } )
    }

    /// Send deployment failed event
    ///
    /// # Errors
    /// Returns an error if the event channel is closed.
    pub fn send_deployment_failed( &self, name : String, error : String ) -> Result< (), &'static str >
    {
      self.send_event( DeploymentNotification::DeploymentFailed { name, error } )
    }

    /// Send scaling event
    ///
    /// # Errors
    /// Returns an error if the event channel is closed.
    pub fn send_scaling_triggered( &self, name : String, from : u32, to : u32 ) -> Result< (), &'static str >
    {
      self.send_event( DeploymentNotification::ScalingTriggered { name, from, to } )
    }
  }

  /// Receiver for deployment events
  #[ derive( Debug ) ]
  pub struct DeploymentEventReceiver
  {
    receiver : mpsc::UnboundedReceiver< DeploymentNotification >,
  }

  impl DeploymentEventReceiver
  {
    /// Try to receive a deployment event (non-blocking)
    pub fn try_recv( &mut self ) -> Option< DeploymentNotification >
    {
      self.receiver.try_recv().ok()
    }

    /// Receive next deployment event (blocking)
    pub async fn recv( &mut self ) -> Option< DeploymentNotification >
    {
      self.receiver.recv().await
    }
  }

  /// Deployment notification types
  #[ derive( Debug, Clone ) ]
  pub enum DeploymentNotification
  {
    /// Deployment started
    DeploymentStarted
    {
      /// Deployment name
      name : String,
      /// Deployment configuration
      config : DeploymentConfig,
    },
    /// Deployment completed successfully
    DeploymentCompleted
    {
      /// Deployment name
      name : String,
    },
    /// Deployment failed
    DeploymentFailed
    {
      /// Deployment name
      name : String,
      /// Error message
      error : String,
    },
    /// Auto-scaling triggered
    ScalingTriggered
    {
      /// Deployment name
      name : String,
      /// Previous replica count
      from : u32,
      /// Target replica count
      to : u32,
    },
    /// Rollback triggered
    RollbackTriggered
    {
      /// Deployment name
      name : String,
      /// Target version
      target_version : String,
    },
  }
}

crate ::mod_interface!
{
  exposed use private::DeploymentStatus;
  exposed use private::DeploymentStrategy;
  exposed use private::DeploymentConfig;
  exposed use private::ResourceRequirements;
  exposed use private::AutoScalingConfig;
  exposed use private::HealthCheckConfig;
  exposed use private::ModelDeployment;
  exposed use private::DeploymentEvent;
  exposed use private::DeploymentEventType;
  exposed use private::DeploymentManager;
  exposed use private::DeploymentManagerConfig;
  exposed use private::DeploymentStats;
  exposed use private::ModelDeploymentUtils;
  exposed use private::DeploymentEventSender;
  exposed use private::DeploymentEventReceiver;
  exposed use private::DeploymentNotification;
}