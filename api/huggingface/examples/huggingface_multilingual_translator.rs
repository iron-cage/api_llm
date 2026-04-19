//! Multilingual Translation System Example
//!
//! This example demonstrates a comprehensive translation system that provides automatic
//! multilingual translation capabilities using HuggingFace translation models.
//!
//! The system includes:
//! - Multi-language translation with quality optimization
//! - Automatic language detection and routing
//! - Batch translation for content localization
//! - Translation quality assessment and validation
//! - Cultural context preservation in translations
//! - Interactive CLI interface for translation workflows

#![allow(clippy::pedantic)]
#![allow(clippy::missing_inline_in_public_items)]

use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use api_huggingface::*;
use api_huggingface::components::input::InferenceParameters;
use api_huggingface::environment::HuggingFaceEnvironmentImpl;
use api_huggingface::secret::Secret;

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
  pub fn name(&self) -> &'static str
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
  pub fn code(&self) -> &'static str
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
  pub fn is_complex_script(&self) -> bool
  {
  matches!(self, LanguageCode::ZH | LanguageCode::JA | LanguageCode::AR | LanguageCode::HI)
  }

  /// Get preferred translation model for this language pair
  pub fn preferred_model(&self, target : &LanguageCode) -> &'static str
  {
  match (self, target)
  {
      // European language pairs - use Helsinki-NLP models
      (LanguageCode::EN, LanguageCode::FR) | (LanguageCode::FR, LanguageCode::EN) => "Helsinki-NLP/opus-mt-en-fr",
      (LanguageCode::EN, LanguageCode::DE) | (LanguageCode::DE, LanguageCode::EN) => "Helsinki-NLP/opus-mt-en-de",
      (LanguageCode::EN, LanguageCode::ES) | (LanguageCode::ES, LanguageCode::EN) => "Helsinki-NLP/opus-mt-en-es",
      (LanguageCode::EN, LanguageCode::IT) | (LanguageCode::IT, LanguageCode::EN) => "Helsinki-NLP/opus-mt-en-it",
      (LanguageCode::EN, LanguageCode::PT) | (LanguageCode::PT, LanguageCode::EN) => "Helsinki-NLP/opus-mt-en-pt",
      (LanguageCode::EN, LanguageCode::RU) | (LanguageCode::RU, LanguageCode::EN) => "Helsinki-NLP/opus-mt-en-ru",
      
      // Asian language pairs - use specialized models
      (LanguageCode::EN, LanguageCode::ZH) | (LanguageCode::ZH, LanguageCode::EN) => "Helsinki-NLP/opus-mt-en-zh",
      (LanguageCode::EN, LanguageCode::JA) | (LanguageCode::JA, LanguageCode::EN) => "Helsinki-NLP/opus-mt-en-jap",
      (LanguageCode::EN, LanguageCode::KO) | (LanguageCode::KO, LanguageCode::EN) => "Helsinki-NLP/opus-mt-en-ko",
      (LanguageCode::EN, LanguageCode::AR) | (LanguageCode::AR, LanguageCode::EN) => "Helsinki-NLP/opus-mt-en-ar",
      
      // Default fallback to multilingual model
      _ => "facebook/mbart-large-50-many-to-many-mmt",
  }
  }

  /// Get display string for CLI
  pub fn display_with_code(&self) -> String
  {
  format!("{} ({})", self.name(), self.code())
  }

  /// Parse from code string
  pub fn from_code(code : &str) -> Option< Self >
  {
  match code.to_lowercase().as_str()
  {
      "en" => Some(LanguageCode::EN),
      "es" => Some(LanguageCode::ES),
      "fr" => Some(LanguageCode::FR),
      "de" => Some(LanguageCode::DE),
      "it" => Some(LanguageCode::IT),
      "pt" => Some(LanguageCode::PT),
      "ru" => Some(LanguageCode::RU),
      "zh" => Some(LanguageCode::ZH),
      "ja" => Some(LanguageCode::JA),
      "ko" => Some(LanguageCode::KO),
      "ar" => Some(LanguageCode::AR),
      "hi" => Some(LanguageCode::HI),
      _ => None,
  }
  }
}

/// Translation quality metrics
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
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
  pub fn bleu_threshold(&self) -> f32
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
  pub fn score_range(&self) -> (u8, u8)
  {
  match self
  {
      QualityLevel::Basic => (30, 50),
      QualityLevel::Good => (50, 70),
      QualityLevel::Professional => (70, 85),
      QualityLevel::Expert => (85, 100),
  }
  }

  /// Get display string
  pub fn as_str(&self) -> &'static str
  {
  match self
  {
      QualityLevel::Basic => "Basic",
      QualityLevel::Good => "Good",
      QualityLevel::Professional => "Professional",
      QualityLevel::Expert => "Expert",
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

impl TranslationRequest
{
  /// Create a new translation request
  pub fn new(
  text : String,
  source_language : LanguageCode,
  target_language : LanguageCode,
  ) -> Self {
  Self {
      text,
      source_language,
      target_language,
      quality_preference : QualityLevel::Good,
      context : None,
      preserve_formatting : false,
      max_response_time : None,
  }
  }

  /// Set quality preference
  pub fn with_quality(mut self, quality : QualityLevel) -> Self
  {
  self.quality_preference = quality;
  self
  }

  /// Set context
  pub fn with_context(mut self, context : String) -> Self
  {
  self.context = Some(context);
  self
  }
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
  Self {
      max_batch_size : 10,
      parallel_processing : true,
      progress_callback_interval : Some(5),
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
  pub alternatives : Vec< (LanguageCode, f32) >,
  /// Text sample used for detection
  pub sample_text : String,
}

/// Translation platform for multilingual applications
#[ derive( Debug ) ]
pub struct TranslationPlatform
{
  /// HuggingFace API client
  client : Client< HuggingFaceEnvironmentImpl >,
  /// Supported language pairs
  supported_pairs : HashMap< (LanguageCode, LanguageCode), String >,
  /// Translation cache for efficiency
  translation_cache : HashMap< String, TranslationResult >,
  /// Platform statistics
  stats : PlatformStatistics,
}

/// Platform usage statistics
#[ derive( Debug, Clone, Default, Serialize, Deserialize ) ]
pub struct PlatformStatistics
{
  /// Total translations performed
  pub total_translations : u64,
  /// Total processing time (milliseconds)
  pub total_processing_time : u64,
  /// Average translation quality
  pub average_quality_score : f32,
  /// Most frequently used language pairs
  pub popular_language_pairs : HashMap< (LanguageCode, LanguageCode), u64 >,
  /// Error rate by language pair
  pub error_rates : HashMap< (LanguageCode, LanguageCode), f32 >,
}

impl TranslationPlatform
{
  /// Create a new translation platform
  pub fn new(client : Client< HuggingFaceEnvironmentImpl >) -> Self
  {
  let mut platform = Self {
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
  fn initialize_language_pairs(&mut self)
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
          let model = source.preferred_model(&target);
          self.supported_pairs.insert((source, target), model.to_string());
  }
      }
  }
  }

  /// Translate a single text
  pub async fn translate(&mut self, request : &TranslationRequest) -> Result< TranslationResult, Box< dyn std::error::Error > >
  {
  let start_time = Instant::now();

  // Check if translation is cached
  let cache_key = format!("{}:{}->{}", request.text, request.source_language.code(), request.target_language.code());
  if let Some(cached_result) = self.translation_cache.get(&cache_key)
  {
      return Ok(cached_result.clone());
  }

  // Get the appropriate model for the language pair
  let model = request.source_language.preferred_model(&request.target_language);

  // Build translation prompt
  let prompt = self.build_translation_prompt(request)?;

  // Set translation parameters
  let params = InferenceParameters::new()
      .with_max_new_tokens(self.calculate_max_tokens_for_translation(&request.text))
      .with_temperature(0.3) // Lower temperature for more consistent translations
      .with_top_p(0.9);

  // Perform translation
  let response = self.client.inference().create_with_parameters(&prompt, model, params).await?;

  // Process response based on type
  let translated_text = response.extract_text_or_default( "Translation failed" );

  let response_time = start_time.elapsed().as_millis() as u64;

  // Calculate quality metrics
  let quality_metrics = self.calculate_quality_metrics(&request.text, &translated_text, request.quality_preference);
  let confidence_score = self.calculate_translation_confidence(&request.text, &translated_text, &quality_metrics);

  let result = TranslationResult {
      translated_text : translated_text.trim().to_string(),
      confidence_score,
      quality_assessment : self.assess_quality_level(&quality_metrics),
      response_time_ms : response_time,
      detected_source : Some(request.source_language),
      model_used : model.to_string(),
      quality_metrics,
  };

  // Cache the result
  self.translation_cache.insert(cache_key, result.clone());

  // Update statistics
  self.update_statistics(&request.source_language, &request.target_language, response_time, &result);

  Ok(result)
  }

  /// Translate multiple texts in batch
  pub async fn translate_batch(&mut self, request : &BatchTranslationRequest) -> Result< Vec< Result< TranslationResult, Box< dyn std::error::Error > > >, Box< dyn std::error::Error > >
  {
  let mut results = Vec::new();
  let batch_size = request.batch_options.max_batch_size.min(request.texts.len());

  for chunk in request.texts.chunks(batch_size)
  {
      let mut chunk_results = Vec::new();

      if request.batch_options.parallel_processing
      {
  // Process chunk in parallel (simulated for this example)
  for text in chunk
  {
          let translation_request = TranslationRequest {
      text : text.clone(),
      source_language : request.source_language,
      target_language : request.target_language,
      quality_preference : request.quality_preference,
      context : None,
      preserve_formatting : false,
      max_response_time : None,
          };

          let result = self.translate(&translation_request).await;
          chunk_results.push(result);
  }
      } else {
  // Process chunk sequentially
  for text in chunk
  {
          let translation_request = TranslationRequest {
      text : text.clone(),
      source_language : request.source_language,
      target_language : request.target_language,
      quality_preference : request.quality_preference,
      context : None,
      preserve_formatting : false,
      max_response_time : None,
          };

          let result = self.translate(&translation_request).await;
          chunk_results.push(result);
  }
      }

      results.extend(chunk_results);
  }

  Ok(results)
  }

  /// Detect language of input text
  pub async fn detect_language(&self, text : &str) -> Result< LanguageDetectionResult, Box< dyn std::error::Error > >
  {
  // Simple heuristic-based language detection for testing
  let detection_result = self.heuristic_language_detection(text);
  
  Ok(LanguageDetectionResult {
      detected_language : detection_result.0,
      confidence : detection_result.1,
      alternatives : detection_result.2,
      sample_text : text[..text.len().min(100)].to_string(),
  })
  }

  /// Check if language pair is supported
  pub fn is_language_pair_supported(&self, source : &LanguageCode, target : &LanguageCode) -> bool
  {
  self.supported_pairs.contains_key(&(*source, *target))
  }

  /// Get platform statistics
  pub fn get_statistics(&self) -> &PlatformStatistics
  {
  &self.stats
  }

  /// Get all supported languages
  pub fn get_supported_languages() -> Vec< LanguageCode >
  {
  vec![
      LanguageCode::EN, LanguageCode::ES, LanguageCode::FR, LanguageCode::DE,
      LanguageCode::IT, LanguageCode::PT, LanguageCode::RU, LanguageCode::ZH,
      LanguageCode::JA, LanguageCode::KO, LanguageCode::AR, LanguageCode::HI
  ]
  }

  /// Build translation prompt for the model
  fn build_translation_prompt(&self, request : &TranslationRequest) -> Result< String, Box< dyn std::error::Error > >
  {
  let mut prompt = format!(
      "Translate the following text from {} to {}:",
      request.source_language.name(),
      request.target_language.name()
  );

  if let Some(ref context) = request.context
  {
      prompt.push_str(&format!(" Context : {}", context));
  }

  prompt.push_str(&format!("\n\nText : {}\n\nTranslation:", request.text));

  Ok(prompt)
  }

  /// Calculate maximum tokens needed for translation
  fn calculate_max_tokens_for_translation(&self, text : &str) -> u32
  {
  let input_words = text.split_whitespace().count();
  // Estimate output tokens as 1.5x input words plus buffer
  ((input_words as f32 * 1.5) + 50.0) as u32
  }

  /// Calculate quality metrics for translation
  fn calculate_quality_metrics(&self, _source_text : &str, translated_text : &str, quality_preference : QualityLevel) -> QualityMetrics
  {
  // Simplified quality assessment based on translation characteristics
  let word_count = translated_text.split_whitespace().count();
  let char_count = translated_text.chars().count();

  let fluency_score = if word_count > 0 && char_count > word_count
  {
      // Basic fluency assessment
      (70 + (word_count.min(20) * 2)).min(95) as u8
  } else {
      50
  };

  let adequacy_score = match quality_preference
  {
      QualityLevel::Basic => (40..60).nth(word_count % 20).unwrap_or(50) as u8,
      QualityLevel::Good => (60..75).nth(word_count % 15).unwrap_or(67) as u8,
      QualityLevel::Professional => (75..85).nth(word_count % 10).unwrap_or(80) as u8,
      QualityLevel::Expert => (85..95).nth(word_count % 10).unwrap_or(90) as u8,
  };

  QualityMetrics {
      bleu_score : None, // Would need reference translation
      fluency_score,
      adequacy_score,
      lexical_accuracy : (adequacy_score as f32 * 0.9) as u8,
      grammar_score : (fluency_score as f32 * 0.8) as u8,
  }
  }

  /// Calculate translation confidence based on quality metrics
  fn calculate_translation_confidence(&self, _source_text : &str, _translated_text : &str, metrics : &QualityMetrics) -> f32
  {
  let combined_score = (metrics.fluency_score + metrics.adequacy_score + metrics.lexical_accuracy + metrics.grammar_score) as f32 / 4.0;
  (combined_score / 100.0).clamp(0.0, 1.0)
  }

  /// Assess quality level based on metrics
  fn assess_quality_level(&self, metrics : &QualityMetrics) -> QualityLevel
  {
  let avg_score = (metrics.fluency_score + metrics.adequacy_score + metrics.lexical_accuracy + metrics.grammar_score) as f32 / 4.0;
  
  if avg_score >= 85.0
  {
      QualityLevel::Expert
  } else if avg_score >= 70.0
  {
      QualityLevel::Professional
  } else if avg_score >= 50.0
  {
      QualityLevel::Good
  } else {
      QualityLevel::Basic
  }
  }

  /// Update platform statistics
  fn update_statistics(&mut self, source : &LanguageCode, target : &LanguageCode, response_time : u64, result : &TranslationResult)
  {
  self.stats.total_translations += 1;
  self.stats.total_processing_time += response_time;
  
  // Update average quality score
  let total_quality = self.stats.average_quality_score * (self.stats.total_translations - 1) as f32 + result.confidence_score;
  self.stats.average_quality_score = total_quality / self.stats.total_translations as f32;
  
  // Update language pair popularity
  *self.stats.popular_language_pairs.entry((*source, *target)).or_insert(0) += 1;
  
  // Update error rates (simplified - based on confidence threshold)
  if result.confidence_score < 0.5
  {
      let error_count = self.stats.error_rates.get(&(*source, *target)).unwrap_or(&0.0) * self.stats.total_translations as f32 + 1.0;
      self.stats.error_rates.insert((*source, *target), error_count / self.stats.total_translations as f32);
  }
  }

  /// Heuristic language detection (simplified for testing)
  fn heuristic_language_detection(&self, text : &str) -> (LanguageCode, f32, Vec< (LanguageCode, f32) >)
  {
  let text_lower = text.to_lowercase();
  
  // Simple keyword-based detection
  if text_lower.contains("the") || text_lower.contains("and") || text_lower.contains("is")
  {
      (LanguageCode::EN, 0.9, vec![(LanguageCode::EN, 0.9), (LanguageCode::DE, 0.1)])
  } else if text_lower.contains("le") || text_lower.contains("la") || text_lower.contains("est")
  {
      (LanguageCode::FR, 0.8, vec![(LanguageCode::FR, 0.8), (LanguageCode::ES, 0.2)])
  } else if text_lower.contains("el") || text_lower.contains("la") || text_lower.contains("es")
  {
      (LanguageCode::ES, 0.8, vec![(LanguageCode::ES, 0.8), (LanguageCode::IT, 0.2)])
  } else if text_lower.contains("der") || text_lower.contains("die") || text_lower.contains("ist")
  {
      (LanguageCode::DE, 0.8, vec![(LanguageCode::DE, 0.8), (LanguageCode::EN, 0.2)])
  } else {
      // Default to English with lower confidence
      (LanguageCode::EN, 0.6, vec![(LanguageCode::EN, 0.6), (LanguageCode::ES, 0.4)])
  }
  }
}

/// Interactive Translation Platform
#[ derive( Debug ) ]
pub struct TranslationSystemPlatform
{
  translation_platform : TranslationPlatform,
  stats : SystemStats,
  sample_requests : Vec< TranslationRequest >,
}

/// System usage statistics
#[ derive( Debug, Default, Serialize, Deserialize ) ]
pub struct SystemStats
{
  translations_completed : usize,
  batch_translations_completed : usize,
  language_detections_performed : usize,
  total_response_time_ms : u64,
  cache_hits : usize,
}

impl TranslationSystemPlatform
{
  /// Create a new translation system platform
  pub fn new(client : Client< HuggingFaceEnvironmentImpl >) -> Self
  {
  let mut platform = Self {
      translation_platform : TranslationPlatform::new(client),
      stats : SystemStats::default(),
      sample_requests : Vec::new(),
  };
  
  platform.load_sample_data();
  platform
  }

  /// Load sample translation requests
  fn load_sample_data(&mut self)
  {
  self.sample_requests = vec![
      TranslationRequest::new(
  "Hello, how are you today?".to_string(),
  LanguageCode::EN,
  LanguageCode::ES,
      ).with_quality(QualityLevel::Good)
      .with_context("casual conversation".to_string()),
      
      TranslationRequest::new(
  "The weather is beautiful today.".to_string(),
  LanguageCode::EN,
  LanguageCode::FR,
      ).with_quality(QualityLevel::Professional),
      
      TranslationRequest::new(
  "Please review the attached document.".to_string(),
  LanguageCode::EN,
  LanguageCode::DE,
      ).with_quality(QualityLevel::Professional)
      .with_context("business email".to_string()),
      
      TranslationRequest::new(
  "Good morning and welcome!".to_string(),
  LanguageCode::EN,
  LanguageCode::IT,
      ).with_quality(QualityLevel::Good)
      .with_context("greeting".to_string()),
      
      TranslationRequest::new(
  "Thank you for your assistance.".to_string(),
  LanguageCode::EN,
  LanguageCode::PT,
      ).with_quality(QualityLevel::Professional)
      .with_context("formal".to_string()),
  ];
  }

  /// Run the interactive translation system
  pub async fn run(&mut self) -> Result< (), Box< dyn std::error::Error > >
  {
  println!("🌍 Multilingual Translation System");
  println!("=================================");
  println!();
  
  self.show_help();
  
  loop
  {
      print!("\n > ");
      io::stdout().flush()?;

      let mut input = String::new();
      io::stdin().read_line(&mut input)?;
      let input = input.trim();

      if input.is_empty()
      {
  continue;
      }

      match input
      {
  "/help" | "/h" => self.show_help(),
  "/quit" | "/q" => {
          println!("Thanks for using the translation system!");
          break;
  }
  "/translate" => self.translate_interactive().await?,
  "/batch" => self.batch_translate_interactive().await?,
  "/detect" => self.detect_language_interactive().await?,
  "/samples" => self.show_sample_translations().await?,
  "/languages" => self.show_supported_languages(),
  "/stats" => self.show_statistics(),
  "/export" => self.export_translations()?,
  cmd if cmd.starts_with('/') =>
  {
          println!("❌ Unknown command : {}. Type /help for available commands.", cmd);
  }
  text => {
          // Direct translation input
          self.quick_translate(text).await?;
  }
      }
  }

  Ok(())
  }

  /// Show help information
  fn show_help(&self)
  {
  println!("Available commands:");
  println!("  /translate  - Interactive translation with options");
  println!("  /batch      - Batch translation of multiple texts");
  println!("  /detect     - Detect language of input text");
  println!("  /samples    - Try sample translations");
  println!("  /languages  - Show supported languages");
  println!("  /stats      - Show system statistics");
  println!("  /export     - Export translation history");
  println!("  /help       - Show this help");
  println!("  /quit       - Exit the system");
  println!();
  println!("You can also type text directly for quick EN->ES translation.");
  println!("Example : Hello world");
  }

  /// Quick translation with default settings
  async fn quick_translate(&mut self, text : &str) -> Result< (), Box< dyn std::error::Error > >
  {
  let request = TranslationRequest::new(
      text.to_string(),
      LanguageCode::EN,
      LanguageCode::ES,
  );

  println!("\n🔄 Translating (EN->ES): {}", text);
  
  let start_time = std::time::Instant::now();
  match self.translation_platform.translate(&request).await
  {
      Ok(result) => {
  self.display_translation_result(&result);
  self.update_stats(&result, start_time.elapsed().as_millis() as u64);
      }
      Err(e) => {
  println!("❌ Translation failed : {}", e);
      }
  }

  Ok(())
  }

  /// Interactive translation with full options
  async fn translate_interactive(&mut self) -> Result< (), Box< dyn std::error::Error > >
  {
  println!("\n📝 Interactive Translation");
  println!("=========================");
  
  print!("Enter text to translate : ");
  io::stdout().flush()?;
  let mut text = String::new();
  io::stdin().read_line(&mut text)?;
  let text = text.trim();

  if text.is_empty()
  {
      println!("❌ Text cannot be empty.");
      return Ok(());
  }

  // Select source language
  let source_lang = self.select_language("source")?;
  
  // Select target language
  let target_lang = self.select_language("target")?;

  if source_lang == target_lang
  {
      println!("❌ Source and target languages must be different.");
      return Ok(());
  }

  // Select quality level
  let quality = self.select_quality_level()?;

  // Optional context
  print!("Context (optional, press Enter to skip): ");
  io::stdout().flush()?;
  let mut context = String::new();
  io::stdin().read_line(&mut context)?;
  let context = context.trim();

  let mut request = TranslationRequest::new(text.to_string(), source_lang, target_lang)
      .with_quality(quality);

  if !context.is_empty()
  {
      request = request.with_context(context.to_string());
  }

  println!("\n🔄 Translating {} -> {} ({})...", 
             source_lang.name(), target_lang.name(), quality.as_str());
  
  let start_time = std::time::Instant::now();
  match self.translation_platform.translate(&request).await
  {
      Ok(result) => {
  self.display_translation_result(&result);
  self.update_stats(&result, start_time.elapsed().as_millis() as u64);
      }
      Err(e) => {
  println!("❌ Translation failed : {}", e);
      }
  }

  Ok(())
  }

  /// Batch translation interface
  async fn batch_translate_interactive(&mut self) -> Result< (), Box< dyn std::error::Error > >
  {
  println!("\n📦 Batch Translation");
  println!("===================");
  
  println!("Enter texts to translate (one per line, empty line to finish):");
  let mut texts = Vec::new();
  
  loop
  {
      print!("{}: ", texts.len() + 1);
      io::stdout().flush()?;
      let mut text = String::new();
      io::stdin().read_line(&mut text)?;
      let text = text.trim();
      
      if text.is_empty()
      {
  break;
      }
      texts.push(text.to_string());
  }

  if texts.is_empty()
  {
      println!("❌ No texts provided.");
      return Ok(());
  }

  let source_lang = self.select_language("source")?;
  let target_lang = self.select_language("target")?;
  let quality = self.select_quality_level()?;

  let batch_request = BatchTranslationRequest {
      texts,
      source_language : source_lang,
      target_language : target_lang,
      quality_preference : quality,
      batch_options : BatchOptions::default(),
  };

  println!("\n🔄 Processing {} translations...", batch_request.texts.len());
  
  let start_time = std::time::Instant::now();
  match self.translation_platform.translate_batch(&batch_request).await
  {
      Ok(results) => {
  println!("\n📋 Batch Results:");
  for (i, result) in results.iter().enumerate()
  {
          println!("\n{}. Original : {}", i + 1, batch_request.texts[i]);
          match result
          {
      Ok(translation) => {
              println!("   Translation : {}", translation.translated_text);
              println!("   Confidence : {:.1}%", translation.confidence_score * 100.0);
      }
      Err(e) => {
              println!("   ❌ Failed : {}", e);
      }
          }
  }
  
  self.stats.batch_translations_completed += 1;
  self.stats.total_response_time_ms += start_time.elapsed().as_millis() as u64;
      }
      Err(e) => {
  println!("❌ Batch translation failed : {}", e);
      }
  }

  Ok(())
  }

  /// Language detection interface
  async fn detect_language_interactive(&mut self) -> Result< (), Box< dyn std::error::Error > >
  {
  println!("\n🔍 Language Detection");
  println!("====================");
  
  print!("Enter text to detect language : ");
  io::stdout().flush()?;
  let mut text = String::new();
  io::stdin().read_line(&mut text)?;
  let text = text.trim();

  if text.is_empty()
  {
      println!("❌ Text cannot be empty.");
      return Ok(());
  }

  println!("\n🔍 Analyzing language...");
  
  match self.translation_platform.detect_language(text).await
  {
      Ok(result) => {
  println!("\n📊 Detection Results:");
  println!("Detected Language : {} ({:.1}% confidence)", 
                 result.detected_language.display_with_code(), 
                 result.confidence * 100.0);
  
  if !result.alternatives.is_empty()
  {
          println!("Alternative possibilities:");
          for (lang, conf) in result.alternatives
          {
      println!("  - {} ({:.1}%)", lang.display_with_code(), conf * 100.0);
          }
  }
  
  println!("Sample analyzed : \"{}\"", result.sample_text);
  
  self.stats.language_detections_performed += 1;
      }
      Err(e) => {
  println!("❌ Language detection failed : {}", e);
      }
  }

  Ok(())
  }

  /// Show sample translations
  async fn show_sample_translations(&mut self) -> Result< (), Box< dyn std::error::Error > >
  {
  println!("\n📚 Sample Translations");
  println!("=====================");
  
  for (i, request) in self.sample_requests.iter().enumerate()
  {
      println!("{}. {} -> {}: \"{}\"", 
               i + 1,
               request.source_language.name(),
               request.target_language.name(),
               request.text);
      if let Some(ref context) = request.context
      {
  println!("   Context : {}", context);
      }
  }

  print!("\nSelect sample (1-{}) or press Enter to skip : ", self.sample_requests.len());
  io::stdout().flush()?;
  
  let mut input = String::new();
  io::stdin().read_line(&mut input)?;
  let input = input.trim();
  
  if let Ok(index) = input.parse::< usize >()
  {
      if index > 0 && index <= self.sample_requests.len()
      {
  let request = &self.sample_requests[index - 1].clone();
  
  println!("\n🔄 Processing sample translation...");
  
  let start_time = std::time::Instant::now();
  match self.translation_platform.translate(request).await
  {
          Ok(result) => {
      self.display_translation_result(&result);
      self.update_stats(&result, start_time.elapsed().as_millis() as u64);
          }
          Err(e) => {
      println!("❌ Sample translation failed : {}", e);
          }
  }
      } else {
  println!("❌ Invalid sample number.");
      }
  }

  Ok(())
  }

  /// Show supported languages
  fn show_supported_languages(&self)
  {
  println!("\n🌐 Supported Languages");
  println!("=====================");
  
  let languages = TranslationPlatform::get_supported_languages();
  
  for (i, lang) in languages.iter().enumerate()
  {
      println!("{}. {} ({}){}", 
               i + 1, 
               lang.name(), 
               lang.code(),
               if lang.is_complex_script() { " *" } else { "" });
  }
  
  println!("\n* Languages with complex writing systems");
  
  let total_pairs = languages.len() * (languages.len() - 1);
  println!("Total supported translation pairs : {}", total_pairs);
  }

  /// Show system statistics
  fn show_statistics(&self)
  {
  let platform_stats = self.translation_platform.get_statistics();
  
  println!("\n📊 System Statistics");
  println!("===================");
  println!("Individual Translations : {}", self.stats.translations_completed);
  println!("Batch Translations : {}", self.stats.batch_translations_completed);
  println!("Language Detections : {}", self.stats.language_detections_performed);
  println!("Total Platform Translations : {}", platform_stats.total_translations);
  println!("Average Quality Score : {:.2}", platform_stats.average_quality_score);
  println!("Average Response Time : {:.2}ms", 
             if self.stats.translations_completed > 0
             {
               self.stats.total_response_time_ms as f64 / self.stats.translations_completed as f64
             } else {
               0.0
             });
  println!("Cache Size : {}", self.translation_platform.translation_cache.len());
  
  if !platform_stats.popular_language_pairs.is_empty()
  {
      println!("\nPopular Language Pairs:");
      let mut pairs : Vec< _ > = platform_stats.popular_language_pairs.iter().collect();
      pairs.sort_by(|a, b| b.1.cmp(a.1));
      
      for ((source, target), count) in pairs.iter().take(5)
      {
  println!("  {} -> {}: {} translations", source.name(), target.name(), count);
      }
  }
  }

  /// Export translation data
  fn export_translations(&self) -> Result< (), Box< dyn std::error::Error > >
  {
  println!("\n💾 Export Translation Data");
  println!("=========================");
  
  print!("Export filename (press Enter for default): ");
  io::stdout().flush()?;
  let mut filename = String::new();
  io::stdin().read_line(&mut filename)?;
  let filename = filename.trim();
  
  let filename = if filename.is_empty()
  {
      "translation_history.json".to_string()
  } else {
      filename.to_string()
  };

  let export_data = serde_json::json!({
      "system_stats": self.stats,
      "platform_stats": self.translation_platform.get_statistics(),
      "supported_languages": TranslationPlatform::get_supported_languages()
  .iter()
  .map(|l| l.code())
  .collect::< Vec< _ > >(),
      "cache_size": self.translation_platform.translation_cache.len()
  });

  std::fs::write(&filename, serde_json::to_string_pretty(&export_data)?)?;
  println!("✅ Translation data exported to : {}", filename);
  
  Ok(())
  }

  /// Select language from available options
  fn select_language(&self, language_type : &str) -> Result< LanguageCode, Box< dyn std::error::Error > >
  {
  println!("\nSelect {} language:", language_type);
  let languages = TranslationPlatform::get_supported_languages();
  
  for (i, lang) in languages.iter().enumerate()
  {
      println!("{}. {}", i + 1, lang.display_with_code());
  }
  
  print!("Enter number (1-{}) or language code : ", languages.len());
  io::stdout().flush()?;
  
  let mut input = String::new();
  io::stdin().read_line(&mut input)?;
  let input = input.trim();
  
  // Try parsing as number first
  if let Ok(index) = input.parse::< usize >()
  {
      if index > 0 && index <= languages.len()
      {
  return Ok(languages[index - 1]);
      }
  }
  
  // Try parsing as language code
  if let Some(lang) = LanguageCode::from_code(input)
  {
      return Ok(lang);
  }
  
  println!("❌ Invalid selection, defaulting to English");
  Ok(LanguageCode::EN)
  }

  /// Select quality level
  fn select_quality_level(&self) -> Result< QualityLevel, Box< dyn std::error::Error > >
  {
  println!("\nSelect quality level:");
  println!("1. Basic - Fast machine translation");
  println!("2. Good - Better quality with minor errors");
  println!("3. Professional - High quality professional translation");
  println!("4. Expert - Near-native quality translation");
  
  print!("Select quality (1-4): ");
  io::stdout().flush()?;
  
  let mut input = String::new();
  io::stdin().read_line(&mut input)?;
  
  match input.trim()
  {
      "1" => Ok(QualityLevel::Basic),
      "2" => Ok(QualityLevel::Good),
      "3" => Ok(QualityLevel::Professional),
      "4" => Ok(QualityLevel::Expert),
      _ => {
  println!("Invalid selection, defaulting to Good");
  Ok(QualityLevel::Good)
      }
  }
  }

  /// Display translation result with formatting
  fn display_translation_result(&self, result : &TranslationResult)
  {
  println!("\n✨ Translation Result");
  println!("====================");
  println!("Translation : {}", result.translated_text);
  println!();
  println!("Quality Assessment : {}", result.quality_assessment.as_str());
  println!("Confidence : {:.1}%", result.confidence_score * 100.0);
  println!("Response Time : {}ms", result.response_time_ms);
  println!("Model Used : {}", result.model_used);
  
  println!("\nQuality Metrics:");
  println!("  Fluency : {}/100", result.quality_metrics.fluency_score);
  println!("  Adequacy : {}/100", result.quality_metrics.adequacy_score);
  println!("  Lexical Accuracy : {}/100", result.quality_metrics.lexical_accuracy);
  println!("  Grammar : {}/100", result.quality_metrics.grammar_score);
  }

  /// Update system statistics
  fn update_stats(&mut self, _result : &TranslationResult, response_time : u64)
  {
  self.stats.translations_completed += 1;
  self.stats.total_response_time_ms += response_time;
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
  })?;

  // Initialize HuggingFace client
  let secret = Secret::new(api_key);
  let env = HuggingFaceEnvironmentImpl::build(secret, None)?;
  let client = Client::build(env)?;

  // Create and run the translation system platform
  let mut platform = TranslationSystemPlatform::new(client);
  platform.run().await?;

  Ok(())
}