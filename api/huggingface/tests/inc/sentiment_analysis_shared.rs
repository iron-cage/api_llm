//! Tests for Sentiment Analysis & Content Moderation Example
//!
//! This test suite verifies the functionality of a sentiment analysis and content moderation system
//! that analyzes text sentiment, emotional tone, and provides content filtering using `HuggingFace` models.

#![ allow( clippy::trivially_copy_pass_by_ref ) ]

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
  #[ must_use ]
  pub fn name( &self ) -> &'static str
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
  #[ must_use ]
  pub fn score_range( &self ) -> ( f32, f32 )
  {
  match self
  {
      SentimentCategory::VeryPositive => ( 0.8, 1.0 ),
      SentimentCategory::Positive => ( 0.6, 0.8 ),
      SentimentCategory::Neutral => ( 0.4, 0.6 ),
      SentimentCategory::Negative => ( 0.2, 0.4 ),
      SentimentCategory::VeryNegative => ( 0.0, 0.2 ),
  }
  }

  /// Get sentiment polarity (-1.0 to 1.0)
  #[ must_use ]
  pub fn polarity( &self ) -> f32
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
  #[ must_use ]
  pub fn preferred_model() -> &'static str
  {
  "cardiffnlp/twitter-roberta-base-sentiment-latest"
  }

  /// Create sentiment category from score (0.0 to 1.0)
  #[ must_use ]
  pub fn from_score( score : f32 ) -> Self
  {
  if score >= 0.8 { SentimentCategory::VeryPositive }
  else if score >= 0.6 { SentimentCategory::Positive }
  else if score >= 0.4 { SentimentCategory::Neutral }
  else if score >= 0.2 { SentimentCategory::Negative }
  else { SentimentCategory::VeryNegative }
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
  #[ must_use ]
  pub fn name( &self ) -> &'static str
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
  #[ must_use ]
  pub fn sentiment_bias( &self ) -> SentimentCategory
  {
  match self
  {
      EmotionalTone::Joy => SentimentCategory::VeryPositive,
      EmotionalTone::Trust | EmotionalTone::Anticipation => SentimentCategory::Positive,
      EmotionalTone::Surprise => SentimentCategory::Neutral,
      EmotionalTone::Sadness | EmotionalTone::Fear => SentimentCategory::Negative,
      EmotionalTone::Anger | EmotionalTone::Disgust => SentimentCategory::VeryNegative,
  }
  }

  /// Get all emotional tones
  #[ must_use ]
  pub fn all_tones() -> Vec< EmotionalTone >
  {
  vec![ 
      EmotionalTone::Joy, EmotionalTone::Sadness, EmotionalTone::Anger, EmotionalTone::Fear,
      EmotionalTone::Surprise, EmotionalTone::Disgust, EmotionalTone::Trust, EmotionalTone::Anticipation
  ]
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
  #[ must_use ]
  pub fn name( &self ) -> &'static str
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
  #[ must_use ]
  pub fn severity_level( &self ) -> u8
  {
  match self
  {
      ContentCategory::Safe => 1,
      ContentCategory::Questionable => 2,
      ContentCategory::Spam => 3,
      ContentCategory::Harmful => 4,
      ContentCategory::HateSpeech | ContentCategory::Violence => 5,
  }
  }

  /// Check if content should be blocked
  #[ must_use ]
  pub fn should_block( &self ) -> bool
  {
  matches!( self, ContentCategory::Harmful | ContentCategory::HateSpeech | ContentCategory::Violence )
  }

  /// Get preferred model for content moderation
  #[ must_use ]
  pub fn preferred_model() -> &'static str
  {
  "unitary/toxic-bert"
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
  pub emotional_tones : Vec< ( EmotionalTone, f32 ) >,
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

/// Moderation action recommendations
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
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
  #[ must_use ]
  pub fn name( &self ) -> &'static str
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
  #[ must_use ]
  pub fn severity( &self ) -> u8
  {
  match self
  {
      ModerationAction::Allow => 1,
      ModerationAction::Review => 2,
      ModerationAction::Flag => 3,
      ModerationAction::Block => 4,
  }
  }
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
  Self
  {
      max_batch_size : 20,
      parallel_processing : true,
      progress_interval : Some( 10 ),
      confidence_threshold : 0.5,
  }
  }
}

/// Statistical analysis of sentiment results
#[ derive( Debug, Clone ) ]
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
  pub top_emotional_tones : Vec< ( EmotionalTone, f32 ) >,
  /// Content moderation summary
  pub moderation_summary : ModerationStatistics,
  /// Processing performance metrics
  pub performance_metrics : PerformanceMetrics,
}

/// Content moderation statistics
#[ derive( Debug, Clone ) ]
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
  pub common_flags : Vec< ( String, usize ) >,
}

/// Performance metrics
#[ derive( Debug, Clone ) ]
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

/// Sentiment analysis and content moderation platform
#[ derive( Debug, Clone ) ]
pub struct SentimentAnalysisPlatform
{
  /// `HuggingFace` API client
  client : Client< HuggingFaceEnvironmentImpl >,
  /// Platform configuration
  config : PlatformConfig,
  /// Analysis history for statistics
  analysis_history : Vec< SentimentResult >,
  /// Performance tracking
  performance_stats : PerformanceMetrics,
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
  Self
  {
      sentiment_model : SentimentCategory::preferred_model().to_string(),
      moderation_model : ContentCategory::preferred_model().to_string(),
      confidence_threshold : 0.6,
      enable_emotional_analysis : true,
      enable_content_moderation : true,
      max_text_length : 512,
  }
  }
}

impl SentimentAnalysisPlatform
{
  /// Create a new sentiment analysis platform
  #[ must_use ]
  pub fn new( client : Client< HuggingFaceEnvironmentImpl > ) -> Self
  {
  Self
  {
      client,
      config : PlatformConfig::default(),
      analysis_history : Vec::new(),
      performance_stats : PerformanceMetrics
      {
  average_processing_time : 0.0,
  total_processing_time : 0,
  throughput : 0.0,
  memory_usage_mb : 0.0,
      },
  }
  }

  /// Create platform with custom configuration
  #[ must_use ]
  pub fn with_config( client : Client< HuggingFaceEnvironmentImpl >, config : PlatformConfig ) -> Self
  {
  Self
  {
      client,
      config,
      analysis_history : Vec::new(),
      performance_stats : PerformanceMetrics
      {
  average_processing_time : 0.0,
  total_processing_time : 0,
  throughput : 0.0,
  memory_usage_mb : 0.0,
      },
  }
  }

  /// Analyze sentiment of a single text
  ///
  /// # Errors
  ///
  /// Returns an error if the text exceeds maximum length or if the API request fails.
  pub async fn analyze_sentiment( &mut self, text : &str ) -> Result< SentimentResult, Box< dyn std::error::Error > >
  {
  let start_time = Instant::now();

  // Validate text length
  if text.len() > self.config.max_text_length
  {
      return Err( format!( "Text length {} exceeds maximum {}", text.len(), self.config.max_text_length ).into() );
  }

  // Build sentiment analysis prompt
  let sentiment_prompt = Self::build_sentiment_prompt( text );

  // Set analysis parameters
  let params = InferenceParameters::new()
      .with_max_new_tokens( 50 )
      .with_temperature( 0.1 ) // Lower temperature for consistent classification
      .with_top_p( 0.8 );

  // Perform sentiment analysis
  let response = self.client.inference().create_with_parameters( 
      &sentiment_prompt, 
      &self.config.sentiment_model, 
      params 
  ).await?;

  // Process sentiment response
  let sentiment_text = response.extract_text_or_default( "neutral" );

  #[ allow( clippy::cast_possible_truncation ) ]
  let processing_time = start_time.elapsed().as_millis() as u64;

  // Parse sentiment classification
  let ( sentiment, confidence, sentiment_score ) = Self::parse_sentiment_response( &sentiment_text );

  // Perform emotional tone analysis if enabled
  let emotional_tones = if self.config.enable_emotional_analysis
  {
      Self::analyze_emotional_tones( text )
  }
  else
  {
      Vec::new()
  };

  // Perform content moderation if enabled
  let content_assessment = if self.config.enable_content_moderation
  {
      Self::moderate_content( text )
  }
  else
  {
      ContentModerationResult
      {
  category : ContentCategory::Safe,
  confidence : 1.0,
  toxicity_score : 0.0,
  flags : Vec::new(),
  recommendation : ModerationAction::Allow,
      }
  };

  let result = SentimentResult
  {
      text : text.to_string(),
      sentiment,
      confidence,
      sentiment_score,
      emotional_tones,
      content_assessment,
      processing_time_ms : processing_time,
  };

  // Update performance statistics
  self.update_performance_stats( processing_time );

  // Store result for statistics
  self.analysis_history.push( result.clone() );

  Ok( result )
  }

  /// Analyze multiple texts in batch
  ///
  /// # Errors
  ///
  /// Returns an error if the batch processing fails.
  pub async fn analyze_batch( &mut self, request : &BatchSentimentRequest ) -> Result< Vec< Result< SentimentResult, Box< dyn std::error::Error > > >, Box< dyn std::error::Error > >
  {
  let mut results = Vec::new();
  let batch_size = request.batch_options.max_batch_size.min( request.texts.len() );

  for ( chunk_idx, chunk ) in request.texts.chunks( batch_size ).enumerate()
  {
      let mut chunk_results = Vec::new();

      if request.batch_options.parallel_processing
      {
  // Process chunk in parallel (simulated for this example)
  for text in chunk
  {
          let result = self.analyze_sentiment( text ).await;
          chunk_results.push( result );
  }
      }
      else
      {
  // Process chunk sequentially
  for text in chunk
  {
          let result = self.analyze_sentiment( text ).await;
          chunk_results.push( result );
  }
      }

      // Report progress if configured
      if let Some( interval ) = request.batch_options.progress_interval
      {
  if ( chunk_idx + 1 ) % interval == 0
  {
          println!( "Processed {chunkplus} batches of {batch_size} texts", chunkplus = chunk_idx + 1 );
  }
      }

      results.extend( chunk_results );
  }

  Ok( results )
  }

  /// Generate statistical analysis of results
  #[ must_use ]
  pub fn generate_statistics( &self ) -> SentimentStatistics
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
      *sentiment_distribution.entry( result.sentiment ).or_insert( 0 ) += 1;
      total_sentiment_score += result.sentiment_score;

      // Emotional tones
      for ( tone, intensity ) in &result.emotional_tones
      {
  *emotional_tone_counts.entry( *tone ).or_insert( 0.0 ) += intensity;
      }

      // Content moderation
      *category_distribution.entry( result.content_assessment.category ).or_insert( 0 ) += 1;
      total_toxicity_score += result.content_assessment.toxicity_score;

      // Flags and actions
      for flag in &result.content_assessment.flags
      {
  *flag_counts.entry( flag.clone() ).or_insert( 0 ) += 1;
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
  let average_sentiment_score = if total_count > 0 { total_sentiment_score / total_count as f32 } else { 0.0 };
  let average_toxicity_score = if total_count > 0 { total_toxicity_score / total_count as f32 } else { 0.0 };

  // Calculate standard deviation
  let sentiment_score_std_dev = if total_count > 1
  {
      let variance = self.analysis_history.iter()
  .map( | result | ( result.sentiment_score - average_sentiment_score ).powi( 2 ) )
  .sum::< f32 >() / ( total_count - 1 ) as f32;
      variance.sqrt()
  }
  else
  {
      0.0
  };

  // Sort emotional tones by frequency
  let mut top_emotional_tones : Vec< ( EmotionalTone, f32 ) > = emotional_tone_counts.into_iter().collect();
  top_emotional_tones.sort_by( | a, b | b.1.partial_cmp( &a.1 ).unwrap_or( core::cmp::Ordering::Equal ) );
  top_emotional_tones.truncate( 5 );

  // Sort flags by frequency
  let mut common_flags : Vec< ( String, usize ) > = flag_counts.into_iter().collect();
  common_flags.sort_by_key( | ( _, count ) | core::cmp::Reverse( *count ) );
  common_flags.truncate( 5 );

  SentimentStatistics
  {
      total_count,
      sentiment_distribution,
      average_sentiment_score,
      sentiment_score_std_dev,
      top_emotional_tones,
      moderation_summary : ModerationStatistics
      {
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
  pub fn clear_history( &mut self )
  {
  self.analysis_history.clear();
  self.performance_stats = PerformanceMetrics
  {
      average_processing_time : 0.0,
      total_processing_time : 0,
      throughput : 0.0,
      memory_usage_mb : 0.0,
  };
  }

  /// Get analysis history
  #[ must_use ]
  pub fn get_history( &self ) -> &Vec< SentimentResult >
  {
  &self.analysis_history
  }

  /// Build sentiment analysis prompt
  fn build_sentiment_prompt( text : &str ) -> String
  {
  format!(
      "Analyze the sentiment of the following text and classify it as very positive, positive, neutral, negative, or very negative:\n\nText : {text}\n\nSentiment:"
  )
  }

  /// Parse sentiment analysis response
  fn parse_sentiment_response( response : &str ) -> ( SentimentCategory, f32, f32 )
  {
  let response_lower = response.trim().to_lowercase();
  
  // Simple keyword-based sentiment classification for testing
  let ( sentiment, base_confidence ) = if response_lower.contains( "very positive" ) || response_lower.contains( "excellent" ) || response_lower.contains( "amazing" )
  {
      ( SentimentCategory::VeryPositive, 0.9 )
  }
  else if response_lower.contains( "positive" ) || response_lower.contains( "good" ) || response_lower.contains( "nice" )
  {
      ( SentimentCategory::Positive, 0.8 )
  }
  else if response_lower.contains( "neutral" ) || response_lower.contains( "okay" )
  {
      ( SentimentCategory::Neutral, 0.7 )
  }
  else if response_lower.contains( "negative" ) || response_lower.contains( "bad" )
  {
      ( SentimentCategory::Negative, 0.8 )
  }
  else if response_lower.contains( "very negative" ) || response_lower.contains( "terrible" ) || response_lower.contains( "awful" )
  {
      ( SentimentCategory::VeryNegative, 0.9 )
  }
  else
  {
      // Default to neutral with lower confidence
      ( SentimentCategory::Neutral, 0.5 )
  };

  let confidence = ( base_confidence * ( 0.8 + ( response.len() as f32 / 100.0 ).min( 0.2 ) ) ).min( 1.0 );
  let sentiment_score = sentiment.polarity() * 0.5 + 0.5; // Convert polarity to 0-1 scale

  ( sentiment, confidence, sentiment_score )
  }

  /// Analyze emotional tones (simplified implementation)
  fn analyze_emotional_tones( text : &str ) -> Vec< ( EmotionalTone, f32 ) >
  {
  // Simplified emotion detection based on keywords
  let mut tones = Vec::new();
  let text_lower = text.to_lowercase();

  // Joy keywords
  if text_lower.contains( "happy" ) || text_lower.contains( "joy" ) || text_lower.contains( "excited" ) || text_lower.contains( "wonderful" )
  {
      tones.push( ( EmotionalTone::Joy, 0.8 ) );
  }

  // Sadness keywords
  if text_lower.contains( "sad" ) || text_lower.contains( "depressed" ) || text_lower.contains( "disappointed" )
  {
      tones.push( ( EmotionalTone::Sadness, 0.7 ) );
  }

  // Anger keywords
  if text_lower.contains( "angry" ) || text_lower.contains( "furious" ) || text_lower.contains( "hate" )
  {
      tones.push( ( EmotionalTone::Anger, 0.8 ) );
  }

  // Fear keywords
  if text_lower.contains( "scared" ) || text_lower.contains( "afraid" ) || text_lower.contains( "worried" )
  {
      tones.push( ( EmotionalTone::Fear, 0.7 ) );
  }

  // If no specific emotions detected, add neutral emotions with low intensity
  if tones.is_empty()
  {
      tones.push( ( EmotionalTone::Trust, 0.3 ) );
  }

  tones
  }

  /// Perform content moderation (simplified implementation)
  fn moderate_content( text : &str ) -> ContentModerationResult
  {
  let text_lower = text.to_lowercase();
  let mut flags = Vec::new();
  let mut toxicity_score : f32 = 0.0;

  // Simple keyword-based moderation
  if text_lower.contains( "hate" ) || text_lower.contains( "stupid" ) || text_lower.contains( "idiot" )
  {
      flags.push( "potential_hate_speech".to_string() );
      toxicity_score += 0.3;
  }

  if text_lower.contains( "kill" ) || text_lower.contains( "violence" ) || text_lower.contains( "hurt" )
  {
      flags.push( "violence_threat".to_string() );
      toxicity_score += 0.4;
  }

  if text_lower.contains( "spam" ) || text_lower.contains( "click here" ) || text_lower.contains( "buy now" )
  {
      flags.push( "promotional_content".to_string() );
      toxicity_score += 0.2;
  }

  // Determine category and recommendation based on flags and score
  let ( category, recommendation ) = if toxicity_score >= 0.7
  {
      if flags.iter().any( | f | f.contains( "violence" ) )
      {
  ( ContentCategory::Violence, ModerationAction::Block )
      }
      else if flags.iter().any( | f | f.contains( "hate" ) )
      {
  ( ContentCategory::HateSpeech, ModerationAction::Block )
      }
      else
      {
  ( ContentCategory::Harmful, ModerationAction::Flag )
      }
  }
  else if toxicity_score >= 0.4
  {
      ( ContentCategory::Questionable, ModerationAction::Review )
  }
  else if toxicity_score >= 0.2
  {
      if flags.iter().any( | f | f.contains( "promotional" ) )
      {
  ( ContentCategory::Spam, ModerationAction::Review )
      }
      else
      {
  ( ContentCategory::Questionable, ModerationAction::Allow )
      }
  }
  else
  {
      ( ContentCategory::Safe, ModerationAction::Allow )
  };

  let confidence = if flags.is_empty() { 0.9 } else { 0.7 + ( flags.len() as f32 * 0.1 ).min( 0.2 ) };

  ContentModerationResult
  {
      category,
      confidence,
      toxicity_score : toxicity_score.min( 1.0 ),
      flags,
      recommendation,
  }
  }

  /// Update performance statistics
  fn update_performance_stats( &mut self, processing_time : u64 )
  {
  let history_count = self.analysis_history.len() as u64;
  
  self.performance_stats.total_processing_time += processing_time;
  self.performance_stats.average_processing_time = 
      self.performance_stats.total_processing_time as f64 / ( history_count + 1 ) as f64;
  
  if self.performance_stats.total_processing_time > 0
  {
      self.performance_stats.throughput = 
  ( history_count + 1 ) as f64 * 1000.0 / self.performance_stats.total_processing_time as f64;
  }

  // Estimate memory usage (simplified)
  self.performance_stats.memory_usage_mb = ( history_count + 1 ) as f32 * 0.1; // ~0.1 MB per analysis
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

// Helper function to create sample texts for testing
fn create_sample_texts() -> Vec< String >
{
  vec![
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
  ]
}
