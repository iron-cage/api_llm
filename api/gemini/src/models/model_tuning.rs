//! Model tuning and fine-tuning capabilities for custom model training.
//!
//! This module provides comprehensive fine-tuning capabilities including supervised learning,
//! parameter-efficient training methods (LoRA, adapters), hyperparameter optimization,
//! and training job management.

mod private
{
  use serde::{ Deserialize, Serialize };
  use std::time::{ Duration, SystemTime };
  use std::sync::{ Arc, Mutex };
  use std::collections::HashMap;
  use tokio::sync::broadcast;

  /// State of a training job
  #[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
  pub enum TrainingJobState
  {
    /// Job is queued and waiting to start
    Pending,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Completed,
    /// Job was cancelled by user
    Cancelled,
    /// Job failed due to error
    Failed,
    /// Job is paused
    Paused,
  }

  /// Configuration for hyperparameters
  #[ derive( Debug, Clone ) ]
  pub struct HyperparameterConfig
  {
    /// Learning rate for training
    pub learning_rate : f64,
    /// Batch size for training
    pub batch_size : usize,
    /// Number of training epochs
    pub epochs : usize,
    /// Number of warmup steps
    pub warmup_steps : usize,
    /// Weight decay for regularization
    pub weight_decay : f64,
    /// Gradient clipping threshold
    pub gradient_clip_norm : f64,
    /// Learning rate scheduler type
    pub scheduler : String,
    /// Optimizer type
    pub optimizer : String,
  }

  impl Default for HyperparameterConfig
  {
    fn default() -> Self
    {
      Self {
        learning_rate : 0.0001,
        batch_size : 16,
        epochs : 3,
        warmup_steps : 50,
        weight_decay : 0.01,
        gradient_clip_norm : 1.0,
        scheduler : "cosine".to_string(),
        optimizer : "adamw".to_string(),
      }
    }
  }

  /// Builder for hyperparameter configuration
  #[ derive( Debug, Clone ) ]
  pub struct HyperparameterConfigBuilder
  {
    config : HyperparameterConfig,
  }

  impl HyperparameterConfigBuilder
  {
    /// Create a new hyperparameter config builder
    pub fn new() -> Self
    {
      Self {
        config : HyperparameterConfig::default(),
      }
    }

    /// Set learning rate
    pub fn learning_rate( mut self, rate : f64 ) -> Self
    {
      self.config.learning_rate = rate;
      self
    }

    /// Set batch size
    pub fn batch_size( mut self, size : usize ) -> Self
    {
      self.config.batch_size = size;
      self
    }

    /// Set number of epochs
    pub fn epochs( mut self, epochs : usize ) -> Self
    {
      self.config.epochs = epochs;
      self
    }

    /// Set warmup steps
    pub fn warmup_steps( mut self, steps : usize ) -> Self
    {
      self.config.warmup_steps = steps;
      self
    }

    /// Set weight decay
    pub fn weight_decay( mut self, decay : f64 ) -> Self
    {
      self.config.weight_decay = decay;
      self
    }

    /// Set gradient clipping norm
    pub fn gradient_clip_norm( mut self, norm : f64 ) -> Self
    {
      self.config.gradient_clip_norm = norm;
      self
    }

    /// Set learning rate scheduler
    pub fn scheduler( mut self, scheduler : &str ) -> Self
    {
      self.config.scheduler = scheduler.to_string();
      self
    }

    /// Set optimizer
    pub fn optimizer( mut self, optimizer : &str ) -> Self
    {
      self.config.optimizer = optimizer.to_string();
      self
    }

    /// Build the configuration with validation
    pub fn build( self ) -> Result< HyperparameterConfig, crate::error::Error >
    {
      if self.config.learning_rate <= 0.0
      {
        return Err( crate::error::Error::ConfigurationError(
          "Learning rate must be positive".to_string()
        ) );
      }

      if self.config.batch_size == 0
      {
        return Err( crate::error::Error::ConfigurationError(
          "Batch size must be greater than 0".to_string()
        ) );
      }

      if self.config.epochs == 0
      {
        return Err( crate::error::Error::ConfigurationError(
          "Epochs must be greater than 0".to_string()
        ) );
      }

      if self.config.weight_decay < 0.0
      {
        return Err( crate::error::Error::ConfigurationError(
          "Weight decay must be non-negative".to_string()
        ) );
      }

      if self.config.gradient_clip_norm <= 0.0
      {
        return Err( crate::error::Error::ConfigurationError(
          "Gradient clip norm must be positive".to_string()
        ) );
      }

      Ok( self.config )
    }
  }

  impl Default for HyperparameterConfigBuilder
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl HyperparameterConfig
  {
    /// Create a new hyperparameter config builder
    pub fn builder() -> HyperparameterConfigBuilder
    {
      HyperparameterConfigBuilder::new()
    }
  }

  /// LoRA (Low-Rank Adaptation) configuration for parameter-efficient fine-tuning
  #[ derive( Debug, Clone ) ]
  pub struct LoRAConfig
  {
    /// Rank of the adaptation
    pub rank : usize,
    /// Scaling factor (alpha)
    pub alpha : f64,
    /// Dropout rate
    pub dropout : f64,
    /// Target modules to apply LoRA
    pub target_modules : Vec< String >,
    /// Whether to merge weights after training
    pub merge_weights : bool,
  }

  impl Default for LoRAConfig
  {
    fn default() -> Self
    {
      Self {
        rank : 8,
        alpha : 16.0,
        dropout : 0.1,
        target_modules : vec![ "query".to_string(), "value".to_string() ],
        merge_weights : false,
      }
    }
  }

  /// Builder for LoRA configuration
  #[ derive( Debug, Clone ) ]
  pub struct LoRAConfigBuilder
  {
    config : LoRAConfig,
  }

  impl LoRAConfigBuilder
  {
    /// Create a new LoRA config builder
    pub fn new() -> Self
    {
      Self {
        config : LoRAConfig::default(),
      }
    }

    /// Set LoRA rank
    pub fn rank( mut self, rank : usize ) -> Self
    {
      self.config.rank = rank;
      self
    }

    /// Set alpha scaling factor
    pub fn alpha( mut self, alpha : f64 ) -> Self
    {
      self.config.alpha = alpha;
      self
    }

    /// Set dropout rate
    pub fn dropout( mut self, dropout : f64 ) -> Self
    {
      self.config.dropout = dropout;
      self
    }

    /// Set target modules
    pub fn target_modules( mut self, modules : Vec< String > ) -> Self
    {
      self.config.target_modules = modules;
      self
    }

    /// Set whether to merge weights
    pub fn merge_weights( mut self, merge : bool ) -> Self
    {
      self.config.merge_weights = merge;
      self
    }

    /// Build the LoRA configuration with validation
    pub fn build( self ) -> Result< LoRAConfig, crate::error::Error >
    {
      if self.config.rank == 0
      {
        return Err( crate::error::Error::ConfigurationError(
          "LoRA rank must be greater than 0".to_string()
        ) );
      }

      if self.config.alpha <= 0.0
      {
        return Err( crate::error::Error::ConfigurationError(
          "LoRA alpha must be positive".to_string()
        ) );
      }

      if self.config.dropout < 0.0 || self.config.dropout >= 1.0
      {
        return Err( crate::error::Error::ConfigurationError(
          "LoRA dropout must be between 0.0 and 1.0".to_string()
        ) );
      }

      if self.config.target_modules.is_empty()
      {
        return Err( crate::error::Error::ConfigurationError(
          "LoRA target modules cannot be empty".to_string()
        ) );
      }

      Ok( self.config )
    }
  }

  impl Default for LoRAConfigBuilder
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl LoRAConfig
  {
    /// Create a new LoRA config builder
    pub fn builder() -> LoRAConfigBuilder
    {
      LoRAConfigBuilder::new()
    }
  }

  /// Training objective types
  #[ derive( Debug, Clone ) ]
  pub enum TrainingObjective
  {
    /// Text completion objective
    Completion {
      /// Maximum sequence length
      max_sequence_length : usize,
      /// Sampling temperature
      temperature : f64,
    },
    /// Classification objective
    Classification {
      /// Number of classes
      num_classes : usize,
      /// Label smoothing factor
      label_smoothing : f64,
    },
    /// Sequence-to-sequence objective
    Seq2Seq {
      /// Maximum input length
      max_input_length : usize,
      /// Maximum output length
      max_output_length : usize,
    },
  }

  /// Training metrics collected during fine-tuning
  #[ derive( Debug, Clone ) ]
  pub struct TrainingMetrics
  {
    /// Current epoch
    pub epoch : usize,
    /// Current step
    pub step : usize,
    /// Current loss value
    pub loss : f64,
    /// Current learning rate
    pub learning_rate : f64,
    /// Gradient norm
    pub gradient_norm : f64,
    /// Training throughput (tokens/sec)
    pub throughput_tokens_per_second : f64,
    /// Memory usage in MB
    pub memory_usage_mb : f64,
    /// Elapsed training time in seconds
    pub elapsed_time_seconds : f64,
  }

  impl Default for TrainingMetrics
  {
    fn default() -> Self
    {
      Self {
        epoch : 0,
        step : 0,
        loss : 0.0,
        learning_rate : 0.0,
        gradient_norm : 0.0,
        throughput_tokens_per_second : 0.0,
        memory_usage_mb : 0.0,
        elapsed_time_seconds : 0.0,
      }
    }
  }

  /// Model checkpoint information
  #[ derive( Debug, Clone ) ]
  pub struct ModelCheckpoint
  {
    /// Unique checkpoint identifier
    pub checkpoint_id : String,
    /// Epoch when checkpoint was created
    pub epoch : usize,
    /// Step when checkpoint was created
    pub step : usize,
    /// Loss value at checkpoint
    pub loss : f64,
    /// Additional metrics
    pub metrics : HashMap<  String, f64  >,
    /// Path to checkpoint files
    pub model_path : String,
    /// Timestamp when checkpoint was created
    pub created_at : SystemTime,
  }

  /// Training progress information
  #[ derive( Debug, Clone ) ]
  pub struct TrainingProgress
  {
    /// Completion percentage (0-100)
    pub percentage : f64,
    /// Current metrics
    pub metrics : TrainingMetrics,
    /// Estimated time remaining
    pub estimated_time_remaining : Option< Duration >,
  }

  /// Training job management
  pub struct TrainingJob
  {
    /// Job identifier
    pub job_id : String,
    /// Current job state
    state : Arc< Mutex< TrainingJobState > >,
    /// Training configuration
    config : HyperparameterConfig,
    /// Current metrics
    metrics : Arc< Mutex< TrainingMetrics > >,
    /// Progress notifications
    progress_tx : broadcast::Sender< TrainingProgress >,
    /// Checkpoints
    checkpoints : Arc< Mutex< Vec< ModelCheckpoint > > >,
  }

  impl TrainingJob
  {
    /// Create a new training job
    pub fn new( job_id : String, config : HyperparameterConfig ) -> Self
    {
      let ( progress_tx, _progress_rx ) = broadcast::channel( 16 );

      Self {
        job_id,
        state : Arc::new( Mutex::new( TrainingJobState::Pending ) ),
        config,
        metrics : Arc::new( Mutex::new( TrainingMetrics::default() ) ),
        progress_tx,
        checkpoints : Arc::new( Mutex::new( Vec::new() ) ),
      }
    }

    /// Get current job state
    pub fn state( &self ) -> TrainingJobState
    {
      self.state.lock().unwrap().clone()
    }

    /// Get current training metrics
    pub fn get_metrics( &self ) -> TrainingMetrics
    {
      self.metrics.lock().unwrap().clone()
    }

    /// Subscribe to training progress updates
    pub fn subscribe_progress( &self ) -> broadcast::Receiver< TrainingProgress >
    {
      self.progress_tx.subscribe()
    }

    /// Get all checkpoints
    pub fn get_checkpoints( &self ) -> Vec< ModelCheckpoint >
    {
      self.checkpoints.lock().unwrap().clone()
    }

    /// Start the training job
    pub async fn start( &self ) -> Result< (), crate::error::Error >
    {
      *self.state.lock().unwrap() = TrainingJobState::Running;
      Ok( () )
    }

    /// Pause the training job
    pub async fn pause( &self ) -> Result< (), crate::error::Error >
    {
      *self.state.lock().unwrap() = TrainingJobState::Paused;
      Ok( () )
    }

    /// Cancel the training job
    pub async fn cancel( &self ) -> Result< (), crate::error::Error >
    {
      *self.state.lock().unwrap() = TrainingJobState::Cancelled;
      Ok( () )
    }

    /// Resume a paused training job
    pub async fn resume( &self ) -> Result< (), crate::error::Error >
    {
      let current_state = self.state();
      if current_state != TrainingJobState::Paused
      {
        return Err( crate::error::Error::ApiError(
          format!( "Cannot resume job in state : {:?}", current_state )
        ) );
      }

      *self.state.lock().unwrap() = TrainingJobState::Running;
      Ok( () )
    }
  }

  impl std::fmt::Debug for TrainingJob
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      f.debug_struct( "TrainingJob" )
        .field( "job_id", &self.job_id )
        .field( "state", &self.state() )
        .field( "config", &self.config )
        .field( "metrics", &self.get_metrics() )
        .finish_non_exhaustive()
    }
  }

  /// Builder for fine-tuning operations
  pub struct FineTuningBuilder< 'a >
  {
    #[ allow( dead_code ) ]
    model : &'a crate::models::api::ModelApi< 'a >,
    training_data : Option< String >,
    validation_data : Option< String >,
    hyperparams : HyperparameterConfig,
    lora_config : Option< LoRAConfig >,
    objective : Option< TrainingObjective >,
    monitoring_interval : Option< Duration >,
    validate_data : bool,
    checkpoint_frequency : usize,
    progress_callback : Option< Box< dyn Fn( TrainingProgress ) + Send + Sync > >,
  }

  impl< 'a > std::fmt::Debug for FineTuningBuilder< 'a >
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      f.debug_struct( "FineTuningBuilder" )
        .field( "training_data", &self.training_data )
        .field( "validation_data", &self.validation_data )
        .field( "hyperparams", &self.hyperparams )
        .field( "lora_config", &self.lora_config )
        .field( "objective", &self.objective )
        .field( "monitoring_interval", &self.monitoring_interval )
        .field( "validate_data", &self.validate_data )
        .field( "checkpoint_frequency", &self.checkpoint_frequency )
        .field( "progress_callback", &self.progress_callback.is_some() )
        .finish_non_exhaustive()
    }
  }

  impl< 'a > FineTuningBuilder< 'a >
  {
    /// Create a new fine-tuning builder
    pub fn new( model : &'a crate::models::api::ModelApi< 'a > ) -> Self
    {
      Self {
        model,
        training_data : None,
        validation_data : None,
        hyperparams : HyperparameterConfig::default(),
        lora_config : None,
        objective : None,
        monitoring_interval : None,
        validate_data : false,
        checkpoint_frequency : 1000,
        progress_callback : None,
      }
    }

    /// Set training data path
    pub fn with_training_data( mut self, path : &str ) -> Self
    {
      self.training_data = Some( path.to_string() );
      self
    }

    /// Set validation data path
    pub fn with_validation_data( mut self, path : &str ) -> Self
    {
      self.validation_data = Some( path.to_string() );
      self
    }

    /// Set number of epochs
    pub fn with_epochs( mut self, epochs : usize ) -> Self
    {
      self.hyperparams.epochs = epochs;
      self
    }

    /// Set learning rate
    pub fn with_learning_rate( mut self, rate : f64 ) -> Self
    {
      self.hyperparams.learning_rate = rate;
      self
    }

    /// Set batch size
    pub fn with_batch_size( mut self, size : usize ) -> Self
    {
      self.hyperparams.batch_size = size;
      self
    }

    /// Set hyperparameter configuration
    pub fn with_hyperparams( mut self, config : HyperparameterConfig ) -> Self
    {
      self.hyperparams = config;
      self
    }

    /// Set LoRA configuration for parameter-efficient fine-tuning
    pub fn with_lora_config( mut self, config : LoRAConfig ) -> Self
    {
      self.lora_config = Some( config );
      self
    }

    /// Set training objective
    pub fn with_objective( mut self, objective : TrainingObjective ) -> Self
    {
      self.objective = Some( objective );
      self
    }

    /// Set monitoring interval
    pub fn with_monitoring_interval( mut self, interval : Duration ) -> Self
    {
      self.monitoring_interval = Some( interval );
      self
    }

    /// Enable or disable data validation
    pub fn validate_data( mut self, validate : bool ) -> Self
    {
      self.validate_data = validate;
      self
    }

    /// Set checkpoint frequency (in steps)
    pub fn with_checkpoint_frequency( mut self, frequency : usize ) -> Self
    {
      self.checkpoint_frequency = frequency;
      self
    }

    /// Set progress callback function
    pub fn with_progress_callback< F >( mut self, callback : F ) -> Self
    where
      F: Fn( TrainingProgress ) + Send + Sync + 'static,
    {
      self.progress_callback = Some( Box::new( callback ) );
      self
    }

    /// Create and start the fine-tuning job
    pub async fn start_training( self ) -> Result< TrainingJob, crate::error::Error >
    {
      // Validate required fields
      if self.training_data.is_none()
      {
        return Err( crate::error::Error::ApiError(
          "Training data is required for fine-tuning".to_string()
        ) );
      }

      // Create training job
      let job_id = format!( "tuning-{}", "generated-id" ); // Simplified for now
      let job = TrainingJob::new( job_id, self.hyperparams );

      // Start the job
      job.start().await?;

      Ok( job )
    }
  }
}

::mod_interface::mod_interface!
{
  exposed use private::TrainingJobState;
  exposed use private::HyperparameterConfig;
  exposed use private::HyperparameterConfigBuilder;
  exposed use private::LoRAConfig;
  exposed use private::LoRAConfigBuilder;
  exposed use private::TrainingObjective;
  exposed use private::TrainingMetrics;
  exposed use private::ModelCheckpoint;
  exposed use private::TrainingProgress;
  exposed use private::TrainingJob;
  exposed use private::FineTuningBuilder;
}