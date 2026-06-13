//! Model tuning validation functions

use super::*;

/// Validate create tuned model request.
///
/// # Arguments
///
/// * `request` - The create tuned model request to validate
///
/// # Returns
///
/// Returns `Ok(())` if the request is valid, or a validation error.
pub fn validate_create_tuned_model_request( request : &CreateTunedModelRequest ) -> Result< (), ValidationError >
{
  // Validate the tuned model
  validate_tuned_model( &request.tuned_model )
    .map_err( |e| ValidationError::InvalidFieldValue {
      field : "tuned_model".to_string(),
      value : "TunedModel".to_string(),
      reason : e.to_string(),
    } )?;

  // Validate tuned model ID if provided
  if let Some( tuned_model_id ) = &request.tuned_model_id
  {
    if tuned_model_id.trim().is_empty()
    {
      return Err( ValidationError::RequiredFieldMissing {
        field : "tuned_model_id".to_string(),
        context : "CreateTunedModelRequest".to_string(),
      } );
    }

    // Validate ID format (should be alphanumeric with hyphens/underscores)
    if !tuned_model_id.chars().all( |c| c.is_alphanumeric() || c == '-' || c == '_' )
    {
      return Err( ValidationError::InvalidFieldValue {
        field : "tuned_model_id".to_string(),
        value : tuned_model_id.clone(),
        reason : "Tuned model ID should contain only alphanumeric characters, hyphens, and underscores".to_string(),
      } );
    }
  }

  Ok( () )
}

/// Validate tuned model.
///
/// # Arguments
///
/// * `model` - The tuned model to validate
///
/// # Returns
///
/// Returns `Ok(())` if the model is valid, or a validation error.
pub fn validate_tuned_model( model : &TunedModel ) -> Result< (), ValidationError >
{
  // Validate base model
  if model.base_model.trim().is_empty()
  {
    return Err( ValidationError::RequiredFieldMissing {
      field : "base_model".to_string(),
      context : "TunedModel".to_string(),
    } );
  }

  validate_model_name( &model.base_model )
    .map_err( |e| ValidationError::InvalidFieldValue {
      field : "base_model".to_string(),
      value : model.base_model.clone(),
      reason : e.to_string(),
    } )?;

  // Validate display name if provided
  if let Some( display_name ) = &model.display_name
  {
    if display_name.trim().is_empty()
    {
      return Err( ValidationError::RequiredFieldMissing {
        field : "display_name".to_string(),
        context : "TunedModel".to_string(),
      } );
    }
  }

  // Validate tuning task if provided
  if let Some( tuning_task ) = &model.tuning_task
  {
    validate_tuning_task( tuning_task )
      .map_err( |e| ValidationError::InvalidFieldValue {
        field : "tuning_task".to_string(),
        value : "TuningTask".to_string(),
        reason : e.to_string(),
      } )?;
  }

  // Validate temperature if provided
  if let Some( temperature ) = model.temperature
  {
    if !( 0.0..=2.0 ).contains( &temperature )
    {
      return Err( ValidationError::ValueOutOfRange {
        field : "temperature".to_string(),
        value : temperature,
        min : Some( 0.0 ),
        max : Some( 2.0 ),
      } );
    }
  }

  // Validate top_p if provided
  if let Some( top_p ) = model.top_p
  {
    if !( 0.0..=1.0 ).contains( &top_p )
    {
      return Err( ValidationError::ValueOutOfRange {
        field : "top_p".to_string(),
        value : top_p,
        min : Some( 0.0 ),
        max : Some( 1.0 ),
      } );
    }
  }

  // Validate top_k if provided
  if let Some( top_k ) = model.top_k
  {
    if top_k <= 0
    {
      return Err( ValidationError::ValueOutOfRange {
        field : "top_k".to_string(),
        value : top_k as f64,
        min : Some( 1.0 ),
        max : None,
      } );
    }
  }

  Ok( () )
}

/// Validate tuning task.
///
/// # Arguments
///
/// * `task` - The tuning task to validate
///
/// # Returns
///
/// Returns `Ok(())` if the task is valid, or a validation error.
pub fn validate_tuning_task( task : &TuningTask ) -> Result< (), ValidationError >
{
  // Validate training data if provided
  if let Some( training_data ) = &task.training_data
  {
    validate_dataset( training_data )
      .map_err( |e| ValidationError::InvalidFieldValue {
        field : "training_data".to_string(),
        value : "Dataset".to_string(),
        reason : e.to_string(),
      } )?;
  }

  // Validate hyperparameters if provided
  if let Some( hyperparameters ) = &task.hyperparameters
  {
    validate_hyperparameters( hyperparameters )
      .map_err( |e| ValidationError::InvalidFieldValue {
        field : "hyperparameters".to_string(),
        value : "Hyperparameters".to_string(),
        reason : e.to_string(),
      } )?;
  }

  Ok( () )
}

/// Validate dataset.
///
/// # Arguments
///
/// * `dataset` - The dataset to validate
///
/// # Returns
///
/// Returns `Ok(())` if the dataset is valid, or a validation error.
pub fn validate_dataset( dataset : &Dataset ) -> Result< (), ValidationError >
{
  // Validate examples if provided
  if let Some( tuning_examples ) = &dataset.examples
  {
    if tuning_examples.examples.is_empty()
    {
      return Err( ValidationError::EmptyCollection {
        field : "examples".to_string(),
        context : "Dataset".to_string(),
      } );
    }

    if tuning_examples.examples.len() > MAX_TUNING_EXAMPLES
    {
      return Err( ValidationError::CollectionTooLarge {
        field : "examples".to_string(),
        size : tuning_examples.examples.len(),
        max : MAX_TUNING_EXAMPLES,
      } );
    }

    // Validate each example
    for ( i, example ) in tuning_examples.examples.iter().enumerate()
    {
      validate_tuning_example( example )
        .map_err( |e| ValidationError::InvalidFieldValue {
          field : format!( "examples[{}]", i ),
          value : "TuningExample".to_string(),
          reason : e.to_string(),
        } )?;
    }
  }

  Ok( () )
}

/// Validate tuning example.
///
/// # Arguments
///
/// * `example` - The tuning example to validate
///
/// # Returns
///
/// Returns `Ok(())` if the example is valid, or a validation error.
pub fn validate_tuning_example( example : &TuningExample ) -> Result< (), ValidationError >
{
  // At least one of input or output should be provided
  let has_input = example.text_input.as_ref().is_some_and( |input| !input.trim().is_empty() );
  let has_output = example.output.as_ref().is_some_and( |output| !output.trim().is_empty() );

  if !has_input && !has_output
  {
    return Err( ValidationError::RequiredFieldMissing {
      field : "text_input_or_output".to_string(),
      context : "TuningExample must have at least text_input or output".to_string(),
    } );
  }

  Ok( () )
}

/// Validate hyperparameters.
///
/// # Arguments
///
/// * `hyperparameters` - The hyperparameters to validate
///
/// # Returns
///
/// Returns `Ok(())` if the hyperparameters are valid, or a validation error.
pub fn validate_hyperparameters( hyperparameters : &Hyperparameters ) -> Result< (), ValidationError >
{
  // Validate learning rate if provided
  if let Some( learning_rate ) = hyperparameters.learning_rate
  {
    if learning_rate <= 0.0 || learning_rate > 1.0
    {
      return Err( ValidationError::ValueOutOfRange {
        field : "learning_rate".to_string(),
        value : learning_rate,
        min : Some( 0.000_001 ),
        max : Some( 1.0 ),
      } );
    }
  }

  // Validate epoch count if provided
  if let Some( epoch_count ) = hyperparameters.epoch_count
  {
    if epoch_count <= 0 || epoch_count > 100
    {
      return Err( ValidationError::ValueOutOfRange {
        field : "epoch_count".to_string(),
        value : epoch_count as f64,
        min : Some( 1.0 ),
        max : Some( 100.0 ),
      } );
    }
  }

  // Validate batch size if provided
  if let Some( batch_size ) = hyperparameters.batch_size
  {
    if batch_size <= 0 || batch_size > 1024
    {
      return Err( ValidationError::ValueOutOfRange {
        field : "batch_size".to_string(),
        value : batch_size as f64,
        min : Some( 1.0 ),
        max : Some( 1024.0 ),
      } );
    }
  }

  // Validate learning rate multiplier if provided
  if let Some( learning_rate_multiplier ) = hyperparameters.learning_rate_multiplier
  {
    if learning_rate_multiplier <= 0.0 || learning_rate_multiplier > 10.0
    {
      return Err( ValidationError::ValueOutOfRange {
        field : "learning_rate_multiplier".to_string(),
        value : learning_rate_multiplier,
        min : Some( 0.1 ),
        max : Some( 10.0 ),
      } );
    }
  }

  Ok( () )
}

/// Validate list tuned models request.
///
/// # Arguments
///
/// * `request` - The list tuned models request to validate
///
/// # Returns
///
/// Returns `Ok(())` if the request is valid, or a validation error.
pub fn validate_list_tuned_models_request( request : &ListTunedModelsRequest ) -> Result< (), ValidationError >
{
  // Validate page size if provided
  if let Some( page_size ) = request.page_size
  {
    if page_size <= 0 || page_size > 1000
    {
      return Err( ValidationError::ValueOutOfRange {
        field : "page_size".to_string(),
        value : page_size as f64,
        min : Some( 1.0 ),
        max : Some( 1000.0 ),
      } );
    }
  }

  Ok( () )
}
