//! Test data factories for OpenAI API test suite
//!
//! This module provides factory functions to generate consistent test data,
//! replacing hardcoded test data patterns and improving test maintainability.

use api_openai::exposed::
{
  components ::
  {
    responses ::
    {
      CreateResponseRequest,
      ResponseObject,
      ResponseInput,
      ResponseItemList,
    },
    input ::
    {
      InputItem,
      InputMessage,
      InputContentPart,
      InputText,
      InputImage,
      ListedInputItem,
    },
    output ::
    {
      OutputItem,
      OutputContentPart,
    },
    common ::
    {
      ModelIdsResponses,
      Metadata,
      ResponseUsage,
    },
    tools ::
    {
      Tool,
      ToolChoice,
      ToolChoiceFunction,
      FunctionTool,
      FunctionParameters,
      FileSearchTool,
      WebSearchTool,
      ComputerTool,
    },
  }
};

use std::collections::HashMap;
use serde_json::json;

/// Factory for creating basic CreateResponseRequest instances
pub struct CreateResponseRequestFactory;

impl CreateResponseRequestFactory
{
  /// Creates a basic request with string input
  pub fn with_simple_string( model : &str, input : &str ) -> CreateResponseRequest
  {
    CreateResponseRequest::former()
    .model( ModelIdsResponses { value : model.to_string() } )
    .input( ResponseInput::String( input.to_string() ) )
    .form()
  }

  /// Creates a request with optional parameters
  pub fn with_optional_params( model : &str, input : &str, temperature : Option< f32 >, top_p : Option< f32 >, store : Option< bool > ) -> CreateResponseRequest
  {
    let mut former = CreateResponseRequest::former()
    .model( ModelIdsResponses { value : model.to_string() } )
    .input( ResponseInput::String( input.to_string() ) );

    if let Some( temp ) = temperature
    {
      former = former.temperature( temp );
    }
    if let Some( p ) = top_p
    {
      former = former.top_p( p );
    }
    if let Some( s ) = store
    {
      former = former.store( s );
    }

    former.form()
  }

  /// Creates a request with message items
  pub fn with_message_items( model : &str, messages : Vec< InputMessage > ) -> CreateResponseRequest
  {
    let input_items = messages.into_iter().map( InputItem::Message ).collect();
    CreateResponseRequest::former()
    .model( ModelIdsResponses { value : model.to_string() } )
    .input( ResponseInput::Items( input_items ) )
    .form()
  }

  /// Creates a request with metadata
  pub fn with_metadata( model : &str, input : &str, metadata_pairs : Vec< ( &str, &str ) > ) -> CreateResponseRequest
  {
    let metadata = Metadata::from( metadata_pairs );
    CreateResponseRequest::former()
    .model( ModelIdsResponses { value : model.to_string() } )
    .input( ResponseInput::String( input.to_string() ) )
    .metadata( metadata )
    .form()
  }

  /// Creates a request with tools and tool choice
  pub fn with_tools( model : &str, input : &str, tools : Vec< Tool >, tool_choice : Option< ToolChoice > ) -> CreateResponseRequest
  {
    let mut former = CreateResponseRequest::former()
    .model( ModelIdsResponses { value : model.to_string() } )
    .input( ResponseInput::String( input.to_string() ) )
    .tools( tools );

    if let Some( choice ) = tool_choice
    {
      former = former.tool_choice( choice );
    }

    former.form()
  }
}

/// Factory for creating input message structures
pub struct InputMessageFactory;

impl InputMessageFactory
{
  /// Creates a simple text message
  pub fn text_message( role : &str, text : &str ) -> InputMessage
  {
    InputMessage::former()
    .r#type( "message".to_string() )
    .role( role.to_string() )
    .content( vec![ InputContentPart::Text( InputText { text : text.to_string() } ) ] )
    .form()
  }

  /// Creates a message with mixed content (text and image)
  pub fn multimodal_message( role : &str, text : &str, image_url : Option< &str >, file_id : Option< &str >, detail : Option< &str > ) -> InputMessage
  {
    let mut content = vec![ InputContentPart::Text( InputText { text : text.to_string() } ) ];

    content.push( InputContentPart::Image( InputImage
    {
      image_url : image_url.map( |s| s.to_string() ),
      file_id : file_id.map( |s| s.to_string() ),
      detail : detail.map( |s| s.to_string() ),
    } ) );

    InputMessage::former()
    .r#type( "message".to_string() )
    .role( role.to_string() )
    .content( content )
    .form()
  }

  /// Creates a conversation with multiple messages
  pub fn conversation( exchanges : Vec< ( &str, &str, &str ) > ) -> Vec< InputMessage >
  {
    exchanges.into_iter()
    .map( |( role, text, _context )| Self::text_message( role, text ) )
    .collect()
  }
}

/// Factory for creating tool structures
pub struct ToolFactory;

impl ToolFactory
{
  /// Creates a basic function tool
  pub fn function_tool( name : &str, description : Option< &str > ) -> Tool
  {
    let mut former = FunctionTool::former()
    .name( name.to_string() )
    .parameters( FunctionParameters::default() );

    if let Some( desc ) = description
    {
      former = former.description( desc.to_string() );
    }

    Tool::Function( former.form() )
  }

  /// Creates a file search tool with default settings
  pub fn file_search_tool() -> Tool
  {
    Tool::FileSearch( FileSearchTool::default() )
  }

  /// Creates a web search tool with default settings
  pub fn web_search_tool() -> Tool
  {
    Tool::WebSearch( WebSearchTool::default() )
  }

  /// Creates a computer use tool with display parameters
  pub fn computer_tool( display_width : Option< f64 >, display_height : Option< f64 >, environment : Option< &str > ) -> Tool
  {
    let mut former = ComputerTool::former();

    if let Some( width ) = display_width
    {
      former = former.display_width( width );
    }
    if let Some( height ) = display_height
    {
      former = former.display_height( height );
    }
    if let Some( env ) = environment
    {
      former = former.environment( env.to_string() );
    }

    Tool::ComputerUse( former.form() )
  }
}

/// Factory for creating tool choice options
pub struct ToolChoiceFactory;

impl ToolChoiceFactory
{
  /// Creates a string-based tool choice
  pub fn string_choice( choice : &str ) -> ToolChoice
  {
    ToolChoice::String( choice.to_string() )
  }

  /// Creates a function-specific tool choice
  pub fn function_choice( function_name : &str ) -> ToolChoice
  {
    ToolChoice::Function
    {
      r#type : "function".to_string(),
      function : ToolChoiceFunction
      {
        name : function_name.to_string(),
      },
    }
  }
}

/// Factory for creating response objects for deserialization tests
pub struct ResponseObjectFactory;

impl ResponseObjectFactory
{
  /// Creates basic response object JSON string
  pub fn simple_completed_response( response_id : &str, message_id : &str, text : &str, model : &str ) -> String
  {
    json!({
      "id": response_id,
      "object": "response",
      "created_at": 1678886400,
      "status": "completed",
      "output": [
        {
          "type": "message",
          "id": message_id,
          "role": "assistant",
          "content": [
            {
              "type": "output_text",
              "text": text
            }
          ],
          "status": "completed"
        }
      ],
      "model": model,
      "parallel_tool_calls": true
    }).to_string()
  }

  /// Creates response object with usage data
  pub fn response_with_usage( response_id : &str, prompt_tokens : u32, completion_tokens : u32, total_tokens : u32 ) -> String
  {
    json!({
      "id": response_id,
      "object": "response",
      "created_at": 1678886400,
      "status": "completed",
      "output": [],
      "model": "gpt-5.1-chat-latest",
      "parallel_tool_calls": true,
      "usage": {
        "prompt_tokens": prompt_tokens,
        "completion_tokens": completion_tokens,
        "total_tokens": total_tokens
      }
    }).to_string()
  }

  /// Creates response object with refusal
  pub fn response_with_refusal( response_id : &str, refusal_reason : &str ) -> String
  {
    json!({
      "id": response_id,
      "object": "response",
      "created_at": 1678886400,
      "status": "completed",
      "output": [
        {
          "type": "message",
          "id": "msg_1",
          "role": "assistant",
          "content": [
            {
              "type": "output_text",
              "refusal": refusal_reason
            }
          ],
          "status": "completed"
        }
      ],
      "model": "gpt-5.1-chat-latest",
      "parallel_tool_calls": true
    }).to_string()
  }

  /// Creates failed response object
  pub fn failed_response( response_id : &str, error_message : &str, error_code : &str ) -> String
  {
    json!({
      "id": response_id,
      "object": "response",
      "created_at": 1678886400,
      "status": "failed",
      "output": [],
      "model": "gpt-5.1-chat-latest",
      "parallel_tool_calls": true,
      "last_error": {
        "code": error_code,
        "message": error_message
      }
    }).to_string()
  }

  /// Creates empty response list
  pub fn empty_response_list() -> String
  {
    json!({
      "object": "list",
      "data": [],
      "first_id": null,
      "last_id": null,
      "has_more": false
    }).to_string()
  }

  /// Creates response list with multiple items
  pub fn response_list_with_items( response_ids : Vec< &str > ) -> String
  {
    let responses : Vec< serde_json::Value > = response_ids.into_iter().map( |id|
    {
      json!({
        "id": id,
        "object": "response",
        "created_at": 1678886400,
        "status": "completed",
        "output": [],
        "model": "gpt-5.1-chat-latest",
        "parallel_tool_calls": true
      })
    } ).collect();

    json!({
      "object": "list",
      "data": responses,
      "first_id": "first",
      "last_id": "last",
      "has_more": false
    }).to_string()
  }

  /// Creates delete response
  pub fn delete_response( response_id : &str, deleted : bool ) -> String
  {
    json!({
      "id": response_id,
      "object": "response.deleted",
      "deleted": deleted
    }).to_string()
  }
}

/// Common test data constants
pub mod constants
{
  pub const DEFAULT_MODEL: &str = "gpt-5.1-chat-latest";
  pub const DEFAULT_MODEL_MINI: &str = "gpt-5-mini";
  pub const COMPUTER_MODEL: &str = "computer-use-preview";

  pub const DEFAULT_TEMPERATURE: f32 = 0.8;
  pub const DEFAULT_TOP_P: f32 = 0.9;

  pub const DEFAULT_DISPLAY_WIDTH: f64 = 1920.0;
  pub const DEFAULT_DISPLAY_HEIGHT: f64 = 1080.0;
  pub const DEFAULT_ENVIRONMENT: &str = "macos";

  pub const SAMPLE_IMAGE_URL: &str = "https://example.com/image.jpg";
  pub const SAMPLE_IMAGE_DETAIL: &str = "high";
}

/// Test scenario builders for common use cases
pub mod scenarios
{
  use super::*;

  /// Creates a basic chat scenario
  pub fn basic_chat( user_message : &str ) -> CreateResponseRequest
  {
    CreateResponseRequestFactory::with_simple_string( constants::DEFAULT_MODEL, user_message )
  }

  /// Creates a multi-turn conversation scenario
  pub fn multi_turn_conversation() -> CreateResponseRequest
  {
    let messages = InputMessageFactory::conversation( vec![
      ( "user", "Who won the world series in 2020?", "sports_question" ),
      ( "assistant", "The Los Angeles Dodgers won the World Series in 2020.", "sports_answer" ),
      ( "user", "Where was it played?", "follow_up_question" ),
    ] );
    CreateResponseRequestFactory::with_message_items( constants::DEFAULT_MODEL, messages )
  }

  /// Creates a multimodal scenario with image
  pub fn multimodal_with_image( text : &str ) -> CreateResponseRequest
  {
    let message = InputMessageFactory::multimodal_message(
      "user",
      text,
      Some( constants::SAMPLE_IMAGE_URL ),
      None,
      Some( constants::SAMPLE_IMAGE_DETAIL )
    );
    CreateResponseRequestFactory::with_message_items( constants::DEFAULT_MODEL, vec![ message ] )
  }

  /// Creates a function calling scenario
  pub fn function_calling( query : &str ) -> CreateResponseRequest
  {
    let tools = vec![ ToolFactory::function_tool( "get_current_weather", Some( "Get the current weather in a given location" ) ) ];
    let tool_choice = Some( ToolChoiceFactory::function_choice( "get_current_weather" ) );
    CreateResponseRequestFactory::with_tools( constants::DEFAULT_MODEL, query, tools, tool_choice )
  }

  /// Creates a file search scenario
  pub fn file_search( query : &str ) -> CreateResponseRequest
  {
    let tools = vec![ ToolFactory::file_search_tool() ];
    let tool_choice = Some( ToolChoiceFactory::string_choice( "auto" ) );
    CreateResponseRequestFactory::with_tools( constants::DEFAULT_MODEL, query, tools, tool_choice )
  }

  /// Creates a web search scenario
  pub fn web_search( query : &str ) -> CreateResponseRequest
  {
    let tools = vec![ ToolFactory::web_search_tool() ];
    let tool_choice = Some( ToolChoiceFactory::string_choice( "auto" ) );
    CreateResponseRequestFactory::with_tools( constants::DEFAULT_MODEL, query, tools, tool_choice )
  }

  /// Creates a computer use scenario
  pub fn computer_use( instruction : &str ) -> CreateResponseRequest
  {
    let tools = vec![ ToolFactory::computer_tool(
      Some( constants::DEFAULT_DISPLAY_WIDTH ),
      Some( constants::DEFAULT_DISPLAY_HEIGHT ),
      Some( constants::DEFAULT_ENVIRONMENT )
    ) ];
    let tool_choice = Some( ToolChoiceFactory::string_choice( "auto" ) );
    CreateResponseRequestFactory::with_tools( constants::COMPUTER_MODEL, instruction, tools, tool_choice )
  }
}