//! Request templates for common use cases.
//!
//! This module provides reusable request configurations to simplify common operations.

use crate::models::
{
  GenerateContentRequest,
  Content,
  Part,
  GenerationConfig,
  SafetySetting,
};

/// Predefined request templates for common use cases.
#[ derive( Debug, Clone ) ]
pub struct RequestTemplate
{
  /// The base request configuration
  request : GenerateContentRequest,
}

impl RequestTemplate
{
  /// Create a simple chat template with minimal configuration.
  #[ must_use ]
  pub fn chat() -> Self
  {
    Self
    {
      request : GenerateContentRequest
      {
        contents : vec![],
        generation_config : Some( GenerationConfig
        {
          temperature : Some( 0.9 ),
          max_output_tokens : Some( 2048 ),
          ..Default::default()
        } ),
        ..Default::default()
      },
    }
  }

  /// Create a code generation template optimized for programming tasks.
  #[ must_use ]
  pub fn code_generation() -> Self
  {
    Self
    {
      request : GenerateContentRequest
      {
        contents : vec![],
        generation_config : Some( GenerationConfig
        {
          temperature : Some( 0.2 ),
          max_output_tokens : Some( 4096 ),
          ..Default::default()
        } ),
        ..Default::default()
      },
    }
  }

  /// Create a creative writing template with higher temperature.
  #[ must_use ]
  pub fn creative_writing() -> Self
  {
    Self
    {
      request : GenerateContentRequest
      {
        contents : vec![],
        generation_config : Some( GenerationConfig
        {
          temperature : Some( 1.2 ),
          max_output_tokens : Some( 8192 ),
          top_p : Some( 0.95 ),
          top_k : Some( 40 ),
          ..Default::default()
        } ),
        ..Default::default()
      },
    }
  }

  /// Create a factual Q&A template with low temperature.
  #[ must_use ]
  pub fn factual_qa() -> Self
  {
    Self
    {
      request : GenerateContentRequest
      {
        contents : vec![],
        generation_config : Some( GenerationConfig
        {
          temperature : Some( 0.1 ),
          max_output_tokens : Some( 1024 ),
          top_p : Some( 0.8 ),
          ..Default::default()
        } ),
        ..Default::default()
      },
    }
  }

  /// Create a summarization template.
  #[ must_use ]
  pub fn summarization() -> Self
  {
    Self
    {
      request : GenerateContentRequest
      {
        contents : vec![],
        generation_config : Some( GenerationConfig
        {
          temperature : Some( 0.3 ),
          max_output_tokens : Some( 2048 ),
          ..Default::default()
        } ),
        ..Default::default()
      },
    }
  }

  /// Set the prompt text for the template.
  #[ must_use ]
  pub fn with_prompt( mut self, prompt : &str ) -> Self
  {
    self.request.contents = vec![ Content
    {
      parts : vec![ Part
      {
        text : Some( prompt.to_string() ),
        ..Default::default()
      } ],
      role : "user".to_string(),
    } ];
    self
  }

  /// Set custom temperature (0.0-2.0).
  #[ must_use ]
  pub fn with_temperature( mut self, temperature : f32 ) -> Self
  {
    if let Some( config ) = &mut self.request.generation_config
    {
      config.temperature = Some( temperature );
    }
    else
    {
      self.request.generation_config = Some( GenerationConfig
      {
        temperature : Some( temperature ),
        ..Default::default()
      } );
    }
    self
  }

  /// Set maximum output tokens.
  #[ must_use ]
  pub fn with_max_tokens( mut self, max_tokens : i32 ) -> Self
  {
    if let Some( config ) = &mut self.request.generation_config
    {
      config.max_output_tokens = Some( max_tokens );
    }
    else
    {
      self.request.generation_config = Some( GenerationConfig
      {
        max_output_tokens : Some( max_tokens ),
        ..Default::default()
      } );
    }
    self
  }

  /// Add safety settings to the template.
  #[ must_use ]
  pub fn with_safety_settings( mut self, settings : Vec< SafetySetting > ) -> Self
  {
    self.request.safety_settings = Some( settings );
    self
  }

  /// Build the final `GenerateContentRequest`.
  #[ must_use ]
  pub fn build( self ) -> GenerateContentRequest
  {
    self.request
  }
}
