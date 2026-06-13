//! Production-ready interactive chat with Claude AI including real streaming responses.
//!
//! **Complexity:** ⭐⭐⭐⭐ (Advanced)
//!
//! This example demonstrates:
//! - Real-time interactive chat loop where you can type and get responses
//! - Real streaming responses that appear as they're generated (with streaming feature)
//! - Simulated streaming fallback (without streaming feature)
//! - Dynamic conversation history building and management
//! - Proper error handling for input/output operations
//! - Feature-conditional behavior based on Claude streaming support
//! - Performance optimization with faster Claude model selection
//! - Clean conversation flow with persistent history across turns
//!
//! ## Usage
//!
//! ```bash
//! # Set your API key
//! export ANTHROPIC_API_KEY="your-api-key-here"
//!
//! # With real streaming (recommended for production)
//! cargo run --example `claude_api_interactive` --features streaming
//!
//! # With all features including streaming
//! cargo run --example `claude_api_interactive` --features full
//!
//! # Basic version with simulated streaming
//! cargo run --example claude_api_interactive
//! ```
//!
//! **Important**: Type your messages and press Enter. Type 'quit', 'exit', or 'bye' to end.
//! Note : This is NOT for automated testing - it's for manual interactive use only.
//!
//! **Target Audience**: Advanced developers building chat applications
//!
//! ## Learning Objectives
//!
//! - Production-ready interactive chat implementation patterns
//! - Real-time streaming response handling with Claude API
//! - Dynamic conversation state management for chat applications
//! - Feature-gated functionality for different deployment scenarios
//! - Error recovery strategies for interactive applications
//! - Performance optimization techniques for chat systems

#[ cfg( feature = "streaming" ) ]
use futures::StreamExt;
use api_claude::{ Client, CreateMessageRequest, Message, Role, Content };
use std::io::{ self, Write };

#[ tokio::main( flavor = "current_thread" ) ]
#[ allow( clippy::too_many_lines ) ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  // Initialize Claude client from environment
  println!( "Initializing Claude client for interactive chat..." );
  let client = Client::from_env()?;

  println!( "🤖 Interactive Claude Chat" );
  println!( "==========================" );
  println!( "Type your messages and press Enter to chat with Claude." );
  println!( "Commands : 'quit', 'exit', or 'bye' to end the conversation." );

  #[ cfg( feature = "streaming" ) ]
  println!( "✨ Streaming mode : Real-time responses enabled" );

  #[ cfg( not( feature = "streaming" ) ) ]
  println!( "📝 Standard mode : Simulated streaming responses" );

  println!( "Model : Claude 3.5 Sonnet (optimized for interactive chat)\n" );

  // Initialize conversation history
  let mut conversation_history : Vec< Message > = Vec::new();

  // Main interactive chat loop
  loop
  {
    // Get user input with prompt
    print!( "💬 You : " );
    io::stdout().flush()?;

    let mut input = String::new();
    match io::stdin().read_line( &mut input )
    {
      Ok( 0 ) =>
      {
        // EOF reached (e.g., when input is not available in non-interactive mode)
        println!( "\n⚠️  No input available. Use this example in interactive terminal only." );
        println!( "Run : cargo run --example claude_api_interactive" );
        break;
      }
      Ok( _ ) => {}
      Err( e ) =>
      {
        println!( "\n❌ Error reading input : {e}" );
        println!( "Please try again or restart the chat application." );
        break;
      }
    }

    let user_message = input.trim().to_string();

    // Handle empty input
    if user_message.is_empty()
    {
      continue;
    }

    // Handle exit commands
    if matches!( user_message.to_lowercase().as_str(), "quit" | "exit" | "bye" )
    {
      println!( "\n👋 Goodbye! Thanks for chatting with Claude!" );
      break;
    }

    // Add user message to conversation history
    conversation_history.push( Message {
      role : Role::User,
      content : vec![ Content::Text {
        r#type : "text".to_string(),
        text : user_message,
      } ],
      cache_control : None,
    });

    // Create request with conversation history
    let request = CreateMessageRequest::builder()
      .model( "claude-sonnet-4-5-20250929".to_string() ) // Current Sonnet model for chat
      .max_tokens( 1024 )
      .messages( conversation_history.clone() )
      .temperature( 0.7 ) // Balanced creativity and coherence
      .build();

    print!( "\n🤖 Claude : " );
    io::stdout().flush()?;

    // Use real streaming if available, otherwise fallback to simulated streaming
    #[ cfg( feature = "streaming" ) ]
    {
      match client.create_message_stream( request ).await
      {
        Ok( mut stream ) =>
        {
          let mut full_response = String::new();

          // Process streaming chunks as they arrive
          while let Some( chunk_result ) = stream.next().await
          {
            match chunk_result
            {
              Ok( event ) =>
              {
                // Handle different event types from Claude streaming API
                if let Some( delta ) = event.delta()
                {
                  if let Some( text_chunk ) = delta.text()
                  {
                    print!( "{text_chunk}" );
                    io::stdout().flush()?;
                    full_response.push_str( text_chunk );
                  }
                }
              }
              Err( e ) =>
              {
                println!( "\n⚠️  Streaming error : {e}" );
                println!( "Continuing with standard request..." );
                break;
              }
            }
          }

          println!( "\n" );

          // Add Claude's response to conversation history
          if !full_response.is_empty()
          {
            conversation_history.push( Message {
              role : Role::Assistant,
              content : vec![ Content::Text {
                r#type : "text".to_string(),
                text : full_response,
              } ],
              cache_control : None,
            });
          }
        }
        Err( e ) =>
        {
          println!( "❌ Streaming error : {e}" );
          println!( "Please try again or type 'quit' to exit.\n" );
        }
      }
    }

    // Fallback to standard generation with simulated streaming
    #[ cfg( not( feature = "streaming" ) ) ]
    {
      match client.create_message( request ).await
      {
        Ok( response ) =>
        {
          if let Some( content ) = response.content.first()
          {
            if content.r#type == "text"
            {
              if let Some( text ) = &content.text
              {
                // Simulate streaming by printing words with realistic delays
                let words : Vec< &str > = text.split_whitespace().collect();
                for ( i, word ) in words.iter().enumerate()
                {
                  print!( "{}", word );
                  if i < words.len() - 1
                  {
                    print!( " " );
                  }
                  io::stdout().flush()?;

                  // Variable delay based on word length for more natural feel
                  let delay_ms = match word.len()
                  {
                    0..=3 => 60,
                    4..=7 => 80,
                    _ => 100,
                  };
                  tokio::time::sleep( tokio::time::Duration::from_millis( delay_ms ) ).await;
                }
                println!( "\n" );

                // Add Claude's response to conversation history
                conversation_history.push( Message {
                  role : Role::Assistant,
                  content : vec![ Content::Text {
                    r#type : "text".to_string(),
                    text : text.clone(),
                  } ],
                  cache_control : None,
                });
              }
              else
              {
                println!( "⚠️  Claude response contained no text content." );
              }
            }
            else
            {
              println!( "⚠️  Claude returned non-text content (unexpected for chat)." );
            }
          }
          else
          {
            println!( "⚠️  Claude generated no response content." );
          }
        }
        Err( e ) =>
        {
          println!( "❌ Error : {}", e );
          println!( "Please try again or type 'quit' to exit.\n" );
        }
      }
    }

    // Show conversation statistics periodically
    if conversation_history.len() % 10 == 0 && !conversation_history.is_empty()
    {
      println!( "📊 Conversation stats : {} messages exchanged", conversation_history.len() );
    }
  }

  // Display session summary
  let user_messages = conversation_history.iter().filter( |m| matches!( m.role, Role::User ) ).count();
  let ai_messages = conversation_history.iter().filter( |m| matches!( m.role, Role::Assistant ) ).count();

  println!( "\n📊 === Chat Session Summary ===" );
  println!( "User messages : {user_messages}" );
  println!( "Claude responses : {ai_messages}" );
  println!( "Total conversation turns : {}", conversation_history.len() );

  #[ cfg( feature = "streaming" ) ]
  println!( "Streaming mode : ✅ Real-time responses" );

  #[ cfg( not( feature = "streaming" ) ) ]
  println!( "Streaming mode : 📝 Simulated responses" );

  println!( "\n🎯 === Production Implementation Notes ===" );
  println!( "• Interactive chat with persistent conversation history" );
  println!( "• Feature-gated streaming for different deployment scenarios" );
  println!( "• Error recovery strategies for network issues" );
  println!( "• Performance-optimized model selection (Claude 3.5 Sonnet)" );
  println!( "• Production-ready input/output handling with proper flushing" );
  println!( "• Graceful exit handling with session statistics" );

  Ok( () )
}