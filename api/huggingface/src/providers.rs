//! `HuggingFace` Inference Providers API for Pro plan model access
//!
//! This module implements the chat completions endpoint that provides access to Pro plan
//! models like Llama-3, Mistral, and `CodeLlama` through various inference providers.

mod private
{
  use crate::
  {
  client::Client,
  error::Result,
  validation::{ validate_input_text, validate_model_identifier },
  };

  #[ cfg( feature = "env-config" ) ]
  use crate::environment::{ HuggingFaceEnvironment, EnvironmentInterface };

  use serde::{ Serialize, Deserialize };

  /// Chat completion request for the Inference Providers API
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ChatCompletionRequest
  {
  /// Model identifier (e.g., "meta-llama/Llama-2-7b-chat-hf")
  pub model : String,
  
  /// List of messages in the conversation
  pub messages : Vec< ChatMessage >,
  
  /// Maximum number of tokens to generate
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub max_tokens : Option< u32 >,
  
  /// Sampling temperature (0.0 to 2.0)
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub temperature : Option< f32 >,
  
  /// Top-p sampling parameter
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub top_p : Option< f32 >,
  
  /// Whether to stream the response
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub stream : Option< bool >,
  }

  pub use crate::components::inference_shared::ChatMessage;

  /// Response from the chat completions endpoint
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ChatCompletionResponse
  {
  /// Unique identifier for the completion
  pub id : String,
  
  /// Object type (should be "chat.completion")
  pub object : String,
  
  /// Unix timestamp of when the completion was created
  pub created : u64,
  
  /// Model that was used for the completion
  pub model : String,
  
  /// List of completion choices
  pub choices : Vec< ChatChoice >,
  
  /// Usage statistics for the request
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub usage : Option< Usage >,
  }

  /// A single choice in the chat completion response
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ChatChoice
  {
  /// Index of this choice in the list
  pub index : u32,
  
  /// The generated message
  pub message : ChatMessage,
  
  /// Reason why the generation finished
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub finish_reason : Option< String >,
  }

  /// Token usage statistics
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct Usage
  {
  /// Number of tokens in the prompt
  pub prompt_tokens : u32,
  
  /// Number of tokens in the completion
  pub completion_tokens : u32,
  
  /// Total number of tokens used
  pub total_tokens : u32,
  }

  /// Supported inference providers
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
  #[ serde( rename_all = "lowercase" ) ]
  pub enum InferenceProvider
  {
  /// OpenAI-compatible provider
  OpenAI,
  
  /// Cohere provider
  Cohere,
  
  /// Together AI provider  
  Together,
  
  /// Groq provider
  Groq,
  
  /// `HuggingFace` inference provider
  HfInference,
  }

  impl InferenceProvider
  {
  /// Get the provider name as used in requests
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
      match self
      {
  Self::OpenAI => "openai",
  Self::Cohere => "cohere", 
  Self::Together => "together",
  Self::Groq => "groq",
  Self::HfInference => "hf-inference",
      }
  }

  /// Get available models for this provider
  #[ inline ]
  #[ must_use ]
  pub fn available_models( self ) -> &'static [ &'static str ]
  {
      match self
      {
  Self::OpenAI => &[
          "meta-llama/Llama-2-7b-chat-hf",
          "meta-llama/Llama-2-13b-chat-hf",  
          "meta-llama/Meta-Llama-3-8B-Instruct",
          "meta-llama/Meta-Llama-3-70B-Instruct",
          "mistralai/Mistral-7B-Instruct-v0.2",
          "mistralai/Mixtral-8x7B-Instruct-v0.1",
          "codellama/CodeLlama-7b-Instruct-hf",
          "codellama/CodeLlama-13b-Instruct-hf",
  ],
  Self::Cohere => &[
          "meta-llama/Llama-2-7b-chat-hf",
          "meta-llama/Meta-Llama-3-8B-Instruct", 
          "mistralai/Mistral-7B-Instruct-v0.2",
  ],
  Self::Together => &[
          "meta-llama/Llama-2-7b-chat-hf",
          "meta-llama/Llama-2-13b-chat-hf",
          "meta-llama/Meta-Llama-3-8B-Instruct",
          "meta-llama/Meta-Llama-3-70B-Instruct",
          "mistralai/Mistral-7B-Instruct-v0.2", 
          "mistralai/Mixtral-8x7B-Instruct-v0.1",
          "codellama/CodeLlama-7b-Instruct-hf",
  ],
  Self::Groq | Self::HfInference => &[
          "meta-llama/Llama-2-7b-chat-hf",
          "meta-llama/Meta-Llama-3-8B-Instruct",
          "mistralai/Mistral-7B-Instruct-v0.2",
          "codellama/CodeLlama-7b-Instruct-hf",
  ],
      }
  }

  /// Find the best provider for a given model
  #[ inline ]
  #[ must_use ]
  pub fn for_model( model : &str ) -> Option< Self >
  {
      [ Self::OpenAI, Self::Together, Self::Cohere, Self::Groq, Self::HfInference ]
  .into_iter()
  .find( | provider | provider.available_models().contains( &model ) )
  }
  }

  /// API group for `HuggingFace` Inference Providers operations
  #[ derive( Debug ) ]
  pub struct Providers< E >
  where
  E : Clone,
  {
  client : Client< E >,
  }

  #[ cfg( feature = "env-config" ) ]
  impl< E > Providers< E >
  where
  E : HuggingFaceEnvironment + EnvironmentInterface + Send + Sync + 'static + Clone,
  {
  /// Create a new Providers API group
  #[ inline ]
  #[ must_use ]
  pub fn new( client : &Client< E > ) -> Self
  {
      Self
      {
  client : (*client).clone(),
      }
  }

  /// Create a chat completion using the Inference Providers API
  ///
  /// # Arguments
  /// - `model`: Model identifier (e.g., "meta-llama/Llama-2-7b-chat-hf")
  /// - `messages`: Conversation messages
  /// - `max_tokens`: Maximum tokens to generate (optional)
  /// - `temperature`: Sampling temperature (optional)
  /// - `top_p`: Top-p sampling parameter (optional)
  ///
  /// # Errors
  /// Returns error if the request fails or response is invalid
  #[ inline ]
  pub async fn chat_completion(
      &self,
      model : impl AsRef< str >,
      messages : Vec< ChatMessage >,
      max_tokens : Option< u32 >,
      temperature : Option< f32 >,
      top_p : Option< f32 >,
  ) -> Result< ChatCompletionResponse >
  {
      let model_id = model.as_ref();
      
      // Validate model identifier
      validate_model_identifier( model_id )?;
      
      // Validate messages
      for message in &messages
      {
  validate_input_text( &message.content )?;
      }

      let request = ChatCompletionRequest
      {
  model : model_id.to_string(),
  messages,
  max_tokens,
  temperature,
  top_p,
  stream : Some( false ), // Non-streaming for now
      };

      let url = self.client.environment.endpoint_url( "/v1/chat/completions" )?;
      
      self.client.post( url.as_str(), &request ).await
  }

  /// Create a simple chat completion with just a user message
  ///
  /// # Arguments  
  /// - `model`: Model identifier
  /// - `user_message`: The user's message
  ///
  /// # Errors
  /// Returns error if the request fails or response is invalid
  #[ inline ]
  pub async fn simple_chat(
      &self,
      model : impl AsRef< str >,
      user_message : impl Into< String >,
  ) -> Result< ChatCompletionResponse >
  {
      let messages = vec!
      [
  ChatMessage
  {
          role : "user".to_string(),
          content : user_message.into(),
          tool_calls : None,
          tool_call_id : None,
  }
      ];

      self.chat_completion( model, messages, None, None, None ).await
  }

  /// Create a math completion with system prompt for better results
  ///
  /// # Arguments
  /// - `model`: Model identifier  
  /// - `math_query`: The mathematical question
  ///
  /// # Errors
  /// Returns error if the request fails or response is invalid
  #[ inline ]
  pub async fn math_completion(
      &self,
      model : impl AsRef< str >,
      math_query : impl Into< String >,
  ) -> Result< ChatCompletionResponse >
  {
      let messages = vec!
      [
  ChatMessage
  {
          role : "system".to_string(),
          content : "You are a helpful math assistant. Solve mathematical problems step by step and provide clear, accurate answers.".to_string(),
          tool_calls : None,
          tool_call_id : None,
  },
  ChatMessage
  {
          role : "user".to_string(),
          content : math_query.into(),
          tool_calls : None,
          tool_call_id : None,
  }
      ];

      self.chat_completion( model, messages, Some( 150 ), Some( 0.1 ), Some( 0.95 ) ).await
  }

  /// Get the recommended model for Pro users
  #[ inline ]
  #[ must_use ]
  pub fn default_pro_model() -> &'static str
  {
      "meta-llama/Meta-Llama-3-8B-Instruct"
  }

  /// Get fallback model for free tier users
  #[ inline ]
  #[ must_use ]
  pub fn fallback_model() -> &'static str
  {
      "facebook/bart-large-cnn"
  }

  /// Get the best provider for a model
  #[ inline ]
  #[ must_use ]
  pub fn get_provider_for_model( model : &str ) -> Option< InferenceProvider >
  {
      InferenceProvider::for_model( model )
  }

  /// Create a chat completion with function calling support
  ///
  /// # Arguments
  /// - `model`: Model identifier (e.g., "moonshotai/Kimi-K2-Instruct-0905:groq")
  /// - `messages`: Conversation messages
  /// - `tools`: List of tools the model may call
  /// - `tool_choice`: Controls tool usage ("auto", "none", "required", or specific function)
  /// - `max_tokens`: Maximum tokens to generate (optional)
  /// - `temperature`: Sampling temperature (optional)
  /// - `top_p`: Top-p sampling parameter (optional)
  ///
  /// # Errors
  /// Returns error if the request fails or response is invalid
  #[ inline ]
  #[ allow( clippy::too_many_arguments ) ]
  pub async fn chat_completion_with_tools(
      &self,
      model : impl AsRef< str >,
      messages : Vec< crate::components::inference_shared::ChatMessage >,
      tools : Vec< crate::components::tools::Tool >,
      tool_choice : Option< String >,
      max_tokens : Option< u32 >,
      temperature : Option< f32 >,
      top_p : Option< f32 >,
  ) -> Result< crate::components::inference_shared::ChatCompletionResponse >
  {
      let model_id = model.as_ref();

      // Validate model identifier
      validate_model_identifier( model_id )?;

      // Validate messages
      // Note: Assistant messages with tool_calls can have empty content
      for message in &messages
      {
  let has_tool_calls = message.tool_calls.as_ref().is_some_and( | tc | !tc.is_empty() );
  if !( message.role == "assistant" && has_tool_calls )
  {
      validate_input_text( &message.content )?;
  }
      }

      // Convert tools to ToolDefinition format
      let tool_definitions : Vec< crate::components::inference_shared::ToolDefinition > = tools
  .into_iter()
  .map( | tool |
  {
          crate::components::inference_shared::ToolDefinition
          {
      tool_type : "function".to_string(),
      function : tool,
          }
  } )
  .collect();

      let request = crate::components::inference_shared::ChatCompletionRequest
      {
  model : model_id.to_string(),
  messages,
  max_tokens,
  temperature,
  top_p,
  stream : Some( false ), // Non-streaming for now
  tools : Some( tool_definitions ),
  tool_choice,
      };

      let url = self.client.environment.endpoint_url( "/v1/chat/completions" )?;

      self.client.post( url.as_str(), &request ).await
  }
  }

  // Basic implementation for when env-config is not available
  #[ cfg( not( feature = "env-config" ) ) ]
  impl< E > Providers< E >
  where
  E : Clone,
  {
  /// Create a new Providers API group
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
}

crate::mod_interface!
{
  exposed use private::
  {
  Providers,
  ChatCompletionRequest,
  ChatCompletionResponse, 
  ChatMessage,
  ChatChoice,
  Usage,
  InferenceProvider,
  };
}