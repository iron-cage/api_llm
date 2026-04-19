//! Sentiment Analysis & Content Moderation System Example
//!
//! This example demonstrates a comprehensive sentiment analysis and content moderation platform
//! that analyzes text sentiment, emotional tone, and provides automated content filtering.
//!
//! The system includes:
//! - Advanced sentiment classification with confidence scoring
//! - Batch processing for large dataset analysis
//! - Content moderation and automated filtering
//! - Emotional tone detection and categorization
//! - Statistical analysis and trend reporting
//! - Real-time sentiment monitoring capabilities

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

/// Sentiment classification categories
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub enum SentimentCategory
{
  /// Very positive sentiment
  VeryPositive,
  /// Positive sentiment
  Positive,
  /// Neutral sentiment
  Neutral,
  /// Negative sentiment
  Negative,
  /// Very negative sentiment
  VeryNegative,
}

impl SentimentCategory
{
  /// Get sentiment name as string
  pub fn name(&self) -> &'static str 
  {
  match self
  {
      SentimentCategory::VeryPositive => "Very Positive",
      SentimentCategory::Positive => "Positive",
      SentimentCategory::Neutral => "Neutral",
      SentimentCategory::Negative => "Negative",
      SentimentCategory::VeryNegative => "Very Negative",
  }
  }

  /// Get sentiment score range (0.0 to 1.0)
  pub fn score_range(&self) -> (f32, f32) 
  {
  match self
  {
      SentimentCategory::VeryPositive => (0.8, 1.0),
      SentimentCategory::Positive => (0.6, 0.8),
      SentimentCategory::Neutral => (0.4, 0.6),
      SentimentCategory::Negative => (0.2, 0.4),
      SentimentCategory::VeryNegative => (0.0, 0.2),
  }
  }

  /// Get sentiment polarity (-1.0 to 1.0)
  pub fn polarity(&self) -> f32 
  {
  match self
  {
      SentimentCategory::VeryPositive => 1.0,
      SentimentCategory::Positive => 0.5,
      SentimentCategory::Neutral => 0.0,
      SentimentCategory::Negative => -0.5,
      SentimentCategory::VeryNegative => -1.0,
  }
  }

  /// Get preferred model for sentiment analysis
  pub fn preferred_model() -> &'static str 
  {
  "cardiffnlp/twitter-roberta-base-sentiment-latest"
  }

  /// Create sentiment category from score (0.0 to 1.0)
  pub fn from_score(score : f32) -> Self 
  {
  if score >= 0.8
  {
      SentimentCategory::VeryPositive
  } else if score >= 0.6
  {
      SentimentCategory::Positive
  } else if score >= 0.4
  {
      SentimentCategory::Neutral
  } else if score >= 0.2
  {
      SentimentCategory::Negative
  } else {
      SentimentCategory::VeryNegative
  }
  }

  /// Get display string with score range
  pub fn display_with_range(&self) -> String 
  {
  let (min, max) = self.score_range();
  format!("{} ({:.1}-{:.1})", self.name(), min, max)
  }
}

/// Emotional tone categories
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub enum EmotionalTone
{
  /// Joy and happiness
  Joy,
  /// Sadness and melancholy
  Sadness,
  /// Anger and frustration
  Anger,
  /// Fear and anxiety
  Fear,
  /// Surprise and amazement
  Surprise,
  /// Disgust and repulsion
  Disgust,
  /// Trust and confidence
  Trust,
  /// Anticipation and excitement
  Anticipation,
}

impl EmotionalTone
{
  /// Get emotion name
  pub fn name(&self) -> &'static str 
  {
  match self
  {
      EmotionalTone::Joy => "Joy",
      EmotionalTone::Sadness => "Sadness",
      EmotionalTone::Anger => "Anger",
      EmotionalTone::Fear => "Fear",
      EmotionalTone::Surprise => "Surprise",
      EmotionalTone::Disgust => "Disgust",
      EmotionalTone::Trust => "Trust",
      EmotionalTone::Anticipation => "Anticipation",
  }
  }

  /// Get associated sentiment bias
  pub fn sentiment_bias(&self) -> SentimentCategory 
  {
  match self
  {
      EmotionalTone::Joy => SentimentCategory::VeryPositive,
      EmotionalTone::Trust => SentimentCategory::Positive,
      EmotionalTone::Anticipation => SentimentCategory::Positive,
      EmotionalTone::Surprise => SentimentCategory::Neutral,
      EmotionalTone::Sadness => SentimentCategory::Negative,
      EmotionalTone::Fear => SentimentCategory::Negative,
      EmotionalTone::Anger => SentimentCategory::VeryNegative,
      EmotionalTone::Disgust => SentimentCategory::VeryNegative,
  }
  }

  /// Get all emotional tones
  pub fn all_tones() -> Vec< EmotionalTone > 
  {
  vec![
      EmotionalTone::Joy,
      EmotionalTone::Sadness,
      EmotionalTone::Anger,
      EmotionalTone::Fear,
      EmotionalTone::Surprise,
      EmotionalTone::Disgust,
      EmotionalTone::Trust,
      EmotionalTone::Anticipation,
  ]
  }

  /// Get emotion icon for display
  pub fn icon(&self) -> &'static str 
  {
  match self
  {
      EmotionalTone::Joy => "😊",
      EmotionalTone::Sadness => "😢",
      EmotionalTone::Anger => "😠",
      EmotionalTone::Fear => "😨",
      EmotionalTone::Surprise => "😮",
      EmotionalTone::Disgust => "🤢",
      EmotionalTone::Trust => "🤝",
      EmotionalTone::Anticipation => "🤗",
  }
  }
}

/// Content moderation categories
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub enum ContentCategory
{
  /// Safe, appropriate content
  Safe,
  /// Potentially inappropriate content
  Questionable,
  /// Harmful or toxic content
  Harmful,
  /// Spam or promotional content
  Spam,
  /// Hate speech or discriminatory content
  HateSpeech,
  /// Violent or threatening content
  Violence,
}

impl ContentCategory
{
  /// Get category name
  pub fn name(&self) -> &'static str 
  {
  match self
  {
      ContentCategory::Safe => "Safe",
      ContentCategory::Questionable => "Questionable",
      ContentCategory::Harmful => "Harmful",
      ContentCategory::Spam => "Spam",
      ContentCategory::HateSpeech => "Hate Speech",
      ContentCategory::Violence => "Violence",
  }
  }

  /// Get severity level (1-5)
  pub fn severity_level(&self) -> u8 
  {
  match self
  {
      ContentCategory::Safe => 1,
      ContentCategory::Questionable => 2,
      ContentCategory::Spam => 3,
      ContentCategory::Harmful => 4,
      ContentCategory::HateSpeech => 5,
      ContentCategory::Violence => 5,
  }
  }

  /// Check if content should be blocked
  pub fn should_block(&self) -> bool 
  {
  matches!(
      self,
      ContentCategory::Harmful | ContentCategory::HateSpeech | ContentCategory::Violence
  )
  }

  /// Get preferred model for content moderation
  pub fn preferred_model() -> &'static str 
  {
  "unitary/toxic-bert"
  }

  /// Get category color for display
  pub fn color(&self) -> &'static str 
  {
  match self
  {
      ContentCategory::Safe => "🟢",
      ContentCategory::Questionable => "🟡",
      ContentCategory::Spam => "🟠",
      ContentCategory::Harmful => "🔴",
      ContentCategory::HateSpeech => "🚫",
      ContentCategory::Violence => "⛔",
  }
  }
}

/// Moderation action recommendations
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
pub enum ModerationAction
{
  /// Allow content to be published
  Allow,
  /// Require review before publishing
  Review,
  /// Block content from being published
  Block,
  /// Flag for administrator attention
  Flag,
}

impl ModerationAction
{
  /// Get action name
  pub fn name(&self) -> &'static str 
  {
  match self
  {
      ModerationAction::Allow => "Allow",
      ModerationAction::Review => "Review",
      ModerationAction::Block => "Block",
      ModerationAction::Flag => "Flag",
  }
  }

  /// Get action severity (1-4)
  pub fn severity(&self) -> u8 
  {
  match self
  {
      ModerationAction::Allow => 1,
      ModerationAction::Review => 2,
      ModerationAction::Flag => 3,
      ModerationAction::Block => 4,
  }
  }

  /// Get action icon
  pub fn icon(&self) -> &'static str 
  {
  match self
  {
      ModerationAction::Allow => "✅",
      ModerationAction::Review => "⚠️",
      ModerationAction::Flag => "🚩",
      ModerationAction::Block => "🚫",
  }
  }
}

/// Sentiment analysis result
#[ derive( Debug, Clone ) ]
pub struct SentimentResult
{
  /// Text that was analyzed
  pub text : String,
  /// Primary sentiment classification
  pub sentiment : SentimentCategory,
  /// Confidence in sentiment classification (0.0-1.0)
  pub confidence : f32,
  /// Sentiment score (0.0-1.0, where 1.0 is most positive)
  pub sentiment_score : f32,
  /// Detected emotional tones with intensity scores
  pub emotional_tones : Vec< (EmotionalTone, f32) >,
  /// Content moderation assessment
  pub content_assessment : ContentModerationResult,
  /// Processing time in milliseconds
  pub processing_time_ms : u64,
}

/// Content moderation result
#[ derive( Debug, Clone ) ]
pub struct ContentModerationResult
{
  /// Content category classification
  pub category : ContentCategory,
  /// Confidence in moderation decision (0.0-1.0)
  pub confidence : f32,
  /// Toxicity score (0.0-1.0, where 1.0 is most toxic)
  pub toxicity_score : f32,
  /// Specific flags that were triggered
  pub flags : Vec< String >,
  /// Recommendation for content handling
  pub recommendation : ModerationAction,
}

/// Batch sentiment analysis request
#[ derive( Debug, Clone ) ]
pub struct BatchSentimentRequest
{
  /// Texts to analyze
  pub texts : Vec< String >,
  /// Include emotional tone analysis
  pub include_emotional_analysis : bool,
  /// Include content moderation
  pub include_moderation : bool,
  /// Batch processing options
  pub batch_options : BatchOptions,
}

/// Batch processing configuration
#[ derive( Debug, Clone ) ]
pub struct BatchOptions
{
  /// Maximum batch size per API call
  pub max_batch_size : usize,
  /// Enable parallel processing
  pub parallel_processing : bool,
  /// Progress reporting interval
  pub progress_interval : Option< usize >,
  /// Minimum confidence threshold for results
  pub confidence_threshold : f32,
}

impl Default for BatchOptions
{
  fn default() -> Self 
  {
  Self {
      max_batch_size : 20,
      parallel_processing : true,
      progress_interval : Some(10),
      confidence_threshold : 0.5,
  }
  }
}

/// Statistical analysis of sentiment results
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct SentimentStatistics
{
  /// Total number of analyzed texts
  pub total_count : usize,
  /// Distribution of sentiment categories
  pub sentiment_distribution : HashMap< SentimentCategory, usize >,
  /// Average sentiment score
  pub average_sentiment_score : f32,
  /// Standard deviation of sentiment scores
  pub sentiment_score_std_dev : f32,
  /// Most common emotional tones
  pub top_emotional_tones : Vec< (EmotionalTone, f32) >,
  /// Content moderation summary
  pub moderation_summary : ModerationStatistics,
  /// Processing performance metrics
  pub performance_metrics : PerformanceMetrics,
}

/// Content moderation statistics
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ModerationStatistics
{
  /// Distribution of content categories
  pub category_distribution : HashMap< ContentCategory, usize >,
  /// Average toxicity score
  pub average_toxicity_score : f32,
  /// Number of blocked contents
  pub blocked_count : usize,
  /// Number of flagged contents
  pub flagged_count : usize,
  /// Most common flags
  pub common_flags : Vec< (String, usize) >,
}

/// Performance metrics
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct PerformanceMetrics
{
  /// Average processing time per text (milliseconds)
  pub average_processing_time : f64,
  /// Total processing time (milliseconds)
  pub total_processing_time : u64,
  /// Processing throughput (texts per second)
  pub throughput : f64,
  /// Memory usage estimate (MB)
  pub memory_usage_mb : f32,
}

/// Platform configuration
#[ derive( Debug, Clone ) ]
pub struct PlatformConfig
{
  /// Default sentiment analysis model
  pub sentiment_model : String,
  /// Default content moderation model
  pub moderation_model : String,
  /// Default confidence threshold
  pub confidence_threshold : f32,
  /// Enable emotional tone analysis
  pub enable_emotional_analysis : bool,
  /// Enable content moderation
  pub enable_content_moderation : bool,
  /// Maximum text length for analysis
  pub max_text_length : usize,
}

impl Default for PlatformConfig
{
  fn default() -> Self 
  {
  Self {
      sentiment_model : SentimentCategory::preferred_model().to_string(),
      moderation_model : ContentCategory::preferred_model().to_string(),
      confidence_threshold : 0.6,
      enable_emotional_analysis : true,
      enable_content_moderation : true,
      max_text_length : 512,
  }
  }
}

/// Sentiment analysis and content moderation platform
#[ derive( Debug ) ]
pub struct SentimentAnalysisPlatform
{
  /// HuggingFace API client
  client : Client< HuggingFaceEnvironmentImpl >,
  /// Platform configuration
  config : PlatformConfig,
  /// Analysis history for statistics
  analysis_history : Vec< SentimentResult >,
  /// Performance tracking
  performance_stats : PerformanceMetrics,
}

impl SentimentAnalysisPlatform
{
  /// Create a new sentiment analysis platform
  pub fn new(client : Client< HuggingFaceEnvironmentImpl >) -> Self 
  {
  Self {
      client,
      config : PlatformConfig::default(),
      analysis_history : Vec::new(),
      performance_stats : PerformanceMetrics {
  average_processing_time : 0.0,
  total_processing_time : 0,
  throughput : 0.0,
  memory_usage_mb : 0.0,
      },
  }
  }

  /// Create platform with custom configuration
  pub fn with_config(client : Client< HuggingFaceEnvironmentImpl >, config : PlatformConfig) -> Self 
  {
  Self {
      client,
      config,
      analysis_history : Vec::new(),
      performance_stats : PerformanceMetrics {
  average_processing_time : 0.0,
  total_processing_time : 0,
  throughput : 0.0,
  memory_usage_mb : 0.0,
      },
  }
  }

  /// Analyze sentiment of a single text
  pub async fn analyze_sentiment(&mut self, text : &str) -> Result< SentimentResult, Box< dyn std::error::Error > > 
  {
  let start_time = Instant::now();

  // Validate text length
  if text.len() > self.config.max_text_length
  {
      return Err(format!(
  "Text length {} exceeds maximum {}",
  text.len(),
  self.config.max_text_length
      )
      .into());
  }

  // Build sentiment analysis prompt
  let sentiment_prompt = self.build_sentiment_prompt(text)?;

  // Set analysis parameters
  let params = InferenceParameters::new()
      .with_max_new_tokens(50)
      .with_temperature(0.1) // Lower temperature for consistent classification
      .with_top_p(0.8);

  // Perform sentiment analysis
  let response = self
      .client
      .inference()
      .create_with_parameters(&sentiment_prompt, &self.config.sentiment_model, params)
      .await?;

  // Process sentiment response
  let sentiment_text = response.extract_text_or_default( "neutral" );

  let processing_time = start_time.elapsed().as_millis() as u64;

  // Parse sentiment classification
  let (sentiment, confidence, sentiment_score) = self.parse_sentiment_response(&sentiment_text)?;

  // Perform emotional tone analysis if enabled
  let emotional_tones = if self.config.enable_emotional_analysis
  {
      self.analyze_emotional_tones(text).await?
  } else {
      Vec::new()
  };

  // Perform content moderation if enabled
  let content_assessment = if self.config.enable_content_moderation
  {
      self.moderate_content(text).await?
  } else {
      ContentModerationResult {
  category : ContentCategory::Safe,
  confidence : 1.0,
  toxicity_score : 0.0,
  flags : Vec::new(),
  recommendation : ModerationAction::Allow,
      }
  };

  let result = SentimentResult {
      text : text.to_string(),
      sentiment,
      confidence,
      sentiment_score,
      emotional_tones,
      content_assessment,
      processing_time_ms : processing_time,
  };

  // Update performance statistics
  self.update_performance_stats(processing_time);

  // Store result for statistics
  self.analysis_history.push(result.clone());

  Ok(result)
  }

  /// Analyze multiple texts in batch
  pub async fn analyze_batch(&mut self, request : &BatchSentimentRequest) -> Result< Vec< Result< SentimentResult, Box< dyn std::error::Error > > >, Box< dyn std::error::Error > > 
  {
  let mut results = Vec::new();
  let batch_size = request.batch_options.max_batch_size.min(request.texts.len());

  for (chunk_idx, chunk) in request.texts.chunks(batch_size).enumerate()
  {
      let mut chunk_results = Vec::new();

      if request.batch_options.parallel_processing
      {
  // Process chunk in parallel (simulated for this example)
  for text in chunk
  {
          let result = self.analyze_sentiment(text).await;
          chunk_results.push(result);
  }
      } else {
  // Process chunk sequentially
  for text in chunk
  {
          let result = self.analyze_sentiment(text).await;
          chunk_results.push(result);
  }
      }

      // Report progress if configured
      if let Some(interval) = request.batch_options.progress_interval
      {
  if (chunk_idx + 1) % interval == 0
  {
          println!(
      "Processed {} batches of {} texts",
      chunk_idx + 1,
      batch_size
          );
  }
      }

      results.extend(chunk_results);
  }

  Ok(results)
  }

  /// Generate statistical analysis of results
  pub fn generate_statistics(&self) -> SentimentStatistics 
  {
  let mut sentiment_distribution = HashMap::new();
  let mut total_sentiment_score = 0.0;
  let mut total_toxicity_score = 0.0;
  let mut emotional_tone_counts : HashMap< EmotionalTone, f32 > = HashMap::new();
  let mut category_distribution = HashMap::new();
  let mut flag_counts : HashMap< String, usize > = HashMap::new();
  let mut blocked_count = 0;
  let mut flagged_count = 0;

  // Collect statistics from analysis history
  for result in &self.analysis_history
  {
      // Sentiment distribution
      *sentiment_distribution.entry(result.sentiment).or_insert(0) += 1;
      total_sentiment_score += result.sentiment_score;

      // Emotional tones
      for (tone, intensity) in &result.emotional_tones
      {
  *emotional_tone_counts.entry(*tone).or_insert(0.0) += intensity;
      }

      // Content moderation
      *category_distribution
  .entry(result.content_assessment.category)
  .or_insert(0) += 1;
      total_toxicity_score += result.content_assessment.toxicity_score;

      // Flags and actions
      for flag in &result.content_assessment.flags
      {
  *flag_counts.entry(flag.clone()).or_insert(0) += 1;
      }

      if result.content_assessment.recommendation == ModerationAction::Block
      {
  blocked_count += 1;
      }

      if result.content_assessment.recommendation == ModerationAction::Flag
      {
  flagged_count += 1;
      }
  }

  let total_count = self.analysis_history.len();
  let average_sentiment_score = if total_count > 0
  {
      total_sentiment_score / total_count as f32
  } else {
      0.0
  };
  let average_toxicity_score = if total_count > 0
  {
      total_toxicity_score / total_count as f32
  } else {
      0.0
  };

  // Calculate standard deviation
  let sentiment_score_std_dev = if total_count > 1
  {
      let variance = self
  .analysis_history
  .iter()
  .map(|result| (result.sentiment_score - average_sentiment_score).powi(2))
  .sum::< f32 >()
  / (total_count - 1) as f32;
      variance.sqrt()
  } else {
      0.0
  };

  // Sort emotional tones by frequency
  let mut top_emotional_tones : Vec< (EmotionalTone, f32) > = emotional_tone_counts.into_iter().collect();
  top_emotional_tones.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
  top_emotional_tones.truncate(5);

  // Sort flags by frequency
  let mut common_flags : Vec< (String, usize) > = flag_counts.into_iter().collect();
  common_flags.sort_by(|a, b| b.1.cmp(&a.1));
  common_flags.truncate(5);

  SentimentStatistics {
      total_count,
      sentiment_distribution,
      average_sentiment_score,
      sentiment_score_std_dev,
      top_emotional_tones,
      moderation_summary : ModerationStatistics {
  category_distribution,
  average_toxicity_score,
  blocked_count,
  flagged_count,
  common_flags,
      },
      performance_metrics : self.performance_stats.clone(),
  }
  }

  /// Clear analysis history
  pub fn clear_history(&mut self) 
  {
  self.analysis_history.clear();
  self.performance_stats = PerformanceMetrics {
      average_processing_time : 0.0,
      total_processing_time : 0,
      throughput : 0.0,
      memory_usage_mb : 0.0,
  };
  }

  /// Get analysis history
  pub fn get_history(&self) -> &Vec< SentimentResult > 
  {
  &self.analysis_history
  }

  /// Build sentiment analysis prompt
  fn build_sentiment_prompt(&self, text : &str) -> Result< String, Box< dyn std::error::Error > > 
  {
  let prompt = format!(
      "Analyze the sentiment of the following text and classify it as very positive, positive, neutral, negative, or very negative:\n\nText : {}\n\nSentiment:",
      text
  );
  Ok(prompt)
  }

  /// Parse sentiment analysis response
  fn parse_sentiment_response(&self, response : &str) -> Result< (SentimentCategory, f32, f32), Box< dyn std::error::Error > > 
  {
  let response_lower = response.trim().to_lowercase();

  // Simple keyword-based sentiment classification for testing
  let (sentiment, base_confidence) = if response_lower.contains("very positive") || response_lower.contains("excellent") || response_lower.contains("amazing")
  {
      (SentimentCategory::VeryPositive, 0.9)
  } else if response_lower.contains("positive") || response_lower.contains("good") || response_lower.contains("nice")
  {
      (SentimentCategory::Positive, 0.8)
  } else if response_lower.contains("neutral") || response_lower.contains("okay")
  {
      (SentimentCategory::Neutral, 0.7)
  } else if response_lower.contains("negative") || response_lower.contains("bad")
  {
      (SentimentCategory::Negative, 0.8)
  } else if response_lower.contains("very negative") || response_lower.contains("terrible") || response_lower.contains("awful")
  {
      (SentimentCategory::VeryNegative, 0.9)
  } else {
      // Default to neutral with lower confidence
      (SentimentCategory::Neutral, 0.5)
  };

  let confidence = (base_confidence * (0.8 + (response.len() as f32 / 100.0).min(0.2))).min(1.0);
  let sentiment_score = sentiment.polarity() * 0.5 + 0.5; // Convert polarity to 0-1 scale

  Ok((sentiment, confidence, sentiment_score))
  }

  /// Analyze emotional tones (simplified implementation)
  async fn analyze_emotional_tones(&self, text : &str) -> Result< Vec< (EmotionalTone, f32) >, Box< dyn std::error::Error > > 
  {
  // Simplified emotion detection based on keywords
  let mut tones = Vec::new();
  let text_lower = text.to_lowercase();

  // Joy keywords
  if text_lower.contains("happy") || text_lower.contains("joy") || text_lower.contains("excited") || text_lower.contains("wonderful")
  {
      tones.push((EmotionalTone::Joy, 0.8));
  }

  // Sadness keywords
  if text_lower.contains("sad") || text_lower.contains("depressed") || text_lower.contains("disappointed")
  {
      tones.push((EmotionalTone::Sadness, 0.7));
  }

  // Anger keywords
  if text_lower.contains("angry") || text_lower.contains("furious") || text_lower.contains("hate")
  {
      tones.push((EmotionalTone::Anger, 0.8));
  }

  // Fear keywords
  if text_lower.contains("scared") || text_lower.contains("afraid") || text_lower.contains("worried")
  {
      tones.push((EmotionalTone::Fear, 0.7));
  }

  // If no specific emotions detected, add neutral emotions with low intensity
  if tones.is_empty()
  {
      tones.push((EmotionalTone::Trust, 0.3));
  }

  Ok(tones)
  }

  /// Perform content moderation (simplified implementation)
  async fn moderate_content(&self, text : &str) -> Result< ContentModerationResult, Box< dyn std::error::Error > > 
  {
  let text_lower = text.to_lowercase();
  let mut flags = Vec::new();
  let mut toxicity_score : f32 = 0.0;

  // Simple keyword-based moderation
  if text_lower.contains("hate") || text_lower.contains("stupid") || text_lower.contains("idiot")
  {
      flags.push("potential_hate_speech".to_string());
      toxicity_score += 0.3;
  }

  if text_lower.contains("kill") || text_lower.contains("violence") || text_lower.contains("hurt")
  {
      flags.push("violence_threat".to_string());
      toxicity_score += 0.4;
  }

  if text_lower.contains("spam") || text_lower.contains("click here") || text_lower.contains("buy now")
  {
      flags.push("promotional_content".to_string());
      toxicity_score += 0.2;
  }

  // Determine category and recommendation based on flags and score
  let (category, recommendation) = if toxicity_score >= 0.7
  {
      if flags.iter().any(|f| f.contains("violence"))
      {
  (ContentCategory::Violence, ModerationAction::Block)
      } else if flags.iter().any(|f| f.contains("hate"))
      {
  (ContentCategory::HateSpeech, ModerationAction::Block)
      } else {
  (ContentCategory::Harmful, ModerationAction::Flag)
      }
  } else if toxicity_score >= 0.4
  {
      (ContentCategory::Questionable, ModerationAction::Review)
  } else if toxicity_score >= 0.2
  {
      if flags.iter().any(|f| f.contains("promotional"))
      {
  (ContentCategory::Spam, ModerationAction::Review)
      } else {
  (ContentCategory::Questionable, ModerationAction::Allow)
      }
  } else {
      (ContentCategory::Safe, ModerationAction::Allow)
  };

  let confidence = if flags.is_empty()
  {
      0.9
  } else {
      0.7 + (flags.len() as f32 * 0.1).min(0.2)
  };

  Ok(ContentModerationResult {
      category,
      confidence,
      toxicity_score : toxicity_score.min(1.0),
      flags,
      recommendation,
  })
  }

  /// Update performance statistics
  fn update_performance_stats(&mut self, processing_time : u64) 
  {
  let history_count = self.analysis_history.len() as u64;

  self.performance_stats.total_processing_time += processing_time;
  self.performance_stats.average_processing_time =
      self.performance_stats.total_processing_time as f64 / (history_count + 1) as f64;

  if self.performance_stats.total_processing_time > 0
  {
      self.performance_stats.throughput = (history_count + 1) as f64 * 1000.0
  / self.performance_stats.total_processing_time as f64;
  }

  // Estimate memory usage (simplified)
  self.performance_stats.memory_usage_mb = (history_count + 1) as f32 * 0.1; // ~0.1 MB per analysis
  }
}

/// Interactive Sentiment Analysis Platform
#[ derive( Debug ) ]
pub struct SentimentSystemPlatform
{
  sentiment_platform : SentimentAnalysisPlatform,
  stats : SystemStats,
  sample_texts : Vec< String >,
}

/// System usage statistics
#[ derive( Debug, Default, Serialize, Deserialize ) ]
pub struct SystemStats
{
  analyses_completed : usize,
  batch_analyses_completed : usize,
  total_response_time_ms : u64,
  emotions_detected : usize,
  content_blocked : usize,
}

impl SentimentSystemPlatform
{
  /// Create a new sentiment system platform
  pub fn new(client : Client< HuggingFaceEnvironmentImpl >) -> Self 
  {
  let mut platform = Self {
      sentiment_platform : SentimentAnalysisPlatform::new(client),
      stats : SystemStats::default(),
      sample_texts : Vec::new(),
  };

  platform.load_sample_data();
  platform
  }

  /// Load sample texts for testing
  fn load_sample_data(&mut self) 
  {
  self.sample_texts = vec![
      "I absolutely love this product! It's amazing and works perfectly.".to_string(),
      "This is okay, nothing special but not bad either.".to_string(),
      "I hate this so much, it's terrible and doesn't work at all.".to_string(),
      "The weather is nice today.".to_string(),
      "I'm so excited about the upcoming vacation!".to_string(),
      "I'm really worried about the exam tomorrow.".to_string(),
      "This movie was incredibly boring and disappointing.".to_string(),
      "Thank you so much for your help, I really appreciate it.".to_string(),
      "Click here to buy now! Amazing deal, don't miss out!".to_string(),
      "I'm feeling sad and depressed lately.".to_string(),
      "This is the best day ever, I'm so happy!".to_string(),
      "I'm scared about what might happen next.".to_string(),
  ];
  }

  /// Run the interactive sentiment analysis system
  pub async fn run(&mut self) -> Result< (), Box< dyn std::error::Error > > 
  {
  println!("🎭 Sentiment Analysis & Content Moderation System");
  println!("================================================");
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
          println!("Thanks for using the sentiment analysis system!");
          break;
  }
  "/analyze" => self.analyze_interactive().await?,
  "/batch" => self.batch_analyze_interactive().await?,
  "/samples" => self.analyze_samples().await?,
  "/emotions" => self.show_emotion_guide(),
  "/moderation" => self.show_moderation_guide(),
  "/stats" => self.show_statistics(),
  "/export" => self.export_results()?,
  "/clear" => self.clear_history(),
  cmd if cmd.starts_with('/') =>
  {
          println!("❌ Unknown command : {}. Type /help for available commands.", cmd);
  }
  text => {
          // Direct sentiment analysis
          self.quick_analyze(text).await?;
  }
      }
  }

  Ok(())
  }

  /// Show help information
  fn show_help(&self) 
  {
  println!("Available commands:");
  println!("  /analyze    - Interactive sentiment analysis with full options");
  println!("  /batch      - Batch analysis of multiple texts");
  println!("  /samples    - Analyze pre-loaded sample texts");
  println!("  /emotions   - Show emotional tone reference guide");
  println!("  /moderation - Show content moderation categories");
  println!("  /stats      - Show comprehensive system statistics");
  println!("  /export     - Export analysis results and statistics");
  println!("  /clear      - Clear analysis history");
  println!("  /help       - Show this help");
  println!("  /quit       - Exit the system");
  println!();
  println!("You can also type text directly for quick sentiment analysis.");
  println!("Example : I love this product!");
  }

  /// Quick sentiment analysis
  async fn quick_analyze(&mut self, text : &str) -> Result< (), Box< dyn std::error::Error > > 
  {
  println!("\n🔍 Analyzing : \"{}\"", text);

  let start_time = std::time::Instant::now();
  match self.sentiment_platform.analyze_sentiment(text).await
  {
      Ok(result) => {
  self.display_analysis_result(&result);
  self.update_stats(&result, start_time.elapsed().as_millis() as u64);
      }
      Err(e) => {
  println!("❌ Analysis failed : {}", e);
      }
  }

  Ok(())
  }

  /// Interactive sentiment analysis with options
  async fn analyze_interactive(&mut self) -> Result< (), Box< dyn std::error::Error > > 
  {
  println!("\n📝 Interactive Sentiment Analysis");
  println!("=================================");

  print!("Enter text to analyze : ");
  io::stdout().flush()?;
  let mut text = String::new();
  io::stdin().read_line(&mut text)?;
  let text = text.trim();

  if text.is_empty()
  {
      println!("❌ Text cannot be empty.");
      return Ok(());
  }

  if text.len() > self.sentiment_platform.config.max_text_length
  {
      println!(
  "⚠️  Text length ({}) exceeds maximum ({}). Truncating...",
  text.len(),
  self.sentiment_platform.config.max_text_length
      );
  }

  println!("\n🔍 Analyzing sentiment and content...");

  let start_time = std::time::Instant::now();
  match self.sentiment_platform.analyze_sentiment(text).await
  {
      Ok(result) => {
  self.display_detailed_analysis(&result);
  self.update_stats(&result, start_time.elapsed().as_millis() as u64);
      }
      Err(e) => {
  println!("❌ Analysis failed : {}", e);
      }
  }

  Ok(())
  }

  /// Batch analysis interface
  async fn batch_analyze_interactive(&mut self) -> Result< (), Box< dyn std::error::Error > > 
  {
  println!("\n📦 Batch Sentiment Analysis");
  println!("===========================");

  println!("Enter texts to analyze (one per line, empty line to finish):");
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

  let batch_request = BatchSentimentRequest {
      texts,
      include_emotional_analysis : true,
      include_moderation : true,
      batch_options : BatchOptions::default(),
  };

  println!("\n🔄 Processing {} texts...", batch_request.texts.len());

  let start_time = std::time::Instant::now();
  match self.sentiment_platform.analyze_batch(&batch_request).await
  {
      Ok(results) => {
  println!("\n📊 Batch Analysis Results:");
  println!("=========================");
  
  for (i, result) in results.iter().enumerate()
  {
          println!("\n{}. \"{}\"", i + 1, batch_request.texts[i]);
          match result
          {
      Ok(analysis) => {
              println!("   Sentiment : {} {}", analysis.sentiment.name(), self.get_sentiment_icon(&analysis.sentiment));
              println!("   Confidence : {:.1}%", analysis.confidence * 100.0);
              if !analysis.emotional_tones.is_empty()
              {
        let emotions : Vec< String > = analysis.emotional_tones.iter()
                  .map(|(tone, intensity)| format!("{} {:.1}", tone.icon(), intensity))
                  .collect();
        println!("   Emotions : {}", emotions.join(" "));
              }
              println!("   Moderation : {} {}", analysis.content_assessment.category.color(), analysis.content_assessment.category.name());
      }
      Err(e) => {
              println!("   ❌ Failed : {}", e);
      }
          }
  }

  self.stats.batch_analyses_completed += 1;
  self.stats.total_response_time_ms += start_time.elapsed().as_millis() as u64;
      }
      Err(e) => {
  println!("❌ Batch analysis failed : {}", e);
      }
  }

  Ok(())
  }

  /// Analyze sample texts
  async fn analyze_samples(&mut self) -> Result< (), Box< dyn std::error::Error > > 
  {
  println!("\n📚 Sample Text Analysis");
  println!("=======================");

  for (i, text) in self.sample_texts.iter().enumerate()
  {
      println!("{}. \"{}\"", i + 1, text);
  }

  print!("\nSelect sample (1-{}) or 'all' for all samples : ", self.sample_texts.len());
  io::stdout().flush()?;

  let mut input = String::new();
  io::stdin().read_line(&mut input)?;
  let input = input.trim();

  if input.eq_ignore_ascii_case("all")
  {
      println!("\n🔄 Analyzing all {} samples...", self.sample_texts.len());
      
      // Clone the sample texts to avoid borrow checker issues
      let sample_texts = self.sample_texts.clone();
      for (i, text) in sample_texts.iter().enumerate()
      {
  println!("\n--- Sample {} ---", i + 1);
  match self.sentiment_platform.analyze_sentiment(text).await
  {
          Ok(result) => {
      self.display_analysis_result(&result);
      self.update_stats(&result, result.processing_time_ms);
          }
          Err(e) => {
      println!("❌ Analysis failed for sample {}: {}", i + 1, e);
          }
  }
      }
  } else if let Ok(index) = input.parse::< usize >()
  {
      if index > 0 && index <= self.sample_texts.len()
      {
  let text = &self.sample_texts[index - 1];
  println!("\n🔍 Analyzing sample {}: \"{}\"", index, text);
  
  let start_time = std::time::Instant::now();
  match self.sentiment_platform.analyze_sentiment(text).await
  {
          Ok(result) => {
      self.display_detailed_analysis(&result);
      self.update_stats(&result, start_time.elapsed().as_millis() as u64);
          }
          Err(e) => {
      println!("❌ Analysis failed : {}", e);
          }
  }
      } else {
  println!("❌ Invalid sample number.");
      }
  }

  Ok(())
  }

  /// Show emotion guide
  fn show_emotion_guide(&self) 
  {
  println!("\n😊 Emotional Tone Reference Guide");
  println!("=================================");
  
  for tone in EmotionalTone::all_tones()
  {
      println!("{} {} - Associated with {}", tone.icon(), tone.name(), tone.sentiment_bias().name());
  }
  
  println!("\nEmotional tones are detected based on keywords and context.");
  println!("Multiple tones can be present in a single text with varying intensities.");
  }

  /// Show moderation guide
  fn show_moderation_guide(&self) 
  {
  println!("\n🛡️  Content Moderation Categories");
  println!("=================================");
  
  let categories = [
      ContentCategory::Safe,
      ContentCategory::Questionable,
      ContentCategory::Spam,
      ContentCategory::Harmful,
      ContentCategory::HateSpeech,
      ContentCategory::Violence,
  ];
  
  for category in categories
  {
      println!("{} {} - Severity Level : {}, Should Block : {}",
               category.color(),
               category.name(),
               category.severity_level(),
               if category.should_block() { "Yes" } else { "No" });
  }
  
  println!("\nModeration Actions:");
  let actions = [
      ModerationAction::Allow,
      ModerationAction::Review,
      ModerationAction::Flag,
      ModerationAction::Block,
  ];
  
  for action in actions
  {
      println!("{} {} - Severity : {}", action.icon(), action.name(), action.severity());
  }
  }

  /// Show system statistics
  fn show_statistics(&self) 
  {
  let platform_stats = self.sentiment_platform.generate_statistics();
  
  println!("\n📊 System Statistics");
  println!("===================");
  println!("Individual Analyses : {}", self.stats.analyses_completed);
  println!("Batch Analyses : {}", self.stats.batch_analyses_completed);
  println!("Total Platform Analyses : {}", platform_stats.total_count);
  println!("Emotions Detected : {}", self.stats.emotions_detected);
  println!("Content Blocked : {}", self.stats.content_blocked);
  
  println!("\n📈 Performance Metrics");
  println!("======================");
  println!("Average Processing Time : {:.2}ms", platform_stats.performance_metrics.average_processing_time);
  println!("Total Processing Time : {}ms", platform_stats.performance_metrics.total_processing_time);
  println!("Throughput : {:.2} texts/second", platform_stats.performance_metrics.throughput);
  println!("Memory Usage : {:.1}MB", platform_stats.performance_metrics.memory_usage_mb);
  
  if !platform_stats.sentiment_distribution.is_empty()
  {
      println!("\n💭 Sentiment Distribution");
      println!("=========================");
      for (sentiment, count) in platform_stats.sentiment_distribution
      {
  println!("{} {}: {}", 
                 self.get_sentiment_icon(&sentiment),
                 sentiment.name(), 
                 count);
      }
      println!("Average Sentiment Score : {:.2}", platform_stats.average_sentiment_score);
      println!("Standard Deviation : {:.2}", platform_stats.sentiment_score_std_dev);
  }
  
  if !platform_stats.top_emotional_tones.is_empty()
  {
      println!("\n😊 Top Emotional Tones");
      println!("======================");
      for (tone, intensity) in platform_stats.top_emotional_tones
      {
  println!("{} {}: {:.2}", tone.icon(), tone.name(), intensity);
      }
  }
  
  if !platform_stats.moderation_summary.category_distribution.is_empty()
  {
      println!("\n🛡️  Content Moderation Summary");
      println!("=============================");
      for (category, count) in platform_stats.moderation_summary.category_distribution
      {
  println!("{} {}: {}", category.color(), category.name(), count);
      }
      println!("Average Toxicity Score : {:.2}", platform_stats.moderation_summary.average_toxicity_score);
      println!("Blocked : {}, Flagged : {}", 
               platform_stats.moderation_summary.blocked_count,
               platform_stats.moderation_summary.flagged_count);
  }
  }

  /// Export results to file
  fn export_results(&self) -> Result< (), Box< dyn std::error::Error > > 
  {
  println!("\n💾 Export Analysis Results");
  println!("==========================");
  
  print!("Export filename (press Enter for default): ");
  io::stdout().flush()?;
  let mut filename = String::new();
  io::stdin().read_line(&mut filename)?;
  let filename = filename.trim();
  
  let filename = if filename.is_empty()
  {
      "sentiment_analysis_results.json".to_string()
  } else {
      filename.to_string()
  };

  let stats = self.sentiment_platform.generate_statistics();
  let export_data = serde_json::json!({
      "system_stats": self.stats,
      "platform_statistics": stats,
      "total_analyses": self.sentiment_platform.analysis_history.len(),
      "export_timestamp": chrono::Utc::now().to_rfc3339()
  });

  std::fs::write(&filename, serde_json::to_string_pretty(&export_data)?)?;
  println!("✅ Results exported to : {}", filename);
  
  Ok(())
  }

  /// Clear analysis history
  fn clear_history(&mut self) 
  {
  self.sentiment_platform.clear_history();
  self.stats = SystemStats::default();
  println!("✅ Analysis history cleared.");
  }

  /// Display analysis result with basic formatting
  fn display_analysis_result(&self, result : &SentimentResult) 
  {
  println!("📊 Sentiment : {} {} (Confidence : {:.1}%)",
             self.get_sentiment_icon(&result.sentiment),
             result.sentiment.name(),
             result.confidence * 100.0);
  
  if !result.emotional_tones.is_empty()
  {
      let emotions : Vec< String > = result.emotional_tones.iter()
  .map(|(tone, intensity)| format!("{} {} ({:.1})", tone.icon(), tone.name(), intensity))
  .collect();
      println!("😊 Emotions : {}", emotions.join(", "));
  }
  
  println!("🛡️  Moderation : {} {} (Toxicity : {:.1}%)",
             result.content_assessment.category.color(),
             result.content_assessment.category.name(),
             result.content_assessment.toxicity_score * 100.0);
  
  if !result.content_assessment.flags.is_empty()
  {
      println!("🚩 Flags : {}", result.content_assessment.flags.join(", "));
  }
  
  println!("⏱️  Processing Time : {}ms", result.processing_time_ms);
  }

  /// Display detailed analysis result
  fn display_detailed_analysis(&self, result : &SentimentResult) 
  {
  println!("\n🎭 Detailed Analysis Results");
  println!("=============================");
  println!("Text : \"{}\"", result.text);
  println!();
  
  println!("📊 Sentiment Analysis:");
  println!("  Category : {} {}", self.get_sentiment_icon(&result.sentiment), result.sentiment.name());
  println!("  Score : {:.2} (Range : {:.1}-{:.1})", 
             result.sentiment_score,
             result.sentiment.score_range().0,
             result.sentiment.score_range().1);
  println!("  Polarity : {:.2}", result.sentiment.polarity());
  println!("  Confidence : {:.1}%", result.confidence * 100.0);
  
  if !result.emotional_tones.is_empty()
  {
      println!("\n😊 Emotional Tones:");
      for (tone, intensity) in &result.emotional_tones
      {
  println!("  {} {} - Intensity : {:.2} (Bias : {})",
                 tone.icon(),
                 tone.name(),
                 intensity,
                 tone.sentiment_bias().name());
      }
  }
  
  println!("\n🛡️  Content Moderation:");
  println!("  Category : {} {}", result.content_assessment.category.color(), result.content_assessment.category.name());
  println!("  Severity Level : {}", result.content_assessment.category.severity_level());
  println!("  Should Block : {}", if result.content_assessment.category.should_block() { "Yes" } else { "No" });
  println!("  Toxicity Score : {:.1}%", result.content_assessment.toxicity_score * 100.0);
  println!("  Confidence : {:.1}%", result.content_assessment.confidence * 100.0);
  println!("  Recommendation : {} {}", result.content_assessment.recommendation.icon(), result.content_assessment.recommendation.name());
  
  if !result.content_assessment.flags.is_empty()
  {
      println!("  Flags : {}", result.content_assessment.flags.join(", "));
  }
  
  println!("\n⏱️  Performance:");
  println!("  Processing Time : {}ms", result.processing_time_ms);
  }

  /// Get sentiment icon
  fn get_sentiment_icon(&self, sentiment : &SentimentCategory) -> &'static str 
  {
  match sentiment
  {
      SentimentCategory::VeryPositive => "😍",
      SentimentCategory::Positive => "😊",
      SentimentCategory::Neutral => "😐",
      SentimentCategory::Negative => "😞",
      SentimentCategory::VeryNegative => "😡",
  }
  }

  /// Update system statistics
  fn update_stats(&mut self, result : &SentimentResult, response_time : u64) 
  {
  self.stats.analyses_completed += 1;
  self.stats.total_response_time_ms += response_time;
  self.stats.emotions_detected += result.emotional_tones.len();
  
  if result.content_assessment.recommendation == ModerationAction::Block
  {
      self.stats.content_blocked += 1;
  }
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

  // Create and run the sentiment analysis system platform
  let mut platform = SentimentSystemPlatform::new(client);
  platform.run().await?;

  Ok(())
}