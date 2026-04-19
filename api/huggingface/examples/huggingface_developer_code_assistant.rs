//! Developer Code Assistant & Documentation Generator
//!
//! This example demonstrates building an intelligent code assistant for developers that provides
//! code completion, documentation generation, technical writing assistance, and code review
//! capabilities using `HuggingFace` models.
//!
//! ## Usage
//!
//! ```bash
//! export HUGGINGFACE_API_KEY="your-api-key-here"
//! cargo run --example developer_code_assistant --features="full"
//! ```
//!
//! ## Commands
//!
//! - `/complete < language > < code >` - Get code completion suggestions
//! - `/document < language > < code >` - Generate documentation for code
//! - `/review < language > < code >` - Get code review and improvement suggestions
//! - `/explain < code >` - Explain what code does
//! - `/templates < language >` - Show code templates for language
//! - `/languages` - List supported programming languages
//! - `/help` - Show available commands
//! - `/quit` - Exit the code assistant

#![allow( missing_docs ) ]
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
use core::fmt;
use std::
{
  collections::HashMap,
  io::{ self, Write as IoWrite },
  time::Instant,
};
use serde::{ Deserialize, Serialize };

/// Programming languages supported by the code assistant
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub enum ProgrammingLanguage
{
  Rust,
  Python, 
  JavaScript,
  TypeScript,
  Java,
  Go,
  CPP,
  C,
}

impl ProgrammingLanguage
{
  /// Parse language from string
  fn from_str( s : &str ) -> Option< Self >
  {
  match s.to_lowercase( ).as_str( )
  {
      "rust" | "rs" => Some( Self::Rust ),
      "python" | "py" => Some( Self::Python ),
      "javascript" | "js" => Some( Self::JavaScript ),
      "typescript" | "ts" => Some( Self::TypeScript ),
      "java" => Some( Self::Java ),
      "go" | "golang" => Some( Self::Go ),
      "cpp" | "c++" => Some( Self::CPP ),
      "c" => Some( Self::C ),
      _ => None,
  }
  }

  /// Get the file extension for the programming language
  fn file_extension( self ) -> &'static str
  {
  match self
  {
      Self::Rust => "rs",
      Self::Python => "py", 
      Self::JavaScript => "js",
      Self::TypeScript => "ts",
      Self::Java => "java",
      Self::Go => "go",
      Self::CPP => "cpp",
      Self::C => "c",
  }
  }

  /// Get the syntax highlighting identifier
  fn syntax_id( self ) -> &'static str
  {
  match self
  {
      Self::Rust => "rust",
      Self::Python => "python",
      Self::JavaScript => "javascript", 
      Self::TypeScript => "typescript",
      Self::Java => "java",
      Self::Go => "go",
      Self::CPP => "cpp",
      Self::C => "c",
  }
  }

  /// Get the preferred model for code generation in this language
  fn preferred_model( self ) -> String
  {
  match self
  {
      Self::Rust | Self::Go | Self::C | Self::CPP => Models::code_llama_7b_instruct( ).to_string( ),
      Self::Python | Self::Java => Models::llama_3_3_70b_instruct( ).to_string( ),
      Self::JavaScript | Self::TypeScript => Models::mistral_7b_instruct( ).to_string( ),
  }
  }

  /// Get language description
  fn description( self ) -> &'static str
  {
  match self
  {
      Self::Rust => "Systems programming language focused on safety and performance",
      Self::Python => "High-level programming language for general-purpose programming",
      Self::JavaScript => "Dynamic language primarily used for web development",
      Self::TypeScript => "Typed superset of JavaScript for large-scale applications",
      Self::Java => "Object-oriented language for enterprise and Android development",
      Self::Go => "Modern language designed for concurrent and networked applications",
      Self::CPP => "Low-level language with object-oriented features",
      Self::C => "Low-level systems programming language",
  }
  }
}

impl fmt::Display for ProgrammingLanguage
{
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
  let name = match self
  {
      Self::Rust => "Rust",
      Self::Python => "Python",
      Self::JavaScript => "JavaScript",
      Self::TypeScript => "TypeScript", 
      Self::Java => "Java",
      Self::Go => "Go",
      Self::CPP => "C++",
      Self::C => "C",
  };
  write!( f, "{name}" )
  }
}

/// Types of code assistance that can be provided
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub enum AssistanceType
{
  CodeCompletion,
  DocumentationGeneration,
  TechnicalWriting,
  CodeReview,
  RefactoringAdvice,
  BugAnalysis,
}

/// Code completion context and requirements
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct CodeCompletionRequest
{
  pub language : ProgrammingLanguage,
  pub context : String,
  pub cursor_position : usize,
  pub max_suggestions : usize,
  pub include_snippets : bool,
  pub completion_type : CompletionType,
}

/// Types of code completion
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default ) ]
pub enum CompletionType
{
  Function,
  Variable,
  Class,
  Import,
  #[ default ]
  Generic,
}

/// Code completion suggestion with metadata
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct CodeSuggestion
{
  pub text : String,
  pub confidence : f32,
  pub suggestion_type : CompletionType,
  pub description : String,
  pub documentation : Option< String >,
}

/// Documentation generation request
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct DocumentationRequest
{
  pub language : ProgrammingLanguage,
  pub code : String,
  pub doc_style : DocumentationStyle,
  pub include_examples : bool,
  pub target_audience : AudienceLevel,
}

/// Documentation styles for different languages
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default ) ]
pub enum DocumentationStyle
{
  RustDoc,
  JavaDoc,
  PyDoc,
  JSDoc,
  Inline,
  #[ default ]
  Markdown,
}

/// Target audience for documentation
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default ) ]
pub enum AudienceLevel
{
  Beginner,
  #[ default ]
  Intermediate,
  Expert,
  Maintainer,
}

/// Generated documentation with metadata
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct GeneratedDocumentation
{
  pub content : String,
  pub style : DocumentationStyle,
  pub completeness : f32,
  pub readability : f32,
  pub examples : Vec< String >,
}

/// Code review request with analysis parameters
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct CodeReviewRequest
{
  pub language : ProgrammingLanguage,
  pub code : String,
  pub focus_areas : Vec< ReviewFocus >,
  pub strictness : ReviewStrictness,
}

/// Areas to focus on during code review
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub enum ReviewFocus
{
  Performance,
  Security,
  Readability,
  Maintainability,
  BestPractices,
  Testing,
}

/// Review strictness levels
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default ) ]
pub enum ReviewStrictness
{
  Lenient,
  #[ default ]
  Standard,
  Strict,
  Pedantic,
}

/// Code review result with suggestions
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct CodeReviewResult
{
  pub overall_score : f32,
  pub issues : Vec< ReviewIssue >,
  pub suggestions : Vec< String >,
  pub compliments : Vec< String >,
}

/// Individual code review issue
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ReviewIssue
{
  pub line : Option< usize >,
  pub severity : IssueSeverity,
  pub category : ReviewFocus,
  pub description : String,
  pub suggestion : Option< String >,
}

/// Issue severity levels
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum IssueSeverity
{
  Info,
  Warning,
  Error,
  Critical,
}

/// Code template for common patterns
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct CodeTemplate
{
  pub name : String,
  pub description : String,
  pub template : String,
  pub language : ProgrammingLanguage,
  pub category : String,
}

/// Advanced code assistant platform
#[ derive( Debug ) ]
pub struct CodeAssistantPlatform
{
  pub client : Client< HuggingFaceEnvironmentImpl >,
  pub language_models : HashMap< ProgrammingLanguage, String >,
  pub code_templates : HashMap< ProgrammingLanguage, Vec< CodeTemplate > >,
  pub usage_stats : AssistantStats,
}

/// Assistant usage statistics
#[ derive( Debug, Clone ) ]
pub struct AssistantStats
{
  pub total_completions : usize,
  pub total_documentation : usize,
  pub total_reviews : usize,
  pub by_language : HashMap< ProgrammingLanguage, usize >,
  pub avg_response_time_ms : f64,
}

impl Default for AssistantStats
{
  fn default() -> Self
  {
  Self
  {
      total_completions : 0,
      total_documentation : 0,
      total_reviews : 0,
      by_language : HashMap::new( ),
      avg_response_time_ms : 0.0,
  }
  }
}

impl CodeAssistantPlatform
{
  /// Create new code assistant platform
  #[ must_use ]
  pub fn new( client : Client< HuggingFaceEnvironmentImpl > ) -> Self
  {
  let mut language_models = HashMap::new( );
  for lang in [ ProgrammingLanguage::Rust, ProgrammingLanguage::Python, ProgrammingLanguage::JavaScript,
                  ProgrammingLanguage::TypeScript, ProgrammingLanguage::Java, ProgrammingLanguage::Go,
                  ProgrammingLanguage::CPP, ProgrammingLanguage::C ]
  {
      language_models.insert( lang, lang.preferred_model( ) );
  }

  let mut platform = Self
  {
      client,
      language_models,
      code_templates : HashMap::new( ),
      usage_stats : AssistantStats::default( ),
  };

  // Add default templates
  platform.add_default_templates( );
  platform
  }

  /// Add default code templates
  fn add_default_templates( &mut self )
  {
  // Rust templates
  let rust_templates = vec![
      CodeTemplate
      {
  name : "function".to_string( ),
  description : "Basic function template".to_string( ),
  template : "fn {name}( {params} ) -> {return_type} {\n    todo!( )\n}".to_string( ),
  language : ProgrammingLanguage::Rust,
  category : "function".to_string( ),
      },
      CodeTemplate
      {
  name : "struct".to_string( ),
  description : "Struct definition template".to_string( ),
  template : "#[ derive( Debug, Clone ) ]\npub struct {Name}\n{\n    // fields\n}".to_string( ),
  language : ProgrammingLanguage::Rust,
  category : "data".to_string( ),
      },
  ];

  // Python templates
  let python_templates = vec![
      CodeTemplate
      {
  name : "function".to_string( ),
  description : "Python function template".to_string( ),
  template : "def {name}( {params} ):\n    \"\"\"Function description.\"\"\"\n    pass".to_string( ),
  language : ProgrammingLanguage::Python,
  category : "function".to_string( ),
      },
      CodeTemplate
      {
  name : "class".to_string( ),
  description : "Python class template".to_string( ),
  template : "class {Name}:\n    \"\"\"Class description.\"\"\"\n    \n    def __init__( self ):\n        pass".to_string( ),
  language : ProgrammingLanguage::Python,
  category : "class".to_string( ),
      },
  ];

  self.code_templates.insert( ProgrammingLanguage::Rust, rust_templates );
  self.code_templates.insert( ProgrammingLanguage::Python, python_templates );
  }

  /// Generate code completion suggestions
  ///
  /// # Errors
  ///
  /// Returns an error if the language is unsupported or if the API call fails.
  pub async fn complete_code( &mut self, request : CodeCompletionRequest ) -> Result< Vec< CodeSuggestion >, Box< dyn std::error::Error > >
  {
  let start_time = Instant::now( );

  let model = self.language_models.get( &request.language )
      .ok_or( "Unsupported language" )?;

  // Build completion prompt
  let prompt = Self::build_completion_prompt( &request );
  
  // Generate completion
  let parameters = InferenceParameters::new( )
      .with_max_new_tokens( 100 )
      .with_temperature( 0.3 )
      .with_top_p( 0.9 );

  let response = self.client
      .inference( )
      .create_with_parameters( &prompt, model, parameters )
      .await?;

  let generated_text = response.extract_text_or_default( "" );

  // Parse suggestions from generated text
  let suggestions = Self::parse_completion_suggestions( &generated_text, &request );

  // Update statistics
  self.usage_stats.total_completions += 1;
  *self.usage_stats.by_language.entry( request.language ).or_insert( 0 ) += 1;
  
  let response_time = start_time.elapsed( ).as_millis( ) as f64;
  self.update_avg_response_time( response_time );

  Ok( suggestions )
  }

  /// Generate documentation for code
  ///
  /// # Errors
  ///
  /// Returns an error if the language is unsupported or if the API call fails.
  pub async fn generate_documentation( &mut self, request : DocumentationRequest ) -> Result< GeneratedDocumentation, Box< dyn std::error::Error > >
  {
  let start_time = Instant::now( );

  let model = self.language_models.get( &request.language )
      .ok_or( "Unsupported language" )?;

  // Build documentation prompt
  let prompt = Self::build_documentation_prompt( &request );

  let parameters = InferenceParameters::new( )
      .with_max_new_tokens( 300 )
      .with_temperature( 0.4 )
      .with_top_p( 0.8 );

  let response = self.client
      .inference( )
      .create_with_parameters( &prompt, model, parameters )
      .await?;

  let generated_text = response.extract_text_or_default( "" );

  let documentation = Self::process_generated_documentation( &generated_text, &request );

  // Update statistics
  self.usage_stats.total_documentation += 1;
  *self.usage_stats.by_language.entry( request.language ).or_insert( 0 ) += 1;
  
  let response_time = start_time.elapsed( ).as_millis( ) as f64;
  self.update_avg_response_time( response_time );

  Ok( documentation )
  }

  /// Review code and provide suggestions
  ///
  /// # Errors
  ///
  /// Returns an error if the language is unsupported or if the API call fails.
  pub async fn review_code( &mut self, request : CodeReviewRequest ) -> Result< CodeReviewResult, Box< dyn std::error::Error > >
  {
  let start_time = Instant::now( );

  let model = self.language_models.get( &request.language )
      .ok_or( "Unsupported language" )?;

  // Build review prompt
  let prompt = Self::build_review_prompt( &request );

  let parameters = InferenceParameters::new( )
      .with_max_new_tokens( 400 )
      .with_temperature( 0.5 )
      .with_top_p( 0.85 );

  let response = self.client
      .inference( )
      .create_with_parameters( &prompt, model, parameters )
      .await?;

  let generated_text = response.extract_text_or_default( "" );

  let review_result = Self::process_review_response( &generated_text, &request );

  // Update statistics
  self.usage_stats.total_reviews += 1;
  *self.usage_stats.by_language.entry( request.language ).or_insert( 0 ) += 1;
  
  let response_time = start_time.elapsed( ).as_millis( ) as f64;
  self.update_avg_response_time( response_time );

  Ok( review_result )
  }

  /// Build completion prompt
  fn build_completion_prompt( request : &CodeCompletionRequest ) -> String
  {
  let lang_name = request.language.to_string( );
  let context = &request.context;
  
  format!( 
      "Complete the following {} code. Provide {} realistic suggestions that would logically continue from this context:\n\n```{}\n{}\n```\n\nCompletion suggestions:",
      lang_name,
      request.max_suggestions,
      request.language.syntax_id( ),
      context
  )
  }

  /// Build documentation prompt
  fn build_documentation_prompt( request : &DocumentationRequest ) -> String
  {
  let style_instruction = match request.doc_style
  {
      DocumentationStyle::RustDoc => "Generate Rust documentation comments using /// syntax",
      DocumentationStyle::JavaDoc => "Generate JavaDoc documentation using /** */ syntax",
      DocumentationStyle::PyDoc => "Generate Python docstrings using triple quotes",
      DocumentationStyle::JSDoc => "Generate JSDoc documentation using /** */ syntax",
      DocumentationStyle::Inline => "Generate inline code comments",
      DocumentationStyle::Markdown => "Generate Markdown documentation",
  };

  let audience_instruction = match request.target_audience
  {
      AudienceLevel::Beginner => "Explain concepts clearly for beginners",
      AudienceLevel::Intermediate => "Provide standard technical documentation",
      AudienceLevel::Expert => "Use technical language appropriate for experts",
      AudienceLevel::Maintainer => "Focus on maintenance and implementation details",
  };

  format!(
      "{style_instruction} for the following {} code. {audience_instruction}{}:\n\n```{}\n{}\n```\n\nGenerated documentation:",
      request.language,
      if request.include_examples { " and include usage examples" } else { "" },
      request.language.syntax_id( ),
      request.code
  )
  }

  /// Build code review prompt
  fn build_review_prompt( request : &CodeReviewRequest ) -> String
  {
  let focus_areas = if request.focus_areas.is_empty( )
  {
      "general code quality".to_string( )
  }
  else
  {
      request.focus_areas.iter( )
  .map( |f| format!( "{f:?}" ).to_lowercase( ) )
  .collect::< Vec< _ > >( )
  .join( ", " )
  };

  let strictness_instruction = match request.strictness
  {
      ReviewStrictness::Lenient => "Be constructive and focus on major issues only",
      ReviewStrictness::Standard => "Provide balanced feedback with practical suggestions",
      ReviewStrictness::Strict => "Be thorough and point out all potential improvements",
      ReviewStrictness::Pedantic => "Be extremely detailed and mention every possible improvement",
  };

  format!(
      "Review the following {} code focusing on : {}. {strictness_instruction}. Provide specific, actionable feedback:\n\n```{}\n{}\n```\n\nCode review:",
      request.language,
      focus_areas,
      request.language.syntax_id( ),
      request.code
  )
  }

  /// Parse completion suggestions from generated text
  fn parse_completion_suggestions( text : &str, request : &CodeCompletionRequest ) -> Vec< CodeSuggestion >
  {
  let lines : Vec< &str > = text.lines( ).take( request.max_suggestions ).collect( );
  let mut suggestions = Vec::new( );

  for ( i, line ) in lines.iter( ).enumerate( )
  {
      if !line.trim( ).is_empty( )
      {
  let idx = i + 1;
  suggestions.push( CodeSuggestion
  {
          text : line.trim( ).to_string( ),
          confidence : 0.8 - ( i as f32 * 0.1 ), // Decreasing confidence
          suggestion_type : request.completion_type,
          description : format!( "Code completion suggestion {idx}" ),
          documentation : None,
  } );
      }
  }

  suggestions
  }

  /// Process generated documentation
  fn process_generated_documentation( text : &str, request : &DocumentationRequest ) -> GeneratedDocumentation
  {
  let cleaned_text = text.trim( ).to_string( );
  
  // Simple quality assessment
  let completeness = if cleaned_text.len( ) > 50 { 0.8 } else { 0.4 };
  let readability = if cleaned_text.contains( '\n' ) { 0.7 } else { 0.5 };

  GeneratedDocumentation
  {
      content : cleaned_text,
      style : request.doc_style,
      completeness,
      readability,
      examples : vec![ ], // Could be extracted from text
  }
  }

  /// Process code review response
  fn process_review_response( text : &str, _request : &CodeReviewRequest ) -> CodeReviewResult
  {
  let cleaned_text = text.trim( );
  
  // Simple parsing - in a real implementation this would be more sophisticated
  let lines : Vec< &str > = cleaned_text.lines( ).collect( );
  let mut issues = Vec::new( );
  let mut suggestions = Vec::new( );

  for line in lines
  {
      if line.contains( "issue" ) || line.contains( "problem" )
      {
  issues.push( ReviewIssue
  {
          line : None,
          severity : IssueSeverity::Warning,
          category : ReviewFocus::BestPractices,
          description : line.to_string( ),
          suggestion : None,
  } );
      }
      else if !line.trim( ).is_empty( )
      {
  suggestions.push( line.to_string( ) );
      }
  }

  let overall_score = if issues.is_empty( ) { 0.9 } else { 0.6 };

  CodeReviewResult
  {
      overall_score,
      issues,
      suggestions,
      compliments : vec![ "Code structure looks good".to_string( ) ],
  }
  }

  /// Update average response time
  fn update_avg_response_time( &mut self, response_time : f64 )
  {
  let total_requests = self.usage_stats.total_completions + self.usage_stats.total_documentation + self.usage_stats.total_reviews;
  if total_requests > 0
  {
      self.usage_stats.avg_response_time_ms = 
  ( self.usage_stats.avg_response_time_ms * ( total_requests - 1 ) as f64 + response_time ) / total_requests as f64;
  }
  }

  /// Get templates for a language
  #[ must_use ]
  pub fn get_templates( &self, language : ProgrammingLanguage ) -> Option< &Vec< CodeTemplate > >
  {
  self.code_templates.get( &language )
  }

  /// Get usage statistics
  #[ must_use ]
  pub fn get_stats( &self ) -> &AssistantStats
  {
  &self.usage_stats
  }

  /// Explain what code does
  ///
  /// # Errors
  ///
  /// Returns an error if the language is unsupported or if the API call fails.
  pub async fn explain_code( &mut self, language : ProgrammingLanguage, code : &str ) -> Result< String, Box< dyn std::error::Error > >
  {
  let model = self.language_models.get( &language )
      .ok_or( "Unsupported language" )?;

  let prompt = format!(
      "Explain what the following {} code does in simple terms:\n\n```{}\n{}\n```\n\nExplanation:",
      language,
      language.syntax_id( ),
      code
  );

  let parameters = InferenceParameters::new( )
      .with_max_new_tokens( 200 )
      .with_temperature( 0.4 );

  let response = self.client
      .inference( )
      .create_with_parameters( &prompt, model, parameters )
      .await?;

  let explanation = response.extract_text_or_default( "Could not generate explanation." );

  Ok( explanation.trim( ).to_string( ) )
  }
}

/// Interactive CLI for the code assistant
#[ derive( Debug ) ]
pub struct CodeAssistantCLI
{
  assistant : CodeAssistantPlatform,
}

impl CodeAssistantCLI
{
  /// Create new code assistant CLI
  #[ must_use ]
  pub fn new( client : Client< HuggingFaceEnvironmentImpl > ) -> Self
  {
  Self
  {
      assistant : CodeAssistantPlatform::new( client ),
  }
  }

  /// Start interactive CLI session
  ///
  /// # Errors
  ///
  /// Returns an error if I/O operations fail or if the command handling fails.
  pub async fn start( &mut self ) -> Result< ( ), Box< dyn std::error::Error > >
  {
  println!( "👨‍💻 Developer Code Assistant & Documentation Generator" );
  println!( "=====================================================" );
  println!( "Type '/help' for commands or start coding assistance!" );
  println!( "Supported languages : Rust, Python, JavaScript, TypeScript, Java, Go, C++, C" );
  println!( );

  let stdin = io::stdin( );
  let mut stdout = io::stdout( );

  loop
  {
      print!( "assistant > " );
      stdout.flush( )?;

      let mut input = String::new( );
      stdin.read_line( &mut input )?;
      let input = input.trim( );

      if input.is_empty( )
      {
  continue;
      }

      // Handle commands
      if input.starts_with( '/' )
      {
  match self.handle_command( input ).await
  {
          Ok( Some( response ) ) => println!( "{response}" ),
          Ok( None ) => {}, // Command handled without output
          Err( e ) => println!( "❌ Error : {e}" ),
  }
  continue;
      }

      println!( "💡 Use commands like:" );
      println!( "  /complete rust 'fn main( ) {{'" );
      println!( "  /document python 'def calculate_sum( a, b ): return a + b'" );
      println!( "  /review rust 'unsafe {{ ptr.read( ) }}'" );
      println!( "Type '/help' for all commands." );
      println!( );
  }
  }

  /// Handle CLI commands
  #[ allow( clippy::too_many_lines ) ]
  async fn handle_command( &mut self, command : &str ) -> Result< Option< String >, Box< dyn std::error::Error > >
  {
  let parts : Vec< &str > = command[ 1.. ].splitn( 2, ' ' ).collect( );
  
  if parts.is_empty( )
  {
      return Ok( Some( "Invalid command. Type '/help' for available commands.".to_string( ) ) );
  }

  match parts[ 0 ].to_lowercase( ).as_str( )
  {
      "help" => Ok( Some( Self::show_help( ) ) ),
      
      "quit" | "exit" => 
      {
  println!( "👋 Happy coding!" );
  std::process::exit( 0 );
      },
      
      "complete" =>
      {
  if parts.len( ) < 2
  {
          return Ok( Some( "Usage : /complete < language > '< code >'".to_string( ) ) );
  }
  
  let sub_parts : Vec< &str > = parts[ 1 ].splitn( 2, ' ' ).collect( );
  if sub_parts.len( ) < 2
  {
          return Ok( Some( "Usage : /complete < language > '< code >'".to_string( ) ) );
  }
  
  if let Some( language ) = ProgrammingLanguage::from_str( sub_parts[ 0 ] )
  {
          let code = sub_parts[ 1 ].trim_matches( '\'' );
          let request = CodeCompletionRequest
          {
      language,
      context : code.to_string( ),
      cursor_position : code.len( ),
      max_suggestions : 3,
      include_snippets : true,
      completion_type : CompletionType::default( ),
          };

          println!( "🔄 Generating code completions for {language}..." );
          
          match self.assistant.complete_code( request ).await
          {
      Ok( suggestions ) => 
      {
              if suggestions.is_empty( )
              {
        Ok( Some( "No completion suggestions generated.".to_string( ) ) )
              }
              else
              {
        use core::fmt::Write;
        let mut result = "✅ Code Completion Suggestions:\n\n".to_string( );
        for ( i, suggestion ) in suggestions.iter( ).enumerate( )
        {
                  let idx = i + 1;
                  let text = &suggestion.text;
                  let confidence = suggestion.confidence;
                  let description = &suggestion.description;
                  writeln!( &mut result, "{idx}. {text} ( confidence : {confidence:.2} )\n   {description}\n" ).ok( );
        }
        Ok( Some( result ) )
              }
      },
      Err( e ) => Ok( Some( format!( "❌ Completion failed : {e}" ) ) ),
          }
  }
  else
  {
          Ok( Some( "❌ Unsupported language. Use '/languages' to see supported languages.".to_string( ) ) )
  }
      },
      
      "document" =>
      {
  if parts.len( ) < 2
  {
          return Ok( Some( "Usage : /document < language > '< code >'".to_string( ) ) );
  }
  
  let sub_parts : Vec< &str > = parts[ 1 ].splitn( 2, ' ' ).collect( );
  if sub_parts.len( ) < 2
  {
          return Ok( Some( "Usage : /document < language > '< code >'".to_string( ) ) );
  }
  
  if let Some( language ) = ProgrammingLanguage::from_str( sub_parts[ 0 ] )
  {
          let code = sub_parts[ 1 ].trim_matches( '\'' );
          let request = DocumentationRequest
          {
      language,
      code : code.to_string( ),
      doc_style : DocumentationStyle::default( ),
      include_examples : true,
      target_audience : AudienceLevel::default( ),
          };

          println!( "🔄 Generating documentation for {language} code..." );
          
          match self.assistant.generate_documentation( request ).await
          {
      Ok( documentation ) => 
      {
              let result = format!(
        "✅ Generated Documentation:\n\n{}\n\n📊 Quality : Completeness {:.1}/1.0, Readability {:.1}/1.0",
        documentation.content, documentation.completeness, documentation.readability
              );
              Ok( Some( result ) )
      },
      Err( e ) => Ok( Some( format!( "❌ Documentation generation failed : {e}" ) ) ),
          }
  }
  else
  {
          Ok( Some( "❌ Unsupported language. Use '/languages' to see supported languages.".to_string( ) ) )
  }
      },
      
      "review" =>
      {
  if parts.len( ) < 2
  {
          return Ok( Some( "Usage : /review < language > '< code >'".to_string( ) ) );
  }
  
  let sub_parts : Vec< &str > = parts[ 1 ].splitn( 2, ' ' ).collect( );
  if sub_parts.len( ) < 2
  {
          return Ok( Some( "Usage : /review < language > '< code >'".to_string( ) ) );
  }
  
  if let Some( language ) = ProgrammingLanguage::from_str( sub_parts[ 0 ] )
  {
          let code = sub_parts[ 1 ].trim_matches( '\'' );
          let request = CodeReviewRequest
          {
      language,
      code : code.to_string( ),
      focus_areas : vec![ ],
      strictness : ReviewStrictness::default( ),
          };

          println!( "🔄 Reviewing {language} code..." );
          
          match self.assistant.review_code( request ).await
          {
      Ok( review ) => 
      {
              let mut result = format!( "✅ Code Review ( Score : {:.1}/1.0 ):\n\n", review.overall_score );
              
              if !review.issues.is_empty( )
              {
        use core::fmt::Write;
        result.push_str( "🔍 Issues Found:\n" );
        for issue in &review.issues
        {
                  let desc = &issue.description;
                  writeln!( &mut result, "• {desc}" ).ok( );
        }
        result.push( '\n' );
              }

              if !review.suggestions.is_empty( )
              {
        use core::fmt::Write;
        result.push_str( "💡 Suggestions:\n" );
        for suggestion in &review.suggestions
        {
                  writeln!( &mut result, "• {suggestion}" ).ok( );
        }
              }
              
              Ok( Some( result ) )
      },
      Err( e ) => Ok( Some( format!( "❌ Code review failed : {e}" ) ) ),
          }
  }
  else
  {
          Ok( Some( "❌ Unsupported language. Use '/languages' to see supported languages.".to_string( ) ) )
  }
      },
      
      "explain" =>
      {
  if parts.len( ) < 2
  {
          return Ok( Some( "Usage : /explain '< code >' ( assumes Rust, or specify language )".to_string( ) ) );
  }
  
  // Try to detect language or default to Rust
  let code = parts[ 1 ].trim_matches( '\'' );
  let language = ProgrammingLanguage::Rust; // Default to Rust

  println!( "🔄 Explaining code..." );
  
  match self.assistant.explain_code( language, code ).await
  {
          Ok( explanation ) => Ok( Some( format!( "💡 Code Explanation:\n{explanation}" ) ) ),
          Err( e ) => Ok( Some( format!( "❌ Explanation failed : {e}" ) ) ),
  }
      },
      
      "templates" =>
      {
  if parts.len( ) < 2
  {
          return Ok( Some( "Usage : /templates < language >".to_string( ) ) );
  }
  
  if let Some( language ) = ProgrammingLanguage::from_str( parts[ 1 ] )
  {
          if let Some( templates ) = self.assistant.get_templates( language )
          {
      use core::fmt::Write;
      let mut result = format!( "📝 {language} Templates:\n\n" );
      for template in templates
      {
              let name = &template.name;
              let category = &template.category;
              let description = &template.description;
              let tmpl = &template.template;
              writeln!( &mut result, "• {name} ( {category} )" ).ok( );
              writeln!( &mut result, "  {description}" ).ok( );
              writeln!( &mut result, "  Template : {tmpl}\n" ).ok( );
      }
      Ok( Some( result ) )
          }
          else
          {
      Ok( Some( format!( "No templates available for {language}." ) ) )
          }
  }
  else
  {
          Ok( Some( "❌ Unsupported language. Use '/languages' to see supported languages.".to_string( ) ) )
  }
      },
      
      "languages" =>
      {
  use core::fmt::Write;
  let mut result = "🌐 Supported Programming Languages:\n\n".to_string( );
  for lang in [ ProgrammingLanguage::Rust, ProgrammingLanguage::Python, ProgrammingLanguage::JavaScript,
                      ProgrammingLanguage::TypeScript, ProgrammingLanguage::Java, ProgrammingLanguage::Go,
                      ProgrammingLanguage::CPP, ProgrammingLanguage::C ]
  {
          let ext = lang.file_extension( );
          let desc = lang.description( );
          writeln!( &mut result, "• {lang} ( .{ext} ) - {desc}" ).ok( );
  }
  Ok( Some( result ) )
      },
      
      "stats" =>
      {
  let stats = self.assistant.get_stats( );
  let result = format!(
          "📊 Assistant Usage Statistics:\n\
           Code Completions : {}\n\
           Documentation Generated : {}\n\
           Code Reviews : {}\n\
           Average Response Time : {:.1}ms\n\
           \n\
           By Language:\n{}",
          stats.total_completions,
          stats.total_documentation,
          stats.total_reviews,
          stats.avg_response_time_ms,
          stats.by_language.iter( )
      .map( |( lang, count )| format!( "  • {lang}: {count}" ) )
      .collect::< Vec< _ > >( )
      .join( "\n" )
  );
  Ok( Some( result ) )
      },
      
      _ =>
      {
  let cmd = parts[ 0 ];
  Ok( Some( format!( "Unknown command : /{cmd}\nType '/help' for available commands." ) ) )
      },
  }
  }

  /// Show help information
  fn show_help() -> String
  {
  r"Available Commands:
===================

/complete < lang > '< code >'  - Get code completion suggestions
/document < lang > '< code >'  - Generate documentation for code
/review < lang > '< code >'    - Get code review and suggestions
/explain '< code >'          - Explain what code does
/templates < lang >          - Show code templates for language
/languages                 - List supported programming languages
/stats                     - Show usage statistics
/help                      - Show this help message
/quit or /exit             - Exit the code assistant

Supported Languages:
===================

rust, python, javascript, typescript, java, go, cpp, c

Example Usage:
==============

/complete rust 'fn main( ) {{'
/document python 'def calculate_sum( a, b ): return a + b'
/review rust 'unsafe {{ ptr.read( ) }}'
/explain 'let mut x = Vec::new( ); x.push( 1 );'
/templates python

Tips:
=====

• Use single quotes around code to avoid shell interpretation
• Code completion works best with meaningful context
• Documentation generation adapts to the target language style
• Code review focuses on common issues and best practices".to_string( )
  }
}

#[ tokio::main ]
async fn main() -> Result< ( ), Box< dyn std::error::Error > >
{
  // Load API key from environment or workspace secrets
  let api_key = std::env::var( "HUGGINGFACE_API_KEY" )
  .or_else( |_| {
      use workspace_tools as workspace;
      let workspace = workspace::workspace( )
  .map_err( |_| std::env::VarError::NotPresent )?; // Convert WorkspaceError
      let secrets = workspace.load_secrets_from_file( "-secrets.sh" )
  .map_err( |_| std::env::VarError::NotPresent )?; // Convert WorkspaceError
      secrets.get( "HUGGINGFACE_API_KEY" )
  .cloned( )
  .ok_or( std::env::VarError::NotPresent )
  } )
  .map_err( |_| "HUGGINGFACE_API_KEY not found in environment or workspace secrets" )?;

  // Build client
  let secret_key = Secret::new( api_key );
  let environment = HuggingFaceEnvironmentImpl::build( secret_key, None )?;
  let client = Client::build( environment )?;

  // Start interactive code assistant CLI
  let mut cli = CodeAssistantCLI::new( client );
  cli.start( ).await?;

  Ok( ( ) )
}