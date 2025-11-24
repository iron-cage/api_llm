//! Model tuning tests
//!
//! This module contains comprehensive tests for the model tuning functionality,
//! testing all core tuning types, configurations, and utilities.

#![ allow( clippy::float_cmp ) ] // Acceptable in tests for exact value checking
#![ allow( clippy::manual_string_new ) ] // Testing edge cases with empty strings
#![ allow( clippy::uninlined_format_args ) ] // Test readability over micro-optimizations

use api_openai::*;
use std::time::SystemTime;
use core::time::Duration;
use std::collections::HashMap;

#[ tokio::test ]
async fn test_tuning_status_variants()
{
  let validating = TuningStatus::Validating;
  assert!( matches!( validating, TuningStatus::Validating ) );

  let queued = TuningStatus::Queued;
  assert!( matches!( queued, TuningStatus::Queued ) );

  let running = TuningStatus::Running;
  assert!( matches!( running, TuningStatus::Running ) );

  let succeeded = TuningStatus::Succeeded;
  assert!( matches!( succeeded, TuningStatus::Succeeded ) );

  let failed = TuningStatus::Failed( "Training error".to_string() );
  assert!( matches!( failed, TuningStatus::Failed( _ ) ) );

  let cancelled = TuningStatus::Cancelled;
  assert!( matches!( cancelled, TuningStatus::Cancelled ) );
}

#[ tokio::test ]
async fn test_training_objective_variants()
{
  let language_modeling = TrainingObjective::LanguageModeling;
  assert!( matches!( language_modeling, TrainingObjective::LanguageModeling ) );

  let supervised = TrainingObjective::SupervisedFineTuning;
  assert!( matches!( supervised, TrainingObjective::SupervisedFineTuning ) );

  let rlhf = TrainingObjective::RLHF;
  assert!( matches!( rlhf, TrainingObjective::RLHF ) );

  let custom = TrainingObjective::Custom
  {
    name : "custom_objective".to_string(),
    parameters : HashMap::new(),
  };
  assert!( matches!( custom, TrainingObjective::Custom { .. } ) );
}

#[ tokio::test ]
async fn test_fine_tuning_method_variants()
{
  let full = FineTuningMethod::Full;
  assert!( matches!( full, FineTuningMethod::Full ) );

  let lora = FineTuningMethod::LoRA
  {
    rank : 16,
    alpha : 32.0,
    dropout : 0.1,
  };
  assert!( matches!( lora, FineTuningMethod::LoRA { .. } ) );

  let adapter = FineTuningMethod::Adapter
  {
    hidden_dim : 256,
    num_layers : 2,
  };
  assert!( matches!( adapter, FineTuningMethod::Adapter { .. } ) );

  let prefix = FineTuningMethod::PrefixTuning
  {
    prefix_length : 10,
    embedding_dim : 768,
  };
  assert!( matches!( prefix, FineTuningMethod::PrefixTuning { .. } ) );
}

#[ tokio::test ]
async fn test_hyperparameters()
{
  let params = HyperParameters
  {
    learning_rate : 1e-4,
    batch_size : 32,
    epochs : 5,
    warmup_steps : 100,
    weight_decay : 0.01,
    gradient_clip_norm : 1.0,
    lr_schedule : "cosine".to_string(),
    custom_params : HashMap::new(),
  };

  assert_eq!( params.learning_rate, 1e-4 );
  assert_eq!( params.batch_size, 32 );
  assert_eq!( params.epochs, 5 );
  assert_eq!( params.warmup_steps, 100 );
  assert_eq!( params.weight_decay, 0.01 );
  assert_eq!( params.gradient_clip_norm, 1.0 );
  assert_eq!( params.lr_schedule, "cosine" );
}

#[ tokio::test ]
async fn test_hyperparameters_default()
{
  let default_params = HyperParameters::default();

  assert_eq!( default_params.learning_rate, 1e-4 );
  assert_eq!( default_params.batch_size, 32 );
  assert_eq!( default_params.epochs, 3 );
  assert_eq!( default_params.warmup_steps, 100 );
  assert_eq!( default_params.weight_decay, 0.01 );
  assert_eq!( default_params.gradient_clip_norm, 1.0 );
  assert_eq!( default_params.lr_schedule, "linear" );
  assert!( default_params.custom_params.is_empty() );
}

#[ tokio::test ]
async fn test_training_data_config()
{
  let mut preprocessing = HashMap::new();
  preprocessing.insert( "normalize".to_string(), "true".to_string() );

  let config = TrainingDataConfig
  {
    training_file : "train.jsonl".to_string(),
    validation_file : Some( "val.jsonl".to_string() ),
    data_format : "jsonl".to_string(),
    max_sequence_length : 2048,
    preprocessing,
  };

  assert_eq!( config.training_file, "train.jsonl" );
  assert_eq!( config.validation_file, Some( "val.jsonl".to_string() ) );
  assert_eq!( config.data_format, "jsonl" );
  assert_eq!( config.max_sequence_length, 2048 );
  assert_eq!( config.preprocessing.len(), 1 );
}

#[ tokio::test ]
async fn test_resource_requirements()
{
  let requirements = TuningResourceRequirements
  {
    gpu_count : 4,
    gpu_type : Some( "A100".to_string() ),
    memory_gb : 64,
    cpu_cores : 16,
    storage_gb : 500,
  };

  assert_eq!( requirements.gpu_count, 4 );
  assert_eq!( requirements.gpu_type, Some( "A100".to_string() ) );
  assert_eq!( requirements.memory_gb, 64 );
  assert_eq!( requirements.cpu_cores, 16 );
  assert_eq!( requirements.storage_gb, 500 );
}

#[ tokio::test ]
async fn test_resource_requirements_default()
{
  let default_requirements = TuningResourceRequirements::default();

  assert_eq!( default_requirements.gpu_count, 1 );
  assert_eq!( default_requirements.gpu_type, None );
  assert_eq!( default_requirements.memory_gb, 16 );
  assert_eq!( default_requirements.cpu_cores, 4 );
  assert_eq!( default_requirements.storage_gb, 100 );
}

#[ tokio::test ]
async fn test_checkpoint_config()
{
  let config = CheckpointConfig
  {
    enabled : true,
    save_interval : 500,
    max_checkpoints : 10,
    save_directory : "./models".to_string(),
  };

  assert!( config.enabled );
  assert_eq!( config.save_interval, 500 );
  assert_eq!( config.max_checkpoints, 10 );
  assert_eq!( config.save_directory, "./models" );
}

#[ tokio::test ]
async fn test_checkpoint_config_default()
{
  let default_config = CheckpointConfig::default();

  assert!( default_config.enabled );
  assert_eq!( default_config.save_interval, 1000 );
  assert_eq!( default_config.max_checkpoints, 5 );
  assert_eq!( default_config.save_directory, "./checkpoints" );
}

#[ tokio::test ]
async fn test_model_checkpoint()
{
  let mut validation_metrics = HashMap::new();
  validation_metrics.insert( "accuracy".to_string(), 0.95 );
  validation_metrics.insert( "f1_score".to_string(), 0.92 );

  let checkpoint = ModelCheckpoint
  {
    checkpoint_id : "checkpoint_1000".to_string(),
    step : 1000,
    loss : 0.25,
    validation_metrics,
    created_at : SystemTime::now(),
    file_path : "./checkpoints/model_1000.pt".to_string(),
  };

  assert_eq!( checkpoint.checkpoint_id, "checkpoint_1000" );
  assert_eq!( checkpoint.step, 1000 );
  assert_eq!( checkpoint.loss, 0.25 );
  assert_eq!( checkpoint.validation_metrics.len(), 2 );
  assert_eq!( checkpoint.file_path, "./checkpoints/model_1000.pt" );
}

#[ tokio::test ]
async fn test_training_metrics()
{
  let mut custom_metrics = HashMap::new();
  custom_metrics.insert( "perplexity".to_string(), 15.5 );

  let metrics = TrainingMetrics
  {
    step : 2500,
    epoch : 2,
    training_loss : 0.18,
    validation_loss : Some( 0.22 ),
    learning_rate : 5e-5,
    throughput : 1200.0,
    eta_seconds : Some( 3600 ),
    custom_metrics,
  };

  assert_eq!( metrics.step, 2500 );
  assert_eq!( metrics.epoch, 2 );
  assert_eq!( metrics.training_loss, 0.18 );
  assert_eq!( metrics.validation_loss, Some( 0.22 ) );
  assert_eq!( metrics.learning_rate, 5e-5 );
  assert_eq!( metrics.throughput, 1200.0 );
  assert_eq!( metrics.eta_seconds, Some( 3600 ) );
  assert_eq!( metrics.custom_metrics.len(), 1 );
}

#[ tokio::test ]
async fn test_tuning_job_config()
{
  let config = TuningJobConfig
  {
    job_name : "bert_finetuning".to_string(),
    base_model : "bert-base-uncased".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : Some( "val.jsonl".to_string() ),
      data_format : "jsonl".to_string(),
      max_sequence_length : 512,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::LoRA
    {
      rank : 16,
      alpha : 32.0,
      dropout : 0.1,
    },
    objective : TrainingObjective::SupervisedFineTuning,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  assert_eq!( config.job_name, "bert_finetuning" );
  assert_eq!( config.base_model, "bert-base-uncased" );
  assert!( matches!( config.method, FineTuningMethod::LoRA { .. } ) );
  assert!( matches!( config.objective, TrainingObjective::SupervisedFineTuning ) );
}

#[ tokio::test ]
async fn test_tuning_job_creation()
{
  let config = TuningJobConfig
  {
    job_name : "test_job".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  let job = TuningJob::new( config );

  assert_eq!( job.config.job_name, "test_job" );
  assert!( matches!( job.status, TuningStatus::Validating ) );
  assert!( job.current_metrics.is_none() );
  assert!( job.checkpoints.is_empty() );
  assert!( job.execution_log.is_empty() );
}

#[ tokio::test ]
async fn test_tuning_job_status_update()
{
  let config = TuningJobConfig
  {
    job_name : "test_job".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  let mut job = TuningJob::new( config );
  assert!( matches!( job.status, TuningStatus::Validating ) );

  job.update_status( TuningStatus::Running );
  assert!( matches!( job.status, TuningStatus::Running ) );
  assert_eq!( job.execution_log.len(), 1 );
}

#[ tokio::test ]
async fn test_tuning_job_metrics_update()
{
  let config = TuningJobConfig
  {
    job_name : "test_job".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  let mut job = TuningJob::new( config );
  assert!( job.current_metrics.is_none() );

  let metrics = TrainingMetrics
  {
    step : 100,
    epoch : 1,
    training_loss : 0.5,
    validation_loss : None,
    learning_rate : 1e-4,
    throughput : 800.0,
    eta_seconds : Some( 7200 ),
    custom_metrics : HashMap::new(),
  };

  job.update_metrics( metrics );
  assert!( job.current_metrics.is_some() );
  assert_eq!( job.execution_log.len(), 1 );
}

#[ tokio::test ]
async fn test_tuning_job_checkpoint()
{
  let config = TuningJobConfig
  {
    job_name : "test_job".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  let mut job = TuningJob::new( config );
  assert!( job.checkpoints.is_empty() );

  let checkpoint = ModelCheckpoint
  {
    checkpoint_id : "checkpoint_500".to_string(),
    step : 500,
    loss : 0.3,
    validation_metrics : HashMap::new(),
    created_at : SystemTime::now(),
    file_path : "./checkpoints/model_500.pt".to_string(),
  };

  job.add_checkpoint( checkpoint );
  assert_eq!( job.checkpoints.len(), 1 );
  assert_eq!( job.execution_log.len(), 1 );
}

#[ tokio::test ]
async fn test_tuning_job_duration()
{
  let config = TuningJobConfig
  {
    job_name : "test_job".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  let job = TuningJob::new( config );
  let duration = job.duration();

  // Duration should be very small (just created)
  assert!( duration.as_secs() < 5 );
}

#[ tokio::test ]
async fn test_tuning_manager_creation()
{
  let manager = TuningManager::new();
  let stats = manager.tuning_stats();

  assert_eq!( stats.total, 0 );
  assert_eq!( stats.validating, 0 );
  assert_eq!( stats.queued, 0 );
  assert_eq!( stats.running, 0 );
  assert_eq!( stats.succeeded, 0 );
  assert_eq!( stats.failed, 0 );
  assert_eq!( stats.cancelled, 0 );
}

#[ tokio::test ]
async fn test_tuning_manager_create_job()
{
  let mut manager = TuningManager::new();

  let config = TuningJobConfig
  {
    job_name : "test_job".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  let result = manager.create_job( config ).await;
  assert!( result.is_ok() );
  assert_eq!( result.unwrap(), "test_job" );

  let stats = manager.tuning_stats();
  assert_eq!( stats.total, 1 );
  assert_eq!( stats.validating, 1 );
}

#[ tokio::test ]
async fn test_tuning_manager_get_job()
{
  let mut manager = TuningManager::new();

  let config = TuningJobConfig
  {
    job_name : "test_job".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  manager.create_job( config ).await.unwrap();
  let job = manager.get_job( "test_job" ).await;

  assert!( job.is_some() );
  assert_eq!( job.unwrap().config.job_name, "test_job" );
}

#[ tokio::test ]
async fn test_tuning_manager_list_jobs()
{
  let mut manager = TuningManager::new();

  let config1 = TuningJobConfig
  {
    job_name : "job1".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train1.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  let config2 = TuningJobConfig
  {
    job_name : "job2".to_string(),
    base_model : "gpt-4".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train2.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 2048,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::LoRA
    {
      rank : 16,
      alpha : 32.0,
      dropout : 0.1,
    },
    objective : TrainingObjective::SupervisedFineTuning,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  manager.create_job( config1 ).await.unwrap();
  manager.create_job( config2 ).await.unwrap();

  let jobs = manager.list_jobs().await;
  assert_eq!( jobs.len(), 2 );

  let stats = manager.tuning_stats();
  assert_eq!( stats.total, 2 );
  assert_eq!( stats.validating, 2 );
}

#[ tokio::test ]
async fn test_tuning_manager_update_status()
{
  let mut manager = TuningManager::new();

  let config = TuningJobConfig
  {
    job_name : "test_job".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  manager.create_job( config ).await.unwrap();
  let result = manager.update_job_status( "test_job", TuningStatus::Running ).await;

  assert!( result.is_ok() );

  let job = manager.get_job( "test_job" ).await.unwrap();
  assert!( matches!( job.status, TuningStatus::Running ) );

  let stats = manager.tuning_stats();
  assert_eq!( stats.running, 1 );
  assert_eq!( stats.validating, 0 );
}

#[ tokio::test ]
async fn test_tuning_manager_cancel_job()
{
  let mut manager = TuningManager::new();

  let config = TuningJobConfig
  {
    job_name : "test_job".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  manager.create_job( config ).await.unwrap();
  let result = manager.cancel_job( "test_job" ).await;

  assert!( result.is_ok() );

  let job = manager.get_job( "test_job" ).await.unwrap();
  assert!( matches!( job.status, TuningStatus::Cancelled ) );

  let stats = manager.tuning_stats();
  assert_eq!( stats.cancelled, 1 );
}

#[ tokio::test ]
async fn test_tuning_manager_delete_job()
{
  let mut manager = TuningManager::new();

  let config = TuningJobConfig
  {
    job_name : "test_job".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  manager.create_job( config ).await.unwrap();
  let result = manager.delete_job( "test_job" ).await;

  assert!( result.is_ok() );

  let job = manager.get_job( "test_job" ).await;
  assert!( job.is_none() );

  let stats = manager.tuning_stats();
  assert_eq!( stats.total, 0 );
}

#[ tokio::test ]
async fn test_tuning_event_notifier()
{
  let ( _sender, _receiver ) = ModelTuningUtils::create_event_notifier();
  // Basic test to ensure the notifier can be created without errors
}

#[ tokio::test ]
async fn test_config_validation_success()
{
  let config = TuningJobConfig
  {
    job_name : "valid_job".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  let result = ModelTuningUtils::validate_config( &config );
  assert!( result.is_ok() );
}

#[ tokio::test ]
async fn test_config_validation_failure()
{
  let config = TuningJobConfig
  {
    job_name : "".to_string(), // Invalid empty name
    base_model : "".to_string(), // Invalid empty model
    training_data : TrainingDataConfig
    {
      training_file : "".to_string(), // Invalid empty file
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters
    {
      learning_rate : -0.1, // Invalid negative learning rate
      batch_size : 0, // Invalid zero batch size
      epochs : 0, // Invalid zero epochs
      warmup_steps : 100,
      weight_decay : 0.01,
      gradient_clip_norm : 1.0,
      lr_schedule : "linear".to_string(),
      custom_params : HashMap::new(),
    },
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  let result = ModelTuningUtils::validate_config( &config );
  assert!( result.is_err() );

  let errors = result.unwrap_err();
  assert!( errors.len() >= 5 ); // Should have multiple validation errors
}

#[ tokio::test ]
async fn test_training_time_estimation()
{
  let config = TuningJobConfig
  {
    job_name : "estimation_test".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters
    {
      epochs : 5,
      ..HyperParameters::default()
    },
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements::default(),
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  let dataset_size = 100_000;
  let estimated_time = ModelTuningUtils::estimate_training_time( &config, dataset_size );

  assert!( estimated_time.as_secs() > 0 );
  assert!( estimated_time.as_secs() >= 500 ); // Should be at least 500 seconds for this dataset
}

#[ tokio::test ]
async fn test_training_cost_estimation()
{
  let config = TuningJobConfig
  {
    job_name : "cost_test".to_string(),
    base_model : "gpt-5-nano".to_string(),
    training_data : TrainingDataConfig
    {
      training_file : "train.jsonl".to_string(),
      validation_file : None,
      data_format : "jsonl".to_string(),
      max_sequence_length : 1024,
      preprocessing : HashMap::new(),
    },
    hyperparameters : HyperParameters::default(),
    method : FineTuningMethod::Full,
    objective : TrainingObjective::LanguageModeling,
    resource_requirements : TuningResourceRequirements
    {
      gpu_count : 2,
      gpu_type : Some( "A100".to_string() ),
      ..TuningResourceRequirements::default()
    },
    checkpointing : CheckpointConfig::default(),
    env_vars : HashMap::new(),
  };

  let duration = Duration::from_secs( 3600 ); // 1 hour
  let estimated_cost = ModelTuningUtils::estimate_training_cost( &config, duration );

  assert!( estimated_cost > 0.0 );
  assert!( estimated_cost == 8.0 ); // 2 A100 GPUs * 1 hour * $4/hour = $8
}

#[ tokio::test ]
async fn test_hyperparameter_suggestions()
{
  let full_method = FineTuningMethod::Full;
  let dataset_size = 50000;
  let suggested = ModelTuningUtils::suggest_hyperparameters( &full_method, dataset_size );

  assert_eq!( suggested.learning_rate, 5e-5 );
  assert_eq!( suggested.batch_size, 32 );

  let lora_method = FineTuningMethod::LoRA { rank : 16, alpha : 32.0, dropout : 0.1 };
  let lora_suggested = ModelTuningUtils::suggest_hyperparameters( &lora_method, dataset_size );

  assert_eq!( lora_suggested.learning_rate, 1e-4 );
  assert_eq!( lora_suggested.batch_size, 64 );
  assert_eq!( lora_suggested.epochs, 5 );
}

#[ tokio::test ]
async fn test_serialization_roundtrip()
{
  let status = TuningStatus::Running;
  let serialized = serde_json::to_string( &status ).unwrap();
  let deserialized : TuningStatus = serde_json::from_str( &serialized ).unwrap();
  assert!( matches!( deserialized, TuningStatus::Running ) );

  let method = FineTuningMethod::LoRA { rank : 16, alpha : 32.0, dropout : 0.1 };
  let serialized = serde_json::to_string( &method ).unwrap();
  let deserialized : FineTuningMethod = serde_json::from_str( &serialized ).unwrap();
  assert!( matches!( deserialized, FineTuningMethod::LoRA { .. } ) );
}

#[ tokio::test ]
async fn test_tuning_stats()
{
  let mut manager = TuningManager::new();

  // Create jobs with different statuses
  for i in 0..3
  {
    let config = TuningJobConfig
    {
      job_name : format!( "job_{}", i ),
      base_model : "gpt-5-nano".to_string(),
      training_data : TrainingDataConfig
      {
        training_file : "train.jsonl".to_string(),
        validation_file : None,
        data_format : "jsonl".to_string(),
        max_sequence_length : 1024,
        preprocessing : HashMap::new(),
      },
      hyperparameters : HyperParameters::default(),
      method : FineTuningMethod::Full,
      objective : TrainingObjective::LanguageModeling,
      resource_requirements : TuningResourceRequirements::default(),
      checkpointing : CheckpointConfig::default(),
      env_vars : HashMap::new(),
    };

    manager.create_job( config ).await.unwrap();
  }

  // Update statuses
  manager.update_job_status( "job_1", TuningStatus::Running ).await.unwrap();
  manager.update_job_status( "job_2", TuningStatus::Succeeded ).await.unwrap();

  let stats = manager.tuning_stats();
  assert_eq!( stats.total, 3 );
  assert_eq!( stats.validating, 1 );
  assert_eq!( stats.running, 1 );
  assert_eq!( stats.succeeded, 1 );
}

#[ tokio::test ]
async fn test_module_exports_availability()
{
  // This test ensures all the main types are exported and accessible
  let _ = TuningStatus::Validating;
  let _ = TrainingObjective::LanguageModeling;
  let _ = FineTuningMethod::Full;
  let _ = HyperParameters::default();
  let _ = TuningResourceRequirements::default();
  let _ = CheckpointConfig::default();
  let _ = TuningManager::new();
  let ( _, _ ) = ModelTuningUtils::create_event_notifier();
}