//! Configuration validation functions

use super::*;

/// Validate enhanced function calling configuration.
///
/// # Arguments
///
/// * `config` - The function calling configuration to validate
///
/// # Returns
///
/// Returns `Ok(())` if the configuration is valid, or a validation error.
pub fn validate_function_calling_config( config : &FunctionCallingConfig ) -> Result< (), ValidationError >
{
  // Validate allowed function names if provided
  if let Some( allowed_names ) = &config.allowed_function_names
  {
    if allowed_names.is_empty()
    {
      return Err( ValidationError::EmptyCollection {
        field : "allowed_function_names".to_string(),
        context : "FunctionCallingConfig".to_string(),
      } );
    }

    if allowed_names.len() > MAX_ALLOWED_FUNCTION_NAMES
    {
      return Err( ValidationError::CollectionTooLarge {
        field : "allowed_function_names".to_string(),
        size : allowed_names.len(),
        max : MAX_ALLOWED_FUNCTION_NAMES,
      } );
    }

    // Validate each function name
    for ( i, function_name ) in allowed_names.iter().enumerate()
    {
      if function_name.trim().is_empty()
      {
        return Err( ValidationError::RequiredFieldMissing {
          field : format!( "allowed_function_names[{}]", i ),
          context : "FunctionCallingConfig".to_string(),
        } );
      }

      // Function names should be valid identifiers
      if !function_name.chars().all( |c| c.is_alphanumeric() || c == '_' )
      {
        return Err( ValidationError::InvalidFieldValue {
          field : format!( "allowed_function_names[{}]", i ),
          value : function_name.clone(),
          reason : "Function name should contain only alphanumeric characters and underscores".to_string(),
        } );
      }
    }
  }

  Ok( () )
}

/// Validate tool configuration.
///
/// # Arguments
///
/// * `config` - The tool configuration to validate
///
/// # Returns
///
/// Returns `Ok(())` if the configuration is valid, or a validation error.
pub fn validate_tool_config( config : &ToolConfig ) -> Result< (), ValidationError >
{
  // Validate function calling config if provided
  if let Some( function_calling_config ) = &config.function_calling_config
  {
    validate_function_calling_config( function_calling_config )
      .map_err( |e| ValidationError::InvalidFieldValue {
        field : "function_calling_config".to_string(),
        value : "FunctionCallingConfig".to_string(),
        reason : e.to_string(),
      } )?;
  }

  // Validate code execution config if provided
  if let Some( code_execution_config ) = &config.code_execution
  {
    validate_code_execution_config( code_execution_config )
      .map_err( |e| ValidationError::InvalidFieldValue {
        field : "code_execution".to_string(),
        value : "CodeExecutionConfig".to_string(),
        reason : e.to_string(),
      } )?;
  }

  Ok( () )
}

/// Validate system instruction.
///
/// # Arguments
///
/// * `instruction` - The system instruction to validate
///
/// # Returns
///
/// Returns `Ok(())` if the instruction is valid, or a validation error.
pub fn validate_system_instruction( instruction : &SystemInstruction ) -> Result< (), ValidationError >
{
  if instruction.parts.is_empty()
  {
    return Err( ValidationError::EmptyCollection {
      field : "parts".to_string(),
      context : "SystemInstruction".to_string(),
    } );
  }

  // Validate each part
  for ( i, part ) in instruction.parts.iter().enumerate()
  {
    validate_part( part )
      .map_err( |e| ValidationError::InvalidFieldValue {
        field : format!( "parts[{}]", i ),
        value : "Part".to_string(),
        reason : e.to_string(),
      } )?;
  }

  // System instruction should primarily contain text
  let has_text_part = instruction.parts.iter().any( |part|
    part.text.as_ref().is_some_and( |t| !t.trim().is_empty() )
  );

  if !has_text_part
  {
    return Err( ValidationError::RequiredFieldMissing {
      field : "text_content".to_string(),
      context : "SystemInstruction should contain at least one text part".to_string(),
    } );
  }

  Ok( () )
}

/// Validate code execution configuration.
///
/// # Arguments
///
/// * `config` - The code execution configuration to validate
///
/// # Returns
///
/// Returns `Ok(())` if the configuration is valid, or a validation error.
pub fn validate_code_execution_config( config : &CodeExecutionConfig ) -> Result< (), ValidationError >
{
  // Validate timeout if provided
  if let Some( timeout ) = config.timeout
  {
    if timeout <= 0
    {
      return Err( ValidationError::ValueOutOfRange {
        field : "timeout".to_string(),
        value : timeout as f64,
        min : Some( 1.0 ),
        max : Some( MAX_CODE_EXECUTION_TIMEOUT as f64 ),
      } );
    }

    if timeout > MAX_CODE_EXECUTION_TIMEOUT
    {
      return Err( ValidationError::ValueOutOfRange {
        field : "timeout".to_string(),
        value : timeout as f64,
        min : Some( 1.0 ),
        max : Some( MAX_CODE_EXECUTION_TIMEOUT as f64 ),
      } );
    }
  }

  Ok( () )
}

/// Validate code execution tool.
///
/// # Arguments
///
/// * `tool` - The code execution tool to validate
///
/// # Returns
///
/// Returns `Ok(())` if the tool is valid, or a validation error.
pub fn validate_code_execution_tool( tool : &CodeExecutionTool ) -> Result< (), ValidationError >
{
  // Validate config if provided
  if let Some( config ) = &tool.config
  {
    validate_code_execution_config( config )
      .map_err( |e| ValidationError::InvalidFieldValue {
        field : "config".to_string(),
        value : "CodeExecutionConfig".to_string(),
        reason : e.to_string(),
      } )?;
  }

  Ok( () )
}
