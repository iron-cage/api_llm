//! Text generation and inference operations for `HuggingFace` API.

mod private
{
use crate::
{
  client::Client,
  components::
  {
  inference_shared::
  {
      InferenceRequest, InferenceResponse, InferenceOptions,
      ChatCompletionRequest, ChatCompletionResponse, ChatMessage,
  },
  input::InferenceParameters,
  output::InferenceOutput,
  },
  error::{ Result, HuggingFaceError },
  validation::{ validate_input_text, validate_model_identifier },
};

#[ cfg( feature = "env-config" ) ]
use crate::environment::{ HuggingFaceEnvironment, EnvironmentInterface };

/// API group for `HuggingFace` inference operations
#[ derive( Debug ) ]
pub struct Inference< E >
where
  E : Clone,
{
  client : Client< E >,
}

#[ cfg( feature = "env-config" ) ]
impl< E > Inference< E >
where
  E : HuggingFaceEnvironment + EnvironmentInterface + Send + Sync + 'static + Clone,
{
  /// Create a new Inference API group
  #[ inline ]
  #[ must_use ]
  pub fn new( client : &Client< E > ) -> Self
  {
  Self
  {
      client : (*client).clone(),
  }
  }
  
  /// Create a text generation inference request
  ///
  /// **Updated for Router API**: Now uses the new chat completions format
  ///
  /// # Arguments
  /// - `inputs`: Input text or prompt
  /// - `model`: Model identifier (e.g., "moonshotai/Kimi-K2-Instruct-0905:groq")
  ///
  /// # Errors
  /// Returns error if the request fails or response is invalid
  #[ inline ]
  pub async fn create(
  &self,
  inputs : impl Into< String >,
  model : impl AsRef< str >
  ) -> Result< InferenceResponse >
  {
  let input_text = inputs.into();
  let model_id = model.as_ref();

  // Validate input parameters
  validate_input_text( &input_text )?;
  validate_model_identifier( model_id )?;

  // Convert to chat completions format
  let chat_request = ChatCompletionRequest
  {
      messages : vec!
      [
  ChatMessage
  {
          role : "user".to_string(),
          content : input_text,
          tool_calls : None,
          tool_call_id : None,
  }
      ],
      model : model_id.to_string(),
      temperature : None,
      max_tokens : None,
      top_p : Some( 1.0 ),
      stream : Some( false ),
      tools : None,
      tool_choice : None,
  };

  let endpoint = "chat/completions";
  let url = self.client.environment.endpoint_url( endpoint )?;

  // Post request and convert response
  let chat_response : ChatCompletionResponse = self.client.post( url.as_str(), &chat_request ).await?;

  // Convert chat completion response to inference response format
  convert_chat_response_to_inference( &chat_response )
  }

  /// Create a text generation inference request with parameters
  ///
  /// **Updated for Router API**: Now uses the new chat completions format
  ///
  /// # Arguments
  /// - `inputs`: Input text or prompt
  /// - `model`: Model identifier
  /// - `parameters`: Inference parameters (temperature, `max_tokens`, etc.)
  ///
  /// # Errors
  /// Returns error if the request fails or response is invalid
  #[ inline ]
  pub async fn create_with_parameters(
  &self,
  inputs : impl Into< String >,
  model : impl AsRef< str >,
  parameters : InferenceParameters
  ) -> Result< InferenceResponse >
  {
  let input_text = inputs.into();
  let model_id = model.as_ref();

  // Validate input parameters
  validate_input_text( &input_text )?;
  validate_model_identifier( model_id )?;
  parameters.validate()?;

  // Convert to chat completions format
  let chat_request = ChatCompletionRequest
  {
      messages : vec!
      [
  ChatMessage
  {
          role : "user".to_string(),
          content : input_text,
          tool_calls : None,
          tool_call_id : None,
  }
      ],
      model : model_id.to_string(),
      temperature : parameters.temperature,
      max_tokens : parameters.max_new_tokens,
      top_p : parameters.top_p,
      stream : Some( false ),
      tools : None,
      tool_choice : None,
  };

  let endpoint = "chat/completions";
  let url = self.client.environment.endpoint_url( endpoint )?;

  // Post request and convert response
  let chat_response : ChatCompletionResponse = self.client.post( url.as_str(), &chat_request ).await?;

  // Convert chat completion response to inference response format
  convert_chat_response_to_inference( &chat_response )
  }

  /// Create a text generation inference request with full options
  ///
  /// # Arguments
  /// - `inputs`: Input text or prompt
  /// - `model`: Model identifier
  /// - `parameters`: Inference parameters
  /// - `options`: Request options
  ///
  /// # Errors
  /// Returns error if the request fails or response is invalid
  #[ inline ]
  pub async fn create_with_options( 
  &self, 
  inputs : impl Into< String >, 
  model : impl AsRef< str >,
  parameters : Option< InferenceParameters >,
  options : Option< InferenceOptions >
  ) -> Result< InferenceResponse >
  {
  let input_text = inputs.into();
  let model_id = model.as_ref();
  
  // Validate input parameters
  validate_input_text( &input_text )?;
  validate_model_identifier( model_id )?;
  
  let mut request = InferenceRequest::new( input_text );
  
  if let Some( params ) = parameters
  {
      params.validate()?;
      request = request.with_parameters( params );
  }
  
  if let Some( opts ) = options
  {
      request = request.with_options( opts );
  }
  
  let endpoint = format!( "models/{model_id}" );
  let url = self.client.environment.endpoint_url( &endpoint )?;
  
  self.client.post( url.as_str(), &request ).await
  }
  
  /// Create a streaming text generation request
  ///
  /// # Arguments
  /// - `inputs`: Input text or prompt
  /// - `model`: Model identifier
  /// - `parameters`: Inference parameters with streaming enabled
  ///
  /// # Returns
  /// A receiver channel for streaming response chunks
  ///
  /// # Errors
  /// Returns error if the request fails
  #[ cfg( feature = "inference-streaming" ) ]
  #[ inline ]
  pub async fn create_stream( 
  &self, 
  inputs : impl Into< String >, 
  model : impl AsRef< str >,
  parameters : InferenceParameters
  ) -> Result< tokio::sync::mpsc::Receiver< Result< String > > >
  {
  let stream_params = parameters.with_streaming( true );
  let request = InferenceRequest::new( inputs ).with_parameters( stream_params );
  let endpoint = format!( "models/{}", model.as_ref() );
  let url = self.client.environment.endpoint_url( &endpoint )?;
  
  self.client.post_stream( url.as_str(), &request ).await
  }

  /// Create a controlled stream with pause/resume/cancel support
  ///
  /// This returns a tuple of (`ControlledStream`, `ControlHandle`) that allows
  /// runtime control of the streaming operation.
  ///
  /// # Arguments
  /// - `inputs`: Text input for generation
  /// - `model`: Model identifier to use
  /// - `parameters`: Inference parameters with streaming enabled
  ///
  /// # Returns
  /// A tuple of (`ControlledStream` for consuming events, `ControlHandle` for control)
  ///
  /// # Errors
  /// Returns error if the request fails
  ///
  /// # Example
  ///
  /// ```rust,ignore
  /// let ( stream, control ) = inference
  ///   .create_controlled_stream( "Hello", "gpt2", params )
  ///   .await?;
  ///
  /// // Pause streaming
  /// control.pause().await?;
  ///
  /// // Resume streaming
  /// control.resume().await?;
  ///
  /// // Consume stream
  /// while let Some( result ) = stream.next().await
  /// {
  ///   match result
  ///   {
  ///     Ok( text ) => println!( "{}", text ),
  ///     Err( e ) => eprintln!( "Error: {}", e ),
  ///   }
  /// }
  /// ```
  #[ cfg( feature = "streaming-control" ) ]
  #[ inline ]
  pub async fn create_controlled_stream(
  &self,
  inputs : impl Into< String >,
  model : impl AsRef< str >,
  parameters : InferenceParameters,
  ) -> Result< ( crate::streaming_control::ControlledStream, crate::streaming_control::ControlHandle ) >
  {
  let receiver = self.create_stream( inputs, model, parameters ).await?;
  Ok( crate::streaming_control::wrap_stream( receiver ) )
  }
}

// Basic implementation for when env-config is not available
#[ cfg( not( feature = "env-config" ) ) ]
impl< E > Inference< E >
where
  E : Clone,
{
  /// Create a new Inference API group
  #[ inline ]
  #[ must_use ]
  pub fn new( client : &Client< E > ) -> Self
  {
  Self
  {
      client : (*client).clone(),
  }
  }
}

/// Helper function to convert chat completion response to inference response format
///
/// This maintains backward compatibility with existing code while using the new API
fn convert_chat_response_to_inference( chat_response : &ChatCompletionResponse ) -> Result< InferenceResponse >
{
  // Extract the generated text from the first choice
  let generated_text = chat_response.choices
  .first()
  .ok_or_else( || HuggingFaceError::Api( crate::error::ApiErrorWrap::new( "No choices in response".to_string() ) ) )?
  .message
  .content
  .clone();

  // Create inference output
  let output = InferenceOutput
  {
  generated_text,
  input_tokens : chat_response.usage.as_ref().map( | u | u.prompt_tokens ),
  generated_tokens : chat_response.usage.as_ref().map( | u | u.completion_tokens ),
  metadata : None,
  };

  // Create inference response
  Ok( InferenceResponse::Single( output ) )
}

} // end mod private

crate::mod_interface!
{
  exposed use 
  {
  private::Inference,
  };
}