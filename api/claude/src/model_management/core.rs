//! Core model management types
//!
//! Basic model types, capabilities, requirements, and related data structures.

#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  use serde::{ Serialize, Deserialize };
  use std::time::Duration;

  /// Model information structure
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ModelInfo
  {
    /// Model identifier
    pub id : String,
    /// Display name
    pub display_name : String,
    /// Model name (user-facing)
    pub name : String,
    /// Maximum tokens supported
    pub max_tokens : u32,
    /// Context length
    pub context_length : u32,
    /// Creation timestamp
    pub created_at : Option< String >,
    /// Model capabilities
    pub supports_tools : bool,
    /// Vision support
    pub supports_vision : bool,
    /// Model version
    pub version : Option< String >,
  }

  /// Model capabilities structure
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ModelCapabilities
  {
    /// Tool calling support
    pub supports_tools : bool,
    /// Vision input support
    pub supports_vision : bool,
    /// Maximum context length
    pub max_context_length : u32,
    /// Maximum tool calls per request
    pub max_tool_calls : Option< u32 >,
    /// Supported input modalities
    pub input_modalities : Vec< String >,
    /// Output modalities
    pub output_modalities : Vec< String >,
  }

  /// Model requirements for selection
  #[ derive( Debug, Clone, Default ) ]
  pub struct ModelRequirements
  {
    /// Requires vision capability
    pub requires_vision : bool,
    /// Requires tool calling
    pub requires_tools : bool,
    /// Minimum context length needed
    pub min_context_length : u32,
    /// Maximum cost tier (1=cheap, 5=expensive)
    pub max_cost_tier : u32,
    /// Prefer speed over capability
    pub prefer_speed : bool,
  }

  /// Builder for model requirements
  #[ derive( Debug, Default ) ]
  pub struct ModelRequirementsBuilder
  {
    requires_vision : bool,
    requires_tools : bool,
    min_context_length : u32,
    max_cost_tier : u32,
    prefer_speed : bool,
  }

  impl ModelRequirementsBuilder
  {
    /// Create new builder
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Set vision requirement
    #[ must_use ]
    pub fn requires_vision( mut self, requires : bool ) -> Self
    {
      self.requires_vision = requires;
      self
    }

    /// Set tools requirement
    #[ must_use ]
    pub fn requires_tools( mut self, requires : bool ) -> Self
    {
      self.requires_tools = requires;
      self
    }

    /// Set minimum context length
    #[ must_use ]
    pub fn min_context_length( mut self, length : u32 ) -> Self
    {
      self.min_context_length = length;
      self
    }

    /// Set maximum cost tier
    #[ must_use ]
    pub fn max_cost_tier( mut self, tier : u32 ) -> Self
    {
      self.max_cost_tier = tier;
      self
    }

    /// Set speed preference
    #[ must_use ]
    pub fn prefer_speed( mut self, prefer : bool ) -> Self
    {
      self.prefer_speed = prefer;
      self
    }

    /// Build requirements
    #[ must_use ]
    pub fn build( self ) -> ModelRequirements
    {
      ModelRequirements
      {
        requires_vision : self.requires_vision,
        requires_tools : self.requires_tools,
        min_context_length : self.min_context_length,
        max_cost_tier : self.max_cost_tier,
        prefer_speed : self.prefer_speed,
      }
    }
  }

  impl ModelRequirements
  {
    /// Create new builder
    #[ must_use ]
    pub fn builder() -> ModelRequirementsBuilder
    {
      ModelRequirementsBuilder::new()
    }
  }

  /// Model availability information
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ModelAvailability
  {
    /// Whether model is currently available
    pub is_available : bool,
    /// Estimated wait time if unavailable
    pub estimated_wait_time : Option< Duration >,
    /// Load factor (0.0 = no load, 1.0 = full capacity)
    pub load_factor : f32,
  }

  /// Model limits and constraints
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ModelLimits
  {
    /// Maximum tokens per request
    pub max_tokens : u32,
    /// Temperature range
    pub temperature_range : TemperatureRange,
    /// Maximum tool calls
    pub max_tool_calls : Option< u32 >,
    /// Rate limits
    pub rate_limits : RateLimits,
  }

  /// Temperature range
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct TemperatureRange
  {
    /// Minimum temperature
    pub min : f32,
    /// Maximum temperature
    pub max : f32,
  }

  /// Rate limiting information
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct RateLimits
  {
    /// Requests per minute
    pub requests_per_minute : u32,
    /// Tokens per minute
    pub tokens_per_minute : u32,
  }

  /// Model performance metrics
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ModelPerformance
  {
    /// Model identifier
    pub model : String,
    /// Tokens per second throughput
    pub tokens_per_second : f32,
    /// Average latency in milliseconds
    pub latency_ms : u32,
    /// Overall throughput score
    pub throughput_score : f32,
    /// Speed score (higher = faster)
    pub speed_score : f32,
    /// Cost tier (1-5, lower = cheaper)
    pub cost_tier : String,
  }

  /// Model status information
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ModelStatus
  {
    /// Whether model is deprecated
    pub is_deprecated : bool,
    /// Deprecation date
    pub deprecation_date : Option< String >,
    /// Sunset/removal date
    pub sunset_date : Option< String >,
    /// Recommended replacement model
    pub replacement_model : String,
  }

  /// Migration path for deprecated models
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct MigrationPath
  {
    /// Recommended replacement model
    pub recommended_replacement : String,
    /// Migration steps
    pub migration_steps : Vec< String >,
    /// Breaking changes information
    pub breaking_changes : Option< Vec< String > >,
  }

  /// Model pricing information
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ModelPricing
  {
    /// Cost per input token
    pub input_cost_per_token : f64,
    /// Cost per output token
    pub output_cost_per_token : f64,
    /// Currency
    pub currency : String,
    /// Usage tier
    pub usage_tier : String,
  }

  /// Cost estimation request
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct EstimatedUsage
  {
    /// Input token count
    pub input_tokens : u32,
    /// Output token count
    pub output_tokens : u32,
    /// Model identifier
    pub model : String,
  }

  /// Cost estimation result
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct CostEstimate
  {
    /// Total estimated cost
    pub total_cost : f64,
    /// Input cost component
    pub input_cost : f64,
    /// Output cost component
    pub output_cost : f64,
    /// Currency
    pub currency : String,
  }

  /// Model filtering criteria
  #[ derive( Debug, Clone, Default ) ]
  pub struct ModelFilter
  {
    /// Filter by tool support
    pub supports_tools : Option< bool >,
    /// Filter by vision support
    pub supports_vision : Option< bool >,
    /// Maximum cost tier
    pub max_cost_tier : Option< u32 >,
    /// Minimum context length
    pub min_context_length : Option< u32 >,
  }

  /// Builder for model filter
  #[ derive( Debug, Default ) ]
  pub struct ModelFilterBuilder
  {
    supports_tools : Option< bool >,
    supports_vision : Option< bool >,
    max_cost_tier : Option< u32 >,
    min_context_length : Option< u32 >,
  }

  impl ModelFilterBuilder
  {
    /// Create new builder
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Filter by tool support
    #[ must_use ]
    pub fn supports_tools( mut self, supports : bool ) -> Self
    {
      self.supports_tools = Some( supports );
      self
    }

    /// Filter by vision support
    #[ must_use ]
    pub fn supports_vision( mut self, supports : bool ) -> Self
    {
      self.supports_vision = Some( supports );
      self
    }

    /// Set maximum cost tier
    #[ must_use ]
    pub fn max_cost_tier( mut self, tier : u32 ) -> Self
    {
      self.max_cost_tier = Some( tier );
      self
    }

    /// Set minimum context length
    #[ must_use ]
    pub fn min_context_length( mut self, length : u32 ) -> Self
    {
      self.min_context_length = Some( length );
      self
    }

    /// Build filter
    #[ must_use ]
    pub fn build( self ) -> ModelFilter
    {
      ModelFilter
      {
        supports_tools : self.supports_tools,
        supports_vision : self.supports_vision,
        max_cost_tier : self.max_cost_tier,
        min_context_length : self.min_context_length,
      }
    }
  }

  impl ModelFilter
  {
    /// Create new builder
    #[ must_use ]
    pub fn builder() -> ModelFilterBuilder
    {
      ModelFilterBuilder::new()
    }
  }

  /// Use case definitions for model recommendations
  #[ derive( Debug, Clone ) ]
  pub enum UseCase
  {
    /// Code generation use case
    CodeGeneration
    {
      /// Programming language
      programming_language : String,
      /// Code complexity level
      complexity : CodeComplexity,
      /// Whether explanation is needed
      requires_explanation : bool,
    },
    /// Creative writing use case
    CreativeWriting
    {
      /// Writing genre
      genre : String,
      /// Content length
      length : ContentLength,
      /// Writing tone
      tone : WritingTone,
    },
  }

  /// Code complexity levels
  #[ derive( Debug, Clone ) ]
  pub enum CodeComplexity
  {
    /// Simple code tasks
    Low,
    /// Moderate complexity
    Medium,
    /// Complex code tasks
    High,
  }

  /// Content length categories
  #[ derive( Debug, Clone ) ]
  pub enum ContentLength
  {
    /// Short content
    Short,
    /// Medium content
    Medium,
    /// Long content
    Long,
  }

  /// Writing tone styles
  #[ derive( Debug, Clone ) ]
  pub enum WritingTone
  {
    /// Professional tone
    Professional,
    /// Casual tone
    Casual,
    /// Creative tone
    Creative,
  }

  /// Model recommendation result
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct ModelRecommendation
  {
    /// Recommended model
    pub recommended_model : String,
    /// Confidence score (0.0-1.0)
    pub confidence_score : f32,
    /// Reasoning for recommendation
    pub reasoning : String,
  }

  /// Feature compatibility information
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct FeatureCompatibility
  {
    /// Whether feature is compatible
    pub is_compatible : bool,
    /// Feature version
    pub feature_version : Option< String >,
    /// Alternative models if incompatible
    pub alternative_models : Option< Vec< String > >,
  }
}

crate::mod_interface!
{
  exposed use ModelInfo;
  exposed use ModelCapabilities;
  exposed use ModelRequirements;
  exposed use ModelRequirementsBuilder;
  exposed use ModelAvailability;
  exposed use ModelLimits;
  exposed use TemperatureRange;
  exposed use RateLimits;
  exposed use ModelPerformance;
  exposed use ModelStatus;
  exposed use MigrationPath;
  exposed use ModelPricing;
  exposed use EstimatedUsage;
  exposed use CostEstimate;
  exposed use ModelFilter;
  exposed use ModelFilterBuilder;
  exposed use UseCase;
  exposed use CodeComplexity;
  exposed use ContentLength;
  exposed use WritingTone;
  exposed use ModelRecommendation;
  exposed use FeatureCompatibility;
}

