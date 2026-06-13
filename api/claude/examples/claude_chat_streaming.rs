//! Streaming conversation example using real Anthropic API
//! Run with : cargo run --example `claude_chat_streaming` --features integration

use api_claude::{ Client, CreateMessageRequest, Message };
use std::io::{ self, Write };

#[ tokio::main( flavor = "current_thread" ) ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
    println!("🤖 AI Streaming Chat Example");
    println!("============================");
    println!("Interactive chat with Claude using streaming responses.");
    println!("Type your messages and press Enter. Type 'quit' to exit.");
    println!("Type 'clear' to clear conversation history.\n");
    
    let client = Client::from_workspace()
        .expect("Must have valid ANTHROPIC_API_KEY in ../../secret/-secrets.sh or environment");
    
    let mut conversation_history = Vec::new();
    let mut conversation_count = 0;
    
    loop
    {
        print!("You : ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let user_message = input.trim().to_string();
        
        match user_message.to_lowercase().as_str()
        {
            "quit" | "exit" | "bye" => {
                println!("👋 Thanks for chatting! Goodbye!");
                break;
            }
            "clear" => {
                conversation_history.clear();
                conversation_count = 0;
                println!("🗑️ Conversation history cleared!");
                continue;
            }
            "" => continue,
            _ => {}
        }
        
        // Add user message to conversation
        conversation_history.push(Message::user(user_message));
        conversation_count += 1;
        
        let stream_request = CreateMessageRequest {
            model : "claude-haiku-4-5-20251001".to_string(), // Fast model for chat
            max_tokens : 500,
            messages : conversation_history.clone(),
            stream : Some(false), // Note : Real streaming implementation would require additional setup
            temperature : Some(0.8),
            system : Some( vec![ api_claude::SystemContent::text( "You are Claude, a helpful AI assistant. Be conversational, engaging, and concise. Show personality while being helpful." ) ] ),
            tools : None,
            tool_choice : None,
        };
        
        print!("Claude : ");
        io::stdout().flush()?;
        
        match client.create_message(stream_request).await
        {
            Ok(response) => {
                if let Some(text_content) = response.content.first()
                {
                    if let Some(text) = &text_content.text
                    {
                        // Simulate streaming effect by printing with delays
                        let words : Vec< &str > = text.split_whitespace().collect();
                        for (i, word) in words.iter().enumerate()
                        {
                            print!("{word}");
                            if i < words.len() - 1
                            {
                                print!(" ");
                            }
                            io::stdout().flush()?;
                            
                            // Small delay to simulate streaming
                            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                        }
                        println!("\n");
                        
                        // Add Claude's response to conversation history
                        conversation_history.push(Message::assistant(text.clone()));
                        
                        // Keep conversation history manageable (last 12 messages)
                        if conversation_history.len() > 12
                        {
                            conversation_history.drain(0..2); // Remove oldest pair
                        }
                        
                        // Show conversation stats
                        if conversation_count % 5 == 0
                        {
                            println!("📊 Chat Stats : {} exchanges, {} tokens in last response", 
                                conversation_count, response.usage.output_tokens);
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Error : {e}");
                println!("Please try again or type 'quit' to exit.");
                // Remove the last user message if there was an error
                conversation_history.pop();
            }
        }
    }
    
    if conversation_count > 0
    {
        println!("\n📈 Session Summary:");
        println!("   • Total exchanges : {conversation_count}");
        let history_len = conversation_history.len(); println!("   • Messages in history : {history_len}");
        println!("   • Model used : claude-haiku-4-5-20251001");
    }
    
    Ok(())
}