//! Enhanced function calling demonstration.
//!
//! Shows how to use parallel tool execution for efficient
//! handling of multiple function calls.
//!
//! Run with:
//! ```bash
//! cargo run --example enhanced_tools_demo --features "integration,enhanced_tools"
//! ```

use api_xai::{
  Client, XaiEnvironmentImpl, Secret, ChatCompletionRequest,
  Message, Tool, ToolResult,
  execute_tool_calls_parallel, ClientApiAccessors
};
use serde_json::json;

#[ allow( clippy::too_many_lines ) ]  // Demo file showcasing tool calling features
#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  println!( "🛠️  XAI Grok API - Enhanced Tools Demo\n" );
  println!( "======================================\n" );

  // Load API key
  let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
  let env = XaiEnvironmentImpl::new( secret )?;
  let client = Client::build( env )?;

  // Define multiple tools
  let weather_tool = Tool::function(
    "get_weather",
    "Get current weather for a location",
    json!({
      "type": "object",
      "properties": {
        "location": {
          "type": "string",
          "description": "City name"
        }
      },
      "required": ["location"]
    })
  );

  let time_tool = Tool::function(
    "get_time",
    "Get current time for a timezone",
    json!({
      "type": "object",
      "properties": {
        "timezone": {
          "type": "string",
          "description": "Timezone (e.g., America/New_York)"
        }
      },
      "required": ["timezone"]
    })
  );

  let stock_tool = Tool::function(
    "get_stock_price",
    "Get stock price for a symbol",
    json!({
      "type": "object",
      "properties": {
        "symbol": {
          "type": "string",
          "description": "Stock symbol (e.g., AAPL)"
        }
      },
      "required": ["symbol"]
    })
  );

  println!( "📋 Available Tools:" );
  println!( "   1. get_weather - Weather information" );
  println!( "   2. get_time - Timezone information" );
  println!( "   3. get_stock_price - Stock prices\n" );

  // Initial request asking for multiple pieces of information
  let request = ChatCompletionRequest::former()
    .model( "grok-2-1212".to_string() )
    .messages( vec![
      Message::user( "What's the weather in Tokyo, the current time in London, and Apple's stock price?" )
    ] )
    .tools( vec![ weather_tool.clone(), time_tool.clone(), stock_tool.clone() ] )
    .form();

  println!( "📤 Sending request with multiple tools..." );

  let response = client.chat().create( request ).await?;

  if let Some( choice ) = response.choices.first()
  {
    if let Some( ref tool_calls ) = choice.message.tool_calls
    {
      println!( "✅ Model requested {} tool call(s)\n", tool_calls.len() );

      // Display requested tool calls
      for ( i, call ) in tool_calls.iter().enumerate()
      {
        println!( "   {}. {} ({})", i + 1, call.function.name, call.id );
        println!( "      Args : {}\n", call.function.arguments );
      }

      // Execute all tool calls in parallel
      println!( "⚡ Executing tool calls in parallel...\n" );

      let start = std::time::Instant::now();

      let results = execute_tool_calls_parallel(
        tool_calls.clone(),
        | call | async move {
          // Simulate tool execution with some delay
          tokio::time::sleep( core::time::Duration::from_millis( 500 ) ).await;

          match call.function.name.as_str()
          {
            "get_weather" =>
            {
              let result = json!({
                "location": "Tokyo",
                "temperature": 18,
                "conditions": "Partly cloudy",
                "humidity": 65
              });
              Ok( ToolResult::new( call.id, &result ) )
            }
            "get_time" =>
            {
              let result = json!({
                "timezone": "Europe/London",
                "time": "14:30 GMT",
                "date": "2025-11-08"
              });
              Ok( ToolResult::new( call.id, &result ) )
            }
            "get_stock_price" =>
            {
              let result = json!({
                "symbol": "AAPL",
                "price": 189.50,
                "change": "+2.30",
                "percent_change": "+1.23%"
              });
              Ok( ToolResult::new( call.id, &result ) )
            }
            _ =>
            {
              Err( format!( "Unknown function : {}", call.function.name ).into() )
            }
          }
        }
      ).await;

      let elapsed = start.elapsed();

      println!( "✅ Executed {} tool calls in {:?}\n", results.len(), elapsed );

      // Display results
      println!( "📊 Tool Results:" );
      for ( i, result ) in results.iter().enumerate()
      {
        match result
        {
          Ok( tool_result ) =>
          {
            println!( "   {}. {} ✓", i + 1, tool_result.tool_call_id );
            println!( "      {}\n", tool_result.result );
          }
          Err( e ) =>
          {
            println!( "   {}. ✗ Error : {}\n", i + 1, e );
          }
        }
      }

      // Prepare tool result messages for the model
      let mut messages = vec![
        Message::user( "What's the weather in Tokyo, the current time in London, and Apple's stock price?" ),
        choice.message.clone(),
      ];

      for tool_result in results.iter().flatten()
      {
        messages.push( Message::tool(
          &tool_result.tool_call_id,
          tool_result.result.clone()
        ) );
      }

      // Send results back to the model
      println!( "📤 Sending tool results to model...\n" );

      let followup_request = ChatCompletionRequest::former()
        .model( "grok-2-1212".to_string() )
        .messages( messages )
        .tools( vec![ weather_tool, time_tool, stock_tool ] )
        .form();

      let followup_response = client.chat().create( followup_request ).await?;

      if let Some( final_choice ) = followup_response.choices.first()
      {
        if let Some( ref content ) = final_choice.message.content
        {
          println!( "🤖 Final Response:\n   {content}\n" );
        }
      }

      println!( "💡 Performance Benefit:" );
      println!( "   - Parallel execution : ~{elapsed:?}" );
      #[ allow( clippy::cast_possible_truncation ) ]  // Small number of tool calls in demo
      let sequential_time = elapsed * tool_calls.len() as u32;
      println!( "   - Sequential would take : ~{sequential_time:?}" );
      println!( "   - Speedup : ~{}x faster\n", tool_calls.len() );
    }
    else if let Some( ref content ) = choice.message.content
    {
      println!( "ℹ️  Model responded directly without using tools:\n   {content}\n" );
    }
  }

  Ok( () )
}
