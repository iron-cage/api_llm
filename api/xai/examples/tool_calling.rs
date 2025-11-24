//! Function calling / tool use example.
//!
//! Demonstrates how to use function calling with the Grok API to execute
//! custom functions based on the model's decisions.
//!
//! Run with:
//! ```bash
//! cargo run --example tool_calling --features integration
//! ```

use api_xai::{ Client, XaiEnvironmentImpl, Secret, ChatCompletionRequest, Message, Tool, ClientApiAccessors };
use serde_json::json;

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  // Setup client
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
  let env = XaiEnvironmentImpl::new( secret )?;
  let client = Client::build( env )?;

  println!( "🚀 XAI Grok API - Function Calling Example\n" );

  // Define a weather function
  let get_weather = Tool::function(
    "get_current_weather",
    "Get the current weather in a given location",
    json!({
      "type": "object",
      "properties": {
        "location": {
          "type": "string",
          "description": "The city and state, e.g. San Francisco, CA"
        },
        "unit": {
          "type": "string",
          "enum": ["celsius", "fahrenheit"],
          "description": "Temperature unit"
        }
      },
      "required": ["location"]
    })
  );

  // Initial request with tool
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![
      Message::user( "What's the weather like in Tokyo?" )
    ] )
    .tools( vec![ get_weather.clone() ] )
    .form();

  println!( "📤 Sending request with weather tool definition...\n" );

  let response = client.chat().create( request ).await?;
  let choice = &response.choices[ 0 ];

  // Check if model wants to call the function
  if let Some( tool_calls ) = &choice.message.tool_calls
  {
    println!( "🔧 Model requested {} function call(s):\n", tool_calls.len() );

    for tool_call in tool_calls
    {
      println!( "   Function : {}", tool_call.function.name );
      println!( "   Arguments : {}", tool_call.function.arguments );

      // Simulate function execution
      let args : serde_json::Value = serde_json::from_str( &tool_call.function.arguments )?;

      let location = args[ "location" ].as_str().unwrap_or( "unknown" );
      println!( "   → Simulating weather lookup for : {location}\n" );

      // Mock weather result
      let weather_result = json!({
        "temperature": 22,
        "unit": "celsius",
        "condition": "sunny",
        "humidity": 65
      });

      // Send function result back to model
      let followup = ChatCompletionRequest::former()
        .model( "grok-2-1212".to_string() )
        .messages( vec![
          Message::user( "What's the weather like in Tokyo?" ),
          choice.message.clone(),
          Message::tool( &tool_call.id, weather_result.to_string() ),
        ] )
        .tools( vec![ get_weather.clone() ] )
        .form();

      println!( "📤 Sending function result back to model...\n" );

      let final_response = client.chat().create( followup ).await?;

      // Display final answer
      if let Some( content ) = &final_response.choices[ 0 ].message.content
      {
        println!( "🤖 Final Answer:\n{content}\n" );
      }
    }
  }
  else
  {
    println!( "ℹ️  Model responded directly without calling function:\n" );
    if let Some( content ) = &choice.message.content
    {
      println!( "{content}\n" );
    }
  }

  Ok( () )
}
