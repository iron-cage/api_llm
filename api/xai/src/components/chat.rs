mod private
{
  use crate::components::common::{ Role, Usage };
  use serde::{ Serialize, Deserialize };
  use former::Former;

  /// A single message in a chat conversation.
  ///
  /// Messages have a role (system, user, assistant, tool) and content.
  /// XAI API allows flexible message ordering unlike `OpenAI`'s strict alternation.
  ///
  /// # Examples
  ///
  /// ```
  /// use api_xai::Message;
  ///
  /// let system = Message::system( "You are a helpful assistant" );
  /// let user = Message::user( "Hello!" );
  /// let assistant = Message::assistant( "Hi there!" );
  /// ```
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct Message
  {
    /// Role of the message sender.
    pub role : Role,

    /// Text content of the message.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub content : Option< String >,

    /// Tool calls made by the assistant (function calling).
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub tool_calls : Option< Vec< ToolCall > >,

    /// ID of the tool call this message responds to (for tool role).
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub tool_call_id : Option< String >,
  }

  impl Message
  {
    /// Creates a system message.
    ///
    /// System messages provide instructions or context to the model.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_xai::Message;
    ///
    /// let msg = Message::system( "You are a helpful coding assistant" );
    /// ```
    pub fn system( content : impl Into< String > ) -> Self
    {
      Self
      {
        role : Role::System,
        content : Some( content.into() ),
        tool_calls : None,
        tool_call_id : None,
      }
    }

    /// Creates a user message.
    ///
    /// User messages contain queries or prompts from the user.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_xai::Message;
    ///
    /// let msg = Message::user( "What is 2 + 2?" );
    /// ```
    pub fn user( content : impl Into< String > ) -> Self
    {
      Self
      {
        role : Role::User,
        content : Some( content.into() ),
        tool_calls : None,
        tool_call_id : None,
      }
    }

    /// Creates an assistant message.
    ///
    /// Assistant messages contain AI-generated responses.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_xai::Message;
    ///
    /// let msg = Message::assistant( "2 + 2 equals 4" );
    /// ```
    pub fn assistant( content : impl Into< String > ) -> Self
    {
      Self
      {
        role : Role::Assistant,
        content : Some( content.into() ),
        tool_calls : None,
        tool_call_id : None,
      }
    }

    /// Creates a tool result message.
    ///
    /// Tool messages provide function execution results back to the model.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_xai::Message;
    ///
    /// let msg = Message::tool( "call_123", r#"{"temperature": 22}"# );
    /// ```
    pub fn tool( tool_call_id : impl Into< String >, content : impl Into< String > ) -> Self
    {
      Self
      {
        role : Role::Tool,
        content : Some( content.into() ),
        tool_calls : None,
        tool_call_id : Some( tool_call_id.into() ),
      }
    }
  }

  /// Tool call made by the assistant.
  ///
  /// When the model decides to call a function, it returns tool calls
  /// that the client must execute and return results for.
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ToolCall
  {
    /// Unique identifier for this tool call.
    pub id : String,

    /// Type of tool (always "function" for function calling).
    #[ serde( rename = "type" ) ]
    pub tool_type : String,

    /// Function call details.
    pub function : FunctionCall,
  }

  /// Function call details.
  ///
  /// Contains the function name and arguments (as JSON string).
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct FunctionCall
  {
    /// Name of the function to call.
    pub name : String,

    /// Function arguments as JSON string.
    pub arguments : String,
  }

  /// Request to create a chat completion.
  ///
  /// This is the primary request type for the chat completions endpoint.
  /// Uses the `Former` builder pattern for fluent construction.
  ///
  /// # Required Fields
  ///
  /// - `model`: Model ID (e.g., "grok-2-1212")
  /// - `messages`: Conversation messages
  ///
  /// # Optional Fields
  ///
  /// - `temperature`: Randomness (0.0-2.0, default varies by model)
  /// - `max_tokens`: Maximum tokens to generate
  /// - `top_p`: Nucleus sampling threshold
  /// - `frequency_penalty`: Repetition reduction
  /// - `presence_penalty`: Topic diversity
  /// - `stream`: Enable SSE streaming
  /// - `tools`: Available functions for tool calling
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use api_xai::{ ChatCompletionRequest, Message };
  ///
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .temperature( 0.7 )
  ///   .max_tokens( 100u32 )
  ///   .form();
  /// ```
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq, Former ) ]
  pub struct ChatCompletionRequest
  {
    /// Model ID (e.g., "grok-2-1212", "grok-4").
    pub model : String,

    /// Conversation messages.
    pub messages : Vec< Message >,

    /// Sampling temperature (0.0-2.0).
    ///
    /// Higher values (0.8-2.0) make output more random.
    /// Lower values (0.0-0.4) make output more focused and deterministic.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub temperature : Option< f32 >,

    /// Maximum tokens to generate.
    ///
    /// Limits the length of the completion. Does not include prompt tokens.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub max_tokens : Option< u32 >,

    /// Nucleus sampling threshold (0.0-1.0).
    ///
    /// Only tokens with cumulative probability up to `top_p` are considered.
    /// Recommended : 0.1-0.9. Do not use with temperature.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub top_p : Option< f32 >,

    /// Frequency penalty (0.0-2.0).
    ///
    /// Reduces repetition of tokens based on their frequency.
    /// Typical range : 0.1-0.8.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub frequency_penalty : Option< f32 >,

    /// Presence penalty (0.0-2.0).
    ///
    /// Encourages the model to talk about new topics.
    /// Typical range : 0.1-0.8.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub presence_penalty : Option< f32 >,

    /// Enable Server-Sent Events streaming.
    ///
    /// When true, the response is streamed as SSE chunks.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub stream : Option< bool >,

    /// Tools (functions) available for the model to call.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub tools : Option< Vec< Tool > >,
  }

  /// Tool definition for function calling.
  ///
  /// Defines a function that the model can choose to call.
  ///
  /// # Examples
  ///
  /// ```
  /// use api_xai::{ Tool, Function };
  /// use serde_json::json;
  ///
  /// // Using convenience method
  /// let tool = Tool::function(
  ///   "get_weather",
  ///   "Get current weather for a location",
  ///   json!({
  ///     "type": "object",
  ///     "properties": {
  ///       "location": {
  ///         "type": "string",
  ///         "description": "City name"
  ///       }
  ///     },
  ///     "required": ["location"]
  ///   })
  /// );
  /// ```
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq, Former ) ]
  pub struct Tool
  {
    /// Type of tool (always "function").
    #[ serde( rename = "type" ) ]
    pub tool_type : String,

    /// Function specification.
    pub function : Function,
  }

  impl Tool
  {
    /// Creates a new function tool.
    ///
    /// # Arguments
    ///
    /// * `name` - Function name
    /// * `description` - What the function does
    /// * `parameters` - JSON Schema for function parameters
    ///
    /// # Examples
    ///
    /// ```
    /// use api_xai::Tool;
    /// use serde_json::json;
    ///
    /// let tool = Tool::function(
    ///   "calculate",
    ///   "Perform arithmetic calculation",
    ///   json!({
    ///     "type": "object",
    ///     "properties": {
    ///       "expression": { "type": "string" }
    ///     }
    ///   })
    /// );
    /// ```
    pub fn function(
      name : impl Into< String >,
      description : impl Into< String >,
      parameters : serde_json::Value
    ) -> Self
    {
      Self
      {
        tool_type : "function".to_string(),
        function : Function
        {
          name : name.into(),
          description : description.into(),
          parameters,
        },
      }
    }
  }

  /// Function specification.
  ///
  /// Describes a function's name, description, and parameters (JSON Schema).
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq, Former ) ]
  pub struct Function
  {
    /// Function name.
    pub name : String,

    /// Human-readable description of what the function does.
    pub description : String,

    /// JSON Schema describing the function parameters.
    pub parameters : serde_json::Value,
  }

  /// Response from chat completion request.
  ///
  /// Contains the model's response including generated text, token usage,
  /// and metadata.
  ///
  /// # Fields
  ///
  /// - `id`: Unique completion ID
  /// - `object`: Type (always "chat.completion")
  /// - `created`: Unix timestamp
  /// - `model`: Model used
  /// - `choices`: Generated completions
  /// - `usage`: Token usage statistics
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ChatCompletionResponse
  {
    /// Unique completion ID.
    pub id : String,

    /// Object type (always "chat.completion").
    pub object : String,

    /// Unix timestamp of creation.
    pub created : u64,

    /// Model ID used for this completion.
    pub model : String,

    /// Generated completion choices.
    pub choices : Vec< Choice >,

    /// Token usage statistics.
    pub usage : Usage,
  }

  /// A single completion choice.
  ///
  /// Contains the generated message and finish reason.
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct Choice
  {
    /// Index of this choice (0-based).
    pub index : u32,

    /// Generated message.
    pub message : Message,

    /// Reason the completion finished.
    ///
    /// Possible values:
    /// - "stop": Natural completion
    /// - "length": `max_tokens` reached
    /// - `tool_calls`: Model wants to call a function
    pub finish_reason : Option< String >,
  }

  /// Streaming chunk from chat completion.
  ///
  /// When streaming is enabled, the response is delivered as a series
  /// of chunks via Server-Sent Events.
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ChatCompletionChunk
  {
    /// Unique completion ID.
    pub id : String,

    /// Object type (always "chat.completion.chunk").
    pub object : String,

    /// Unix timestamp of creation.
    pub created : u64,

    /// Model ID used for this completion.
    pub model : String,

    /// Streaming choices with deltas.
    pub choices : Vec< ChunkChoice >,
  }

  /// A single streaming choice.
  ///
  /// Contains incremental updates (deltas) to the message.
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ChunkChoice
  {
    /// Index of this choice (0-based).
    pub index : u32,

    /// Incremental message update.
    pub delta : Delta,

    /// Reason the completion finished (only in final chunk).
    pub finish_reason : Option< String >,
  }

  /// Incremental message update in streaming.
  ///
  /// Contains partial content that should be appended to the full message.
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq, Default ) ]
  pub struct Delta
  {
    /// Role (only in first chunk).
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub role : Option< Role >,

    /// Partial content to append.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub content : Option< String >,

    /// Tool calls (for function calling in streaming).
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub tool_calls : Option< Vec< ToolCall > >,
  }
}

crate::mod_interface!
{
  exposed use
  {
    Message,
    ToolCall,
    FunctionCall,
    ChatCompletionRequest,
    Tool,
    Function,
    ChatCompletionResponse,
    Choice,
    ChatCompletionChunk,
    ChunkChoice,
    Delta,
  };
}
