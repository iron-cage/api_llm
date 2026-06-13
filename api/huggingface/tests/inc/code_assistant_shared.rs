//! Code Assistant & Documentation Generator Example Tests
//!
//! This module contains comprehensive tests for a code assistant and documentation generator
//! system that helps developers with code completion, documentation generation, technical
//! writing, code review suggestions, and multi-language programming support.
//!
//! The tests cover:
//! - Code completion and suggestion generation
//! - Documentation generation from code analysis
//! - Technical writing assistance for developers
//! - Code review and improvement suggestions
//! - Multi-language programming support
//! - Integration with development workflows
//!
//! All tests follow Test-Driven Development (TDD) principles and are designed to guide
//! the implementation of a comprehensive code assistant platform.

#![ allow( clippy::pedantic, clippy::upper_case_acronyms ) ]

use std::collections::HashMap;

use serde::{ Deserialize, Serialize };

use api_huggingface::*;
use api_huggingface::components::input::InferenceParameters;
use api_huggingface::environment::HuggingFaceEnvironmentImpl;

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
  /// Get the file extension for the programming language
  pub fn file_extension( &self ) -> &'static str
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
  pub fn syntax_id( &self ) -> &'static str
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

  /// Get the appropriate model for code generation in this language
  pub fn preferred_model( &self ) -> &'static str
  {
  // Use Qwen model for all languages (new Router API)
  // Provides better code understanding and generation than legacy models
  "meta-llama/Llama-3.3-70B-Instruct"
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
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum CompletionType
{
  Function,
  Variable,
  Class,
  Import,
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
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum DocumentationStyle
{
  RustDoc,
  JavaDoc, 
  PyDoc,
  JSDoc,
  Inline,
  Markdown,
}

/// Target audience for documentation
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum AudienceLevel
{
  Beginner,
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
  pub review_focus : Vec< ReviewFocus >,
  pub severity_threshold : SeverityLevel,
  pub include_suggestions : bool,
}

/// Areas of focus for code review
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub enum ReviewFocus
{
  Performance,
  Security,
  Maintainability,
  Style,
  BestPractices,
  Testing,
}

/// Severity levels for review issues
#[ derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize ) ]
pub enum SeverityLevel
{
  Info,
  Warning,
  Error,
  Critical,
}

/// Code review finding with details
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ReviewFinding
{
  pub line_number : usize,
  pub severity : SeverityLevel,
  pub category : ReviewFocus,
  pub message : String,
  pub suggestion : Option< String >,
  pub confidence : f32,
}

/// Technical writing assistance request
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct TechnicalWritingRequest
{
  pub content_type : TechnicalContentType,
  pub topic : String,
  pub audience : AudienceLevel,
  pub include_code_examples : bool,
  pub target_length : WritingLength,
}

/// Types of technical content that can be generated
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum TechnicalContentType
{
  APIDocumentation,
  Tutorial,
  TechnicalBlog,
  README,
  ArchitectureDoc,
  Troubleshooting,
}

/// Target length for technical writing
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum WritingLength
{
  Brief,      // 100-300 words
  Standard,   // 300-800 words  
  Detailed,   // 800-1500 words
  Comprehensive, // 1500+ words
}

impl WritingLength
{
  /// Get the word count range for this length category
  pub fn word_count_range( &self ) -> ( usize, usize )
  {
  match self
  {
      Self::Brief => ( 100, 300 ),
      Self::Standard => ( 300, 800 ),
      Self::Detailed => ( 800, 1500 ),
      Self::Comprehensive => ( 1500, 3000 ),
  }
  }
}

/// Generated technical content with metrics
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct TechnicalContent
{
  pub content : String,
  pub word_count : usize,
  pub readability_score : f32,
  pub technical_accuracy : f32,
  pub code_examples : Vec< String >,
}

/// Code assistant platform with multi-language support
#[ derive( Debug ) ]
pub struct CodeAssistantPlatform
{
  pub client : Client< HuggingFaceEnvironmentImpl >,
  pub language_models : HashMap< ProgrammingLanguage, String >,
  pub code_templates : HashMap< ProgrammingLanguage, Vec< CodeTemplate > >,
  pub review_rules : HashMap< ReviewFocus, Vec< ReviewRule > >,
}

/// Code template for common patterns
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct CodeTemplate
{
  pub name : String,
  pub language : ProgrammingLanguage,
  pub template : String,
  pub variables : Vec< String >,
  pub description : String,
}

/// Review rule for code analysis
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ReviewRule
{
  pub pattern : String,
  pub severity : SeverityLevel,
  pub message : String,
  pub suggestion : Option< String >,
}

impl CodeAssistantPlatform
{
  /// Create a new code assistant platform
  pub fn new( client : Client< HuggingFaceEnvironmentImpl > ) -> Self
  {
  // Use Qwen model for all languages (new Router API)
  let kimi_model = "meta-llama/Llama-3.3-70B-Instruct".to_string();

  let mut language_models = HashMap::new();
  language_models.insert( ProgrammingLanguage::Rust, kimi_model.clone() );
  language_models.insert( ProgrammingLanguage::Python, kimi_model.clone() );
  language_models.insert( ProgrammingLanguage::JavaScript, kimi_model.clone() );
  language_models.insert( ProgrammingLanguage::TypeScript, kimi_model.clone() );
  language_models.insert( ProgrammingLanguage::Java, kimi_model.clone() );
  language_models.insert( ProgrammingLanguage::Go, kimi_model.clone() );
  language_models.insert( ProgrammingLanguage::CPP, kimi_model.clone() );
  language_models.insert( ProgrammingLanguage::C, kimi_model );

  Self
  {
      client,
      language_models,
      code_templates : HashMap::new(),
      review_rules : HashMap::new(),
  }
  }

  /// Generate code completion suggestions
  pub async fn complete_code( &self, request : &CodeCompletionRequest ) -> Result< Vec< CodeSuggestion >, Box< dyn std::error::Error > >
  {
  let model = self.language_models.get( &request.language )
      .ok_or( "Unsupported language" )?;

  let prompt = Self::build_completion_prompt( request );
  
  let params = InferenceParameters::new()
      .with_temperature( 0.3 )
      .with_max_new_tokens( 150 )
      .with_top_p( 0.9 );

  let result = self.client.inference().create_with_parameters( &prompt, model, params ).await?;

  let generated_text = result.extract_text_or_default( "" );
  let suggestions = Self::parse_code_suggestions( &generated_text, request );
  Ok( suggestions )
  }

  /// Generate documentation for code
  pub async fn generate_documentation( &self, request : &DocumentationRequest ) -> Result< GeneratedDocumentation, Box< dyn std::error::Error > >
  {
  let model = self.language_models.get( &request.language )
      .ok_or( "Unsupported language" )?;

  let prompt = Self::build_documentation_prompt( request );
  
  let params = InferenceParameters::new()
      .with_temperature( 0.4 )
      .with_max_new_tokens( 500 )
      .with_top_p( 0.8 );

  let result = self.client.inference().create_with_parameters( &prompt, model, params ).await?;

  let generated_text = result.extract_text_or_default( "" );
  let documentation = Self::parse_documentation_response( &generated_text, request );
  Ok( documentation )
  }

  /// Perform code review analysis
  pub async fn review_code( &self, request : &CodeReviewRequest ) -> Result< Vec< ReviewFinding >, Box< dyn std::error::Error > >
  {
  let model = self.language_models.get( &request.language )
      .ok_or( "Unsupported language" )?;

  let prompt = Self::build_review_prompt( request );
  
  let params = InferenceParameters::new()
      .with_temperature( 0.2 )
      .with_max_new_tokens( 400 )
      .with_top_p( 0.7 );

  let result = self.client.inference().create_with_parameters( &prompt, model, params ).await?;

  let generated_text = result.extract_text_or_default( "" );
  let findings = Self::parse_review_findings( &generated_text, request );
  Ok( findings )
  }

  /// Generate technical writing content
  pub async fn generate_technical_content( &self, request : &TechnicalWritingRequest ) -> Result< TechnicalContent, Box< dyn std::error::Error > >
  {
  let model = "meta-llama/Llama-3.3-70B-Instruct";

  let prompt = Self::build_technical_writing_prompt( request );
  
  let params = InferenceParameters::new()
      .with_temperature( 0.6 )
      .with_max_new_tokens( Self::get_max_tokens_for_length( request.target_length ) )
      .with_top_p( 0.9 );

  let result = self.client.inference().create_with_parameters( &prompt, model, params ).await?;

  let generated_text = result.extract_text_or_default( "" );
  let content = Self::parse_technical_content( &generated_text, request );
  Ok( content )
  }

  /// Add a code template for a specific language
  pub fn add_template( &mut self, template : CodeTemplate )
  {
  self.code_templates
      .entry( template.language )
      .or_default()
      .push( template );
  }

  /// Get templates for a specific language
  pub fn get_templates_for_language( &self, language : ProgrammingLanguage ) -> Vec< &CodeTemplate >
  {
  self.code_templates
      .get( &language )
      .map( | templates | templates.iter().collect() )
      .unwrap_or_default()
  }

  /// Add a review rule for a specific focus area
  pub fn add_review_rule( &mut self, focus : ReviewFocus, rule : ReviewRule )
  {
  self.review_rules
      .entry( focus )
      .or_default()
      .push( rule );
  }

  /// Get platform statistics
  pub fn get_platform_stats( &self ) -> AssistantStats
  {
  let total_languages = self.language_models.len();
  let total_templates = self.code_templates.values().map( Vec::len ).sum();
  let total_review_rules = self.review_rules.values().map( Vec::len ).sum();

  AssistantStats
  {
      supported_languages : total_languages,
      available_templates : total_templates,
      active_review_rules : total_review_rules,
      language_distribution : self.calculate_language_distribution(),
  }
  }

  /// Build prompt for code completion
  fn build_completion_prompt( request : &CodeCompletionRequest ) -> String
  {
  let language = request.language.syntax_id();
  let context = &request.context;
  let completion_type = format!( "{:?}", request.completion_type ).to_lowercase();
  let max_suggestions = request.max_suggestions;

  format!(
      "Complete the following {language} code. Focus on {completion_type} completion:\n\n```{language}\n{context}\n```\n\nProvide {max_suggestions} high-quality suggestions:"
  )
  }

  /// Build prompt for documentation generation
  fn build_documentation_prompt( request : &DocumentationRequest ) -> String
  {
  let style = format!( "{:?}", request.doc_style );
  let audience = format!( "{:?}", request.target_audience ).to_lowercase();
  let language = request.language.syntax_id();
  let examples_note = if request.include_examples { " Include practical examples." } else { "" };
  let code = &request.code;

  format!(
      "Generate {style} documentation for the following {language} code, targeting {audience} developers.{examples_note}\n\n```{language}\n{code}\n```\n\nDocumentation:"
  )
  }

  /// Build prompt for code review
  fn build_review_prompt( request : &CodeReviewRequest ) -> String
  {
  let language = request.language.syntax_id();
  let focus_areas = request.review_focus.iter()
      .map( | f | format!( "{f:?}" ).to_lowercase() )
      .collect::< Vec< _ > >()
      .join( ", " );
  let code = &request.code;

  format!(
      "Review the following {language} code focusing on : {focus_areas}. Identify issues and provide suggestions:\n\n```{language}\n{code}\n```\n\nReview findings:"
  )
  }

  /// Build prompt for technical writing
  fn build_technical_writing_prompt( request : &TechnicalWritingRequest ) -> String
  {
  let content_type = format!( "{:?}", request.content_type );
  let audience = format!( "{:?}", request.audience ).to_lowercase();
  let length = format!( "{:?}", request.target_length ).to_lowercase();
  let examples_note = if request.include_code_examples { " Include relevant code examples." } else { "" };
  let ( min_words, max_words ) = request.target_length.word_count_range();
  let topic = &request.topic;

  format!(
      "Write {content_type} about '{topic}' for {audience} developers. Target length : {length} ({min_words}-{max_words} words).{examples_note}\n\nContent:"
  )
  }

  /// Parse code suggestions from response
  fn parse_code_suggestions( response : &str, request : &CodeCompletionRequest ) -> Vec< CodeSuggestion >
  {
  let lines = response.lines().collect::< Vec< _ > >();
  let mut suggestions = Vec::new();

  for ( i, line ) in lines.iter().enumerate()
  {
      if !line.trim().is_empty() && i < request.max_suggestions
      {
  let completion_type = request.completion_type;
  suggestions.push( CodeSuggestion
  {
          text : line.trim().to_string(),
          confidence : 0.8 - ( i as f32 * 0.1 ),
          suggestion_type : request.completion_type,
          description : format!( "{completion_type:?} completion suggestion" ),
          documentation : None,
  } );
      }
  }

  if suggestions.is_empty()
  {
      suggestions.push( CodeSuggestion
      {
  text : "// No suggestions available".to_string(),
  confidence : 0.1,
  suggestion_type : request.completion_type,
  description : "Fallback suggestion".to_string(),
  documentation : None,
      } );
  }

  suggestions
  }

  /// Parse documentation from response
  fn parse_documentation_response( response : &str, request : &DocumentationRequest ) -> GeneratedDocumentation
  {
  let _word_count = response.split_whitespace().count();
  let readability = Self::calculate_readability_score( response );
  let completeness = Self::calculate_completeness_score( response, request );
  let examples = Self::extract_code_examples( response );

  GeneratedDocumentation
  {
      content : response.to_string(),
      style : request.doc_style,
      completeness,
      readability,
      examples,
  }
  }

  /// Parse review findings from response
  fn parse_review_findings( response : &str, request : &CodeReviewRequest ) -> Vec< ReviewFinding >
  {
  let mut findings = Vec::new();
  let lines = response.lines().enumerate();

  for ( line_num, line ) in lines
  {
      if !line.trim().is_empty()
      {
  findings.push( ReviewFinding
  {
          line_number : line_num + 1,
          severity : if line.contains( "error" ) || line.contains( "critical" ) { SeverityLevel::Error }
                     else if line.contains( "warning" ) { SeverityLevel::Warning }
                     else { SeverityLevel::Info },
          category : request.review_focus.first().copied().unwrap_or( ReviewFocus::BestPractices ),
          message : line.trim().to_string(),
          suggestion :
          {
      let trimmed = line.trim();
      Some( format!( "Consider improving : {trimmed}" ) )
          },
          confidence : 0.7,
  } );
      }
  }

  findings
  }

  /// Parse technical content from response
  fn parse_technical_content( response : &str, request : &TechnicalWritingRequest ) -> TechnicalContent
  {
  let word_count = response.split_whitespace().count();
  let readability = Self::calculate_readability_score( response );
  let technical_accuracy = Self::calculate_technical_accuracy( response, request );
  let code_examples = if request.include_code_examples { Self::extract_code_examples( response ) } else { Vec::new() };

  TechnicalContent
  {
      content : response.to_string(),
      word_count,
      readability_score : readability,
      technical_accuracy,
      code_examples,
  }
  }

  /// Get maximum tokens for writing length
  fn get_max_tokens_for_length( length : WritingLength ) -> u32
  {
  match length
  {
      WritingLength::Brief => 200u32,
      WritingLength::Standard => 600u32,
      WritingLength::Detailed => 1200u32,
      WritingLength::Comprehensive => 2000u32,
  }
  }

  /// Calculate readability score for text
  fn calculate_readability_score( text : &str ) -> f32
  {
  let word_count = text.split_whitespace().count();
  let sentence_count = text.matches( '.' ).count() + text.matches( '!' ).count() + text.matches( '?' ).count();
  let avg_word_length = text.chars().filter( | c | c.is_alphabetic() ).count() as f32 / word_count.max( 1 ) as f32;
  
  let sentences = sentence_count.max( 1 ) as f32;
  let words_per_sentence = word_count as f32 / sentences;
  
  let readability = 206.835 - ( 1.015 * words_per_sentence ) - ( 84.6 * avg_word_length );
  ( readability / 100.0 ).clamp( 0.0, 1.0 )
  }

  /// Calculate completeness score for documentation
  fn calculate_completeness_score( documentation : &str, request : &DocumentationRequest ) -> f32
  {
  let mut score : f32 = 0.0;
  let text_lower = documentation.to_lowercase();

  // Check for common documentation elements
  if text_lower.contains( "param" ) || text_lower.contains( "argument" ) { score += 0.2; }
  if text_lower.contains( "return" ) || text_lower.contains( "output" ) { score += 0.2; }
  if text_lower.contains( "example" ) && request.include_examples { score += 0.3; }
  if text_lower.contains( "error" ) || text_lower.contains( "exception" ) { score += 0.1; }
  if documentation.len() > 100 { score += 0.2; }

  score.min( 1.0 )
  }

  /// Calculate technical accuracy score
  fn calculate_technical_accuracy( content : &str, request : &TechnicalWritingRequest ) -> f32
  {
  let mut accuracy : f32 = 0.7; // Base accuracy
  let text_lower = content.to_lowercase();

  // Boost accuracy based on content type
  match request.content_type
  {
      TechnicalContentType::APIDocumentation if text_lower.contains( "api" ) || text_lower.contains( "endpoint" ) => accuracy += 0.2,
      TechnicalContentType::Tutorial if text_lower.contains( "step" ) || text_lower.contains( "guide" ) => accuracy += 0.2,
      TechnicalContentType::README if text_lower.contains( "install" ) || text_lower.contains( "usage" ) => accuracy += 0.2,
      _ => {}
  }

  if request.include_code_examples && ( text_lower.contains( "```" ) || text_lower.contains( "code" ) )
  {
      accuracy += 0.1;
  }

  accuracy.min( 1.0 )
  }

  /// Extract code examples from text
  fn extract_code_examples( text : &str ) -> Vec< String >
  {
  let mut examples = Vec::new();
  let lines = text.lines().collect::< Vec< _ > >();
  let mut in_code_block = false;
  let mut current_example = String::new();

  for line in lines
  {
      if line.starts_with( "```" )
      {
  if in_code_block && !current_example.trim().is_empty()
  {
          examples.push( current_example.trim().to_string() );
          current_example.clear();
  }
  in_code_block = !in_code_block;
      }
      else if in_code_block
      {
  if !current_example.is_empty() { current_example.push( '\n' ); }
  current_example.push_str( line );
      }
  }

  // Also look for inline code patterns
  if examples.is_empty()
  {
      for line in text.lines()
      {
  if line.contains( "def " ) || line.contains( "function " ) || line.contains( "fn " )
  {
          examples.push( line.trim().to_string() );
  }
      }
  }

  examples
  }

  /// Calculate language distribution statistics
  fn calculate_language_distribution( &self ) -> HashMap< ProgrammingLanguage, f32 >
  {
  let total_languages = self.language_models.len() as f32;
  let mut distribution = HashMap::new();

  for language in self.language_models.keys()
  {
      distribution.insert( *language, 1.0 / total_languages );
  }

  distribution
  }
}

/// Platform statistics
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct AssistantStats
{
  pub supported_languages : usize,
  pub available_templates : usize,
  pub active_review_rules : usize,
  pub language_distribution : HashMap< ProgrammingLanguage, f32 >,
}

