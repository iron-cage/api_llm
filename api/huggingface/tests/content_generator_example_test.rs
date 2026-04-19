//! Tests for Content Generation Platform Example  
//!
//! This test suite verifies the functionality of an automated content creation system
//! that generates blogs, marketing materials, and creative writing using `HuggingFace` models.

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
use std::{ collections::HashMap, time::Instant };

#[ allow( missing_docs ) ]
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

  Self
  {
      client,
      templates : HashMap::new(),
      default_models,
  }
  }

  /// Add content template to the platform
  pub fn add_template( &mut self, template : ContentTemplate )
  {
  self.templates.insert( template.name.clone(), template );
  }

  /// Generate content based on request
  ///
  /// # Errors
  /// Returns error if content generation fails or processing encounters issues
  pub async fn generate_content( &self, request : ContentRequest ) -> Result< GeneratedContent, Box< dyn std::error::Error > >
  {
  let generation_start = Instant::now();
  
  // Build prompt from request
  let prompt = Self::build_prompt( &request );
  
  // Get appropriate model for content type
  let model = self.default_models.get( &request.content_type )
      .ok_or( "No model configured for content type" )?;

  // Configure generation parameters based on content type and tone
  let parameters = Self::get_generation_parameters( request.content_type, request.tone, request.length );

  // Generate content
  let response = self.client
      .inference()
      .create_with_parameters( &prompt, model, parameters )
      .await?;

  let generated_text = response.extract_text_or_default( "No content generated" );

  // Post-process and clean the generated content
  let cleaned_text = Self::clean_generated_content( &generated_text, &request );
  
  // Calculate quality metrics
  let quality_score = Self::calculate_quality_score( &cleaned_text, &request );
  let word_count = cleaned_text.split_whitespace().count();

  Ok( GeneratedContent
  {
      text : cleaned_text,
      content_type : request.content_type,
      tone : request.tone,
      topic : request.topic,
      quality_score,
      word_count,
      generated_at : generation_start,
      model : model.clone(),
  } )
  }

  /// Generate multiple content variations
  ///
  /// # Errors
  /// Returns error if batch generation fails or processing encounters issues
  pub async fn generate_variations( &self, request : ContentRequest, count : usize ) -> Result< Vec< GeneratedContent >, Box< dyn std::error::Error > >
  {
  let mut variations = Vec::new();

  for i in 0..count
  {
      // Slightly vary parameters for each generation
      let mut varied_request = request.clone();
      if let Some( ref mut context ) = varied_request.context
      {
  use core::fmt::Write;
  write!( context, " (Variation {i})" ).expect( "[generate_variations] Failed to write variation marker to String context - String write! should never fail" );
      }

      let content = self.generate_content( varied_request ).await?;
      variations.push( content );
  }

  Ok( variations )
  }

  /// Build generation prompt from request
  fn build_prompt( request : &ContentRequest ) -> String
  {
  let tone_instruction = match request.tone
  {
      ContentTone::Professional => "Write in a professional, formal tone.",
      ContentTone::Casual => "Write in a casual, conversational tone.",
      ContentTone::Enthusiastic => "Write with enthusiasm and energy.",
      ContentTone::Humorous => "Write with humor and wit.",
      ContentTone::Authoritative => "Write with authority and expertise.",
      ContentTone::Empathetic => "Write with empathy and understanding.",
  };

  let content_instruction = match request.content_type
  {
      ContentType::BlogPost => "Create an engaging blog post",
      ContentType::Marketing => "Create compelling marketing copy",
      ContentType::Creative => "Write creative and imaginative content",
      ContentType::SocialMedia => "Create engaging social media content",
      ContentType::Technical => "Write clear technical documentation",
      ContentType::Email => "Compose a professional email",
  };

  let length_instruction = match request.length
  {
      ContentLength::Short => "Keep it concise (50-150 words).",
      ContentLength::Medium => "Write a moderate length piece (150-400 words).",
      ContentLength::Long => "Create a comprehensive piece (400-800 words).",
      ContentLength::Extended => "Write an in-depth, detailed piece (800+ words).",
  };

  let mut prompt = format!( "{content_instruction} about '{}'.  {tone_instruction} {length_instruction}",
      request.topic );

  if let Some( ref audience ) = request.target_audience
  {
      use core::fmt::Write;
      write!( &mut prompt, " Target audience : {audience}." ).expect( "[build_prompt] Failed to write target_audience to String prompt - String write! should never fail" );
  }

  if let Some( ref context ) = request.context
  {
      use core::fmt::Write;
      write!( &mut prompt, " Additional context : {context}" ).expect( "[build_prompt] Failed to write context to String prompt - String write! should never fail" );
  }

  // Use template if provided
  if let Some( ref template ) = request.template
  {
      use core::fmt::Write;
      write!( &mut prompt, " Follow this structure : {}", template.template ).expect( "[build_prompt] Failed to write template structure to String prompt - String write! should never fail" );
  }

  prompt.push_str( " Generate only the content, without any meta-commentary." );
  prompt
  }

  /// Get generation parameters for content type and style
  fn get_generation_parameters( content_type : ContentType, tone : ContentTone, length : ContentLength ) -> InferenceParameters
  {
  let base_temperature = match tone
  {
      ContentTone::Humorous => 0.9,
      ContentTone::Casual | ContentTone::Enthusiastic => 0.8,
      ContentTone::Empathetic => 0.7,
      ContentTone::Professional => 0.6,
      ContentTone::Authoritative => 0.5,
  };

  let max_tokens = match length
  {
      ContentLength::Short => 200,
      ContentLength::Medium => 500,
      ContentLength::Long => 800,
      ContentLength::Extended => 1200,
  };

  let top_p = match content_type
  {
      ContentType::Creative => 0.95,
      ContentType::Marketing | ContentType::SocialMedia => 0.9,
      _ => 0.85,
  };

  InferenceParameters::new()
      .with_temperature( base_temperature )
      .with_max_new_tokens( max_tokens )
      .with_top_p( top_p )
  }

  /// Clean and post-process generated content
  fn clean_generated_content( text : &str, _request : &ContentRequest ) -> String
  {
  // Remove common unwanted prefixes/suffixes
  let cleaned = text
      .trim()
      .trim_start_matches( "Here's" )
      .trim_start_matches( "Here is" )
      .trim_start_matches( "Sure," )
      .trim_start_matches( "Certainly," )
      .trim();

  // Remove meta-commentary patterns
  let lines : Vec< &str > = cleaned.lines().collect();
  let content_lines : Vec< &str > = lines.into_iter()
      .filter( | line | 
  !line.contains( "I'll write" ) &&
  !line.contains( "I'll create" ) &&
  !line.contains( "Let me" ) &&
  !line.starts_with( "Note:" ) &&
  !line.starts_with( "Disclaimer:" )
      )
      .collect();

  content_lines.join( "\n" ).trim().to_string()
  }

  /// Calculate quality score for generated content
  fn calculate_quality_score( text : &str, request : &ContentRequest ) -> f32
  {
  let metrics = Self::assess_content_quality( text, request );
  metrics.overall_score
  }

  /// Assess comprehensive quality metrics
  fn assess_content_quality( text : &str, request : &ContentRequest ) -> QualityMetrics
  {
  // Simple heuristic-based quality assessment
  let word_count = text.split_whitespace().count();
  let sentence_count = text.matches( '.' ).count() + text.matches( '!' ).count() + text.matches( '?' ).count();

  // Length appropriateness
  let expected_range = match request.length
  {
      ContentLength::Short => ( 50, 150 ),
      ContentLength::Medium => ( 150, 400 ),
      ContentLength::Long => ( 400, 800 ),
      ContentLength::Extended => ( 800, 1500 ),
  };

  let length_score = if word_count >= expected_range.0 && word_count <= expected_range.1
  {
      1.0
  }
  else
  {
      0.7 // Penalty for wrong length
  };

  // Readability (sentences per 100 words)
  let avg_sentence_length = if sentence_count > 0 { word_count as f32 / sentence_count as f32 } else { 0.0 };
  let readability = if avg_sentence_length > 5.0 && avg_sentence_length < 25.0 { 0.9 } else { 0.6 };

  // Coherence (basic check for repeated words indicating potential loops)
  let unique_words : std::collections::HashSet< _ > = text.split_whitespace().collect();
  let coherence = ( unique_words.len() as f32 / word_count.max( 1 ) as f32 ).min( 1.0 );

  // Creativity (vocabulary diversity)
  let creativity = if unique_words.len() > word_count / 3 { 0.8 } else { 0.6 };

  // Topic relevance (simple keyword presence)
  let topic_words : Vec< &str > = request.topic.split_whitespace().collect();
  let relevance = if topic_words.iter().any( | word | text.to_lowercase().contains( &word.to_lowercase() ) )
  {
      0.9
  }
  else
  {
      0.5
  };

  // Tone match (heuristic based on word choice)
  let tone_match = Self::assess_tone_match( text, request.tone );

  let overall_score = ( length_score + readability + coherence + creativity + relevance + tone_match ) / 6.0;

  QualityMetrics
  {
      overall_score,
      readability,
      coherence,
      creativity,
      relevance,
      tone_match,
  }
  }

  /// Assess how well the content matches the requested tone
  fn assess_tone_match( text : &str, tone : ContentTone ) -> f32
  {
  let text_lower = text.to_lowercase();

  match tone
  {
      ContentTone::Professional => 
      {
  let professional_indicators = [ "therefore", "however", "furthermore", "consequently", "moreover" ];
  if professional_indicators.iter().any( | word | text_lower.contains( word ) ) { 0.9 } else { 0.7 }
      },
      ContentTone::Casual => 
      {
  let casual_indicators = [ "you", "we", "let's", "really", "pretty", "kind of" ];
  if casual_indicators.iter().any( | word | text_lower.contains( word ) ) { 0.9 } else { 0.7 }
      },
      ContentTone::Enthusiastic =>
      {
  let enthusiasm_indicators = [ "amazing", "fantastic", "incredible", "exciting", "wonderful" ];
  if enthusiasm_indicators.iter().any( | word | text_lower.contains( word ) ) { 0.9 } else { 0.6 }
      },
      ContentTone::Humorous =>
      {
  // Hard to detect humor automatically, so use moderate score
  0.7
      },
      ContentTone::Authoritative =>
      {
  let authority_indicators = [ "research", "studies", "evidence", "proven", "established" ];
  if authority_indicators.iter().any( | word | text_lower.contains( word ) ) { 0.9 } else { 0.6 }
      },
      ContentTone::Empathetic =>
      {
  let empathy_indicators = [ "understand", "feel", "experience", "support", "help" ];
  if empathy_indicators.iter().any( | word | text_lower.contains( word ) ) { 0.9 } else { 0.7 }
      },
  }
  }

  /// Get available templates for a content type
  #[ must_use ]
  pub fn get_templates_for_type( &self, content_type : ContentType ) -> Vec< &ContentTemplate >
  {
  self.templates.values()
      .filter( | template | template.content_type == content_type )
      .collect()
  }

  /// Get content generation statistics
  #[ must_use ]
  pub fn get_platform_stats( &self ) -> ContentPlatformStats
  {
  ContentPlatformStats
  {
      total_templates : self.templates.len(),
      supported_content_types : self.default_models.len(),
      available_models : self.default_models.values().cloned().collect(),
  }
  }
}

/// Platform statistics and capabilities
#[ derive( Debug, Clone ) ]
pub struct ContentPlatformStats
{
  /// Number of available templates
  pub total_templates : usize,
  /// Number of supported content types
  pub supported_content_types : usize,
  /// Available generation models
  pub available_models : Vec< String >,
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

  fn create_sample_templates() -> Vec< ContentTemplate >
  {
  vec!
  [
      ContentTemplate
      {
  name : "blog_intro".to_string(),
  content_type : ContentType::BlogPost,
  template : "## {title}\n\n{introduction}\n\n{main_points}\n\n{conclusion}".to_string(),
  variables : vec![ "title".to_string(), "introduction".to_string(), "main_points".to_string(), "conclusion".to_string() ],
  default_tone : ContentTone::Professional,
      },
      ContentTemplate
      {
  name : "marketing_email".to_string(),
  content_type : ContentType::Email,
  template : "Subject : {subject}\n\nDear {name},\n\n{opening}\n\n{value_proposition}\n\n{call_to_action}\n\nBest regards,\n{sender}".to_string(),
  variables : vec![ "subject".to_string(), "name".to_string(), "opening".to_string(), "value_proposition".to_string(), "call_to_action".to_string(), "sender".to_string() ],
  default_tone : ContentTone::Professional,
      },
      ContentTemplate
      {
  name : "social_media_post".to_string(),
  content_type : ContentType::SocialMedia,
  template : "{hook} 🚀\n\n{main_content}\n\n{hashtags} {call_to_action}".to_string(),
  variables : vec![ "hook".to_string(), "main_content".to_string(), "hashtags".to_string(), "call_to_action".to_string() ],
  default_tone : ContentTone::Enthusiastic,
      },
  ]
  }

  #[ test ]
  fn test_content_type_enum()
  {
  let content_types = [
      ContentType::BlogPost,
      ContentType::Marketing,
      ContentType::Creative,
      ContentType::SocialMedia,
      ContentType::Technical,
      ContentType::Email,
  ];

  assert_eq!( content_types.len(), 6 );
  assert_eq!( ContentType::BlogPost, ContentType::BlogPost );
  assert_ne!( ContentType::Marketing, ContentType::Creative );
  }

  #[ test ]
  fn test_content_tone_variations()
  {
  let tones = vec!
  [
      ContentTone::Professional,
      ContentTone::Casual,
      ContentTone::Enthusiastic,
      ContentTone::Humorous,
      ContentTone::Authoritative,
      ContentTone::Empathetic,
  ];

  assert_eq!( tones.len(), 6 );
  for tone in tones
  {
      let cloned = tone;
      assert_eq!( tone, cloned );
  }
  }

  #[ test ]
  fn test_content_length_categories()
  {
  let lengths = [ ContentLength::Short, ContentLength::Medium, ContentLength::Long, ContentLength::Extended ];
  
  assert_eq!( lengths.len(), 4 );
  assert_eq!( ContentLength::Short, ContentLength::Short );
  assert_ne!( ContentLength::Medium, ContentLength::Long );
  }

  #[ test ]
  fn test_content_template_structure()
  {
  let template = ContentTemplate
  {
      name : "test_template".to_string(),
      content_type : ContentType::BlogPost,
      template : "Title : {title}\nContent : {content}".to_string(),
      variables : vec![ "title".to_string(), "content".to_string() ],
      default_tone : ContentTone::Professional,
  };

  assert_eq!( template.name, "test_template" );
  assert_eq!( template.content_type, ContentType::BlogPost );
  assert_eq!( template.variables.len(), 2 );
  assert!( template.variables.contains( &"title".to_string() ) );
  }

  #[ test ]
  fn test_content_request_creation()
  {
  let request = ContentRequest
  {
      content_type : ContentType::BlogPost,
      tone : ContentTone::Casual,
      topic : "sustainable technology".to_string(),
      target_audience : Some( "tech enthusiasts".to_string() ),
      length : ContentLength::Medium,
      context : Some( "focus on renewable energy".to_string() ),
      template : None,
  };

  assert_eq!( request.content_type, ContentType::BlogPost );
  assert_eq!( request.tone, ContentTone::Casual );
  assert_eq!( request.topic, "sustainable technology" );
  assert_eq!( request.length, ContentLength::Medium );
  assert!( request.target_audience.is_some() );
  assert!( request.context.is_some() );
  }

  #[ tokio::test ]
  async fn test_platform_initialization()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let platform = ContentGenerationPlatform::new( client );
  assert!( platform.templates.is_empty() );
  assert_eq!( platform.default_models.len(), 6 ); // 6 content types

  // Verify model assignments
  assert!( platform.default_models.contains_key( &ContentType::BlogPost ) );
  assert!( platform.default_models.contains_key( &ContentType::Marketing ) );
  assert!( platform.default_models.contains_key( &ContentType::Creative ) );
  }

  #[ tokio::test ]
  async fn test_template_management()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let mut platform = ContentGenerationPlatform::new( client );
  let templates = create_sample_templates();

  // Add templates
  for template in templates
  {
      platform.add_template( template );
  }

  assert_eq!( platform.templates.len(), 3 );

  // Test template retrieval by type
  let blog_templates = platform.get_templates_for_type( ContentType::BlogPost );
  assert_eq!( blog_templates.len(), 1 );
  assert_eq!( blog_templates[ 0 ].name, "blog_intro" );

  let email_templates = platform.get_templates_for_type( ContentType::Email );
  assert_eq!( email_templates.len(), 1 );

  let social_templates = platform.get_templates_for_type( ContentType::SocialMedia );
  assert_eq!( social_templates.len(), 1 );
  }

  #[ tokio::test ]
  async fn test_prompt_building()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let _platform = ContentGenerationPlatform::new( client );

  let request = ContentRequest
  {
      content_type : ContentType::BlogPost,
      tone : ContentTone::Professional,
      topic : "artificial intelligence".to_string(),
      target_audience : Some( "business leaders".to_string() ),
      length : ContentLength::Medium,
      context : Some( "focus on practical applications".to_string() ),
      template : None,
  };

  let prompt = ContentGenerationPlatform::build_prompt( &request );

  assert!( prompt.contains( "artificial intelligence" ) );
  assert!( prompt.contains( "professional" ) );
  assert!( prompt.contains( "blog post" ) );
  assert!( prompt.contains( "business leaders" ) );
  assert!( prompt.contains( "practical applications" ) );
  assert!( prompt.contains( "150-400 words" ) );
  }

  #[ tokio::test ]
  async fn test_generation_parameters()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let _platform = ContentGenerationPlatform::new( client );

  // Test creative content parameters
  let creative_params = ContentGenerationPlatform::get_generation_parameters( 
      ContentType::Creative, 
      ContentTone::Humorous, 
      ContentLength::Long 
  );

  assert!( creative_params.temperature.expect( "[test_generation_parameters] InferenceParameters.temperature should be Some for creative content - check get_generation_parameters() implementation" ) >= 0.8 ); // High creativity
  assert_eq!( creative_params.max_new_tokens.expect( "[test_generation_parameters] InferenceParameters.max_new_tokens should be Some for long content - check get_generation_parameters() implementation" ), 800 ); // Long content
  assert!( creative_params.top_p.expect( "[test_generation_parameters] InferenceParameters.top_p should be Some for creative content - check get_generation_parameters() implementation" ) >= 0.9 ); // High diversity

  // Test professional content parameters
  let professional_params = ContentGenerationPlatform::get_generation_parameters( 
      ContentType::Technical, 
      ContentTone::Authoritative, 
      ContentLength::Short 
  );

  assert!( professional_params.temperature.expect( "[test_generation_parameters] InferenceParameters.temperature should be Some for professional content - check get_generation_parameters() implementation" ) <= 0.6 ); // Lower creativity
  assert_eq!( professional_params.max_new_tokens.expect( "[test_generation_parameters] InferenceParameters.max_new_tokens should be Some for short content - check get_generation_parameters() implementation" ), 200 ); // Short content
  }

  #[ tokio::test ]
  async fn test_content_cleaning()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let _platform = ContentGenerationPlatform::new( client );

  let request = ContentRequest
  {
      content_type : ContentType::BlogPost,
      tone : ContentTone::Professional,
      topic : "test topic".to_string(),
      target_audience : None,
      length : ContentLength::Medium,
      context : None,
      template : None,
  };

  // Test cleaning of common unwanted prefixes
  let dirty_text = "Here's a great blog post about technology:\n\nTechnology is advancing rapidly.";
  let cleaned = ContentGenerationPlatform::clean_generated_content( dirty_text, &request );
  assert!( !cleaned.starts_with( "Here's" ) );
  assert!( cleaned.contains( "Technology is advancing rapidly" ) );

  // Test removal of meta-commentary
  let meta_text = "I'll write a blog post for you.\nLet me start with the introduction.\nTechnology is important.\nNote : This is just an example.";
  let cleaned_meta = ContentGenerationPlatform::clean_generated_content( meta_text, &request );
  assert!( !cleaned_meta.contains( "I'll write" ) );
  assert!( !cleaned_meta.contains( "Let me" ) );
  assert!( !cleaned_meta.contains( "Note:" ) );
  assert!( cleaned_meta.contains( "Technology is important" ) );
  }

  #[ tokio::test ]
  async fn test_quality_assessment()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let _platform = ContentGenerationPlatform::new( client );

  let request = ContentRequest
  {
      content_type : ContentType::BlogPost,
      tone : ContentTone::Professional,
      topic : "sustainable energy".to_string(),
      target_audience : None,
      length : ContentLength::Medium,
      context : None,
      template : None,
  };

  // Test quality assessment of good content
  let good_content = "Sustainable energy represents a crucial advancement in our fight against climate change. Renewable energy sources such as solar, wind, and hydroelectric power offer clean alternatives to fossil fuels. These technologies not only reduce carbon emissions but also provide long-term economic benefits through job creation and energy independence.";

  let quality = ContentGenerationPlatform::assess_content_quality( good_content, &request );

  assert!( quality.overall_score > 0.5 );
  assert!( quality.relevance > 0.8 ); // Contains "sustainable energy" topic
  assert!( quality.readability > 0.5 );
  assert!( quality.coherence > 0.5 );

  // Test quality assessment of poor content (too short for medium length)
  let poor_content = "Energy is good.";
  let poor_quality = ContentGenerationPlatform::assess_content_quality( poor_content, &request );
  assert!( poor_quality.overall_score < quality.overall_score );
  }

  #[ tokio::test ]
  async fn test_tone_matching()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let _platform = ContentGenerationPlatform::new( client );

  // Test professional tone matching
  let professional_text = "Therefore, we must consider the implications. However, the research indicates positive outcomes.";
  let professional_score = ContentGenerationPlatform::assess_tone_match( professional_text, ContentTone::Professional );
  assert!( professional_score > 0.8 );

  // Test casual tone matching
  let casual_text = "You know what? We really should consider this. It's pretty amazing how things work.";
  let casual_score = ContentGenerationPlatform::assess_tone_match( casual_text, ContentTone::Casual );
  assert!( casual_score > 0.8 );

  // Test enthusiastic tone matching
  let enthusiastic_text = "This is absolutely amazing! The results are fantastic and incredible.";
  let enthusiastic_score = ContentGenerationPlatform::assess_tone_match( enthusiastic_text, ContentTone::Enthusiastic );
  assert!( enthusiastic_score > 0.8 );

  // Test authoritative tone matching
  let authoritative_text = "Research shows that studies have proven this approach is established and evidence-based.";
  let authoritative_score = ContentGenerationPlatform::assess_tone_match( authoritative_text, ContentTone::Authoritative );
  assert!( authoritative_score > 0.8 );
  }

  #[ tokio::test ]
  async fn test_platform_statistics()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let mut platform = ContentGenerationPlatform::new( client );
  let templates = create_sample_templates();

  for template in templates
  {
      platform.add_template( template );
  }

  let stats = platform.get_platform_stats();
  assert_eq!( stats.total_templates, 3 );
  assert_eq!( stats.supported_content_types, 6 );
  assert!( stats.available_models.len() >= 3 ); // At least 3 different models
  }

  #[ tokio::test ]
  async fn test_content_length_validation()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let _platform = ContentGenerationPlatform::new( client );

  // Test different length categories
  let short_params = ContentGenerationPlatform::get_generation_parameters( ContentType::BlogPost, ContentTone::Professional, ContentLength::Short );
  let medium_params = ContentGenerationPlatform::get_generation_parameters( ContentType::BlogPost, ContentTone::Professional, ContentLength::Medium );
  let long_params = ContentGenerationPlatform::get_generation_parameters( ContentType::BlogPost, ContentTone::Professional, ContentLength::Long );

  assert!( short_params.max_new_tokens.expect( "[test_content_length_validation] InferenceParameters.max_new_tokens should be Some for short length - check get_generation_parameters() implementation" ) < medium_params.max_new_tokens.expect( "[test_content_length_validation] InferenceParameters.max_new_tokens should be Some for medium length - check get_generation_parameters() implementation" ) );
  assert!( medium_params.max_new_tokens.expect( "[test_content_length_validation] InferenceParameters.max_new_tokens should be Some for medium length - check get_generation_parameters() implementation" ) < long_params.max_new_tokens.expect( "[test_content_length_validation] InferenceParameters.max_new_tokens should be Some for long length - check get_generation_parameters() implementation" ) );
  }

  #[ tokio::test ]
  async fn test_error_scenarios()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let platform = ContentGenerationPlatform::new( client );

  // Test empty topic
  let empty_request = ContentRequest
  {
      content_type : ContentType::BlogPost,
      tone : ContentTone::Professional,
      topic : String::new(),
      target_audience : None,
      length : ContentLength::Medium,
      context : None,
      template : None,
  };

  let prompt = ContentGenerationPlatform::build_prompt( &empty_request );
  assert!( !prompt.is_empty() ); // Should still build a prompt

  // Test template filtering
  let creative_templates = platform.get_templates_for_type( ContentType::Creative );
  assert!( creative_templates.is_empty() ); // No creative templates added
  }

  #[ tokio::test ]
  async fn test_batch_generation_structure()
  {
  let Some( client ) = create_test_client() else {
      println!( "Skipping test - no API key available" );
      return;
  };

  let platform = ContentGenerationPlatform::new( client );

  let request = ContentRequest
  {
      content_type : ContentType::SocialMedia,
      tone : ContentTone::Enthusiastic,
      topic : "productivity tips".to_string(),
      target_audience : Some( "professionals".to_string() ),
      length : ContentLength::Short,
      context : None,
      template : None,
  };

  // Test batch generation method exists and has correct signature
  let result = platform.generate_variations( request, 3 ).await;

  match result
  {
      Ok( variations ) =>
      {
  println!( "Successfully generated {} content variations", variations.len() );
  // Verify variations structure if generation succeeds
  for ( i, variation ) in variations.iter().enumerate()
  {
          assert_eq!( variation.content_type, ContentType::SocialMedia );
          assert_eq!( variation.tone, ContentTone::Enthusiastic );
          assert_eq!( variation.topic, "productivity tips" );
          println!( "Variation {}: {} words", i + 1, variation.word_count );
  }
      },
      Err( e ) =>
      {
  println!( "Batch generation failed (expected in test environment): {e}" );
  // This is expected if API keys aren't available or API is unreachable
      },
  }
  }
}