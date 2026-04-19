//! Interactive `OpenAI` Chat with Caching
//!
//! This example demonstrates an advanced interactive chat application that showcases
//! many features of the `OpenAI` API client including:
//! - Persistent conversation sessions
//! - Intelligent response caching
//! - Context window management
//! - Rich formatting and syntax highlighting
//! - Export/import functionality
//! - Multi-session support
//!
//! Usage:
//! ```bash
//! # Basic usage
//! cargo run --example openai_cached_interactive_chat
//!
//! # With specific model and system prompt
//! cargo run --example openai_cached_interactive_chat -- \
//!   --model gpt-4-turbo \
//!   --system "You are a helpful coding assistant"
//!
//! # Load existing session
//! cargo run --example openai_cached_interactive_chat -- \
//!   --session "my-coding-session"
//!
//! # Export conversation
//! cargo run --example openai_cached_interactive_chat -- \
//!   --export markdown
//! ```

#![ allow( missing_docs, missing_debug_implementations ) ]
#![allow(clippy::missing_inline_in_public_items)]

use api_openai::ClientApiAccessors;
use api_openai::{
  Client,
  components ::{
    responses ::{ CreateResponseRequest, ResponseInput, ResponseObject },
    input ::{ InputItem, InputMessage, InputContentPart, InputText },
    output ::{ OutputItem, OutputContentPart },
    common ::ModelIdsResponses,
  },
};
use serde::{ Deserialize, Serialize };
use std::{
  collections ::HashMap,
  env,
  fs,
  io ::{ self, Write },
  path ::PathBuf,
};
use core::{
  fmt ::Write as FmtWrite,
  time ::Duration,
};

/// Chat session data structure containing conversation history and metadata
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ChatSession
{
  /// Unique identifier for the session
  pub id : String,
  /// Human-readable name for the session
  pub name : String,
  /// Conversation messages
  pub messages : Vec< Message >,
  /// `OpenAI` model being used
  pub model : String,
  /// Optional system prompt
  pub system_prompt : Option< String >,
  /// Session creation timestamp
  pub created_at : String,
  /// Last update timestamp
  pub last_updated : String,
}

impl ChatSession
{
  /// Create a new chat session
  ///
  /// # Panics
  /// Panics if the system time is before the Unix epoch.
  #[ must_use ]
  pub fn new( name : String, model : String, system_prompt : Option< String > ) -> Self
  {
    let now = std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .unwrap()
      .as_secs()
      .to_string();
    Self
    {
      id : uuid::Uuid::new_v4().to_string(),
      name,
      messages : Vec::new(),
      model,
      system_prompt,
      created_at : now.clone(),
      last_updated : now,
    }
  }

  /// Add a message to the session
  ///
  /// # Panics
  /// Panics if the system time is before the Unix epoch.
  pub fn add_message( &mut self, message : Message )
  {
    self.messages.push( message );
    self.last_updated = std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .unwrap()
      .as_secs()
      .to_string();
  }

  /// Calculate total token count (estimated)
  #[ must_use ]
  pub fn estimate_total_tokens( &self ) -> u32
  {
    self.messages.iter()
      .map( | m | m.token_count.unwrap_or_else( || Self::estimate_tokens( &m.content ) ) )
      .sum()
  }

  /// Simple token estimation (rough approximation)
  fn estimate_tokens( text : &str ) -> u32
  {
    // Rough approximation : 1 token per 4 characters
    // Use safe conversion to avoid clippy warnings
    let char_count = text.len();
    if char_count == 0
    {
      return 0;
    }
    // Safely handle potential overflow and casting
    let char_count_u32 = u32::try_from( char_count ).unwrap_or( u32::MAX );
    let tokens = f64::from( char_count_u32 ) / 4.0;
    #[ allow(clippy::cast_possible_truncation, clippy::cast_sign_loss) ]
    let result = tokens.ceil() as u32;
    result
  }

  /// Clear conversation history
  ///
  /// # Panics
  /// Panics if the system time is before the Unix epoch.
  pub fn clear_history( &mut self )
  {
    self.messages.clear();
    self.last_updated = std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .unwrap()
      .as_secs()
      .to_string();
  }
}

/// Individual message in a conversation
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct Message
{
  /// Message role (user, assistant, system)
  pub role : String,
  /// Message content
  pub content : String,
  /// Message timestamp
  pub timestamp : String,
  /// Token count for this message
  pub token_count : Option< u32 >,
  /// Whether response was served from cache
  pub cached : bool,
}

impl Message
{
  /// Create a new message
  ///
  /// # Panics
  /// Panics if the system time is before the Unix epoch.
  #[ must_use ]
  pub fn new( role : String, content : String, cached : bool ) -> Self
  {
    Self
    {
      role,
      content,
      timestamp : std::time::SystemTime::now()
        .duration_since( std::time::UNIX_EPOCH )
        .unwrap()
        .as_secs()
        .to_string(),
      token_count : None,
      cached,
    }
  }

  /// Create user message
  #[ must_use ]
  pub fn user( content : String ) -> Self
  {
    Self::new( "user".to_string(), content, false )
  }

  /// Create assistant message
  #[ must_use ]
  pub fn assistant( content : String, cached : bool ) -> Self
  {
    Self::new( "assistant".to_string(), content, cached )
  }

  /// Create system message
  #[ must_use ]
  pub fn system( content : String ) -> Self
  {
    Self::new( "system".to_string(), content, false )
  }
}

/// Response caching system
pub struct ChatCache
{
  /// In-memory cache of responses
  cache : HashMap<  String, String  >,
  /// File system storage path
  storage_path : PathBuf,
}

impl ChatCache
{
  /// Create new cache with storage path
  ///
  /// # Errors
  /// Returns an error if cache directory creation fails or cache loading fails.
  pub fn new( storage_path : PathBuf ) -> Result< Self, Box< dyn std::error::Error > >
  {
    // Ensure cache directory exists
    if let Some( parent ) = storage_path.parent()
    {
      fs ::create_dir_all( parent )?;
    }

    let mut cache = Self
    {
      cache : HashMap::new(),
      storage_path,
    };

    // Load existing cache from disk
    cache.load_from_disk()?;
    Ok( cache )
  }

  /// Generate cache key from conversation context
  #[ must_use ]
  pub fn generate_key( &self, messages : &[ Message ], model : &str ) -> String
  {
    use std::collections::hash_map::DefaultHasher;
    use core::hash::{ Hash, Hasher };

    let mut hasher = DefaultHasher::new();
    model.hash( &mut hasher );

    // Hash last few messages for context
    for message in messages.iter().rev().take( 5 )
    {
      message.role.hash( &mut hasher );
      message.content.hash( &mut hasher );
    }

    format!( "{:x}", hasher.finish() )
  }

  /// Get cached response
  #[ must_use ]
  pub fn get( &self, key : &str ) -> Option< String >
  {
    self.cache.get( key ).cloned()
  }

  /// Store response in cache
  ///
  /// # Errors
  /// Returns an error if cache persistence to disk fails.
  pub fn insert( &mut self, key : String, response : String ) -> Result< (), Box< dyn std::error::Error > >
  {
    self.cache.insert( key, response );
    self.save_to_disk()?;
    Ok( () )
  }

  /// Load cache from disk
  fn load_from_disk( &mut self ) -> Result< (), Box< dyn std::error::Error > >
  {
    if self.storage_path.exists()
    {
      let content = fs::read_to_string( &self.storage_path )?;
      self.cache = serde_json::from_str( &content ).unwrap_or_default();
    }
    Ok( () )
  }

  /// Save cache to disk
  fn save_to_disk( &self ) -> Result< (), Box< dyn std::error::Error > >
  {
    let content = serde_json::to_string_pretty( &self.cache )?;
    fs ::write( &self.storage_path, content )?;
    Ok( () )
  }

  /// Clear all cached entries
  ///
  /// # Errors
  /// Returns an error if cache persistence to disk fails.
  pub fn clear( &mut self ) -> Result< (), Box< dyn std::error::Error > >
  {
    self.cache.clear();
    self.save_to_disk()?;
    Ok( () )
  }
}

/// Chat application configuration
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ChatConfig
{
  /// Default model to use
  pub default_model : String,
  /// Maximum context tokens before truncation
  pub max_context_tokens : u32,
  /// Whether caching is enabled
  pub cache_enabled : bool,
  /// Cache time-to-live
  pub cache_ttl : Duration,
  /// Auto-save sessions
  pub auto_save : bool,
  /// Enable syntax highlighting
  pub syntax_highlighting : bool,
  /// Directory for session storage
  pub session_directory : PathBuf,
}

impl Default for ChatConfig
{
  fn default() -> Self
  {
    Self
    {
      default_model : "gpt-4".to_string(),
      max_context_tokens : 4000,
      cache_enabled : true,
      cache_ttl : Duration::from_secs( 3600 ), // 1 hour
      auto_save : true,
      syntax_highlighting : true,
      session_directory : PathBuf::from( ".chat_sessions" ),
    }
  }
}

/// Available export formats
#[ derive( Debug, Clone, Copy ) ]
pub enum ExportFormat
{
  Json,
  Markdown,
  Plain,
}

impl core::str::FromStr for ExportFormat
{
  type Err = String;

  fn from_str( s : &str ) -> Result< Self, Self::Err >
  {
    match s.to_lowercase().as_str()
    {
      "json" => Ok( ExportFormat::Json ),
      "markdown" => Ok( ExportFormat::Markdown ),
      "plain" => Ok( ExportFormat::Plain ),
      _ => Err( format!( "Unknown export format : {s}" ) ),
    }
  }
}

/// Chat commands that users can execute
#[ derive( Debug, Clone ) ]
pub enum ChatCommand
{
  Help,
  NewSession( Option< String > ),
  LoadSession( String ),
  ListSessions,
  SaveSession,
  ExportSession( ExportFormat, PathBuf ),
  SetModel( String ),
  SetSystemPrompt( String ),
  ClearHistory,
  ShowStats,
  ToggleCache,
  SetMaxTokens( u32 ),
  Quit,
}

impl ChatCommand
{
  /// Parse command from user input
  ///
  /// # Errors
  /// Returns an error if the input is empty or contains an invalid command.
  pub fn parse( input : &str ) -> Result< Self, String >
  {
    let parts : Vec< &str > = input.split_whitespace().collect();
    if parts.is_empty()
    {
      return Err( "Empty command".to_string() );
    }

    match parts[ 0 ]
    {
      "/help" | "/h" => Ok( ChatCommand::Help ),
      "/new" => Ok( ChatCommand::NewSession( parts.get( 1 ).map( | s | (*s).to_string() ) ) ),
      "/load" =>
      {
        if parts.len() < 2
        {
          Err( "Usage : /load < session_name >".to_string() )
        }
        else
        {
          Ok( ChatCommand::LoadSession( parts[ 1 ].to_string() ) )
        }
      },
      "/sessions" | "/list" => Ok( ChatCommand::ListSessions ),
      "/save" => Ok( ChatCommand::SaveSession ),
      "/export" =>
      {
        if parts.len() < 2
        {
          Err( "Usage : /export < format > [filename]".to_string() )
        }
        else
        {
          let format = parts[ 1 ].parse::< ExportFormat >()
            .map_err( | e | format!( "Invalid format : {e}" ) )?;
          let filename = parts.get( 2 ).map_or_else(|| PathBuf::from( format!( "conversation.{}", parts[ 1 ] ) ), PathBuf::from);
          Ok( ChatCommand::ExportSession( format, filename ) )
        }
      },
      "/model" =>
      {
        if parts.len() < 2
        {
          Err( "Usage : /model < model_name >".to_string() )
        }
        else
        {
          Ok( ChatCommand::SetModel( parts[ 1 ].to_string() ) )
        }
      },
      "/system" =>
      {
        if parts.len() < 2
        {
          Err( "Usage : /system < prompt >".to_string() )
        }
        else
        {
          let prompt = parts[ 1.. ].join( " " );
          Ok( ChatCommand::SetSystemPrompt( prompt ) )
        }
      },
      "/clear" => Ok( ChatCommand::ClearHistory ),
      "/stats" => Ok( ChatCommand::ShowStats ),
      "/cache" => Ok( ChatCommand::ToggleCache ),
      "/tokens" =>
      {
        if parts.len() < 2
        {
          Err( "Usage : /tokens < number >".to_string() )
        }
        else
        {
          let tokens = parts[ 1 ].parse::< u32 >()
            .map_err( | _ | "Invalid token count".to_string() )?;
          Ok( ChatCommand::SetMaxTokens( tokens ) )
        }
      },
      "/quit" | "/exit" | "/q" => Ok( ChatCommand::Quit ),
      _ => Err( format!( "Unknown command : {}. Type /help for available commands.", parts[ 0 ] ) ),
    }
  }
}

/// User input types
#[ derive( Debug ) ]
pub enum UserInput
{
  Message( String ),
  Command( ChatCommand ),
  Exit,
}

/// Output formatter for rich text display
pub struct OutputFormatter
{
  colors_enabled : bool,
}

impl OutputFormatter
{
  /// Create new formatter
  #[ must_use ]
  pub fn new( colors_enabled : bool ) -> Self
  {
    Self { colors_enabled }
  }

  /// Format assistant message
  #[ must_use ]
  pub fn format_assistant_message( &self, content : &str, cached : bool ) -> String
  {
    if self.colors_enabled
    {
      let cache_indicator = if cached { " 🔄" } else { "" };
      format!( "\x1b[36m🤖 Assistant{cache_indicator}\x1b[0m : {content}" )
    }
    else
    {
      let cache_indicator = if cached { " (cached)" } else { "" };
      format!( "Assistant{cache_indicator}: {content}" )
    }
  }

  /// Format user message
  #[ must_use ]
  pub fn format_user_message( &self, content : &str ) -> String
  {
    if self.colors_enabled
    {
      format!( "\x1b[32m👤 You\x1b[0m : {content}" )
    }
    else
    {
      format!( "You : {content}" )
    }
  }

  /// Format system message
  #[ must_use ]
  pub fn format_system_message( &self, content : &str ) -> String
  {
    if self.colors_enabled
    {
      format!( "\x1b[33mℹ️  System\x1b[0m : {content}" )
    }
    else
    {
      format!( "System : {content}" )
    }
  }

  /// Format error message
  #[ must_use ]
  pub fn format_error( &self, content : &str ) -> String
  {
    if self.colors_enabled
    {
      format!( "\x1b[31m❌ Error\x1b[0m : {content}" )
    }
    else
    {
      format!( "Error : {content}" )
    }
  }

  /// Format info message
  #[ must_use ]
  pub fn format_info( &self, content : &str ) -> String
  {
    if self.colors_enabled
    {
      format!( "\x1b[34mℹ️  \x1b[0m{content}" )
    }
    else
    {
      content.to_string()
    }
  }
}

/// Main interactive chat application
pub struct InteractiveChatApp
{
  client : Client< api_openai::environment::OpenaiEnvironmentImpl >,
  cache : ChatCache,
  current_session : ChatSession,
  config : ChatConfig,
  formatter : OutputFormatter,
}

impl InteractiveChatApp
{
  /// Create new interactive chat application
  ///
  /// # Errors
  /// Returns an error if cache directory creation fails or cache initialization fails.
  pub fn new(
    client : Client< api_openai::environment::OpenaiEnvironmentImpl >,
    config : ChatConfig,
    session_name : Option< String >
  ) -> Result< Self, Box< dyn std::error::Error > >
  {
    // Create cache directory
    fs ::create_dir_all( &config.session_directory )?;

    let cache_path = config.session_directory.join( "cache.json" );
    let cache = ChatCache::new( cache_path )?;

    let session_name = session_name.unwrap_or_else( || "default".to_string() );
    let current_session = ChatSession::new(
      session_name,
      config.default_model.clone(),
      None
    );

    let formatter = OutputFormatter::new( config.syntax_highlighting );

    Ok( Self
    {
      client,
      cache,
      current_session,
      config,
      formatter,
    } )
  }

  /// Main application loop
  ///
  /// # Errors
  /// Returns an error if user input reading fails, message processing fails, or session saving fails.
  pub async fn run( &mut self ) -> Result< (), Box< dyn std::error::Error > >
  {
    self.display_welcome();

    loop
    {
      match self.read_user_input()?
      {
        UserInput::Message( content ) =>
        {
          if let Err( e ) = self.process_message( content ).await
          {
            println!( "{}", self.formatter.format_error( &e.to_string() ) );
          }
        },
        UserInput::Command( cmd ) =>
        {
          if !self.handle_command( cmd ).await?
          {
            break;
          }
        },
        UserInput::Exit => break,
      }
    }

    if self.config.auto_save
    {
      self.save_session()?;
    }

    println!( "{}", self.formatter.format_info( "Goodbye! 👋" ) );
    Ok( () )
  }

  /// Display welcome message
  fn display_welcome( &self )
  {
    println!( "{}", self.formatter.format_info( "🚀 Welcome to OpenAI Interactive Chat!" ) );
    println!( "{}", self.formatter.format_info( &format!( "Session : {} | Model : {}", self.current_session.name, self.current_session.model ) ) );
    println!( "{}", self.formatter.format_info( "Type your message or use /help for commands." ) );
    println!();
  }

  /// Read user input from stdin
  fn read_user_input( &self ) -> Result< UserInput, Box< dyn std::error::Error > >
  {
    print!( "> " );
    io ::stdout().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();

    stdin.read_line( &mut line )?;
    let input = line.trim();

    if input.is_empty()
    {
      return Ok( UserInput::Exit );
    }

    if input.starts_with( '/' )
    {
      match ChatCommand::parse( input )
      {
        Ok( cmd ) => Ok( UserInput::Command( cmd ) ),
        Err( e ) =>
        {
          println!( "{}", self.formatter.format_error( &e ) );
          self.read_user_input()
        }
      }
    }
    else
    {
      Ok( UserInput::Message( input.to_string() ) )
    }
  }

  /// Process user message and get AI response
  async fn process_message( &mut self, content : String ) -> Result< (), Box< dyn std::error::Error > >
  {
    // Add user message to session
    let user_message = Message::user( content );
    println!( "{}", self.formatter.format_user_message( &user_message.content ) );
    self.current_session.add_message( user_message );

    // Check context window
    self.manage_context_window();

    // Try to get cached response
    let cache_key = self.cache.generate_key( &self.current_session.messages, &self.current_session.model );

    let ( response_content, from_cache ) = if self.config.cache_enabled
    {
      if let Some( cached_response ) = self.cache.get( &cache_key )
      {
        ( cached_response, true )
      }
      else
      {
        let response = self.get_ai_response().await?;
        self.cache.insert( cache_key, response.clone() )?;
        ( response, false )
      }
    }
    else
    {
      ( self.get_ai_response().await?, false )
    };

    // Add assistant response to session
    let assistant_message = Message::assistant( response_content.clone(), from_cache );
    self.current_session.add_message( assistant_message );

    // Display response
    println!( "{}", self.formatter.format_assistant_message( &response_content, from_cache ) );
    println!();

    Ok( () )
  }

  /// Get AI response from `OpenAI` API
  ///
  /// # Errors
  /// Returns an error if the API request fails or response parsing fails.
  async fn get_ai_response( &self ) -> Result< String, Box< dyn std::error::Error > >
  {
    // Convert session messages to API format
    let mut api_messages = Vec::new();

    // Add system prompt if set
    if let Some( ref system_prompt ) = self.current_session.system_prompt
    {
      api_messages.push( InputItem::Message(
        InputMessage::former()
          .role( "system" )
          .content( vec![
            InputContentPart::Text(
              InputText::former()
                .text( system_prompt.clone() )
                .form()
            )
          ] )
          .form()
      ) );
    }

    // Add conversation messages
    for message in &self.current_session.messages
    {
      let api_message = InputItem::Message(
        InputMessage::former()
          .role( message.role.clone() )
          .content( vec![
            InputContentPart::Text(
              InputText::former()
                .text( message.content.clone() )
                .form()
            )
          ] )
          .form()
      );
      api_messages.push( api_message );
    }

    // Create response request
    let request = CreateResponseRequest::former()
      .model( ModelIdsResponses::from( self.current_session.model.clone() ) )
      .input( ResponseInput::Items( api_messages ) )
      .temperature( 0.7 )
      .max_output_tokens( 2048 )
      .form();

    // Send request
    let response : ResponseObject = self.client.responses().create( request ).await?;

    // Extract response content
    if let Some( OutputItem::Message( message_struct ) ) = response.output.first()
    {
      if let Some( OutputContentPart::Text { text, .. } ) = message_struct.content.first()
      {
        Ok( text.clone() )
      }
      else
      {
        Err( "No text content found in response".into() )
      }
    }
    else
    {
      Err( "No message output received in response".into() )
    }
  }

  /// Handle chat commands
  async fn handle_command( &mut self, command : ChatCommand ) -> Result< bool, Box< dyn std::error::Error > >
  {
    match command
    {
      ChatCommand::Help => self.show_help(),
      ChatCommand::NewSession( name ) => self.new_session( name ).await?,
      ChatCommand::LoadSession( name ) => self.load_session( name ).await?,
      ChatCommand::ListSessions => self.list_sessions()?,
      ChatCommand::SaveSession => self.save_session()?,
      ChatCommand::ExportSession( format, path ) => self.export_session( format, &path )?,
      ChatCommand::SetModel( model ) => self.set_model( &model ),
      ChatCommand::SetSystemPrompt( prompt ) => self.set_system_prompt( &prompt ),
      ChatCommand::ClearHistory => self.clear_history(),
      ChatCommand::ShowStats => self.show_stats(),
      ChatCommand::ToggleCache => self.toggle_cache(),
      ChatCommand::SetMaxTokens( tokens ) => self.set_max_tokens( tokens ),
      ChatCommand::Quit => return Ok( false ),
    }
    Ok( true )
  }

  /// Show help information
  fn show_help( &self )
  {
    let help_text = "
📋 Available Commands:

/help           - Show this help message
/new [name]     - Start new conversation session
/load < name >    - Load existing session
/sessions       - List all saved sessions
/save           - Save current session
/export < fmt >   - Export conversation (json/markdown/plain)
/model < name >   - Switch to different model
/system < text >  - Set system prompt
/clear          - Clear conversation history
/stats          - Show session statistics
/cache          - Toggle response caching
/tokens < num >   - Set max context tokens
/quit           - Exit application

💡 Tips:
- Messages are automatically saved when auto-save is enabled
- Cached responses are marked with a cache indicator
- Use /stats to monitor token usage
- Export conversations in different formats for sharing
";
    println!( "{}", self.formatter.format_info( help_text ) );
  }

  /// Create new session
  async fn new_session( &mut self, name : Option< String > ) -> Result< (), Box< dyn std::error::Error > >
  {
    tokio ::task::yield_now().await;
    // Save current session if auto-save is enabled
    if self.config.auto_save
    {
      self.save_session()?;
    }

    let session_name = name.unwrap_or_else( || format!( "session_{}",
      std ::time::SystemTime::now()
        .duration_since( std::time::UNIX_EPOCH )
        .unwrap()
        .as_secs() ) );
    self.current_session = ChatSession::new(
      session_name.clone(),
      self.config.default_model.clone(),
      self.current_session.system_prompt.clone()
    );

    println!( "{}", self.formatter.format_info( &format!( "Started new session : {session_name}" ) ) );
    Ok( () )
  }

  /// Load existing session
  async fn load_session( &mut self, name : String ) -> Result< (), Box< dyn std::error::Error > >
  {
    tokio ::task::yield_now().await;
    let session_path = self.config.session_directory.join( format!( "{name}.json" ) );

    if !session_path.exists()
    {
      return Err( format!( "Session '{name}' not found" ).into() );
    }

    // Save current session if auto-save is enabled
    if self.config.auto_save
    {
      self.save_session()?;
    }

    let content = fs::read_to_string( session_path )?;
    self.current_session = serde_json::from_str( &content )?;

    println!( "{}", self.formatter.format_info( &format!( "Loaded session : {name}" ) ) );
    println!( "{}", self.formatter.format_info( &format!( "Messages : {} | Model : {}",
      self.current_session.messages.len(), self.current_session.model ) ) );
    Ok( () )
  }

  /// List all saved sessions
  fn list_sessions( &self ) -> Result< (), Box< dyn std::error::Error > >
  {
    let sessions_dir = &self.config.session_directory;

    if !sessions_dir.exists()
    {
      println!( "{}", self.formatter.format_info( "No sessions directory found." ) );
      return Ok( () );
    }

    let mut sessions = Vec::new();
    for entry in fs::read_dir( sessions_dir )?
    {
      let entry = entry?;
      if let Some( name ) = entry.file_name().to_str()
      {
        if std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
          let session_name = name.trim_end_matches( ".json" );
          sessions.push( session_name.to_string() );
        }
      }
    }

    if sessions.is_empty()
    {
      println!( "{}", self.formatter.format_info( "No saved sessions found." ) );
    }
    else
    {
      println!( "{}", self.formatter.format_info( "📁 Saved Sessions:" ) );
      for session in sessions
      {
        let current_marker = if session == self.current_session.name { " (current)" } else { "" };
        println!( "  • {session}{current_marker}" );
      }
    }

    Ok( () )
  }

  /// Save current session
  fn save_session( &self ) -> Result< (), Box< dyn std::error::Error > >
  {
    let session_path = self.config.session_directory.join( format!( "{}.json", self.current_session.name ) );
    let content = serde_json::to_string_pretty( &self.current_session )?;
    fs ::write( session_path, content )?;

    println!( "{}", self.formatter.format_info( &format!( "Session '{}' saved.", self.current_session.name ) ) );
    Ok( () )
  }

  /// Export session in specified format
  fn export_session( &self, format : ExportFormat, path : &std::path::Path ) -> Result< (), Box< dyn std::error::Error > >
  {
    let content = match format
    {
      ExportFormat::Json => serde_json::to_string_pretty( &self.current_session )?,
      ExportFormat::Markdown => self.to_markdown(),
      ExportFormat::Plain => self.to_plain_text(),
    };

    fs ::write( path, content )?;
    println!( "{}", self.formatter.format_info( &format!( "Conversation exported to : {}", path.display() ) ) );
    Ok( () )
  }

  /// Convert session to markdown format
  fn to_markdown( &self ) -> String
  {
    let mut output = String::new();

    write!( &mut output, "# Chat Session : {}\n\n", self.current_session.name ).unwrap();
    writeln!( &mut output, "**Model**: {}", self.current_session.model ).unwrap();
    writeln!( &mut output, "**Created**: {}", self.current_session.created_at ).unwrap();
    write!( &mut output, "**Last Updated**: {}\n\n", self.current_session.last_updated ).unwrap();

    if let Some( ref system_prompt ) = self.current_session.system_prompt
    {
      write!( &mut output, "**System Prompt**: {system_prompt}\n\n" ).unwrap();
    }

    output.push_str( "---\n\n" );

    for message in &self.current_session.messages
    {
      let role_emoji = match message.role.as_str()
      {
        "user" => "👤",
        "assistant" => "🤖",
        "system" => "ℹ️",
        _ => "❓",
      };

      let cached_indicator = if message.cached { " 🔄" } else { "" };

      write!( &mut output, "## {} {}{}\n\n", role_emoji, message.role, cached_indicator ).unwrap();
      write!( &mut output, "{}\n\n", message.content ).unwrap();
      write!( &mut output, "*{}*\n\n", message.timestamp ).unwrap();
    }

    output
  }

  /// Convert session to plain text format
  fn to_plain_text( &self ) -> String
  {
    let mut output = String::new();

    writeln!( &mut output, "Chat Session : {}", self.current_session.name ).unwrap();
    writeln!( &mut output, "Model : {}", self.current_session.model ).unwrap();
    writeln!( &mut output, "Created : {}", self.current_session.created_at ).unwrap();
    write!( &mut output, "Last Updated : {}\n\n", self.current_session.last_updated ).unwrap();

    if let Some( ref system_prompt ) = self.current_session.system_prompt
    {
      write!( &mut output, "System Prompt : {system_prompt}\n\n" ).unwrap();
    }

    output.push_str( &"=" .repeat( 50 ) );
    output.push_str( "\n\n" );

    for message in &self.current_session.messages
    {
      let cached_indicator = if message.cached { " (cached)" } else { "" };
      writeln!( &mut output, "{}{}: {}", message.role, cached_indicator, message.content ).unwrap();
      write!( &mut output, "[{}]\n\n", message.timestamp ).unwrap();
    }

    output
  }

  /// Set model
  fn set_model( &mut self, model : &str )
  {
    self.current_session.model = model.to_string();
    println!( "{}", self.formatter.format_info( &format!( "Model set to : {model}" ) ) );
  }

  /// Set system prompt
  fn set_system_prompt( &mut self, prompt : &str )
  {
    self.current_session.system_prompt = Some( prompt.to_string() );
    println!( "{}", self.formatter.format_info( &format!( "System prompt set : {prompt}" ) ) );
  }

  /// Clear conversation history
  fn clear_history( &mut self )
  {
    self.current_session.clear_history();
    println!( "{}", self.formatter.format_info( "Conversation history cleared." ) );
  }

  /// Show session statistics
  fn show_stats( &self )
  {
    let total_messages = self.current_session.messages.len();
    let user_messages = self.current_session.messages.iter().filter( | m | m.role == "user" ).count();
    let assistant_messages = self.current_session.messages.iter().filter( | m | m.role == "assistant" ).count();
    let cached_messages = self.current_session.messages.iter().filter( | m | m.cached ).count();
    let estimated_tokens = self.current_session.estimate_total_tokens();

    let stats = format!( "
📊 Session Statistics:

Session : {}
Model : {}
Created : {}
Last Updated : {}

Messages:
  • Total : {}
  • User : {}
  • Assistant : {}
  • Cached : {}

Tokens:
  • Estimated Total : {}
  • Max Context : {}
  • Remaining : {}

Cache : {}
Auto-save : {}
",
      self.current_session.name,
      self.current_session.model,
      self.current_session.created_at,
      self.current_session.last_updated,
      total_messages,
      user_messages,
      assistant_messages,
      cached_messages,
      estimated_tokens,
      self.config.max_context_tokens,
      self.config.max_context_tokens.saturating_sub(estimated_tokens),
      if self.config.cache_enabled { "Enabled" } else { "Disabled" },
      if self.config.auto_save { "Enabled" } else { "Disabled" }
    );

    println!( "{}", self.formatter.format_info( &stats ) );
  }

  /// Toggle cache
  fn toggle_cache( &mut self )
  {
    self.config.cache_enabled = !self.config.cache_enabled;
    let status = if self.config.cache_enabled { "enabled" } else { "disabled" };
    println!( "{}", self.formatter.format_info( &format!( "Response caching {status}" ) ) );
  }

  /// Set maximum tokens
  fn set_max_tokens( &mut self, tokens : u32 )
  {
    self.config.max_context_tokens = tokens;
    println!( "{}", self.formatter.format_info( &format!( "Max context tokens set to : {tokens}" ) ) );
  }

  /// Manage context window - truncate old messages if needed
  fn manage_context_window( &mut self )
  {
    let estimated_tokens = self.current_session.estimate_total_tokens();

    if estimated_tokens > self.config.max_context_tokens
    {
      // Keep system prompt and recent messages
      let system_messages : Vec< _ > = self.current_session.messages
        .iter()
        .filter( | m | m.role == "system" )
        .cloned()
        .collect();

      let mut recent_messages = Vec::new();
      let mut token_count = 0u32;

      // Keep most recent messages within token limit
      for message in self.current_session.messages.iter().rev()
      {
        if message.role != "system"
        {
          let msg_tokens = message.token_count.unwrap_or_else( || ChatSession::estimate_tokens( &message.content ) );
          if token_count + msg_tokens <= self.config.max_context_tokens / 2 // Leave room for response
          {
            recent_messages.insert( 0, message.clone() );
            token_count += msg_tokens;
          }
          else
          {
            break;
          }
        }
      }

      // Combine system messages with recent messages
      let mut new_messages = system_messages;
      new_messages.extend( recent_messages );

      let removed_count = self.current_session.messages.len() - new_messages.len();
      self.current_session.messages = new_messages;

      if removed_count > 0
      {
        println!( "{}", self.formatter.format_info( &format!(
          "Truncated {removed_count} old messages to stay within token limit"
        ) ) );
      }
    }
  }
}

/// Command line arguments structure
#[ derive( Debug, Default ) ]
struct CliArgs
{
  session : Option< String >,
  model : Option< String >,
  system : Option< String >,
  no_cache : bool,
  export : Option< String >,
  max_tokens : Option< u32 >,
  help : bool,
}

/// Parse command line arguments
fn parse_args() -> CliArgs
{
  let args : Vec< String > = env::args().collect();
  let mut parsed = CliArgs::default();

  let mut i = 1;
  while i < args.len()
  {
    match args[ i ].as_str()
    {
      "--session" | "-s" =>
      {
        if i + 1 < args.len()
        {
          parsed.session = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      },
      "--model" | "-m" =>
      {
        if i + 1 < args.len()
        {
          parsed.model = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      },
      "--system" =>
      {
        if i + 1 < args.len()
        {
          parsed.system = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      },
      "--no-cache" =>
      {
        parsed.no_cache = true;
      },
      "--export" =>
      {
        if i + 1 < args.len()
        {
          parsed.export = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      },
      "--max-tokens" =>
      {
        if i + 1 < args.len()
        {
          if let Ok( tokens ) = args[ i + 1 ].parse::< u32 >()
          {
            parsed.max_tokens = Some( tokens );
          }
          i += 1;
        }
      },
      "--help" | "-h" =>
      {
        parsed.help = true;
      },
      _ => {} // Ignore unknown arguments
    }
    i += 1;
  }

  parsed
}

/// Display help information
fn show_cli_help()
{
  println!( "OpenAI Interactive Chat with Caching v1.0.0" );
  println!();
  println!( "USAGE:" );
  println!( "    cargo run --example openai_cached_interactive_chat [OPTIONS]" );
  println!();
  println!( "OPTIONS:" );
  println!( "    -s, --session < NAME >      Load existing session or create new one" );
  println!( "    -m, --model < MODEL >       OpenAI model to use (default : gpt-4)" );
  println!( "        --system < PROMPT >     System prompt to use" );
  println!( "        --no-cache            Disable response caching" );
  println!( "        --export < FORMAT >     Export format (json, markdown, plain)" );
  println!( "        --max-tokens < NUMBER > Maximum context tokens" );
  println!( "    -h, --help                Print help information" );
  println!();
  println!( "EXAMPLES:" );
  println!( "    # Basic usage" );
  println!( "    cargo run --example openai_cached_interactive_chat" );
  println!();
  println!( "    # With specific model and system prompt" );
  println!( "    cargo run --example openai_cached_interactive_chat -- \\" );
  println!( "      --model gpt-4-turbo \\" );
  println!( "      --system \"You are a helpful coding assistant\"" );
  println!();
  println!( "    # Load existing session" );
  println!( "    cargo run --example openai_cached_interactive_chat -- \\" );
  println!( "      --session \"my-coding-session\"" );
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  // Parse command line arguments
  let args = parse_args();

  // Show help if requested
  if args.help
  {
    show_cli_help();
    return Ok( () );
  }

  // Create OpenAI client
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

  // Create configuration
  let mut config = ChatConfig::default();

  if let Some( ref model ) = args.model
  {
    config.default_model.clone_from(model);
  }

  if args.no_cache
  {
    config.cache_enabled = false;
  }

  if let Some( max_tokens ) = args.max_tokens
  {
    config.max_context_tokens = max_tokens;
  }

  // Get session name
  let session_name = args.session;

  // Create and run application
  let mut app = InteractiveChatApp::new( client, config, session_name )?;

  // Set system prompt if provided
  if let Some( ref system_prompt ) = args.system
  {
    app.current_session.system_prompt = Some( system_prompt.clone() );
  }

  // Run the application
  app.run().await?;

  Ok( () )
}