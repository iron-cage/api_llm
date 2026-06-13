#![ allow( clippy::all, clippy::pedantic ) ]
//! Tests for Language Translation & Localization Example
//!
//! This test suite verifies the functionality of a multilingual translation system
//! that provides automatic translation capabilities using HuggingFace translation models.

use api_huggingface::
{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  components::
  {
  input::InferenceParameters,
  },
  secret::Secret,
};
use std::{ collections::HashMap, time::Instant };
use serde::{ Serialize, Deserialize };

#[ allow( missing_docs ) ]
/// Supported language codes using ISO 639-1 standard
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub enum LanguageCode
{
  /// English
  EN,
  /// Spanish
  ES,
  /// French
  FR,
  /// German
  DE,
  /// Italian
  IT,
  /// Portuguese
  PT,
  /// Russian
  RU,
  /// Chinese (Simplified)
  ZH,
  /// Japanese
  JA,
  /// Korean
  KO,
  /// Arabic
  AR,
  /// Hindi
  HI,
}

impl LanguageCode
{
  /// Get language name in English
  pub fn name( &self ) -> &'static str
  {
  match self
  {
      LanguageCode::EN => "English",
      LanguageCode::ES => "Spanish", 
      LanguageCode::FR => "French",
      LanguageCode::DE => "German",
      LanguageCode::IT => "Italian",
      LanguageCode::PT => "Portuguese",
      LanguageCode::RU => "Russian",
      LanguageCode::ZH => "Chinese",
      LanguageCode::JA => "Japanese",
      LanguageCode::KO => "Korean",
      LanguageCode::AR => "Arabic",
      LanguageCode::HI => "Hindi",
  }
  }

  /// Get ISO 639-1 code as string
  pub fn code( &self ) -> &'static str
  {
  match self
  {
      LanguageCode::EN => "en",
      LanguageCode::ES => "es",
      LanguageCode::FR => "fr",
      LanguageCode::DE => "de",
      LanguageCode::IT => "it",
      LanguageCode::PT => "pt",
      LanguageCode::RU => "ru",
      LanguageCode::ZH => "zh",
      LanguageCode::JA => "ja",
      LanguageCode::KO => "ko",
      LanguageCode::AR => "ar",
      LanguageCode::HI => "hi",
  }
  }

  /// Check if language uses complex writing systems
  pub fn is_complex_script( &self ) -> bool
  {
  matches!( self, LanguageCode::ZH | LanguageCode::JA | LanguageCode::AR | LanguageCode::HI )
  }

  /// Get preferred translation model for this language pair
  pub fn preferred_model( &self, target : &LanguageCode ) -> &'static str
  {
  match ( self, target )
  {
      // European language pairs - use Helsinki-NLP models
      ( LanguageCode::EN, LanguageCode::FR ) | ( LanguageCode::FR, LanguageCode::EN ) => "Helsinki-NLP/opus-mt-en-fr",
      ( LanguageCode::EN, LanguageCode::DE ) | ( LanguageCode::DE, LanguageCode::EN ) => "Helsinki-NLP/opus-mt-en-de", 
      ( LanguageCode::EN, LanguageCode::ES ) | ( LanguageCode::ES, LanguageCode::EN ) => "Helsinki-NLP/opus-mt-en-es",
      ( LanguageCode::EN, LanguageCode::IT ) | ( LanguageCode::IT, LanguageCode::EN ) => "Helsinki-NLP/opus-mt-en-it",
      ( LanguageCode::EN, LanguageCode::PT ) | ( LanguageCode::PT, LanguageCode::EN ) => "Helsinki-NLP/opus-mt-en-pt",
      ( LanguageCode::EN, LanguageCode::RU ) | ( LanguageCode::RU, LanguageCode::EN ) => "Helsinki-NLP/opus-mt-en-ru",
      
      // Asian language pairs - use specialized models
      ( LanguageCode::EN, LanguageCode::ZH ) | ( LanguageCode::ZH, LanguageCode::EN ) => "Helsinki-NLP/opus-mt-en-zh",
      ( LanguageCode::EN, LanguageCode::JA ) | ( LanguageCode::JA, LanguageCode::EN ) => "Helsinki-NLP/opus-mt-en-jap",
      ( LanguageCode::EN, LanguageCode::KO ) | ( LanguageCode::KO, LanguageCode::EN ) => "Helsinki-NLP/opus-mt-en-ko",
      ( LanguageCode::EN, LanguageCode::AR ) | ( LanguageCode::AR, LanguageCode::EN ) => "Helsinki-NLP/opus-mt-en-ar",
      
      // Default fallback to multilingual model
      _ => "facebook/mbart-large-50-many-to-many-mmt",
  }
  }
}

/// Translation quality metrics
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
pub enum QualityLevel
{
  /// Machine translation quality
  Basic,
  /// Good quality with minor errors
  Good,
  /// High quality professional translation
  Professional,
  /// Near-native quality translation
  Expert,
}

impl QualityLevel
{
  /// Get minimum BLEU score threshold for this quality level
  pub fn bleu_threshold( &self ) -> f32
  {
  match self
  {
      QualityLevel::Basic => 0.3,
      QualityLevel::Good => 0.5,
      QualityLevel::Professional => 0.7,
      QualityLevel::Expert => 0.85,
  }
  }

  /// Get quality score range (0-100)  
  pub fn score_range( &self ) -> ( u8, u8 )
  {
  match self
  {
      QualityLevel::Basic => ( 30, 50 ),
      QualityLevel::Good => ( 50, 70 ),
      QualityLevel::Professional => ( 70, 85 ),
      QualityLevel::Expert => ( 85, 100 ),
  }
  }
}

/// Translation request configuration
#[ derive( Debug, Clone ) ]
pub struct TranslationRequest
{
  /// Source text to translate
  pub text : String,
  /// Source language
  pub source_language : LanguageCode,
  /// Target language
  pub target_language : LanguageCode,
  /// Quality level preference
  pub quality_preference : QualityLevel,
  /// Context for better translation (domain, style, etc.)
  pub context : Option< String >,
  /// Whether to preserve formatting
  pub preserve_formatting : bool,
  /// Maximum allowed response time (seconds)
  pub max_response_time : Option< u32 >,
}

/// Translation result with quality metrics
#[ derive( Debug, Clone ) ]
pub struct TranslationResult
{
  /// Translated text
  pub translated_text : String,
  /// Confidence score (0.0 - 1.0)
  pub confidence_score : f32,
  /// Estimated quality level
  pub quality_assessment : QualityLevel,
  /// Processing time in milliseconds
  pub response_time_ms : u64,
  /// Source language (detected if auto-detected)
  pub detected_source : Option< LanguageCode >,
  /// Model used for translation
  pub model_used : String,
  /// Additional quality metrics
  pub quality_metrics : QualityMetrics,
}

/// Detailed quality assessment metrics
#[ derive( Debug, Clone ) ]
pub struct QualityMetrics
{
  /// Estimated BLEU score (if reference available)
  pub bleu_score : Option< f32 >,
  /// Fluency score (0-100)
  pub fluency_score : u8,
  /// Adequacy score (0-100) 
  pub adequacy_score : u8,
  /// Lexical accuracy (0-100)
  pub lexical_accuracy : u8,
  /// Grammar correctness (0-100)
  pub grammar_score : u8,
}

/// Batch translation request
#[ derive( Debug, Clone ) ]
pub struct BatchTranslationRequest
{
  /// List of texts to translate
  pub texts : Vec< String >,
  /// Source language for all texts
  pub source_language : LanguageCode,
  /// Target language for all texts
  pub target_language : LanguageCode,
  /// Quality preference
  pub quality_preference : QualityLevel,
  /// Batch processing options
  pub batch_options : BatchOptions,
}

/// Batch processing configuration
#[ derive( Debug, Clone ) ]
pub struct BatchOptions
{
  /// Maximum batch size
  pub max_batch_size : usize,
  /// Enable parallel processing
  pub parallel_processing : bool,
  /// Progress callback interval
  pub progress_callback_interval : Option< usize >,
  /// Retry failed translations
  pub retry_failures : bool,
}

impl Default for BatchOptions
{
  fn default() -> Self
  {
  Self
  {
      max_batch_size : 10,
      parallel_processing : true,
      progress_callback_interval : Some( 5 ),
      retry_failures : true,
  }
  }
}

/// Language detection result
#[ derive( Debug, Clone ) ]
pub struct LanguageDetectionResult
{
  /// Detected language
  pub detected_language : LanguageCode,
  /// Confidence in detection (0.0 - 1.0)
  pub confidence : f32,
  /// Alternative language possibilities
  pub alternatives : Vec< ( LanguageCode, f32 ) >,
  /// Text sample used for detection
  pub sample_text : String,
}

/// Translation platform for multilingual applications
#[ derive( Debug, Clone ) ]
pub struct TranslationPlatform
{
  /// HuggingFace API client
  client : Client< HuggingFaceEnvironmentImpl >,
  /// Supported language pairs
  supported_pairs : HashMap< ( LanguageCode, LanguageCode ), String >,
  /// Translation cache for efficiency
  translation_cache : HashMap< String, TranslationResult >,
  /// Platform statistics
  stats : PlatformStatistics,
}

/// Platform usage statistics
#[ derive( Debug, Clone, Default ) ]
pub struct PlatformStatistics
{
  /// Total translations performed
  pub total_translations : u64,
  /// Total processing time (milliseconds)
  pub total_processing_time : u64,
  /// Average translation quality
  pub average_quality_score : f32,
  /// Most frequently used language pairs
  pub popular_language_pairs : HashMap< ( LanguageCode, LanguageCode ), u64 >,
  /// Error rate by language pair
  pub error_rates : HashMap< ( LanguageCode, LanguageCode ), f32 >,
}

impl TranslationPlatform
{
  /// Create a new translation platform
  pub fn new( client : Client< HuggingFaceEnvironmentImpl > ) -> Self
  {
  let mut platform = Self
  {
      client,
      supported_pairs : HashMap::new(),
      translation_cache : HashMap::new(),
      stats : PlatformStatistics::default(),
  };

  // Initialize supported language pairs
  platform.initialize_language_pairs();
  platform
  }

  /// Initialize supported language pairs with their preferred models
  fn initialize_language_pairs( &mut self )
  {
  let languages = [ 
      LanguageCode::EN, LanguageCode::ES, LanguageCode::FR, LanguageCode::DE,
      LanguageCode::IT, LanguageCode::PT, LanguageCode::RU, LanguageCode::ZH,
      LanguageCode::JA, LanguageCode::KO, LanguageCode::AR, LanguageCode::HI
  ];

  for &source in &languages
  {
      for &target in &languages
      {
  if source != target
  {
          let model = source.preferred_model( &target );
          self.supported_pairs.insert( ( source, target ), model.to_string() );
  }
      }
  }
  }

  /// Translate a single text
  pub async fn translate( &mut self, request : &TranslationRequest ) -> Result< TranslationResult, Box< dyn std::error::Error > >
  {
  let start_time = Instant::now();

  // Check if translation is cached
  let cache_key = format!( "{}:{}->{}", request.text, request.source_language.code(), request.target_language.code() );
  if let Some( cached_result ) = self.translation_cache.get( &cache_key )
  {
      return Ok( cached_result.clone() );
  }

  // Get the appropriate model for the language pair
  let model = request.source_language.preferred_model( &request.target_language );

  // Build translation prompt
  let prompt = self.build_translation_prompt( request )?;

  // Set translation parameters
  let params = InferenceParameters::new()
      .with_max_new_tokens( self.calculate_max_tokens_for_translation( &request.text ) )
      .with_temperature( 0.3 ) // Lower temperature for more consistent translations
      .with_top_p( 0.9 );

  // Perform translation
  let response = self.client.inference().create_with_parameters( &prompt, model, params ).await?;

  // Process response based on type
  let translated_text = response.extract_text_or_default( "Translation failed" );

  let response_time = start_time.elapsed().as_millis() as u64;

  // Calculate quality metrics
  let quality_metrics = self.calculate_quality_metrics( &request.text, &translated_text, request.quality_preference );
  let confidence_score = self.calculate_translation_confidence( &request.text, &translated_text, &quality_metrics );

  let result = TranslationResult
  {
      translated_text : translated_text.trim().to_string(),
      confidence_score,
      quality_assessment : self.assess_quality_level( &quality_metrics ),
      response_time_ms : response_time,
      detected_source : Some( request.source_language ),
      model_used : model.to_string(),
      quality_metrics,
  };

  // Cache the result
  self.translation_cache.insert( cache_key, result.clone() );

  // Update statistics
  self.update_statistics( &request.source_language, &request.target_language, response_time, &result );

  Ok( result )
  }

  /// Translate multiple texts in batch
  pub async fn translate_batch( &mut self, request : &BatchTranslationRequest ) -> Result< Vec< Result< TranslationResult, Box< dyn std::error::Error > > >, Box< dyn std::error::Error > >
  {
  let mut results = Vec::new();
  let batch_size = request.batch_options.max_batch_size.min( request.texts.len() );

  for chunk in request.texts.chunks( batch_size )
  {
      let mut chunk_results = Vec::new();

      if request.batch_options.parallel_processing
      {
  // Process chunk in parallel (simulated for this example)
  for text in chunk
  {
          let translation_request = TranslationRequest
          {
      text : text.clone(),
      source_language : request.source_language,
      target_language : request.target_language,
      quality_preference : request.quality_preference,
      context : None,
      preserve_formatting : false,
      max_response_time : None,
          };

          let result = self.translate( &translation_request ).await;
          chunk_results.push( result );
  }
      }
      else
      {
  // Process chunk sequentially
  for text in chunk
  {
          let translation_request = TranslationRequest
          {
      text : text.clone(),
      source_language : request.source_language,
      target_language : request.target_language,
      quality_preference : request.quality_preference,
      context : None,
      preserve_formatting : false,
      max_response_time : None,
          };

          let result = self.translate( &translation_request ).await;
          chunk_results.push( result );
  }
      }

      results.extend( chunk_results );
  }

  Ok( results )
  }

  /// Detect language of input text
  pub async fn detect_language( &self, text : &str ) -> Result< LanguageDetectionResult, Box< dyn std::error::Error > >
  {
  // Simple heuristic-based language detection for testing
  let detection_result = self.heuristic_language_detection( text );
  
  Ok( LanguageDetectionResult
  {
      detected_language : detection_result.0,
      confidence : detection_result.1,
      alternatives : detection_result.2,
      sample_text : text[ ..text.len().min( 100 ) ].to_string(),
  } )
  }

  /// Check if language pair is supported
  pub fn is_language_pair_supported( &self, source : &LanguageCode, target : &LanguageCode ) -> bool
  {
  self.supported_pairs.contains_key( &( *source, *target ) )
  }

  /// Get platform statistics
  pub fn get_statistics( &self ) -> &PlatformStatistics
  {
  &self.stats
  }

  /// Build translation prompt for the model
  fn build_translation_prompt( &self, request : &TranslationRequest ) -> Result< String, Box< dyn std::error::Error > >
  {
  let mut prompt = format!(
      "Translate the following text from {} to {}:",
      request.source_language.name(),
      request.target_language.name()
  );

  if let Some( ref context ) = request.context
  {
      prompt.push_str( &format!( " Context : {}", context ) );
  }

  prompt.push_str( &format!( "\n\nText : {}\n\nTranslation:", request.text ) );

  Ok( prompt )
  }

  /// Calculate maximum tokens needed for translation
  fn calculate_max_tokens_for_translation( &self, text : &str ) -> u32
  {
  let input_words = text.split_whitespace().count();
  // Estimate output tokens as 1.5x input words plus buffer
  ( ( input_words as f32 * 1.5 ) + 50.0 ) as u32
  }

  /// Calculate quality metrics for translation
  fn calculate_quality_metrics( &self, _source_text : &str, translated_text : &str, quality_preference : QualityLevel ) -> QualityMetrics
  {
  // Simplified quality assessment based on translation characteristics
  let word_count = translated_text.split_whitespace().count();
  let char_count = translated_text.chars().count();

  let fluency_score = if word_count > 0 && char_count > word_count
  {
      // Basic fluency assessment
      ( 70 + ( word_count.min( 20 ) * 2 ) ).min( 95 ) as u8
  }
  else
  {
      50
  };

  let adequacy_score = match quality_preference
  {
      QualityLevel::Basic => ( 40..60 ).into_iter().nth( word_count % 20 ).unwrap_or( 50 ) as u8,
      QualityLevel::Good => ( 60..75 ).into_iter().nth( word_count % 15 ).unwrap_or( 67 ) as u8,
      QualityLevel::Professional => ( 75..85 ).into_iter().nth( word_count % 10 ).unwrap_or( 80 ) as u8,
      QualityLevel::Expert => ( 85..95 ).into_iter().nth( word_count % 10 ).unwrap_or( 90 ) as u8,
  };

  QualityMetrics
  {
      bleu_score : None, // Would need reference translation
      fluency_score,
      adequacy_score,
      lexical_accuracy : ( adequacy_score as f32 * 0.9 ) as u8,
      grammar_score : ( fluency_score as f32 * 0.8 ) as u8,
  }
  }

  /// Calculate translation confidence based on quality metrics
  fn calculate_translation_confidence( &self, _source_text : &str, _translated_text : &str, metrics : &QualityMetrics ) -> f32
  {
  let combined_score = ( metrics.fluency_score + metrics.adequacy_score + metrics.lexical_accuracy + metrics.grammar_score ) as f32 / 4.0;
  ( combined_score / 100.0 ).min( 1.0 ).max( 0.0 )
  }

  /// Assess quality level based on metrics
  fn assess_quality_level( &self, metrics : &QualityMetrics ) -> QualityLevel
  {
  let avg_score = ( metrics.fluency_score + metrics.adequacy_score + metrics.lexical_accuracy + metrics.grammar_score ) as f32 / 4.0;
  
  if avg_score >= 85.0
  {
      QualityLevel::Expert
  }
  else if avg_score >= 70.0
  {
      QualityLevel::Professional
  }
  else if avg_score >= 50.0
  {
      QualityLevel::Good
  }
  else
  {
      QualityLevel::Basic
  }
  }

  /// Update platform statistics
  fn update_statistics( &mut self, source : &LanguageCode, target : &LanguageCode, response_time : u64, result : &TranslationResult )
  {
  self.stats.total_translations += 1;
  self.stats.total_processing_time += response_time;
  
  // Update average quality score
  let total_quality = self.stats.average_quality_score * ( self.stats.total_translations - 1 ) as f32 + result.confidence_score;
  self.stats.average_quality_score = total_quality / self.stats.total_translations as f32;
  
  // Update language pair popularity
  *self.stats.popular_language_pairs.entry( ( *source, *target ) ).or_insert( 0 ) += 1;
  
  // Update error rates (simplified - based on confidence threshold)
  if result.confidence_score < 0.5
  {
      let error_count = self.stats.error_rates.get( &( *source, *target ) ).unwrap_or( &0.0 ) * self.stats.total_translations as f32 + 1.0;
      self.stats.error_rates.insert( ( *source, *target ), error_count / self.stats.total_translations as f32 );
  }
  }

  /// Heuristic language detection (simplified for testing)
  fn heuristic_language_detection( &self, text : &str ) -> ( LanguageCode, f32, Vec< ( LanguageCode, f32 ) > )
  {
  let text_lower = text.to_lowercase();
  
  // Simple keyword-based detection
  if text_lower.contains( "the" ) || text_lower.contains( "and" ) || text_lower.contains( "is" )
  {
      ( LanguageCode::EN, 0.9, vec![ ( LanguageCode::EN, 0.9 ), ( LanguageCode::DE, 0.1 ) ] )
  }
  else if text_lower.contains( "le" ) || text_lower.contains( "la" ) || text_lower.contains( "est" )
  {
      ( LanguageCode::FR, 0.8, vec![ ( LanguageCode::FR, 0.8 ), ( LanguageCode::ES, 0.2 ) ] )
  }
  else if text_lower.contains( "el" ) || text_lower.contains( "la" ) || text_lower.contains( "es" )
  {
      ( LanguageCode::ES, 0.8, vec![ ( LanguageCode::ES, 0.8 ), ( LanguageCode::IT, 0.2 ) ] )
  }
  else if text_lower.contains( "der" ) || text_lower.contains( "die" ) || text_lower.contains( "ist" )
  {
      ( LanguageCode::DE, 0.8, vec![ ( LanguageCode::DE, 0.8 ), ( LanguageCode::EN, 0.2 ) ] )
  }
  else
  {
      // Default to English with lower confidence
      ( LanguageCode::EN, 0.6, vec![ ( LanguageCode::EN, 0.6 ), ( LanguageCode::ES, 0.4 ) ] )
  }
  }
}

// Helper function to create test client
fn create_test_client() -> Option< Client< HuggingFaceEnvironmentImpl > >
{
  let api_key = super::get_api_key_for_testing()?;
  let secret = Secret::new( api_key );
  let env = HuggingFaceEnvironmentImpl::build( secret, None ).ok()?;
  Client::build( env ).ok()
}

// Helper function to create sample translation requests
fn create_sample_translation_requests() -> Vec< TranslationRequest >
{
  vec![
  TranslationRequest
  {
      text : "Hello, how are you today?".to_string(),
      source_language : LanguageCode::EN,
      target_language : LanguageCode::ES,
      quality_preference : QualityLevel::Good,
      context : Some( "casual conversation".to_string() ),
      preserve_formatting : false,
      max_response_time : Some( 10 ),
  },
  TranslationRequest
  {
      text : "The weather is beautiful today.".to_string(),
      source_language : LanguageCode::EN,
      target_language : LanguageCode::FR,
      quality_preference : QualityLevel::Professional,
      context : None,
      preserve_formatting : false,
      max_response_time : None,
  },
  TranslationRequest
  {
      text : "Please review the attached document.".to_string(),
      source_language : LanguageCode::EN,
      target_language : LanguageCode::DE,
      quality_preference : QualityLevel::Professional,
      context : Some( "business email".to_string() ),
      preserve_formatting : true,
      max_response_time : Some( 15 ),
  },
  ]
}

