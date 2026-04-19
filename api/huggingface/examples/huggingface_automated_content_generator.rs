//! Automated Content Generation Platform
//!
//! This example demonstrates building an intelligent content generation system that creates
//! blogs, marketing materials, and creative writing using `HuggingFace` models. Features include
//! template-driven generation, multiple writing styles, quality assessment, and batch processing.
//!
//! ## Usage
//!
//! ```bash
//! export HUGGINGFACE_API_KEY="your-api-key-here"
//! cargo run --example automated_content_generator --features="full"
//! ```
//!
//! ## Commands
//!
//! - `/generate < type > < tone > < topic >` - Generate content with specified parameters
//! - `/template < name >` - Use a specific template for generation
//! - `/batch < count > < type > < topic >` - Generate multiple content pieces
//! - `/quality < text >` - Assess content quality
//! - `/export < format >` - Export last generated content
//! - `/templates` - List available templates
//! - `/types` - Show supported content types and tones
//! - `/help` - Show available commands
//! - `/quit` - Exit the content generator

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
use core::fmt::Write as FmtWrite;
use std::
{
  collections::HashMap,
  fmt,
  io::{ self, Write as IoWrite },
  time::{ Instant, SystemTime, UNIX_EPOCH },
  fs,
};

/// Content types supported by the generator
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
pub enum ContentType
{
  /// Blog posts and articles
  BlogPost,
  /// Marketing copy and advertisements
  Marketing,
  /// Creative writing and storytelling
  Creative,
  /// Social media posts
  SocialMedia,
  /// Technical documentation
  Technical,
  /// Email content
  Email,
}

impl ContentType
{
  /// Parse content type from string
  fn from_str( s : &str ) -> Option< Self >
  {
  match s.to_lowercase().as_str()
  {
      "blog" | "blogpost" | "article" => Some( Self::BlogPost ),
      "marketing" | "ad" | "advertisement" => Some( Self::Marketing ),
      "creative" | "story" | "fiction" => Some( Self::Creative ),
      "social" | "socialmedia" | "post" => Some( Self::SocialMedia ),
      "technical" | "tech" | "documentation" => Some( Self::Technical ),
      "email" | "mail" => Some( Self::Email ),
      _ => None,
  }
  }

  /// Get content type description
  fn description( self ) -> &'static str
  {
  match self
  {
      Self::BlogPost => "Blog posts and articles",
      Self::Marketing => "Marketing copy and advertisements",
      Self::Creative => "Creative writing and storytelling",
      Self::SocialMedia => "Social media posts",
      Self::Technical => "Technical documentation",
      Self::Email => "Email content",
  }
  }
}

impl fmt::Display for ContentType
{
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
  let name = match self
  {
      Self::BlogPost => "blogpost",
      Self::Marketing => "marketing",
      Self::Creative => "creative",
      Self::SocialMedia => "social",
      Self::Technical => "technical",
      Self::Email => "email",
  };
  write!( f, "{name}" )
  }
}

/// Writing tones and styles
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
pub enum ContentTone
{
  /// Professional and formal
  Professional,
  /// Casual and conversational  
  Casual,
  /// Enthusiastic and energetic
  Enthusiastic,
  /// Humorous and light-hearted
  Humorous,
  /// Authoritative and expert
  Authoritative,
  /// Empathetic and caring
  Empathetic,
}

impl ContentTone
{
  /// Parse tone from string
  fn from_str( s : &str ) -> Option< Self >
  {
  match s.to_lowercase().as_str()
  {
      "professional" | "formal" => Some( Self::Professional ),
      "casual" | "conversational" => Some( Self::Casual ),
      "enthusiastic" | "energetic" => Some( Self::Enthusiastic ),
      "humorous" | "funny" | "witty" => Some( Self::Humorous ),
      "authoritative" | "expert" => Some( Self::Authoritative ),
      "empathetic" | "caring" => Some( Self::Empathetic ),
      _ => None,
  }
  }

  /// Get tone instruction for prompt
  fn instruction( self ) -> &'static str
  {
  match self
  {
      Self::Professional => "Write in a professional, formal tone.",
      Self::Casual => "Write in a casual, conversational tone.",
      Self::Enthusiastic => "Write with enthusiasm and energy.",
      Self::Humorous => "Write with humor and wit.",
      Self::Authoritative => "Write with authority and expertise.",
      Self::Empathetic => "Write with empathy and understanding.",
  }
  }
}

impl fmt::Display for ContentTone
{
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
  let name = match self
  {
      Self::Professional => "professional",
      Self::Casual => "casual",
      Self::Enthusiastic => "enthusiastic",
      Self::Humorous => "humorous",
      Self::Authoritative => "authoritative",
      Self::Empathetic => "empathetic",
  };
  write!( f, "{name}" )
  }
}

/// Content length preferences
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
pub enum ContentLength
{
  /// Short content (50-150 words)
  Short,
  /// Medium content (150-400 words)
  Medium,
  /// Long content (400-800 words)
  Long,
  /// Extended content (800+ words)
  Extended,
}

impl ContentLength
{
  /// Parse length from string
  #[ allow( dead_code ) ]
  fn from_str( s : &str ) -> Option< Self >
  {
  match s.to_lowercase().as_str()
  {
      "short" | "brief" => Some( Self::Short ),
      "medium" | "standard" => Some( Self::Medium ),
      "long" | "detailed" => Some( Self::Long ),
      "extended" | "comprehensive" => Some( Self::Extended ),
      _ => None,
  }
  }

  /// Get length instruction for prompt
  fn instruction( self ) -> &'static str
  {
  match self
  {
      Self::Short => "Keep it concise (50-150 words).",
      Self::Medium => "Write a moderate length piece (150-400 words).",
      Self::Long => "Create a comprehensive piece (400-800 words).",
      Self::Extended => "Write an in-depth, detailed piece (800+ words).",
  }
  }

  /// Get target word count range
  fn word_range( self ) -> ( usize, usize )
  {
  match self
  {
      Self::Short => ( 50, 150 ),
      Self::Medium => ( 150, 400 ),
      Self::Long => ( 400, 800 ),
      Self::Extended => ( 800, 1200 ),
  }
  }

  /// Get max tokens for this length
  fn max_tokens( self ) -> usize
  {
  match self
  {
      Self::Short => 200,
      Self::Medium => 500,
      Self::Long => 1000,
      Self::Extended => 1500,
  }
  }
}

/// Content generation template
#[ derive( Debug, Clone ) ]
pub struct ContentTemplate
{
  /// Template name
  pub name : String,
  /// Content type
  pub content_type : ContentType,
  /// Template structure with placeholders
  pub template : String,
  /// Available variables for substitution
  pub variables : Vec< String >,
  /// Suggested tone for this template
  pub default_tone : ContentTone,
}

/// Content generation request  
#[ derive( Debug, Clone ) ]
pub struct ContentRequest
{
  /// Type of content to generate
  pub content_type : ContentType,
  /// Desired tone and style
  pub tone : ContentTone,
  /// Topic or subject matter
  pub topic : String,
  /// Target audience
  pub target_audience : Option< String >,
  /// Content length preference
  pub length : ContentLength,
  /// Additional context or requirements
  pub context : Option< String >,
  /// Template to use (optional)
  pub template : Option< ContentTemplate >,
}

impl Default for ContentRequest
{
  fn default() -> Self
  {
  Self
  {
      content_type : ContentType::BlogPost,
      tone : ContentTone::Professional,
      topic : String::new(),
      target_audience : None,
      length : ContentLength::Medium,
      context : None,
      template : None,
  }
  }
}

/// Generated content with metadata
#[ derive( Debug, Clone ) ]
pub struct GeneratedContent
{
  /// The generated text
  pub text : String,
  /// Content type
  pub content_type : ContentType,
  /// Applied tone
  pub tone : ContentTone,
  /// Original topic
  pub topic : String,
  /// Quality score (0.0 to 1.0)
  pub quality_score : f32,
  /// Word count
  pub word_count : usize,
  /// Generation timestamp
  pub generated_at : Instant,
  /// Model used for generation
  pub model : String,
}

/// Content quality metrics
#[ derive( Debug, Clone ) ]
pub struct QualityMetrics
{
  /// Overall quality score (0.0 to 1.0)
  pub overall_score : f32,
  /// Readability score (0.0 to 1.0)
  pub readability : f32,
  /// Coherence score (0.0 to 1.0)
  pub coherence : f32,
  /// Creativity score (0.0 to 1.0)
  pub creativity : f32,
  /// Relevance to topic (0.0 to 1.0)
  pub relevance : f32,
  /// Tone appropriateness (0.0 to 1.0)
  pub tone_match : f32,
}

/// Advanced content generation platform
#[ derive( Debug ) ]
pub struct ContentGenerationPlatform
{
  client : Client< HuggingFaceEnvironmentImpl >,
  templates : HashMap< String, ContentTemplate >,
  default_models : HashMap< ContentType, String >,
  generation_stats : GenerationStats,
}

/// Platform statistics
#[ derive( Debug, Clone ) ]
pub struct GenerationStats
{
  /// Total content pieces generated
  pub total_generated : usize,
  /// Generation by content type
  pub by_content_type : HashMap< ContentType, usize >,
  /// Generation by tone
  pub by_tone : HashMap< ContentTone, usize >,
  /// Average quality score
  pub avg_quality : f32,
  /// Total words generated
  pub total_words : usize,
}

impl Default for GenerationStats
{
  fn default() -> Self
  {
  Self
  {
      total_generated : 0,
      by_content_type : HashMap::new(),
      by_tone : HashMap::new(),
      avg_quality : 0.0,
      total_words : 0,
  }
  }
}

impl ContentGenerationPlatform
{
  /// Create new content generation platform
  #[ must_use ]
  pub fn new( client : Client< HuggingFaceEnvironmentImpl > ) -> Self
  {
  let mut default_models = HashMap::new();
  default_models.insert( ContentType::BlogPost, Models::llama_3_3_70b_instruct().to_string() );
  default_models.insert( ContentType::Marketing, Models::llama_3_3_70b_instruct().to_string() );
  default_models.insert( ContentType::Creative, Models::llama_3_3_70b_instruct().to_string() );
  default_models.insert( ContentType::SocialMedia, Models::mistral_7b_instruct().to_string() );
  default_models.insert( ContentType::Technical, Models::code_llama_7b_instruct().to_string() );
  default_models.insert( ContentType::Email, Models::mistral_7b_instruct().to_string() );

  let mut platform = Self
  {
      client,
      templates : HashMap::new(),
      default_models,
      generation_stats : GenerationStats::default(),
  };

  // Add default templates
  platform.add_default_templates();
  platform
  }

  /// Add default content templates
  fn add_default_templates( &mut self )
  {
  // Blog post template
  self.add_template( ContentTemplate
  {
      name : "standard_blog".to_string(),
      content_type : ContentType::BlogPost,
      template : "Write a compelling blog post about {topic}. Start with an engaging introduction, develop the main points with examples, and conclude with actionable insights.".to_string(),
      variables : vec![ "topic".to_string() ],
      default_tone : ContentTone::Professional,
  } );

  // Marketing template
  self.add_template( ContentTemplate
  {
      name : "marketing_copy".to_string(),
      content_type : ContentType::Marketing,
      template : "Create persuasive marketing copy for {topic}. Highlight key benefits, address pain points, and include a strong call to action.".to_string(),
      variables : vec![ "topic".to_string() ],
      default_tone : ContentTone::Enthusiastic,
  } );

  // Social media template
  self.add_template( ContentTemplate
  {
      name : "social_post".to_string(),
      content_type : ContentType::SocialMedia,
      template : "Create an engaging social media post about {topic}. Make it shareable and include relevant hashtags.".to_string(),
      variables : vec![ "topic".to_string() ],
      default_tone : ContentTone::Casual,
  } );
  }

  /// Add content template to the platform
  pub fn add_template( &mut self, template : ContentTemplate )
  {
  self.templates.insert( template.name.clone(), template );
  }

  /// Generate content based on request
  ///
  /// # Errors
  ///
  /// Returns an error if the content generation fails or if no model is configured for the content type.
  pub async fn generate_content( &mut self, request : ContentRequest ) -> Result< GeneratedContent, Box< dyn std::error::Error > >
  {
  let generation_start = Instant::now();
  
  // Build prompt from request
  let prompt = Self::build_prompt( &request );
  
  // Get appropriate model for content type
  let model = self.default_models.get( &request.content_type )
      .ok_or( "No model configured for content type" )?;

  // Configure generation parameters based on content type and tone
  let parameters = Self::get_generation_parameters( &request );

  // Generate content
  let response = self.client
      .inference()
      .create_with_parameters( &prompt, model, parameters )
      .await?;

  let generated_text = response.extract_text_or_default( "Sorry, I couldn't generate content." );

  // Clean up the generated content
  let cleaned_text = Self::clean_generated_content( &generated_text, &request );
  let word_count = cleaned_text.split_whitespace().count();

  // Assess content quality
  let quality_score = Self::assess_content_quality( &cleaned_text, &request );

  let generated_content = GeneratedContent
  {
      text : cleaned_text,
      content_type : request.content_type,
      tone : request.tone,
      topic : request.topic.clone(),
      quality_score,
      word_count,
      generated_at : generation_start,
      model : model.clone(),
  };

  // Update statistics
  self.update_stats( &generated_content );

  Ok( generated_content )
  }

  /// Build generation prompt from request
  fn build_prompt( request : &ContentRequest ) -> String
  {
  let tone_instruction = request.tone.instruction();
  
  let content_instruction = match request.content_type
  {
      ContentType::BlogPost => "Create an engaging blog post",
      ContentType::Marketing => "Create compelling marketing copy",
      ContentType::Creative => "Write creative and imaginative content",
      ContentType::SocialMedia => "Create engaging social media content",
      ContentType::Technical => "Write clear technical documentation",
      ContentType::Email => "Compose a professional email",
  };

  let length_instruction = request.length.instruction();

  let mut prompt = format!( "{content_instruction} about '{}'.  {tone_instruction} {length_instruction}",
      request.topic, 
  );

  // Add audience information if provided
  if let Some( audience ) = &request.target_audience
  {
      let _ = write!( &mut prompt, " Target audience : {audience}." );
  }

  // Add additional context if provided
  if let Some( context ) = &request.context
  {
      let _ = write!( &mut prompt, " Additional context : {context}" );
  }

  // Use template if provided
  if let Some( template ) = &request.template
  {
      prompt = template.template.replace( "{topic}", &request.topic );
  }

  prompt
  }

  /// Get generation parameters based on request
  fn get_generation_parameters( request : &ContentRequest ) -> InferenceParameters
  {
  let max_tokens = request.length.max_tokens();
  
  let temperature = match request.tone
  {
      ContentTone::Humorous => 0.9,
      ContentTone::Professional => 0.3,
      ContentTone::Enthusiastic => 0.8,
      _ => 0.7,
  };

  InferenceParameters::new()
      .with_max_new_tokens( max_tokens.try_into().unwrap_or( u32::MAX ) )
      .with_temperature( temperature )
      .with_top_p( 0.9 )
  }

  /// Clean generated content
  fn clean_generated_content( text : &str, _request : &ContentRequest ) -> String
  {
  let mut cleaned = text.trim().to_string();
  
  // Remove common prompt echoes
  let prefixes_to_remove = [
      "Create an engaging blog post about",
      "Create compelling marketing copy",
      "Write creative and imaginative content",
      "Create engaging social media content",
      "Write clear technical documentation",
      "Compose a professional email",
  ];
  
  for prefix in &prefixes_to_remove
  {
      if let Some( pos ) = cleaned.find( prefix )
      {
  if let Some( end_pos ) = cleaned[ pos.. ].find( '.' ).map( |p| p + pos + 1 )
  {
          cleaned = cleaned[ end_pos.. ].trim().to_string();
  }
      }
  }
  
  // Remove excessive newlines
  while cleaned.contains( "\n\n\n" )
  {
      cleaned = cleaned.replace( "\n\n\n", "\n\n" );
  }
  
  cleaned
  }

  /// Assess content quality (simplified implementation)
  fn assess_content_quality( content : &str, request : &ContentRequest ) -> f32
  {
  let mut score : f32 = 0.5; // Base score

  // Length appropriateness
  let word_count = content.split_whitespace().count();
  let ( min_words, max_words ) = request.length.word_range();
  
  if word_count >= min_words && word_count <= max_words
  {
      score += 0.2;
  }
  else if word_count >= min_words / 2 && word_count <= max_words * 2
  {
      score += 0.1;
  }

  // Content structure (simple heuristics)
  if content.contains( '\n' ) // Has paragraphs
  {
      score += 0.1;
  }
  
  if content.ends_with( '.' ) || content.ends_with( '!' ) || content.ends_with( '?' )
  {
      score += 0.1;
  }

  // Avoid very short sentences as they might indicate truncation
  if content.len() > 50
  {
      score += 0.1;
  }

  score.min( 1.0 )
  }

  /// Generate multiple content pieces
  ///
  /// # Errors
  ///
  /// Returns an error if batch generation fails. Individual content generation errors are logged but don't stop the batch process.
  pub async fn generate_batch(
  &mut self,
  request : ContentRequest,
  count : usize
  ) -> Result< Vec< GeneratedContent >, Box< dyn std::error::Error > >
  {
  let mut results = Vec::new();
  
  for i in 0..count
  {
      // Add variation to topics for batch generation
      let mut varied_request = request.clone();
      if count > 1
      {
  varied_request.topic = format!( "{} (variation {})", request.topic, i + 1 );
      }
      
      match self.generate_content( varied_request ).await
      {
  Ok( content ) => results.push( content ),
  Err( e ) => eprintln!( "Failed to generate content #{}: {e}", i + 1 ),
      }
  }
  
  Ok( results )
  }

  /// Update generation statistics
  fn update_stats( &mut self, content : &GeneratedContent )
  {
  self.generation_stats.total_generated += 1;
  self.generation_stats.total_words += content.word_count;
  
  *self.generation_stats.by_content_type.entry( content.content_type ).or_insert( 0 ) += 1;
  *self.generation_stats.by_tone.entry( content.tone ).or_insert( 0 ) += 1;
  
  // Update rolling average quality
  let total = self.generation_stats.total_generated as f32;
  self.generation_stats.avg_quality = 
      ( self.generation_stats.avg_quality * ( total - 1.0 ) + content.quality_score ) / total;
  }

  /// Get platform statistics
  #[ must_use ]
  pub fn get_stats( &self ) -> &GenerationStats
  {
  &self.generation_stats
  }

  /// List available templates
  #[ must_use ]
  pub fn list_templates( &self ) -> Vec< &ContentTemplate >
  {
  self.templates.values().collect()
  }

  /// Get template by name
  #[ must_use ]
  pub fn get_template( &self, name : &str ) -> Option< &ContentTemplate >
  {
  self.templates.get( name )
  }
}

/// Interactive CLI for content generation
#[ derive( Debug ) ]
pub struct ContentGeneratorCLI
{
  platform : ContentGenerationPlatform,
  last_generated : Option< GeneratedContent >,
}

impl ContentGeneratorCLI
{
  /// Create new content generator CLI
  #[ must_use ]
  pub fn new( client : Client< HuggingFaceEnvironmentImpl > ) -> Self
  {
  Self
  {
      platform : ContentGenerationPlatform::new( client ),
      last_generated : None,
  }
  }

  /// Start interactive CLI session
  ///
  /// # Errors
  ///
  /// Returns an error if the CLI session fails to start or encounters I/O errors.
  pub async fn start( &mut self ) -> Result< (), Box< dyn std::error::Error > >
  {
  println!( "✍️ Automated Content Generation Platform" );
  println!( "========================================" );
  println!( "Type '/help' for commands or start generating content!" );
  println!( "Supported types : blog, marketing, creative, social, technical, email" );
  println!( "Supported tones : professional, casual, enthusiastic, humorous, authoritative, empathetic" );
  println!();

  let stdin = io::stdin();
  let mut stdout = io::stdout();

  loop
  {
      print!( "generate > " );
      stdout.flush()?;

      let mut input = String::new();
      stdin.read_line( &mut input )?;
      let input = input.trim();

      if input.is_empty()
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

      // Handle quick generation : "type tone topic"
      let parts : Vec< &str > = input.splitn( 3, ' ' ).collect();
      if parts.len() >= 3
      {
  if let ( Some( content_type ), Some( tone ) ) = ( 
          ContentType::from_str( parts[ 0 ] ),
          ContentTone::from_str( parts[ 1 ] )
  )
  {
          let topic = parts[ 2 ].to_string();
          let request = ContentRequest
          {
      content_type,
      tone,
      topic : topic.clone(),
      length : ContentLength::Medium,
      ..ContentRequest::default()
          };

          println!( "🔄 Generating {content_type} content with {tone} tone about '{topic}'..." );
          
          match self.platform.generate_content( request ).await
          {
      Ok( content ) => 
      {
              println!( "\n✅ Generated Content:" );
              println!( "===================" );
              println!( "{}", content.text );
              println!( "\n📊 Stats : {} words | Quality : {:.2} | Model : {}", 
        content.word_count, content.quality_score, content.model );
              
              self.last_generated = Some( content );
      },
      Err( e ) => println!( "❌ Generation failed : {e}" ),
          }
  }
  else
  {
          println!( "❌ Invalid type or tone. Type '/types' to see available options." );
  }
      }
      else
      {
  println!( "💡 Usage : < type > < tone > < topic >" );
  println!( "Example : blog professional 'artificial intelligence trends'" );
  println!( "Or use commands like '/generate blog professional AI trends'" );
      }

      println!();
  }
  }

  /// Handle CLI commands
  ///
  /// # Errors
  ///
  /// Returns an error if command execution fails or if invalid parameters are provided.
  #[ allow( clippy::too_many_lines ) ]
  async fn handle_command( &mut self, command : &str ) -> Result< Option< String >, Box< dyn std::error::Error > >
  {
  let parts : Vec< &str > = command[ 1.. ].splitn( 2, ' ' ).collect();
  
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
      
      "generate" =>
      {
  if parts.len() < 2
  {
          return Ok( Some( "Usage : /generate < type > < tone > < topic >".to_string() ) );
  }
  
  let gen_parts : Vec< &str > = parts[ 1 ].splitn( 3, ' ' ).collect();
  if gen_parts.len() < 3
  {
          return Ok( Some( "Usage : /generate < type > < tone > < topic >".to_string() ) );
  }
  
  if let ( Some( content_type ), Some( tone ) ) = ( 
          ContentType::from_str( gen_parts[ 0 ] ),
          ContentTone::from_str( gen_parts[ 1 ] )
  )
  {
          let topic = gen_parts[ 2 ].to_string();
          let request = ContentRequest
          {
      content_type,
      tone,
      topic : topic.clone(),
      length : ContentLength::Medium,
      ..ContentRequest::default()
          };

          println!( "🔄 Generating {content_type} content with {tone} tone about '{topic}'..." );
          
          match self.platform.generate_content( request ).await
          {
      Ok( content ) => 
      {
              self.last_generated = Some( content.clone() );
              Ok( Some( format!(
        "✅ Generated Content:\n{}\n\n📊 Stats : {} words | Quality : {:.2} | Model : {}",
        content.text, content.word_count, content.quality_score, content.model
              ) ) )
      },
      Err( e ) => Ok( Some( format!( "❌ Generation failed : {e}" ) ) ),
          }
  }
  else
  {
          Ok( Some( "❌ Invalid type or tone. Use '/types' to see options.".to_string() ) )
  }
      },
      
      "batch" =>
      {
  if parts.len() < 2
  {
          return Ok( Some( "Usage : /batch < count > < type > < topic >".to_string() ) );
  }
  
  let batch_parts : Vec< &str > = parts[ 1 ].splitn( 3, ' ' ).collect();
  if batch_parts.len() < 3
  {
          return Ok( Some( "Usage : /batch < count > < type > < topic >".to_string() ) );
  }
  
  let count : usize = batch_parts[ 0 ].parse()
          .map_err( |_| "Invalid count number" )?;
          
  if count > 10
  {
          return Ok( Some( "Maximum batch size is 10.".to_string() ) );
  }
  
  if let Some( content_type ) = ContentType::from_str( batch_parts[ 1 ] )
  {
          let topic = batch_parts[ 2 ].to_string();
          let request = ContentRequest
          {
      content_type,
      tone : ContentTone::Professional,
      topic : topic.clone(),
      length : ContentLength::Medium,
      ..ContentRequest::default()
          };

          println!( "🔄 Generating {count} {content_type} pieces about '{topic}'..." );
          
          match self.platform.generate_batch( request, count ).await
          {
      Ok( contents ) => 
      {
              let mut result = format!( "✅ Generated {} content pieces:\n\n", contents.len() );
              for ( i, content ) in contents.iter().enumerate()
              {
        let _ = writeln!( &mut result, "=== Piece {} ===\n{}\n", i + 1, content.text );
              }
              Ok( Some( result ) )
      },
      Err( e ) => Ok( Some( format!( "❌ Batch generation failed : {e}" ) ) ),
          }
  }
  else
  {
          Ok( Some( "❌ Invalid content type. Use '/types' to see options.".to_string() ) )
  }
      },
      
      "templates" =>
      {
  let templates = self.platform.list_templates();
  if templates.is_empty()
  {
          Ok( Some( "No templates available.".to_string() ) )
  }
  else
  {
          let mut result = "📝 Available Templates:\n\n".to_string();
          for template in templates
          {
      let _ = writeln!( &mut result,
              "• {} ({}) - {}\n  Default tone : {}\n",
              template.name, template.content_type,
              template.content_type.description(), template.default_tone
      );
          }
          Ok( Some( result ) )
  }
      },
      
      "types" =>
      {
  let mut result = "📋 Content Types:\n".to_string();
  for content_type in [ ContentType::BlogPost, ContentType::Marketing, ContentType::Creative,
                             ContentType::SocialMedia, ContentType::Technical, ContentType::Email ]
  {
          let _ = writeln!( &mut result, "• {} - {}", content_type, content_type.description() );
  }

  result.push_str( "\n🎭 Content Tones:\n" );
  for tone in [ ContentTone::Professional, ContentTone::Casual, ContentTone::Enthusiastic,
                     ContentTone::Humorous, ContentTone::Authoritative, ContentTone::Empathetic ]
  {
          let _ = writeln!( &mut result, "• {} - {}", tone, tone.instruction() );
  }
  
  Ok( Some( result ) )
      },
      
      "export" =>
      {
  if let Some( content ) = &self.last_generated
  {
          let format = parts.get( 1 ).unwrap_or( &"txt" );
          let filename = Self::export_content( content, format )?;
          Ok( Some( format!( "✅ Content exported to : {filename}" ) ) )
  }
  else
  {
          Ok( Some( "No content to export. Generate something first!".to_string() ) )
  }
      },
      
      "stats" =>
      {
  let stats = self.platform.get_stats();
  let result = format!(
          "📊 Generation Statistics:\n\
           Total Generated : {}\n\
           Total Words : {}\n\
           Average Quality : {:.2}\n\
           \n\
           By Content Type:\n{}\n\
           By Tone:\n{}",
          stats.total_generated,
          stats.total_words,
          stats.avg_quality,
          stats.by_content_type.iter()
      .map( |( t, c )| format!( "  • {t}: {c}" ) )
      .collect::< Vec< _ > >()
      .join( "\n" ),
          stats.by_tone.iter()
      .map( |( t, c )| format!( "  • {t}: {c}" ) )
      .collect::< Vec< _ > >()
      .join( "\n" )
  );
  Ok( Some( result ) )
      },
      
      _ => Ok( Some( format!( "Unknown command : /{}\nType '/help' for available commands.", parts[ 0 ] ) ) ),
  }
  }

  /// Export content to file
  ///
  /// # Errors
  ///
  /// Returns an error if file writing fails.
  fn export_content( content : &GeneratedContent, format : &str ) -> Result< String, Box< dyn std::error::Error > >
  {
  let timestamp = SystemTime::now()
      .duration_since( UNIX_EPOCH )?
      .as_secs();
  
  let filename = format!( "content_{}_{}.{}", content.content_type, timestamp, format );
  
  let export_content = match format
  {
      "md" | "markdown" => format!( 
  "# Generated Content\n\n**Type:** {}\n**Tone:** {}\n**Topic:** {}\n**Quality:** {:.2}\n**Words:** {}\n\n---\n\n{}\n",
  content.content_type, content.tone, content.topic, 
  content.quality_score, content.word_count, content.text
      ),
      "html" => format!(
  "<!DOCTYPE html >\n< html >\n< head >< title >{}</title ></head >\n< body >\n< h1 >Generated Content</h1 >\n< p >< strong >Type:</strong > {}</p >\n< p >< strong >Tone:</strong > {}</p >\n< p >< strong >Topic:</strong > {}</p >\n< hr >\n< div >{}</div >\n</body >\n</html >\n",
  content.topic, content.content_type, content.tone, content.topic,
  content.text.replace( '\n', "< br >\n" )
      ),
      _ => format!( 
  "Generated Content\n================\nType : {}\nTone : {}\nTopic : {}\nQuality : {:.2}\nWords : {}\n\n{}\n",
  content.content_type, content.tone, content.topic, 
  content.quality_score, content.word_count, content.text
      ),
  };
  
  fs::write( &filename, export_content )?;
  Ok( filename )
  }

  /// Show help information
  fn show_help() -> String
  {
  r#"Available Commands:
===================

/generate < type > < tone > < topic > - Generate content with specified parameters
/batch < count > < type > < topic >   - Generate multiple content pieces (max 10)
/templates                      - List available content templates
/types                          - Show supported content types and tones
/export < format >                - Export last generated content (txt, md, html)
/stats                          - Show generation statistics
/help                           - Show this help message
/quit or /exit                  - Exit the content generator

Quick Generation:
=================

Type : < type > < tone > < topic >
Example : blog professional "sustainable energy solutions"

Content Types:
• blog, marketing, creative, social, technical, email

Content Tones:
• professional, casual, enthusiastic, humorous, authoritative, empathetic

Examples:
• marketing enthusiastic "new fitness app"
• creative humorous "office life during remote work"
• technical professional "microservices architecture patterns""#.to_string()
  }
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  // Load API key from environment or workspace secrets
  let api_key = std::env::var("HUGGINGFACE_API_KEY")
  .or_else(|_| {
      use workspace_tools as workspace;
      let workspace = workspace::workspace()
  .map_err(|_| std::env::VarError::NotPresent)?; // Convert WorkspaceError
      let secrets = workspace.load_secrets_from_file("-secrets.sh")
  .map_err(|_| std::env::VarError::NotPresent)?; // Convert WorkspaceError
      secrets.get("HUGGINGFACE_API_KEY")
  .cloned()
  .ok_or(std::env::VarError::NotPresent)
  })
  .map_err(|_| "HUGGINGFACE_API_KEY not found in environment or workspace secrets")?;

  // Build client
  let secret_key = Secret::new( api_key );
  let environment = HuggingFaceEnvironmentImpl::build( secret_key, None )?;
  let client = Client::build( environment )?;

  // Start interactive content generator CLI
  let mut cli = ContentGeneratorCLI::new( client );
  cli.start().await?;

  Ok( () )
}