//! AI Chatbot/Conversational Interface Example
//!
//! This example demonstrates building an intelligent conversational AI chatbot using `HuggingFace` models.
//! Features include conversation context management, multiple personality styles, streaming responses,
//! and session persistence.
//!
//! ## Usage
//!
//! ```bash
//! export HUGGINGFACE_API_KEY="your-api-key-here"
//! cargo run --example huggingface_chat_conversational --features="full"
//! ```
//!
//! ## Commands
//!
//! - `/style < casual|formal|creative|technical|supportive >` - Change conversation style
//! - `/model < model-name >` - Change AI model
//! - `/history` - Show conversation history
//! - `/clear` - Clear conversation history
//! - `/export` - Export conversation to file
//! - `/help` - Show available commands
//! - `/quit` - Exit the chatbot

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
  providers::ChatMessage,
  secret::Secret,
};
use std::
{
  collections::HashMap,
  io::{ self, Write as IoWrite },
  fs,
  time::{ SystemTime, UNIX_EPOCH },
};
use core::fmt::Write;

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
  /// Session start time
  pub started_at : SystemTime,
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

impl ConversationStyle
{
  /// Parse style from string
  fn from_str( s : &str ) -> Option< Self >
  {
  match s.to_lowercase().as_str()
  {
      "casual" => Some( Self::Casual ),
      "formal" => Some( Self::Formal ),
      "creative" => Some( Self::Creative ),
      "technical" => Some( Self::Technical ),
      "supportive" => Some( Self::Supportive ),
      _ => None,
  }
  }

  /// Get style description
  fn description( self ) -> &'static str
  {
  match self
  {
      Self::Casual => "Friendly, relaxed conversation style",
      Self::Formal => "Professional, precise conversation style",
      Self::Creative => "Imaginative, expressive conversation style", 
      Self::Technical => "Detailed, technical conversation style",
      Self::Supportive => "Empathetic, encouraging conversation style",
  }
  }
}

/// Chatbot system for managing conversations
#[ derive( Debug ) ]
pub struct ChatbotSystem
{
  client : Client< HuggingFaceEnvironmentImpl >,
  sessions : HashMap< String, ConversationContext >,
  current_session : String,
}

impl ChatbotSystem
{
  /// Create new chatbot system with client
  /// 
  /// # Panics
  /// 
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  #[ must_use ]
  pub fn new( client : Client< HuggingFaceEnvironmentImpl > ) -> Self
  {
  let current_session = format!( "session_{}", SystemTime::now().duration_since( UNIX_EPOCH ).unwrap().as_secs() );
  
  Self
  {
      client,
      sessions : HashMap::new(),
      current_session,
  }
  }


  /// Start new conversation session
  /// 
  /// # Panics
  /// 
  /// Panics if the session was not properly inserted (should never happen)
  pub fn start_conversation( &mut self, session_id : &str, style : ConversationStyle ) -> &ConversationContext
  {
  let context = ConversationContext
  {
      session_id : session_id.to_string(),
      history : Vec::new(),
      style,
      model : match style
      {
  ConversationStyle::Technical => "codellama/CodeLlama-7b-Instruct-hf".to_string(),
  _ => "meta-llama/Meta-Llama-3-8B-Instruct".to_string(),
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
      started_at : SystemTime::now(),
  };

  self.sessions.insert( session_id.to_string(), context );
  self.current_session = session_id.to_string();
  
  // Safe unwrap since we just inserted
  self.sessions.get( session_id ).unwrap()
  }

  /// Process user input and generate response using Pro models with fallback
  /// 
  /// # Errors
  /// 
  /// Returns error if session not found, API call fails, or response processing fails
  pub async fn process_input( 
  &mut self, 
  session_id : &str, 
  user_input : &str 
  ) -> Result< String, Box< dyn std::error::Error > >
  {
  // Get model and context with immutable borrow
  let ( model, style ) = {
      let context = self.sessions.get( session_id )
  .ok_or( "Session not found" )?;
      ( context.model.clone(), context.style )
  };

  // Try Pro models first using the Providers API
  let bot_response = if Self::is_pro_model( &model )
  {
      match self.try_pro_model_response( session_id, user_input, &model, style ).await
      {
  Ok( response ) => response,
  Err( e ) =>
  {
          println!( "⚠️  Pro model {model} failed : {e}" );
          println!( "🔄 Falling back to BART model..." );
          
          // Update model to fallback model
          if let Some( context ) = self.sessions.get_mut( session_id )
          {
      context.model = "facebook/bart-large-cnn".to_string();
          }
          
          self.try_fallback_model_response( session_id, user_input ).await?
  }
      }
  }
  else
  {
      // Use fallback model directly
      self.try_fallback_model_response( session_id, user_input ).await?
  };

  // Update conversation history with mutable borrow
  let context = self.sessions.get_mut( session_id )
      .ok_or( "Session not found" )?;
  
  context.history.push( ( user_input.to_string(), bot_response.clone() ) );

  // Keep only last 5 exchanges to manage context length
  if context.history.len() > 5
  {
      context.history.remove( 0 );
  }

  Ok( bot_response )
  }

  /// Try to get response using Pro models through Providers API
  async fn try_pro_model_response(
  &self,
  session_id : &str,
  user_input : &str,
  model : &str,
  style : ConversationStyle,
  ) -> Result< String, Box< dyn std::error::Error > >
  {
  let providers = self.client.providers();
  
  // Build conversation messages for chat completion format
  let messages = self.build_chat_messages( session_id, user_input, style )?;
  
  // Set parameters based on conversation style
  let ( max_tokens, temperature, top_p ) = match style
  {
      ConversationStyle::Creative => ( Some( 200 ), Some( 0.9 ), Some( 0.95 ) ),
      ConversationStyle::Formal => ( Some( 150 ), Some( 0.3 ), Some( 0.85 ) ),
      ConversationStyle::Technical => ( Some( 250 ), Some( 0.2 ), Some( 0.9 ) ),
      ConversationStyle::Supportive => ( Some( 180 ), Some( 0.7 ), Some( 0.9 ) ),
      ConversationStyle::Casual => ( Some( 150 ), Some( 0.7 ), Some( 0.9 ) ),
  };

  let response = if user_input.contains( '=' ) || user_input.contains( '*' ) || user_input.contains( '/' ) || user_input.contains( '+' ) || user_input.contains( '-' )
  {
      // Use specialized math completion for mathematical queries
      providers.math_completion( model, user_input ).await?
  }
  else
  {
      // Use regular chat completion
      providers.chat_completion( model, messages, max_tokens, temperature, top_p ).await?
  };

  if let Some( choice ) = response.choices.first()
  {
      let content = choice.message.content.trim().to_string();
      if content.is_empty()
      {
  Err( "Empty response from Pro model".into() )
      }
      else
      {
  Ok( Self::clean_response( &content, user_input ) )
      }
  }
  else
  {
      Err( "No response choices from Pro model".into() )
  }
  }

  /// Try fallback model (BART) with special prompting
  async fn try_fallback_model_response(
  &self,
  session_id : &str,
  user_input : &str,
  ) -> Result< String, Box< dyn std::error::Error > >
  {
  let context = self.sessions.get( session_id )
      .ok_or( "Session not found" )?;
  
  let prompt = Self::build_contextual_prompt( context, user_input );
  
  let response = self.client
      .inference()
      .create( &prompt, "facebook/bart-large-cnn" )
      .await?;

  let bot_response = response.extract_text_or_default( "Sorry, I couldn't generate a response." );
  
  Ok( Self::clean_response( &bot_response, user_input ) )
  }

  /// Build chat messages for the Providers API format
  fn build_chat_messages( 
  &self, 
  session_id : &str, 
  user_input : &str, 
  style : ConversationStyle 
  ) -> Result< Vec< ChatMessage >, Box< dyn std::error::Error > >
  {
  let context = self.sessions.get( session_id )
      .ok_or( "Session not found" )?;

  let mut messages = Vec::new();

  // System message based on style
  let system_content = match style
  {
      ConversationStyle::Casual => "You are a helpful, friendly AI assistant. Be conversational and casual in your responses.",
      ConversationStyle::Formal => "You are a professional AI assistant. Respond formally and precisely with accurate information.",
      ConversationStyle::Creative => "You are a creative AI assistant. Be imaginative, expressive, and think outside the box.",
      ConversationStyle::Technical => "You are a technical AI assistant. Provide detailed, accurate, and technically precise information.",
      ConversationStyle::Supportive => "You are a supportive AI assistant. Be empathetic, encouraging, and helpful.",
  };

  messages.push( ChatMessage
  {
      role : "system".to_string(),
      content : system_content.to_string(),
  } );

  // Add conversation history
  for ( user_msg, bot_msg ) in &context.history
  {
      messages.push( ChatMessage
      {
  role : "user".to_string(),
  content : user_msg.clone(),
      } );
      
      messages.push( ChatMessage
      {
  role : "assistant".to_string(),
  content : bot_msg.clone(),
      } );
  }

  // Add current user input
  messages.push( ChatMessage
  {
      role : "user".to_string(),
      content : user_input.to_string(),
  } );

  Ok( messages )
  }

  /// Check if a model is a Pro model that should use Providers API
  fn is_pro_model( model : &str ) -> bool
  {
  model.contains( "meta-llama" ) || 
  model.contains( "Llama" ) ||
  model.contains( "mistralai" ) ||
  model.contains( "Mistral" ) ||
  model.contains( "codellama" ) ||
  model.contains( "CodeLlama" ) ||
  ( !model.contains( "bart" ) && !model.contains( "flan" ) && !model.contains( "t5" ) )
  }

  /// Clean up AI response by removing prompt echoes
  fn clean_response( response : &str, user_input : &str ) -> String
  {
  let response = response.trim();
  
  // Remove common prompt echoes
  let prefixes_to_remove = [
      &format!( "User : {user_input}\nAssistant:" ),
      &format!( "User : {user_input}" ),
      "Assistant:",
      "AI:",
  ];
  
  let mut cleaned = response.to_string();
  for prefix in &prefixes_to_remove
  {
      if cleaned.starts_with( prefix )
      {
  cleaned = cleaned[ prefix.len().. ].trim().to_string();
      }
  }
  
  // Remove excessive newlines
  while cleaned.contains( "\n\n\n" )
  {
      cleaned = cleaned.replace( "\n\n\n", "\n\n" );
  }
  
  cleaned.trim().to_string()
  }

  /// Build contextual prompt from conversation history
  /// For BART (summarization model), we frame the task differently to get conversational output
  fn build_contextual_prompt( context : &ConversationContext, user_input : &str ) -> String
  {
  // Special prompting strategy for BART - frame as summarization of helpful response
  if context.model.contains( "bart" )
  {
      let style_desc = match context.style
      {
  ConversationStyle::Casual => "friendly and casual manner",
  ConversationStyle::Formal => "professional and formal manner", 
  ConversationStyle::Creative => "creative and expressive manner",
  ConversationStyle::Technical => "detailed technical manner",
  ConversationStyle::Supportive => "supportive and encouraging manner",
      };
      
      format!(
  "Question : {user_input}\n\nAnswer : A {style_desc} AI assistant responds with helpful information. \
         Summary of the response:"
      )
  }
  else if context.model.contains( "llama" ) || context.model.contains( "Llama" )
  {
      // Llama-specific prompting with proper chat format
      let system_msg = match context.style
      {
  ConversationStyle::Casual => "You are a helpful, friendly AI assistant. Be conversational and casual.",
  ConversationStyle::Formal => "You are a professional AI assistant. Respond formally and precisely.",
  ConversationStyle::Creative => "You are a creative AI assistant. Be imaginative and expressive.",
  ConversationStyle::Technical => "You are a technical AI assistant. Provide detailed, accurate information.",
  ConversationStyle::Supportive => "You are a supportive AI assistant. Be empathetic and encouraging.",
      };

      let mut prompt = format!( "< s >[INST] << SYS > >\n{system_msg}\n<</SYS > >\n\n" );

      // Add conversation history with conversational labels
      for ( user_msg, bot_msg ) in &context.history
      {
  write!( &mut prompt, "User : {user_msg} [/INST] Assistant : {bot_msg} </s >< s >[INST] " ).unwrap();
      }

      write!( &mut prompt, "User : {user_input} [/INST] Assistant : " ).unwrap();
      prompt
  }
  else
  {
      // Standard prompting for other models
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
  write!( &mut prompt, "User : {user_msg}\nAssistant : {bot_msg}\n\n" ).unwrap();
      }

      write!( &mut prompt, "User : {user_input}\nAssistant : " ).unwrap();
      prompt
  }
  }

  /// Get current session context
  #[ must_use ]
  pub fn get_current_context( &self ) -> Option< &ConversationContext >
  {
  self.sessions.get( &self.current_session )
  }

  /// Change conversation style
  /// 
  /// # Errors
  /// 
  /// Returns error if no active session exists
  pub fn change_style( &mut self, style : ConversationStyle ) -> Result< (), &'static str >
  {
  let session_id = self.current_session.clone();
  let context = self.sessions.get_mut( &session_id )
      .ok_or( "No active session" )?;
      
  context.style = style;
  
  // Update model and parameters for new style
  context.model = match style
  {
      ConversationStyle::Technical => Models::mistral_7b_instruct().to_string(),
      _ => Models::llama_3_3_70b_instruct().to_string(),
  };
  
  context.parameters = match style
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
  };
  
  Ok( () )
  }

  /// Change AI model
  /// 
  /// # Errors
  /// 
  /// Returns error if no active session exists
  pub fn change_model( &mut self, model : &str ) -> Result< (), &'static str >
  {
  let session_id = self.current_session.clone();
  let context = self.sessions.get_mut( &session_id )
      .ok_or( "No active session" )?;
      
  context.model = model.to_string();
  Ok( () )
  }

  /// Clear conversation history
  /// 
  /// # Errors
  /// 
  /// Returns error if no active session exists
  pub fn clear_history( &mut self ) -> Result< (), &'static str >
  {
  let session_id = self.current_session.clone();
  let context = self.sessions.get_mut( &session_id )
      .ok_or( "No active session" )?;
      
  context.history.clear();
  Ok( () )
  }

  /// Export conversation to file
  /// 
  /// # Errors
  /// 
  /// Returns error if no active session, no history, or file write fails
  pub fn export_conversation( &self ) -> Result< String, Box< dyn std::error::Error > >
  {
  let context = self.get_current_context()
      .ok_or( "No active session" )?;
      
  let timestamp = SystemTime::now()
      .duration_since( UNIX_EPOCH )?
      .as_secs();
  let filename = format!( "conversation_{}_{}.txt", context.session_id, timestamp );
  
  let mut export_content = format!(
      "Chatbot Conversation Export\n\
       Session ID: {}\n\
       Style : {:?}\n\
       Model : {}\n\
       Started : {:?}\n\
       Exported : {:?}\n\
       \n\
       === Conversation History ===\n\n",
      context.session_id,
      context.style,
      context.model,
      context.started_at,
      SystemTime::now()
  );
  
  for ( i, ( user_msg, bot_msg ) ) in context.history.iter().enumerate()
  {
      write!( &mut export_content, "Turn {}:\nUser : {}\nAssistant : {}\n\n", i + 1, user_msg, bot_msg )?;
  }
  
  fs::write( &filename, export_content )?;
  Ok( filename )
  }
}

/// Interactive chatbot interface
#[ derive( Debug ) ]
pub struct InteractiveChatbot
{
  system : ChatbotSystem,
}

impl InteractiveChatbot
{
  /// Create new interactive chatbot
  #[ must_use ]
  pub fn new( client : Client< HuggingFaceEnvironmentImpl > ) -> Self
  {
  Self
  {
      system : ChatbotSystem::new( client ),
  }
  }


  /// Start interactive chat session
  /// 
  /// # Errors
  /// 
  /// Returns error if I/O operations fail or API calls fail
  pub async fn start( &mut self ) -> Result< (), Box< dyn std::error::Error > >
  {
  println!( "🤖 AI Chatbot - HuggingFace Edition (Pro Models Support)" );
  println!( "========================================================" );
  println!( "🎯 Automatically uses Pro plan models (Llama-3, Mistral, CodeLlama) when available" );
  println!( "🔄 Falls back to BART model for free tier users" );
  println!( "📚 Type '/help' for commands or start chatting!" );
  println!();

  // Start default conversation
  self.system.start_conversation( &self.system.current_session.clone(), ConversationStyle::Casual );
  
  if let Some( context ) = self.system.get_current_context()
  {
      println!( "Session : {} | Style : {:?} | Model : {}", 
  context.session_id, context.style, context.model );
  }
  println!();

  let stdin = io::stdin();
  let mut stdout = io::stdout();

  loop
  {
      print!( "You : " );
      stdout.flush()?;

      let mut input = String::new();
      let bytes_read = stdin.read_line( &mut input )?;
      
      // If we read 0 bytes, stdin is closed (EOF reached)
      if bytes_read == 0
      {
  println!("Goodbye!");
  break Ok(());
      }
      
      let input = input.trim();
      if input.is_empty()
      {
  continue;
      }

      // Handle commands
      if input.starts_with( '/' )
      {
  match self.handle_command( input )
  {
          Ok( Some( response ) ) => println!( "🤖 {response}" ),
          Ok( None ) => {}, // Command handled without output
          Err( e ) => println!( "❌ Error : {e}" ),
  }
  continue;
      }

      // Process regular chat input
      print!( "🤖 " );
      stdout.flush()?;
      
      match self.system.process_input( &self.system.current_session.clone(), input ).await
      {
  Ok( response ) => println!( "{response}" ),
  Err( e ) => println!( "❌ Sorry, I encountered an error : {e}" ),
      }
      
      println!();
  }
  }


  /// Handle chat commands
  fn handle_command( &mut self, command : &str ) -> Result< Option< String >, Box< dyn std::error::Error > >
  {
  let parts : Vec< &str > = command[ 1.. ].split_whitespace().collect();
  
  if parts.is_empty()
  {
      return Ok( Some( "Invalid command. Type '/help' for available commands.".to_string() ) );
  }

  match parts[ 0 ].to_lowercase().as_str()
  {
      "help" => Ok( Some( Self::show_help() ) ),
      
      "quit" | "exit" => 
      {
  println!( "👋 Goodbye!" );
  std::process::exit( 0 );
      },
      
      "style" =>
      {
  if parts.len() < 2
  {
          return Ok( Some( "Usage : /style < casual|formal|creative|technical|supportive >".to_string() ) );
  }
  
  match ConversationStyle::from_str( parts[ 1 ] )
  {
          Some( style ) =>
          {
      self.system.change_style( style )?;
      Ok( Some( format!( "Changed to {} style : {}", parts[ 1 ], style.description() ) ) )
          },
          None => Ok( Some( "Invalid style. Use : casual, formal, creative, technical, or supportive".to_string() ) ),
  }
      },
      
      "model" =>
      {
  if parts.len() < 2
  {
          return Ok( Some( "Usage : /model < model-name >".to_string() ) );
  }
  
  let model = parts[ 1.. ].join( " " );
  self.system.change_model( &model )?;
  Ok( Some( format!( "Changed to model : {model}" ) ) )
      },
      
      "history" =>
      {
  match self.system.get_current_context()
  {
          Some( context ) if !context.history.is_empty() =>
          {
      let mut history = String::from( "Conversation History:\n" );
      for ( i, ( user_msg, bot_msg ) ) in context.history.iter().enumerate()
      {
              write!( &mut history, "\n{}. You : {}\n   Bot : {}\n", i + 1, user_msg, bot_msg )?;
      }
      Ok( Some( history ) )
          },
          Some( _ ) => Ok( Some( "No conversation history yet.".to_string() ) ),
          None => Ok( Some( "No active session.".to_string() ) ),
  }
      },
      
      "clear" =>
      {
  self.system.clear_history()?;
  Ok( Some( "Conversation history cleared.".to_string() ) )
      },
      
      "export" =>
      {
  match self.system.export_conversation()
  {
          Ok( filename ) => Ok( Some( format!( "Conversation exported to : {filename}" ) ) ),
          Err( e ) => Ok( Some( format!( "Export failed : {e}" ) ) ),
  }
      },
      
      "status" =>
      {
  match self.system.get_current_context()
  {
          Some( context ) =>
          {
      let duration = SystemTime::now().duration_since( context.started_at )?;
      Ok( Some( format!(
              "Session : {}\nStyle : {:?} ({})\nModel : {}\nHistory : {} turns\nDuration : {}m {}s",
              context.session_id,
              context.style,
              context.style.description(),
              context.model,
              context.history.len(),
              duration.as_secs() / 60,
              duration.as_secs() % 60
      ) ) )
          },
          None => Ok( Some( "No active session.".to_string() ) ),
  }
      },
      
      _ => Ok( Some( format!( "Unknown command : /{}\nType '/help' for available commands.", parts[ 0 ] ) ) ),
  }
  }

  /// Show help information
  fn show_help() -> String
  {
  r"Available Commands:
===================

/style < type >     - Change conversation style
          Options : casual, formal, creative, technical, supportive
/model < name >     - Change AI model (Pro : Llama-3, Mistral, CodeLlama)
/history          - Show conversation history
/clear            - Clear conversation history
/export           - Export conversation to file
/status           - Show session information
/help             - Show this help message
/quit or /exit    - Exit the chatbot

Pro Model Features:
==================

🎯 Intelligent Model Selection:
• Pro plan users get access to advanced models (Llama-3, Mistral, CodeLlama)
• Automatic fallback to BART for free tier users
• Math questions use specialized math completion

🤖 Advanced Capabilities:
• Proper conversational AI (not summarization)
• Mathematical reasoning and calculations
• Context-aware responses
• Multi-turn conversation memory

Conversation Styles:
===================

• casual     - Friendly, relaxed conversation (Llama-3-8B)
• formal     - Professional, precise responses (Llama-3-8B)
• creative   - Imaginative, expressive replies (Llama-3-8B)
• technical  - Detailed, technical information (CodeLlama-7b)
• supportive - Empathetic, encouraging responses (Llama-3-8B)

Simply type your message to start chatting!
Try math questions like : 'x=13, what is x*3?' for better results with Pro models!".to_string()
  }
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  // Load API key from environment or workspace secrets using workspace_tools
  let api_key = std::env::var("HUGGINGFACE_API_KEY")
  .or_else(|_| {
      use workspace_tools as workspace;
      let workspace = workspace::workspace()
  .map_err(|_| std::env::VarError::NotPresent)?;
      
      // Load project-specific secrets using workspace_tools properly  
      let secrets = workspace.load_secrets_from_file("-secrets.sh")
  .map_err(|_| std::env::VarError::NotPresent)?;
  
      secrets.get("HUGGINGFACE_API_KEY")
  .cloned()
  .ok_or(std::env::VarError::NotPresent)
  })
  .map_err(|_| "HUGGINGFACE_API_KEY not found in environment or workspace secrets (./secret/-secrets.sh or -secrets.sh)")?;

  // Build client
  let secret_key = Secret::new( api_key );
  let environment = HuggingFaceEnvironmentImpl::build( secret_key, None )?;
  let client = Client::build( environment )?;

  // Start interactive chatbot
  let mut chatbot = InteractiveChatbot::new( client );
  chatbot.start().await?;

  Ok( () )
}