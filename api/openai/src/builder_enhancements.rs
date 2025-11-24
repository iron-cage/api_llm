//! Builder Pattern Enhancements
//!
//! This module provides enhanced functionality for the Former-based builder patterns
//! used throughout the `OpenAI` API client. It includes:
//!
//! - Validation traits and implementations
//! - Convenience constructor methods
//! - Enhanced error handling for common builder mistakes
//! - Performance optimizations for builder usage

mod private
{
  use crate::
  {
    components ::
    {
      responses ::{ CreateResponseRequest, ResponseInput },
      common ::ModelIdsResponses,
      input ::{ InputMessage, InputContentPart, InputText, InputItem },
      tools ::{ FunctionTool, FunctionParameters },
    },
  };

  /// Validation errors for `CreateResponseRequest` builders
  #[ derive( Debug, Clone, PartialEq ) ]
  pub enum ResponseRequestValidationError
  {
    /// Model is empty or invalid
    InvalidModel(String),
    /// Input is empty or invalid
    InvalidInput(String),
    /// Temperature is out of valid range
    InvalidTemperature(f32),
    /// Max tokens is out of valid range
    InvalidMaxTokens(i32),
  }

  impl core::fmt::Display for ResponseRequestValidationError
  {
    #[ inline ]
    fn fmt(&self, f : &mut core::fmt::Formatter< '_ >) -> core::fmt::Result
    {
      match self
      {
        Self::InvalidModel(model) => write!(f, "Invalid model : {model}"),
        Self::InvalidInput(input) => write!(f, "Invalid input : {input}"),
        Self::InvalidTemperature(temp) => write!(f, "Invalid temperature {temp}: must be between 0.0 and 2.0"),
        Self::InvalidMaxTokens(tokens) => write!(f, "Invalid max tokens {tokens}: must be positive"),
      }
    }
  }

  impl core::error::Error for ResponseRequestValidationError
  {}

  /// Enhanced builder methods for `CreateResponseRequest`
  pub trait CreateResponseRequestEnhancements
  {
    /// Create a simple text request
    ///
    /// # Arguments
    /// * `model` - The model identifier (e.g., "gpt-4", "gpt-5-nano")
    /// * `text` - The text input for the model
    ///
    /// # Returns
    /// A configured `CreateResponseRequest`
    ///
    /// # Example
    /// ```
    /// use api_openai::builder_enhancements::CreateResponseRequestEnhancements;
    /// use api_openai::exposed::responses::CreateResponseRequest;
    ///
    /// let request = CreateResponseRequest::with_simple_text("gpt-4", "Tell me a story");
    /// ```
    fn with_simple_text(model : &str, text : &str) -> CreateResponseRequest;

    /// Create a request with message-based input
    ///
    /// # Arguments
    /// * `model` - The model identifier
    /// * `messages` - Vector of input messages
    ///
    /// # Returns
    /// A configured `CreateResponseRequest` with message input
    fn with_messages(model : &str, messages : Vec< InputMessage >) -> CreateResponseRequest;

    /// Validate the request configuration
    ///
    /// # Errors
    ///
    /// Returns `ResponseRequestValidationError` if the request configuration is invalid.
    ///
    /// # Returns
    /// Ok(()) if valid, Err(ValidationError) if invalid
    fn validate_request(&self) -> core::result::Result< (), ResponseRequestValidationError >;
  }

  impl CreateResponseRequestEnhancements for CreateResponseRequest
  {
    #[ inline ]
    fn with_simple_text(model : &str, text : &str) -> CreateResponseRequest
    {
      CreateResponseRequest::former()
        .model(ModelIdsResponses::from(model.to_string()))
        .input(ResponseInput::String(text.to_string()))
        .form()
    }

    #[ inline ]
    fn with_messages(model : &str, messages : Vec< InputMessage >) -> CreateResponseRequest
    {
      let input_items : Vec< InputItem > = messages.into_iter()
        .map(InputItem::Message)
        .collect();

      CreateResponseRequest::former()
        .model(ModelIdsResponses::from(model.to_string()))
        .input(ResponseInput::Items(input_items))
        .form()
    }

    #[ inline ]
    fn validate_request(&self) -> core::result::Result< (), ResponseRequestValidationError >
    {
      // Validate model
      if self.model.value.is_empty()
      {
        return Err(ResponseRequestValidationError::InvalidModel("Model cannot be empty".to_string()));
      }

      // Validate input
      match &self.input
      {
        ResponseInput::String(s) if s.is_empty() =>
        {
          return Err(ResponseRequestValidationError::InvalidInput("String input cannot be empty".to_string()));
        },
        ResponseInput::Items(items) if items.is_empty() =>
        {
          return Err(ResponseRequestValidationError::InvalidInput("Items input cannot be empty".to_string()));
        },
        _ => {} // Valid input
      }

      // Validate temperature range
      if let Some(temp) = self.temperature
      {
        if !(0.0..=2.0).contains(&temp)
        {
          return Err(ResponseRequestValidationError::InvalidTemperature(temp));
        }
      }

      // Validate max tokens
      if let Some(max_tokens) = self.max_output_tokens
      {
        if max_tokens <= 0
        {
          return Err(ResponseRequestValidationError::InvalidMaxTokens(max_tokens));
        }
      }

      Ok(())
    }
  }

  /// Enhanced builder methods for `InputMessage`
  pub trait InputMessageEnhancements
  {
    /// Create a simple user message with text content
    ///
    /// # Arguments
    /// * `text` - The text content of the message
    ///
    /// # Returns
    /// An `InputMessage` with role "user" and text content
    fn user_text(text : &str) -> InputMessage;

    /// Create a system message with text content
    ///
    /// # Arguments
    /// * `text` - The system message content
    ///
    /// # Returns
    /// An `InputMessage` with role "system" and text content
    fn system_text(text : &str) -> InputMessage;

    /// Create an assistant message with text content
    ///
    /// # Arguments
    /// * `text` - The assistant message content
    ///
    /// # Returns
    /// An `InputMessage` with role "assistant" and text content
    fn assistant_text(text : &str) -> InputMessage;
  }

  impl InputMessageEnhancements for InputMessage
  {
    #[ inline ]
    fn user_text(text : &str) -> InputMessage
    {
      InputMessage::former()
        .role("user".to_string())
        .content(vec![InputContentPart::Text(InputText { text : text.to_string() })])
        .form()
    }

    #[ inline ]
    fn system_text(text : &str) -> InputMessage
    {
      InputMessage::former()
        .role("system".to_string())
        .content(vec![InputContentPart::Text(InputText { text : text.to_string() })])
        .form()
    }

    #[ inline ]
    fn assistant_text(text : &str) -> InputMessage
    {
      InputMessage::former()
        .role("assistant".to_string())
        .content(vec![InputContentPart::Text(InputText { text : text.to_string() })])
        .form()
    }
  }

  /// Enhanced builder methods for `FunctionTool`
  pub trait FunctionToolEnhancements
  {
    /// Create a simple function tool with basic parameters
    ///
    /// # Arguments
    /// * `name` - The function name
    /// * `description` - Description of what the function does
    ///
    /// # Returns
    /// A `FunctionTool` with basic configuration
    fn simple_function(name : &str, description : &str) -> FunctionTool;

    /// Create a function tool with object parameters
    ///
    /// # Arguments
    /// * `name` - The function name
    /// * `description` - Description of what the function does
    /// * `properties` - JSON object describing the parameters
    ///
    /// # Returns
    /// A `FunctionTool` with structured parameters
    fn with_object_params(name : &str, description : &str, properties : serde_json::Value) -> FunctionTool;
  }

  impl FunctionToolEnhancements for FunctionTool
  {
    #[ inline ]
    fn simple_function(name : &str, description : &str) -> FunctionTool
    {
      let params = serde_json::json!({
        "type": "object",
        "properties": {},
        "required": []
      });

      FunctionTool::former()
        .name(name.to_string())
        .description(description.to_string())
        .parameters(FunctionParameters::new(params))
        .form()
    }

    #[ inline ]
    fn with_object_params(name : &str, description : &str, properties : serde_json::Value) -> FunctionTool
    {
      let params = serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": []
      });

      FunctionTool::former()
        .name(name.to_string())
        .description(description.to_string())
        .parameters(FunctionParameters::new(params))
        .form()
    }
  }

  #[ cfg( test ) ]
  mod tests
  {
    use super::*;

    #[ test ]
    fn test_simple_text_convenience_method()
    {
      let request = CreateResponseRequest::with_simple_text("gpt-4", "Tell me a story");

      assert_eq!(request.model.value, "gpt-4");
      assert_eq!(request.input, ResponseInput::String("Tell me a story".to_string()));
    }

    #[ test ]
    fn test_user_message_convenience_method()
    {
      let message = InputMessage::user_text("Hello world");

      assert_eq!(message.role, "user");
      assert_eq!(message.content.len(), 1);
      if let InputContentPart::Text(text_part) = &message.content[0]
      {
        assert_eq!(text_part.text, "Hello world");
      }
      else
      {
        panic!("Expected text content");
      }
    }

    #[ test ]
    fn test_request_validation_success()
    {
      let request = CreateResponseRequest::with_simple_text("gpt-4", "Valid input");

      let validation_result = request.validate_request();
      assert!(validation_result.is_ok());
    }

    #[ test ]
    fn test_request_validation_invalid_temperature()
    {
      let mut request = CreateResponseRequest::with_simple_text("gpt-4", "Valid input");
      request.temperature = Some(3.0); // Invalid - too high

      let validation_result = request.validate_request();
      assert!(validation_result.is_err());

      if let Err(ResponseRequestValidationError::InvalidTemperature(temp)) = validation_result
      {
        assert!((temp - 3.0).abs() < f32::EPSILON);
      }
      else
      {
        panic!("Expected InvalidTemperature error");
      }
    }

    #[ test ]
    fn test_simple_function_tool()
    {
      let tool = FunctionTool::simple_function("test_func", "A test function");

      assert_eq!(tool.name, "test_func");
      assert_eq!(tool.description, Some("A test function".to_string()));
      assert_eq!(tool.parameters.0["type"], "object");
    }

    #[ test ]
    fn test_messages_convenience_method()
    {
      let messages = vec![
        InputMessage::user_text("Hello"),
        InputMessage::assistant_text("Hi there"),
        InputMessage::system_text("You are a helpful assistant"),
      ];

      let request = CreateResponseRequest::with_messages("gpt-4", messages);

      assert_eq!(request.model.value, "gpt-4");
      if let ResponseInput::Items(items) = request.input
      {
        assert_eq!(items.len(), 3);
        let InputItem::Message(msg) = &items[0];
        assert_eq!(msg.role, "user");
      }
      else
      {
        panic!("Expected Items input");
      }
    }
  }
}

crate ::mod_interface!
{
  orphan use ResponseRequestValidationError;
  orphan use CreateResponseRequestEnhancements;
  orphan use InputMessageEnhancements;
  orphan use FunctionToolEnhancements;
}