//! Multi-turn conversation example with the `OpenAI` API.
//!
//! This example demonstrates:
//! - Building conversation history across multiple turns
//! - Context-aware responses that remember previous interactions
//! - Proper role alternation (user → assistant → user → assistant)
//! - Context retention and conversation state management
//! - Token usage growth with conversation length
//! - Conversation history building patterns
//!
//! ## Usage
//!
//! ```bash
//! # Set your API key
//! export OPENAI_API_KEY="your-api-key-here"
//!
//! # Run the example
//! cargo run --example openai_multi_turn_conversation
//! ```
//!
//! ## What You'll Learn
//!
//! - How to maintain conversation context across API calls
//! - Building and managing conversation history
//! - Role management in multi-turn scenarios
//! - Context preservation techniques
//! - Token usage implications of longer conversations
//! - Conversation flow patterns
//!
//! **Complexity**: ⭐⭐⭐ (Intermediate)
//! **Target Audience**: Intermediate developers learning conversation patterns
//!
//! This example shows how to build natural, context-aware conversations.

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput, ResponseObject },
    output ::{ OutputItem, OutputContentPart },
    common ::ModelIdsResponses,
  },
};
use core::fmt::Write;

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  println!( "OpenAI Multi-Turn Conversation Example" );
  println!( "=====================================\n" );

  // Initialize the client with API key from environment variable
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

  // Start with conversation context as a growing string
  let mut conversation_context = String::new();

  // Turn 1: User asks about Japan travel
  println!( "🔄 Turn 1: Starting conversation about Japan travel\n" );

  let user_question_1 = "I'm planning a trip to Japan in spring. What are some must-visit places for cherry blossom viewing?";
  writeln!( conversation_context, "User : {user_question_1}" ).unwrap();

  println!( "User : {user_question_1}" );

  let request_1 = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5-mini".to_string() ) )
    .input( ResponseInput::String( conversation_context.clone() ) )
    .temperature( 0.8 )
    .max_output_tokens( 2048 )
    .form();

  let response_1 : ResponseObject = client.responses().create( request_1 ).await?;

  if let Some( OutputItem::Message( message_struct ) ) = response_1.output.first()
  {
    if let Some( OutputContentPart::Text { text, .. } ) = message_struct.content.first()
    {
      println!( "AI: {text}\n" );
      writeln!( conversation_context, "AI: {text}" ).unwrap();
    }
  }

  if let Some( usage ) = &response_1.usage
  {
    println!( "📊 Tokens used (Turn 1): {} prompt + {} completion = {} total\n",
      usage.prompt_tokens,
      usage.completion_tokens.unwrap_or( 0 ),
      usage.total_tokens
    );
  }

  // Turn 2: User asks for more specific information
  println!( "🔄 Turn 2: Follow-up question about timing\n" );

  let user_question_2 = "That sounds amazing! What's the best time in spring to visit? I want to catch the peak bloom.";
  writeln!( conversation_context, "User : {user_question_2}" ).unwrap();

  println!( "User : {user_question_2}" );

  let request_2 = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5-mini".to_string() ) )
    .input( ResponseInput::String( conversation_context.clone() ) )
    .temperature( 0.8 )
    .max_output_tokens( 2048 )
    .form();

  let response_2 : ResponseObject = client.responses().create( request_2 ).await?;

  if let Some( OutputItem::Message( message_struct ) ) = response_2.output.first()
  {
    if let Some( OutputContentPart::Text { text, .. } ) = message_struct.content.first()
    {
      println!( "AI: {text}\n" );
      writeln!( conversation_context, "AI: {text}" ).unwrap();
    }
  }

  if let Some( usage ) = &response_2.usage
  {
    println!( "📊 Tokens used (Turn 2): {} prompt + {} completion = {} total\n",
      usage.prompt_tokens,
      usage.completion_tokens.unwrap_or( 0 ),
      usage.total_tokens
    );
  }

  // Turn 3: User asks about practical details
  println!( "🔄 Turn 3: Practical planning question\n" );

  let user_question_3 = "Perfect! How many days would you recommend for the trip, and should I book accommodations in advance?";
  writeln!( conversation_context, "User : {user_question_3}" ).unwrap();

  println!( "User : {user_question_3}" );

  let request_3 = CreateResponseRequest::former()
    .model( ModelIdsResponses::from( "gpt-5-mini".to_string() ) )
    .input( ResponseInput::String( conversation_context.clone() ) )
    .temperature( 0.8 )
    .max_output_tokens( 2048 )
    .form();

  let response_3 : ResponseObject = client.responses().create( request_3 ).await?;

  if let Some( OutputItem::Message( message_struct ) ) = response_3.output.first()
  {
    if let Some( OutputContentPart::Text { text, .. } ) = message_struct.content.first()
    {
      println!( "AI: {text}\n" );
    }
  }

  if let Some( usage ) = &response_3.usage
  {
    println!( "📊 Tokens used (Turn 3): {} prompt + {} completion = {} total\n",
      usage.prompt_tokens,
      usage.completion_tokens.unwrap_or( 0 ),
      usage.total_tokens
    );
  }

  // Show conversation summary
  println!( "📋 Conversation Summary" );
  println!( "======================" );
  println!( "Total turns : 3" );
  println!( "Conversation context length : {} characters", conversation_context.len() );

  if let ( Some( usage_1 ), Some( usage_2 ), Some( usage_3 ) ) = ( &response_1.usage, &response_2.usage, &response_3.usage )
  {
    println!( "Token usage progression:" );
    println!( "  Turn 1: {} total tokens", usage_1.total_tokens );
    println!( "  Turn 2: {} total tokens (+{})", usage_2.total_tokens,
      usage_2.total_tokens - usage_1.total_tokens );
    println!( "  Turn 3: {} total tokens (+{})", usage_3.total_tokens,
      usage_3.total_tokens - usage_2.total_tokens );
  }

  println!( "\n💡 Technical Notes:" );
  println!( "- Each turn includes ALL previous conversation for context" );
  println!( "- Token usage grows with conversation length" );
  println!( "- Context preservation requires careful string management" );
  println!( "- Consider conversation truncation for very long chats" );
  println!( "- This approach uses string concatenation for simplicity" );

  Ok( () )
}