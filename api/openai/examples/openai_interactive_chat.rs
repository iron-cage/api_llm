//! Interactive Chat Example - Real-time streaming conversation
//!
//! This example demonstrates:
//! - Interactive chat loop where you can type and get responses
//! - Simulated streaming responses that appear word by word
//! - Clean conversation flow with dynamic history
//! - Proper error handling and recovery
//! - Graceful error handling and recovery
//! - Performance optimization with faster models
//!
//! ## Usage
//!
//! ```bash
//! # Run the example
//! cargo run --example openai_chat_interactive
//! ```
//!
//! Type your messages and press Enter. Type 'quit', 'exit', or 'bye' to end.
//!
//! ## What You'll Learn
//!
//! - Real-time user input handling with stdin
//! - Simulated streaming response display
//! - Dynamic conversation history management
//! - Production-ready chat application patterns
//! - Performance optimization techniques
//!
//! **Complexity**: ⭐⭐⭐⭐ (Advanced)
//! **Target Audience**: Advanced developers building chat applications
//!
//! Note : This is NOT for automated testing - it's for manual interactive use only.

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput },
    input ::{ InputItem, InputMessage, InputContentPart, InputText },
    output ::{ OutputItem, OutputContentPart },
  },
};
use std::io::{ self, Write };

#[ tokio::main ]
#[ allow( clippy::too_many_lines ) ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  println!( "Interactive OpenAI Chat" );
  println!( "{}", "=".repeat( 50 ) );
  println!( "📝 Simulated streaming mode - responses appear word by word." );
  println!( "Type your messages and press Enter." );
  println!( "Type 'quit', 'exit', or 'bye' to end the conversation.\n" );

  // Initialize the client
  let secret = api_openai::secret::Secret::load_with_fallbacks( "OPENAI_API_KEY" )
    .expect( "Failed to load OPENAI_API_KEY. Please set environment variable or add to workspace secrets file." );

  let env = api_openai::environment::OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    api_openai ::environment::OpenAIRecommended::base_url().to_string(),
    api_openai ::environment::OpenAIRecommended::realtime_base_url().to_string()
  ).expect( "Failed to create environment" );

  let client = Client::build( env ).expect( "Failed to create client" );

  // Use faster model for interactive experience
  let model = "gpt-5-nano".to_string();
  println!( "🤖 Using model : {model} (optimized for speed)" );
  println!( "🎯 Temperature : 0.7 (balanced creativity)" );
  println!();

  // Initialize conversation state
  let mut conversation_history = Vec::new();
  let mut turn_number = 0;

  // Interactive conversation loop
  let stdin = io::stdin();

  loop
  {
    // Get user input
    print!( "You : " );
    io ::stdout().flush()?;

    let mut input = String::new();
    match stdin.read_line( &mut input )
    {
      Ok( _ ) =>
      {
        let user_input = input.trim();

        // Handle empty input
        if user_input.is_empty()
        {
          continue;
        }

        // Check for exit commands
        let lower_input = user_input.to_lowercase();
        if lower_input == "quit" || lower_input == "exit" || lower_input == "bye"
        {
          println!( "\n👋 Goodbye! Thanks for chatting!" );
          println!( "Total conversation turns : {turn_number}" );
          break;
        }

        turn_number += 1;

        // Add user message to conversation history
        conversation_history.push( InputItem::Message(
          InputMessage::former()
            .role( "user" )
            .content( vec![
              InputContentPart::Text(
                InputText::former()
                  .text( user_input.to_string() )
                  .form()
              )
            ])
            .form()
        ));

        // Simulated streaming implementation
        let request = CreateResponseRequest::former()
          .model( model.clone() )
          .input( ResponseInput::Items( conversation_history.clone() ) )
          .temperature( 0.7 )
          .max_output_tokens( 1024 )
          .form();

        print!( "AI: " );
        io ::stdout().flush()?;

        // Show typing indicator
        print!( "🤔 thinking..." );
        io ::stdout().flush()?;

        // Make the request
        match client.responses().create( request ).await
        {
          Ok( response ) =>
          {
            // Clear typing indicator
            print!( "\r          \r" ); // Clear the thinking message

            // Extract response text
            let mut ai_response = String::new();
            if let Some( OutputItem::Message( message_struct ) ) = response.output.first()
            {
              if let Some( OutputContentPart::Text { text, .. } ) = message_struct.content.first()
              {
                ai_response.push_str( text );
              }
            }

              // Simulate streaming by printing word by word
              let words : Vec< &str > = ai_response.split_whitespace().collect();
              for ( i, word ) in words.iter().enumerate()
              {
                print!( "{word}" );
                if i < words.len() - 1
                {
                  print!( " " );
                }
                io ::stdout().flush()?;

                // Small delay to simulate streaming
                tokio ::time::sleep( tokio::time::Duration::from_millis( 50 ) ).await;
              }
              println!(); // New line after response

              // Add AI response to conversation history
              if !ai_response.is_empty()
              {
                conversation_history.push( InputItem::Message(
                  InputMessage::former()
                    .role( "assistant" )
                    .content( vec![
                      InputContentPart::Text(
                        InputText::former()
                          .text( ai_response )
                          .form()
                      )
                    ])
                    .form()
                ));
              }
            }
            Err( e ) =>
            {
              print!( "\r          \r" ); // Clear thinking message
              println!( "❌ Error : {e}" );
              println!( "💡 Please check your connection and try again." );
            }
          }

        // Show conversation stats occasionally
        if turn_number % 5 == 0
        {
          println!( "\n📊 Conversation stats : {} turns, {} messages in history",
                   turn_number, conversation_history.len() );
        }
      }
      Err( e ) =>
      {
        println!( "\n❌ Input error : {e}" );
        println!( "💡 Please try typing your message again." );
      }
    }

    println!(); // Extra spacing between turns
  }

  // Final statistics
  println!( "\n=== Session Summary ===" );
  println!( "Total turns : {turn_number}" );
  println!( "Final history length : {} messages", conversation_history.len() );

  println!( "Streaming mode : Simulated 📝" );

  println!( "\n🎯 Key Features Demonstrated:" );
  println!( "✅ Real-time user interaction" );
  println!( "✅ Dynamic conversation building" );
  println!( "✅ Simulated streaming experience" );

  println!( "✅ Graceful error handling" );
  println!( "✅ Performance-optimized model selection" );
  println!( "\n💼 This pattern is ready for production chat applications!" );

  Ok( () )
}