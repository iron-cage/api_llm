//! Tests for AI Chatbot/Conversational Interface Example
//!
//! This test suite verifies the functionality of an AI chatbot system that provides
//! intelligent conversational capabilities using the `HuggingFace` API.

#![allow(clippy::missing_inline_in_public_items)]

use api_huggingface::
{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  components::
  {
  input::InferenceParameters,
  models::Models,
  },
  secret::Secret,
};
use std::collections::HashMap;
use core::fmt::Write;

#[ allow( missing_docs ) ]
/// Represents conversation context for maintaining dialogue state
#[ derive( Debug, Clone ) ]
pub struct ConversationContext
{
  /// Unique conversation identifier
  pub session_id : String,
  /// Conversation history for context preservation
  pub history : Vec< ( String, String ) >, // (user_input, bot_response)
  /// Current conversation style/personality
  pub style : ConversationStyle,
  /// Model being used for this conversation
  pub model : String,
  /// Custom parameters for this conversation
  pub parameters : InferenceParameters,
}

/// Different conversation styles for personality customization
#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub enum ConversationStyle
{
  /// Friendly, relaxed conversation style
  Casual,
  /// Professional, precise conversation style  
  Formal,
  /// Imaginative, expressive conversation style
  Creative,
  /// Detailed, technical conversation style
  Technical,
  /// Empathetic, encouraging conversation style
  Supportive,
}

/// Chatbot system for managing conversations
#[ derive( Debug ) ]
pub struct ChatbotSystem
{
  client : Client< HuggingFaceEnvironmentImpl >,
  active_sessions : HashMap< String, ConversationContext >,
}

impl ChatbotSystem
{
  /// Create new chatbot system with client
  #[ must_use ]
  pub fn new( client : Client< HuggingFaceEnvironmentImpl > ) -> Self
  {
  Self
  {
      client,
      active_sessions : HashMap::new(),
  }
  }

  /// Start new conversation session
  ///
  /// # Panics
  /// Panics if the session context cannot be retrieved after insertion
  pub fn start_conversation( &mut self, session_id : &str, style : ConversationStyle ) -> &ConversationContext
  {
  let context = ConversationContext
  {
      session_id : session_id.to_string(),
      history : Vec::new(),
      style,
      model : match style
      {
  ConversationStyle::Technical => Models::mistral_7b_instruct().to_string(),
  _ => Models::llama_3_3_70b_instruct().to_string(),
      },
      parameters : match style
      {
  ConversationStyle::Creative => InferenceParameters::new()
          .with_temperature( 0.9 )
          .with_max_new_tokens( 150 ),
  ConversationStyle::Formal => InferenceParameters::new()
          .with_temperature( 0.3 )
          .with_max_new_tokens( 200 ),
  ConversationStyle::Technical => InferenceParameters::new()
          .with_temperature( 0.5 )
          .with_max_new_tokens( 300 ),
  _ => InferenceParameters::new()
          .with_temperature( 0.7 )
          .with_max_new_tokens( 150 ),
      },
  };

  self.active_sessions.insert( session_id.to_string(), context );
  self.active_sessions.get( session_id ).expect( "[start_conversation] Session should exist in active_sessions immediately after insert - check HashMap::insert() and HashMap::get() implementation" )
  }

  /// Process user input and generate response
  ///
  /// # Errors
  /// Returns error if API request fails, session not found, or response generation fails
  pub async fn process_input( 
  &mut self, 
  session_id : &str, 
  user_input : &str 
  ) -> Result< String, Box< dyn std::error::Error > >
  {
  // Build prompt with immutable borrow
  let prompt = {
      let context = self.active_sessions.get( session_id )
  .ok_or( "Session not found" )?;
      Self::build_contextual_prompt( context, user_input )
  };
  
  // Get model and parameters with immutable borrow
  let ( model, parameters ) = {
      let context = self.active_sessions.get( session_id )
  .ok_or( "Session not found" )?;
      ( context.model.clone(), context.parameters.clone() )
  };

  // Generate response
  let response = self.client
      .inference()
      .create_with_parameters( &prompt, &model, parameters )
      .await?;

  let bot_response = response.extract_text_or_default( "Sorry, I couldn't generate a response." );

  // Update conversation history with mutable borrow
  let context = self.active_sessions.get_mut( session_id )
      .ok_or( "Session not found" )?;
  
  context.history.push( ( user_input.to_string(), bot_response.clone() ) );

  // Keep only last 5 exchanges to manage context length
  if context.history.len() > 5
  {
      context.history.remove( 0 );
  }

  Ok( bot_response )
  }

  /// Build contextual prompt from conversation history
  fn build_contextual_prompt( context : &ConversationContext, user_input : &str ) -> String
  {
  let style_prefix = match context.style
  {
      ConversationStyle::Casual => "You are a friendly, casual AI assistant. Respond in a relaxed, conversational way.",
      ConversationStyle::Formal => "You are a professional AI assistant. Respond formally and precisely.",
      ConversationStyle::Creative => "You are a creative AI assistant. Be imaginative and expressive in your responses.",
      ConversationStyle::Technical => "You are a technical AI assistant. Provide detailed, accurate technical information.",
      ConversationStyle::Supportive => "You are a supportive AI assistant. Be empathetic and encouraging.",
  };

  let mut prompt = format!( "{style_prefix}\n\n" );

  // Add recent conversation history for context
  for ( user_msg, bot_msg ) in &context.history
  {
      write!( &mut prompt, "User : {user_msg}\nAssistant : {bot_msg}\n\n" ).expect( "[build_contextual_prompt] Failed to write conversation history to String - String write! should never fail" );
  }

  write!( &mut prompt, "User : {user_input}\nAssistant : " ).expect( "[build_contextual_prompt] Failed to write user input to String - String write! should never fail" );
  prompt
  }

  /// Get conversation context
  #[ must_use ]
  pub fn get_context( &self, session_id : &str ) -> Option< &ConversationContext >
  {
  self.active_sessions.get( session_id )
  }

  /// End conversation session
  pub fn end_conversation( &mut self, session_id : &str ) -> Option< ConversationContext >
  {
  self.active_sessions.remove( session_id )
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;
  use workspace_tools as workspace;

  fn get_api_key_for_testing() -> Option< String >
  {
  let workspace = workspace::workspace().ok()?;
  let secrets = workspace.load_secrets_from_file( "-secrets.sh" ).ok()?;
  secrets.get( "HUGGINGFACE_API_KEY" ).cloned()
  }

  fn create_test_client() -> Option< Client< HuggingFaceEnvironmentImpl > >
  {
  let api_key = get_api_key_for_testing()?;
  let secret = Secret::new( api_key );
  let env = HuggingFaceEnvironmentImpl::build( secret, None ).ok()?;
  Client::build( env ).ok()
  }

  #[ test ]
  fn test_conversation_context_creation()
  {
  let context = ConversationContext
  {
      session_id : "test-session".to_string(),
      history : Vec::new(),
      style : ConversationStyle::Casual,
      model : Models::llama_3_3_70b_instruct().to_string(),
      parameters : InferenceParameters::default(),
  };

  assert_eq!( context.session_id, "test-session" );
  assert_eq!( context.style, ConversationStyle::Casual );
  assert!( context.history.is_empty() );
  }

  #[ test ]
  fn test_conversation_style_parameters()
  {
  // Test Creative style parameters
  let creative_params = match ConversationStyle::Creative
  {
      ConversationStyle::Creative => InferenceParameters::new()
  .with_temperature( 0.9 )
  .with_max_new_tokens( 150 ),
      _ => panic!( "Unexpected style" ),
  };

  assert_eq!( creative_params.temperature, Some( 0.9 ) );
  assert_eq!( creative_params.max_new_tokens, Some( 150 ) );

  // Test Formal style parameters
  let formal_params = match ConversationStyle::Formal
  {
      ConversationStyle::Formal => InferenceParameters::new()
  .with_temperature( 0.3 )
  .with_max_new_tokens( 200 ),
      _ => panic!( "Unexpected style" ),
  };

  assert_eq!( formal_params.temperature, Some( 0.3 ) );
  assert_eq!( formal_params.max_new_tokens, Some( 200 ) );
  }

  #[ tokio::test ]
  async fn test_chatbot_system_creation()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let chatbot = ChatbotSystem::new( client );
  assert!( chatbot.active_sessions.is_empty() );
  }

  #[ tokio::test ]
  async fn test_conversation_session_management()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let mut chatbot = ChatbotSystem::new( client );

  // Start new conversation
  let session_id = "test-session-123";
  let context = chatbot.start_conversation( session_id, ConversationStyle::Casual );

  assert_eq!( context.session_id, session_id );
  assert_eq!( context.style, ConversationStyle::Casual );
  assert!( context.history.is_empty() );

  // Verify session is active
  assert!( chatbot.get_context( session_id ).is_some() );

  // End conversation
  let ended_context = chatbot.end_conversation( session_id );
  assert!( ended_context.is_some() );
  assert!( chatbot.get_context( session_id ).is_none() );
  }

  #[ tokio::test ]
  async fn test_contextual_prompt_building()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let _chatbot = ChatbotSystem::new( client );

  let context = ConversationContext
  {
      session_id : "test".to_string(),
      history : vec!
      [
  ( "Hello".to_string(), "Hi there!".to_string() ),
  ( "How are you?".to_string(), "I'm doing well, thank you!".to_string() ),
      ],
      style : ConversationStyle::Casual,
      model : Models::llama_3_3_70b_instruct().to_string(),
      parameters : InferenceParameters::default(),
  };

  let prompt = ChatbotSystem::build_contextual_prompt( &context, "What's the weather like?" );

  assert!( prompt.contains( "friendly, casual AI assistant" ) );
  assert!( prompt.contains( "User : Hello" ) );
  assert!( prompt.contains( "Assistant : Hi there!" ) );
  assert!( prompt.contains( "User : How are you?" ) );
  assert!( prompt.contains( "User : What's the weather like?" ) );
  }

  #[ tokio::test ]
  async fn test_different_conversation_styles()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let _chatbot = ChatbotSystem::new( client );

  // Test different style prompts
  let casual_context = ConversationContext
  {
      session_id : "casual".to_string(),
      history : Vec::new(),
      style : ConversationStyle::Casual,
      model : Models::llama_3_3_70b_instruct().to_string(),
      parameters : InferenceParameters::default(),
  };

  let formal_context = ConversationContext
  {
      session_id : "formal".to_string(),
      history : Vec::new(),
      style : ConversationStyle::Formal,
      model : Models::llama_3_3_70b_instruct().to_string(),
      parameters : InferenceParameters::default(),
  };

  let casual_prompt = ChatbotSystem::build_contextual_prompt( &casual_context, "Hello" );
  let formal_prompt = ChatbotSystem::build_contextual_prompt( &formal_context, "Hello" );

  assert!( casual_prompt.contains( "friendly, casual" ) );
  assert!( formal_prompt.contains( "professional" ) );
  assert_ne!( casual_prompt, formal_prompt );
  }

  #[ tokio::test ]
  async fn test_model_selection_by_style()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let mut chatbot = ChatbotSystem::new( client );

  // Test Creative style uses Llama
  let creative_context = chatbot.start_conversation(
      "creative-test",
      ConversationStyle::Creative
  );
  assert_eq!( creative_context.model, Models::llama_3_3_70b_instruct() );

  // Test Technical style uses Mistral
  let technical_context = chatbot.start_conversation(
      "technical-test",
      ConversationStyle::Technical
  );
  assert_eq!( technical_context.model, Models::mistral_7b_instruct() );
  }

  #[ tokio::test ]
  async fn test_error_handling_invalid_session()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let mut chatbot = ChatbotSystem::new( client );

  // Try to process input for non-existent session
  let result = chatbot.process_input( "non-existent", "Hello" ).await;
  assert!( result.is_err() );
  assert!( result.unwrap_err().to_string().contains( "Session not found" ) );
  }

  #[ tokio::test ]
  async fn test_conversation_style_enum_completeness()
  {
  // Verify all conversation styles are properly defined
  let styles = vec!
  [
      ConversationStyle::Casual,
      ConversationStyle::Formal,
      ConversationStyle::Creative,
      ConversationStyle::Technical,
      ConversationStyle::Supportive,
  ];

  for style in styles
  {
      // Each style should be cloneable and debuggable
      let cloned = style;
      assert_eq!( format!( "{style:?}" ), format!( "{cloned:?}" ) );
  }
  }

  #[ tokio::test ]
  async fn test_parameter_optimization_per_style()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let mut chatbot = ChatbotSystem::new( client );

  // Test that different styles get different parameters (create separately to avoid borrowing conflicts)
  let creative_temp;
  let creative_tokens;
  {
      let creative = chatbot.start_conversation( "c1", ConversationStyle::Creative );
      creative_temp = creative.parameters.temperature.expect( "[test_conversation_personality_styles] Creative InferenceParameters.temperature should be Some - check start_conversation() Creative style parameters" );
      creative_tokens = creative.parameters.max_new_tokens.expect( "[test_conversation_personality_styles] Creative InferenceParameters.max_new_tokens should be Some - check start_conversation() Creative style parameters" );
  }

  let formal_temp;
  {
      let formal = chatbot.start_conversation( "f1", ConversationStyle::Formal );
      formal_temp = formal.parameters.temperature.expect( "[test_conversation_personality_styles] Formal InferenceParameters.temperature should be Some - check start_conversation() Formal style parameters" );
  }

  let technical_tokens;
  {
      let technical = chatbot.start_conversation( "t1", ConversationStyle::Technical );
      technical_tokens = technical.parameters.max_new_tokens.expect( "[test_conversation_personality_styles] Technical InferenceParameters.max_new_tokens should be Some - check start_conversation() Technical style parameters" );
  }

  // Creative should have higher temperature
  assert!( creative_temp > formal_temp );
  
  // Technical should have more tokens than creative
  assert!( technical_tokens > creative_tokens );
  
  // Formal should have lowest temperature
  assert!( ( formal_temp - 0.3 ).abs() < f32::EPSILON );
  }
}