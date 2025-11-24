//! Tests for API component types (messages, requests, models).

use api_xai::
{
  Message, Role, ChatCompletionRequest, Usage,
  Model, ListModelsResponse
};

#[ test ]
fn message_system_constructor_works()
{
  let msg = Message::system( "You are a helpful assistant" );

  assert_eq!( msg.role, Role::System );
  assert_eq!( msg.content, Some( "You are a helpful assistant".to_string() ) );
  assert_eq!( msg.tool_calls, None );
  assert_eq!( msg.tool_call_id, None );
}

#[ test ]
fn message_user_constructor_works()
{
  let msg = Message::user( "Hello!" );

  assert_eq!( msg.role, Role::User );
  assert_eq!( msg.content, Some( "Hello!".to_string() ) );
  assert_eq!( msg.tool_calls, None );
  assert_eq!( msg.tool_call_id, None );
}

#[ test ]
fn message_assistant_constructor_works()
{
  let msg = Message::assistant( "Hi there!" );

  assert_eq!( msg.role, Role::Assistant );
  assert_eq!( msg.content, Some( "Hi there!".to_string() ) );
  assert_eq!( msg.tool_calls, None );
  assert_eq!( msg.tool_call_id, None );
}

#[ test ]
fn message_tool_constructor_works()
{
  let msg = Message::tool( "call_123", r#"{"result": "ok"}"# );

  assert_eq!( msg.role, Role::Tool );
  assert_eq!( msg.content, Some( r#"{"result": "ok"}"#.to_string() ) );
  assert_eq!( msg.tool_call_id, Some( "call_123".to_string() ) );
  assert_eq!( msg.tool_calls, None );
}

#[ test ]
fn chat_request_serializes_correctly()
{
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Test" ) ] )
    .temperature( 0.7 )
    .max_tokens( 100u32 )
    .form();

  let json = serde_json::to_value( &request ).unwrap();

  assert_eq!( json[ "model" ], "grok-2-1212" );
  assert!( ( json[ "temperature" ].as_f64().unwrap() - 0.7 ).abs() < 0.0001 );
  assert_eq!( json[ "max_tokens" ], 100 );
  assert!( json[ "messages" ].is_array() );
}

#[ test ]
fn chat_request_omits_none_fields()
{
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Test" ) ] )
    .form();

  let json = serde_json::to_string( &request ).unwrap();

  // Should not contain "temperature", "max_tokens", etc.
  assert!( !json.contains( "\"temperature\"" ) );
  assert!( !json.contains( "\"max_tokens\"" ) );
  assert!( !json.contains( "\"top_p\"" ) );
}

#[ test ]
fn chat_request_includes_some_fields()
{
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Test" ) ] )
    .temperature( 0.5 )
    .form();

  let json = serde_json::to_string( &request ).unwrap();

  assert!( json.contains( "\"temperature\"" ) );
  assert!( json.contains( "0.5" ) );
}

#[ test ]
fn former_builder_works()
{
  let request = ChatCompletionRequest::former()
    .model( "grok-4".to_string() )
    .messages( vec![
      Message::system( "You are helpful" ),
      Message::user( "Hello" ),
    ] )
    .temperature( 0.8 )
    .max_tokens( 200u32 )
    .top_p( 0.9 )
    .frequency_penalty( 0.1 )
    .presence_penalty( 0.2 )
    .stream( true )
    .form();

  assert_eq!( request.model, "grok-4" );
  assert_eq!( request.messages.len(), 2 );
  assert_eq!( request.temperature, Some( 0.8 ) );
  assert_eq!( request.max_tokens, Some( 200 ) );
  assert_eq!( request.top_p, Some( 0.9 ) );
  assert_eq!( request.frequency_penalty, Some( 0.1 ) );
  assert_eq!( request.presence_penalty, Some( 0.2 ) );
  assert_eq!( request.stream, Some( true ) );
}

#[ test ]
fn role_serializes_as_lowercase()
{
  let system = serde_json::to_string( &Role::System ).unwrap();
  assert_eq!( system, r#""system""# );

  let user = serde_json::to_string( &Role::User ).unwrap();
  assert_eq!( user, r#""user""# );

  let assistant = serde_json::to_string( &Role::Assistant ).unwrap();
  assert_eq!( assistant, r#""assistant""# );

  let tool = serde_json::to_string( &Role::Tool ).unwrap();
  assert_eq!( tool, r#""tool""# );
}

#[ test ]
fn role_deserializes_from_lowercase()
{
  let system : Role = serde_json::from_str( r#""system""# ).unwrap();
  assert_eq!( system, Role::System );

  let user : Role = serde_json::from_str( r#""user""# ).unwrap();
  assert_eq!( user, Role::User );

  let assistant : Role = serde_json::from_str( r#""assistant""# ).unwrap();
  assert_eq!( assistant, Role::Assistant );

  let tool : Role = serde_json::from_str( r#""tool""# ).unwrap();
  assert_eq!( tool, Role::Tool );
}

#[ test ]
fn usage_serializes_correctly()
{
  let usage = Usage
  {
    prompt_tokens : 10,
    completion_tokens : 20,
    total_tokens : 30,
  };

  let json = serde_json::to_value( &usage ).unwrap();

  assert_eq!( json[ "prompt_tokens" ], 10 );
  assert_eq!( json[ "completion_tokens" ], 20 );
  assert_eq!( json[ "total_tokens" ], 30 );
}

#[ test ]
fn usage_deserializes_correctly()
{
  let json = r#"{
    "prompt_tokens": 15,
    "completion_tokens": 25,
    "total_tokens": 40
  }"#;

  let usage : Usage = serde_json::from_str( json ).unwrap();

  assert_eq!( usage.prompt_tokens, 15 );
  assert_eq!( usage.completion_tokens, 25 );
  assert_eq!( usage.total_tokens, 40 );
}

#[ test ]
fn model_deserializes_correctly()
{
  let json = r#"{
    "id": "grok-2-1212",
    "object": "model",
    "created": 1234567890,
    "owned_by": "xai"
  }"#;

  let model : Model = serde_json::from_str( json ).unwrap();

  assert_eq!( model.id, "grok-2-1212" );
  assert_eq!( model.object, "model" );
  assert_eq!( model.created, 1_234_567_890 );
  assert_eq!( model.owned_by, "xai" );
}

#[ test ]
fn list_models_response_deserializes_correctly()
{
  let json = r#"{
    "object": "list",
    "data": [
      {
        "id": "grok-2-1212",
        "object": "model",
        "created": 1234567890,
        "owned_by": "xai"
      },
      {
        "id": "grok-4",
        "object": "model",
        "created": 1234567891,
        "owned_by": "xai"
      }
    ]
  }"#;

  let response : ListModelsResponse = serde_json::from_str( json ).unwrap();

  assert_eq!( response.object, "list" );
  assert_eq!( response.data.len(), 2 );
  assert_eq!( response.data[ 0 ].id, "grok-2-1212" );
  assert_eq!( response.data[ 1 ].id, "grok-4" );
}

#[ test ]
fn message_with_none_content_omits_field()
{
  let mut msg = Message::user( "test" );
  msg.content = None;

  let json = serde_json::to_string( &msg ).unwrap();

  // Should not serialize content field if None
  assert!( !json.contains( "\"content\"" ) );
}

#[ test ]
fn message_with_some_content_includes_field()
{
  let msg = Message::user( "hello world" );

  let json = serde_json::to_string( &msg ).unwrap();

  assert!( json.contains( "\"content\"" ) );
  assert!( json.contains( "hello world" ) );
}

#[ test ]
fn chat_request_clone_works()
{
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![ Message::user( "Test" ) ] )
    .form();

  let cloned = request.clone();

  assert_eq!( request, cloned );
}

#[ test ]
fn role_clone_and_partial_eq_work()
{
  let role1 = Role::User;
  let role2 = role1.clone();

  assert_eq!( role1, role2 );
}

#[ test ]
fn usage_clone_and_partial_eq_work()
{
  let usage1 = Usage
  {
    prompt_tokens : 10,
    completion_tokens : 20,
    total_tokens : 30,
  };

  let usage2 = usage1.clone();

  assert_eq!( usage1, usage2 );
}
