// src/components/common.rs
//! Defines common data structures (components) used across various `OpenAI` API responses and requests.
//! Based on the components/schemas section of the `OpenAPI` specification.

/// Define a private namespace for all its items.
mod private
{
  // Use full paths from crate root for components
  use crate::components::tools as tools;

  // Standard library imports
  use std::collections::HashMap;
  // Serde imports
  use serde::{ Serialize, Deserialize };
  use serde_json::Value;
  use former::Former;
  // derive_tools import
  use derive_tools::From; // Import the derive macro

  /// Represents metadata as key-value pairs, allowing attachment of custom information.
  ///
  /// Set of 16 key-value pairs that can be attached to an object. This can be
  /// useful for storing additional information about the object in a structured
  /// format, and querying for objects via API or the dashboard.
  /// Keys are strings with a maximum length of 64 characters. Values are strings
  /// with a maximum length of 512 characters.
  ///
  /// # Used By
  /// - `/assistants` (POST)
  /// - `/assistants/{assistant_id}` (POST)
  /// - `/batches` (POST, GET)
  /// - `/batches/{batch_id}` (GET)
  /// - `/chat/completions` (GET, POST)
  /// - `/chat/completions/{completion_id}` (POST)
  /// - `/fine_tuning/jobs` (POST, GET)
  /// - `/fine_tuning/jobs/{fine_tuning_job_id}` (GET)
  /// - `/threads` (POST)
  /// - `/threads/{thread_id}` (POST)
  /// - `/threads/{thread_id}/messages` (POST)
  /// - `/threads/{thread_id}/messages/{message_id}` (POST)
  /// - `/threads/{thread_id}/runs` (POST)
  /// - `/threads/{thread_id}/runs/{run_id}` (POST)
  /// - `/threads/{thread_id}/runs/{run_id}/steps/{step_id}` (GET)
  /// - `/responses` (POST, GET)
  /// - `/responses/{response_id}` (GET)
  /// - `/vector_stores` (POST)
  /// - `/vector_stores/{vector_store_id}` (POST)
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq, Default ) ]
  #[ serde( transparent ) ]
  pub struct Metadata
  (
    /// The underlying map storing key-value pairs.
    pub HashMap<  String, String  >
  );

  impl From< HashMap<  String, String  > > for Metadata
  {
    /// Creates Metadata from a `HashMap`.
    #[ inline ]
    fn from( map : HashMap<  String, String  > ) -> Self
    {
      Metadata( map )
    }
  }
  impl< const N : usize > From< [ ( &str, &str ); N ] > for Metadata
  {
    /// Creates Metadata from an array of string slice tuples.
    #[ inline ]
    fn from( arr : [ ( &str, &str ); N ] ) -> Self
    {
      Metadata( arr.into_iter().map( | ( k, v ) | ( k.to_string(), v.to_string() ) ).collect() )
    }
  }

  /// Represents an error object returned in specific contexts, like `last_error`.
  ///
  /// This structure holds a subset of the fields from the main `Error` object,
  /// typically used for errors associated with specific sub-objects (e.g., vector store files).
  ///
  /// # Used By (as `last_error` or similar nested error structure)
  /// - `VectorStoreFileObject` (within `/vector_stores/{vector_store_id}/files/{file_id}` GET)
  /// - `RunObject` (within `/threads/{thread_id}/runs/{run_id}` GET)
  /// - `RunStepObject` (within `/threads/{thread_id}/runs/{run_id}/steps/{step_id}` GET)
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ] // Added Serialize
  pub struct ResponseError
  {
    /// A machine-readable error code string (e.g., "`server_error`", "`rate_limit_exceeded`").
    pub code : String,
    /// A human-readable description of the error.
    pub message : String,
  }

  /// Represents token usage statistics for a completed response.
  ///
  /// # Used By
  /// - `CreateChatCompletionResponse`
  /// - `CreateCompletionResponse`
  /// - `CreateEmbeddingResponse`
  /// - `RunObject`
  /// - `RunStepObject`
  /// - `Response`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ] // Added Serialize
  pub struct ResponseUsage
  {
    /// The number of prompt tokens.
    #[ serde( alias = "input_tokens" ) ]
    pub prompt_tokens : u32,
    /// The number of completion tokens (optional for embeddings).
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    #[ serde( alias = "output_tokens" ) ]
    pub completion_tokens : Option< u32 >,
    /// The total number of tokens used.
    pub total_tokens : u32,
  }

  /// Configuration for the text response format.
  ///
  /// # Used By
  /// - `TextResponseFormatConfigurationOptions`
  /// - `AssistantsApiResponseFormatOption`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq, Former ) ]
  pub struct TextResponseFormatConfiguration
  {
    /// The type of response format, must be "text".
    pub r#type : String,
  }
  impl Default for TextResponseFormatConfiguration
  {
    /// Creates a default configuration with `type` set to "text".
    #[ inline ]
    fn default() -> Self
    {
      Self { r#type : "text".to_string() }
    }
  }

  /// Wrapper for text response format with nested format field.
  ///
  /// # Used By
  /// - Response object deserialization
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct TextResponseFormatWrapper
  {
    /// The nested format configuration.
    pub format : TextResponseFormatConfiguration,
  }

  /// Represents a 2D coordinate (x, y).
  ///
  /// # Used By
  /// - `ComputerAction::Click`
  /// - `ComputerAction::DoubleClick`
  /// - `ComputerAction::Drag` (within path)
  /// - `ComputerAction::Move`
  /// - `ComputerAction::Scroll`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct Coordinate
  {
    /// The x-coordinate.
    pub x : i64,
    /// The y-coordinate.
    pub y : i64,
  }

  /// Represents attributes for a vector store file.
  ///
  /// Set of 16 key-value pairs that can be attached to an object. This can be
  /// useful for storing additional information about the object in a structured
  /// format, and querying for objects via API or the dashboard. Keys are strings
  /// with a maximum length of 64 characters. Values are strings with a maximum
  /// length of 512 characters, booleans, or numbers.
  ///
  /// # Used By
  /// - `/vector_stores/{vector_store_id}/file_batches` (POST)
  /// - `/vector_stores/{vector_store_id}/files` (POST)
  /// - `/vector_stores/{vector_store_id}/files/{file_id}` (GET, POST)
  /// - `/vector_stores/{vector_store_id}/search` (POST)
  /// - `VectorStoreFileObject`
  /// - `VectorStoreSearchResultItem`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq, Default ) ] // Added Serialize
  #[ serde( transparent ) ]
  pub struct VectorStoreFileAttributes
  (
    /// The underlying map storing key-value pairs (string, number, or boolean).
    pub HashMap<  String, Value  >
  );

  /// Represents a generic API error structure, used in top-level error responses.
  ///
  /// # Used By
  /// - `ErrorResponse` (Wrapper for general API errors)
  /// - `RealtimeServerEventError` (Realtime API specific error event)
  /// - `FineTuningJob.error` (Error details for a fine-tuning job)
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct Error
  {
    /// The type of error (e.g., "`invalid_request_error`", "`api_error`").
    pub r#type : String,
    /// A human-readable description of the error.
    pub message : String,
    /// The specific parameter that caused the error, if applicable.
    pub param : Option< String >,
    /// A short machine-readable error code.
    pub code : Option< String >,
  }

  /// Represents a generic error response wrapper containing an `Error` object.
  ///
  /// Used as the schema for various error responses (e.g., 4xx, 5xx status codes)
  /// across multiple endpoints like:
  /// - `/organization/projects/{project_id}` (POST - update default project)
  /// - `/organization/projects/{project_id}/api_keys/{key_id}` (DELETE)
  /// - `/organization/projects/{project_id}/rate_limits/{rate_limit_id}` (POST)
  /// - `/organization/projects/{project_id}/service_accounts` (GET, POST)
  /// - `/organization/projects/{project_id}/users` (GET, POST)
  /// - `/organization/projects/{project_id}/users/{user_id}` (POST, DELETE)
  /// - `/responses/{response_id}` (DELETE - 404 Not Found)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct ErrorResponse
  {
    /// The detailed error object.
    pub error : Error,
  }

  /// Represents log probability properties for a token.
  ///
  /// # Used By
  /// - `ChatCompletionTokenLogprob`
  /// - `LogProb`
  /// - `CreateTranscriptionResponseJson`
  /// - `TranscriptTextDeltaEvent`
  /// - `TranscriptTextDoneEvent`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ] // Added Serialize
  pub struct LogProbProperties
  {
    /// The token text.
    pub token : String,
    /// The log probability of this token.
    pub logprob : f64,
    /// A list of integers representing the UTF-8 bytes representation of the token. Can be null.
    pub bytes : Option< Vec< i32 > >,
  }

  /// Represents log probability information, including top alternative tokens.
  ///
  /// # Used By
  /// - `ChatCompletionTokenLogprob` (as `top_logprobs`)
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ] // Added Serialize
  pub struct LogProb
  {
    /// The token text.
    pub token : String,
    /// The log probability of this token.
    pub logprob : f64,
    /// A list of integers representing the UTF-8 bytes representation of the token. Can be null.
    pub bytes : Option< Vec< i32 > >,
    /// List of the most likely alternative tokens and their log probabilities at this position.
    pub top_logprobs : Option< Vec< LogProbProperties > >,
  }

  /// Represents voice options for text-to-speech generation.
  ///
  /// # Used By
  /// - `/audio/speech` (POST)
  /// - `RealtimeSession`
  /// - `RealtimeSessionCreateResponse`
  /// - `RealtimeResponseCreateParams`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct VoiceIdsShared( pub String );

  /// Represents audio response format options.
  ///
  /// # Used By
  /// - `/audio/speech` (POST)
  /// - `/audio/transcriptions` (POST)
  /// - `/audio/translations` (POST)
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct AudioResponseFormat
  {
    /// The selected audio format (e.g., "mp3", "wav", "json").
    pub value : String,
  }

  /// Represents options for including additional data in transcription responses.
  ///
  /// # Used By
  /// - `/audio/transcriptions` (POST)
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct TranscriptionInclude
  {
    /// The field to include (currently only "logprobs").
    pub value : String,
  }

  /// Represents model identifiers shared across multiple API endpoints.
  ///
  /// # Used By
  /// - `/chat/completions` (POST)
  /// - `/responses` (POST)
  /// - `/embeddings` (POST)
  /// - `/fine_tuning/jobs` (POST)
  /// - `/assistants` (POST)
  /// - `/assistants/{assistant_id}` (POST)
  /// - `/threads/runs` (POST)
  /// - `/threads/{thread_id}/runs` (POST)
  /// - `/models/{model}` (GET)
  /// - etc. (Many endpoints accept a model ID)
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ModelIdsShared
  {
    /// The model identifier string (e.g., "gpt-5.1-chat-latest", "text-embedding-ada-002").
    pub value : String,
  }

  /// Represents model identifiers specific to the Responses API.
  ///
  /// # Used By
  /// - `/responses` (POST)
  /// - `ResponseProperties`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq, From ) ] // Added From derive
  #[ serde( transparent ) ]
  pub struct ModelIdsResponses
  {
    /// The model identifier string compatible with the Responses API.
    pub value : String,
  }

  /// Represents all possible model identifiers, handling different enum subsets.
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  #[ serde( untagged ) ]
  pub enum ModelIds
  {
    /// A raw string model identifier.
    String( String ),
    /// A model identifier from the shared set.
    Shared( ModelIdsShared ),
    /// A model identifier specific to the Responses API.
    Responses( ModelIdsResponses ),
  }

  /// Represents reasoning effort options for o-series models.
  ///
  /// # Used By
  /// - `/chat/completions` (POST)
  /// - `/responses` (POST)
  /// - `ResponseProperties`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  #[ serde( transparent ) ]
  pub struct ReasoningEffort
  {
    /// The reasoning effort level ("low", "medium", "high").
    pub value : String,
  }

  /// Represents parallel tool call settings.
  ///
  /// # Used By
  /// - `/chat/completions` (POST)
  /// - `/responses` (POST)
  /// - `/assistants` (POST)
  /// - `/assistants/{assistant_id}` (POST)
  /// - `/threads/runs` (POST)
  /// - `/threads/{thread_id}/runs` (POST)
  /// - `FineTuneChatRequestInput`
  /// - `FineTunePreferenceInputData`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ParallelToolCalls
  {
    /// Whether parallel tool calls are enabled.
    pub value : bool,
  }

  /// Represents stop sequence configuration for completions.
  ///
  /// # Used By
  /// - `/chat/completions` (POST)
  /// - `/completions` (POST)
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  #[ serde( untagged ) ]
  pub enum StopConfiguration
  {
    /// A single stop sequence string.
    String( String ),
    /// An array of up to 4 stop sequence strings.
    Array( Vec< String > ),
  }

  /// Represents options for streaming chat completions.
  ///
  /// # Used By
  /// - `/chat/completions` (POST)
  /// - `/completions` (POST)
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ChatCompletionStreamOptions
  {
    /// If set, an additional chunk with usage statistics will be streamed.
    pub include_usage : Option< bool >,
  }

  /// Represents includable fields in Responses API calls.
  ///
  /// # Used By
  /// - `/responses` (POST)
  /// - `/responses/{response_id}` (GET)
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct Includable
  {
    /// The field name to include (e.g., "`file_search_call.results`").
    pub value : String,
  }

  /// Represents the JSON object response format configuration.
  ///
  /// # Used By
  /// - `TextResponseFormatConfigurationOptions`
  /// - `AssistantsApiResponseFormatOption`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ResponseFormatJsonObject
  {
    /// The type, always "`json_object`".
    pub r#type : String,
  }

  /// Represents the JSON schema object used within `ResponseFormatJsonSchema`.
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq, Default ) ]
  #[ serde( transparent ) ]
  pub struct ResponseFormatJsonSchemaSchema( pub serde_json::Value );

  /// Represents the JSON schema response format configuration.
  ///
  /// # Used By
  /// - `TextResponseFormatConfigurationOptions`
  /// - `AssistantsApiResponseFormatOption`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ResponseFormatJsonSchema
  {
    /// The type, always "`json_schema`".
    pub r#type : String,
    /// Details of the JSON schema.
    pub json_schema : ResponseFormatJsonSchemaSchemaDetails,
  }

  /// Details for the JSON schema response format.
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ResponseFormatJsonSchemaSchemaDetails
  {
    /// The name of the response format.
    pub name : String,
    /// An optional description of the response format.
    pub description : Option< String >,
    /// The JSON schema object.
    pub schema : ResponseFormatJsonSchemaSchema,
    /// Whether to enable strict schema adherence. Defaults to false.
    pub strict : Option< bool >,
  }

  /// Represents the overall text response format configuration options.
  ///
  /// # Used By
  /// - `/chat/completions` (POST)
  /// - `ResponseProperties`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  #[ serde( untagged ) ]
  pub enum TextResponseFormatConfigurationOptions
  {
    /// Plain text format.
    Text( TextResponseFormatConfiguration ),
    /// Text format with wrapper (for response deserialization).
    TextWrapper( TextResponseFormatWrapper ),
    /// JSON schema format.
    JsonSchema( ResponseFormatJsonSchema ),
    /// JSON object format.
    JsonObject( ResponseFormatJsonObject ),
  }

  /// Represents the reasoning configuration for o-series models.
  ///
  /// # Used By
  /// - `/chat/completions` (POST)
  /// - `/responses` (POST)
  /// - `ResponseProperties`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct Reasoning
  {
    /// The desired reasoning effort level.
    pub effort : Option< ReasoningEffort >,
    /// Whether to generate a concise or detailed summary of reasoning.
    pub generate_summary : Option< String >, // Enum : concise, detailed
  }

  /// Represents the properties common to model responses across different APIs.
  ///
  /// # Used By
  /// - `CreateChatCompletionRequest`
  /// - `CreateResponse`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ModelResponseProperties
  {
    /// Optional metadata associated with the request.
    pub metadata : Option< Metadata >,
    /// Sampling temperature.
    pub temperature : Option< f32 >,
    /// Nucleus sampling probability.
    pub top_p : Option< f32 >,
    /// An identifier for the end-user.
    pub user : Option< String >,
  }

  /// Represents the properties specific to the main Response object.
  ///
  /// # Used By
  /// - `CreateResponse`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ResponseProperties
  {
    /// The ID of the previous response in a conversation.
    pub previous_response_id : Option< String >,
    /// The model used for the response. *** CHANGED TO String ***
    pub model : String,
    /// Reasoning configuration.
    pub reasoning : Option< Reasoning >,
    /// Maximum number of output tokens allowed.
    pub max_output_tokens : Option< i32 >,
    /// System instructions for the model.
    pub instructions : Option< String >,
    /// Text response format configuration.
    pub text : Option< TextResponseFormatConfigurationOptions >,
    /// List of available tools.
    pub tools : Option< Vec< tools::Tool > >,
    /// Strategy for choosing tools.
    pub tool_choice : Option< tools::ToolChoice >,
    /// Truncation strategy ("auto" or "disabled").
    pub truncation : Option< String >,
  }

  /// Represents the structure for a deleted response confirmation.
  ///
  /// # Used By
  /// - `/responses/{response_id}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct DeleteResponse
  {
    /// The ID of the deleted response.
    pub id : String,
    /// The object type, always "response.deleted".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted file confirmation.
  ///
  /// # Used By
  /// - `/files/{file_id}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct DeleteFileResponse
  {
    /// The ID of the deleted file.
    pub id : String,
    /// The object type, always "file".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted model confirmation.
  ///
  /// # Used By
  /// - `/models/{model}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct DeleteModelResponse
  {
    /// The ID of the deleted model.
    pub id : String,
    /// The object type, always "model".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted assistant confirmation.
  ///
  /// # Used By
  /// - `/assistants/{assistant_id}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct DeleteAssistantResponse
  {
    /// The ID of the deleted assistant.
    pub id : String,
    /// The object type, always "assistant.deleted".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted thread confirmation.
  ///
  /// # Used By
  /// - `/threads/{thread_id}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct DeleteThreadResponse
  {
    /// The ID of the deleted thread.
    pub id : String,
    /// The object type, always "thread.deleted".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted message confirmation.
  ///
  /// # Used By
  /// - `/threads/{thread_id}/messages/{message_id}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct DeleteMessageResponse
  {
    /// The ID of the deleted message.
    pub id : String,
    /// The object type, always "thread.message.deleted".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted vector store confirmation.
  ///
  /// # Used By
  /// - `/vector_stores/{vector_store_id}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct DeleteVectorStoreResponse
  {
    /// The ID of the deleted vector store.
    pub id : String,
    /// The object type, always "`vector_store.deleted`".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted vector store file confirmation.
  ///
  /// # Used By
  /// - `/vector_stores/{vector_store_id}/files/{file_id}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct DeleteVectorStoreFileResponse
  {
    /// The ID of the deleted vector store file.
    pub id : String,
    /// The object type, always "`vector_store.file.deleted`".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted fine-tuning checkpoint permission confirmation.
  ///
  /// # Used By
  /// - `/fine_tuning/checkpoints/{permission_id}/permissions` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct DeleteFineTuningCheckpointPermissionResponse
  {
    /// The ID of the deleted permission.
    pub id : String,
    /// The object type, always "checkpoint.permission".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted invite confirmation.
  ///
  /// # Used By
  /// - `/organization/invites/{invite_id}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct InviteDeleteResponse
  {
    /// The ID of the deleted invite.
    pub id : String,
    /// The object type, always "organization.invite.deleted".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted project confirmation (Note : Projects are archived, not deleted).
  /// This might represent the response from an archive operation.
  ///
  /// # Used By
  /// - `/organization/projects/{project_id}/archive` (POST) - *Likely represents the archived project object*
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct ProjectDeleteResponse // Consider renaming to ProjectArchiveResponse if appropriate
  {
    /// The ID of the project.
    pub id : String,
    /// The object type, likely "organization.project".
    pub object : String,
    /// Indicates if the operation was successful (e.g., archived status).
    pub deleted : bool, // Or perhaps 'archived': bool
  }

  /// Represents the structure for a deleted project API key confirmation.
  ///
  /// # Used By
  /// - `/organization/projects/{project_id}/api_keys/{key_id}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct ProjectApiKeyDeleteResponse
  {
    /// The ID of the deleted API key.
    pub id : String,
    /// The object type, always "`organization.project.api_key.deleted`".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted project service account confirmation.
  ///
  /// # Used By
  /// - `/organization/projects/{project_id}/service_accounts/{service_account_id}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct ProjectServiceAccountDeleteResponse
  {
    /// The ID of the deleted service account.
    pub id : String,
    /// The object type, always "`organization.project.service_account.deleted`".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted project user confirmation.
  ///
  /// # Used By
  /// - `/organization/projects/{project_id}/users/{user_id}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct ProjectUserDeleteResponse
  {
    /// The ID of the deleted user from the project.
    pub id : String,
    /// The object type, always "organization.project.user.deleted".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a deleted user confirmation.
  ///
  /// # Used By
  /// - `/organization/users/{user_id}` (DELETE)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct UserDeleteResponse
  {
    /// The ID of the deleted user.
    pub id : String,
    /// The object type, always "organization.user.deleted".
    pub object : String,
    /// Indicates if the deletion was successful.
    pub deleted : bool,
  }

  /// Represents the structure for a default project error response.
  ///
  /// # Used By
  /// - `/organization/projects/{project_id}` (POST - when updating default project)
  #[ derive( Debug, Deserialize, Clone, PartialEq ) ]
  pub struct DefaultProjectErrorResponse
  {
    /// Error code.
    pub code : i32,
    /// Error message.
    pub message : String,
  }

  /// Represents usage statistics for completions, including token details.
  ///
  /// # Used By
  /// - `CreateChatCompletionResponse`
  /// - `CreateCompletionResponse`
  /// - `RunObject`
  /// - `RunStepObject`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ] // Added Serialize
  pub struct CompletionUsage
  {
    /// Number of tokens in the generated completion.
    pub completion_tokens : i32,
    /// Number of tokens in the prompt.
    pub prompt_tokens : i32,
    /// Total number of tokens used (prompt + completion).
    pub total_tokens : i32,
    /// Detailed breakdown of completion tokens.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub completion_tokens_details : Option< CompletionTokensDetails >,
    /// Detailed breakdown of prompt tokens.
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    pub prompt_tokens_details : Option< PromptTokensDetails >,
  }

  /// Detailed breakdown of completion tokens.
  ///
  /// # Used By
  /// - `CompletionUsage`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ] // Added Serialize
  pub struct CompletionTokensDetails
  {
    /// Tokens from accepted predictions (Predicted Outputs feature).
    pub accepted_prediction_tokens : Option< i32 >,
    /// Audio output tokens generated by the model.
    pub audio_tokens : Option< i32 >,
    /// Tokens generated by the model for reasoning steps.
    pub reasoning_tokens : Option< i32 >,
    /// Tokens from rejected predictions (Predicted Outputs feature).
    pub rejected_prediction_tokens : Option< i32 >,
  }

  /// Detailed breakdown of prompt tokens.
  ///
  /// # Used By
  /// - `CompletionUsage`
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ] // Added Serialize
  pub struct PromptTokensDetails
  {
    /// Audio input tokens present in the prompt.
    pub audio_tokens : Option< i32 >,
    /// Cached tokens present in the prompt (Prompt Caching feature).
    pub cached_tokens : Option< i32 >,
  }

  /// Represents query parameters for listing operations, specifically for pagination.
  ///
  /// # Used By
  /// - `/responses/{response_id}/input_items` (GET)
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ] // Removed From derive
  pub struct ListQuery
  {
    /// The maximum number of items to return.
    #[ serde( skip_serializing_if = "Option::is_none" ) ] // Add skip_serializing_if
    pub limit : Option< u32 >,
  }
} // end mod private

crate ::mod_interface!
{
  exposed use
  {
    Metadata,
    ResponseError,
    ResponseUsage,
    TextResponseFormatConfiguration,
    Coordinate,
    VectorStoreFileAttributes,
    Error,
    ErrorResponse,
    LogProbProperties,
    LogProb,
    VoiceIdsShared,
    AudioResponseFormat,
    TranscriptionInclude,
    ModelIdsShared,
    ModelIdsResponses,
    ModelIds,
    ReasoningEffort,
    ParallelToolCalls,
    StopConfiguration,
    ChatCompletionStreamOptions,
    Includable,
    ResponseFormatJsonObject,
    ResponseFormatJsonSchemaSchema,
    ResponseFormatJsonSchema,
    ResponseFormatJsonSchemaSchemaDetails,
    TextResponseFormatConfigurationOptions,
    Reasoning,
    ModelResponseProperties,
    ResponseProperties,
    DeleteResponse,
    DeleteFileResponse,
    DeleteModelResponse,
    DeleteAssistantResponse,
    DeleteThreadResponse,
    DeleteMessageResponse,
    DeleteVectorStoreResponse,
    DeleteVectorStoreFileResponse,
    DeleteFineTuningCheckpointPermissionResponse,
    InviteDeleteResponse,
    ProjectDeleteResponse,
    ProjectApiKeyDeleteResponse,
    ProjectServiceAccountDeleteResponse,
    ProjectUserDeleteResponse,
    UserDeleteResponse,
    DefaultProjectErrorResponse,
    CompletionUsage,
    CompletionTokensDetails,
    PromptTokensDetails,
    ListQuery,
  };
}