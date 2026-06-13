//! Enhanced model management types and features
//!
//! Extended model details, performance profiles, comparisons, and advanced features.

#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  use super::super::core::orphan::*;
  use serde::{ Serialize, Deserialize };
  use std::collections::HashMap;

/// Extended model details with performance profiles and capabilities
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct EnhancedModelDetails
{
  /// Basic model information
  pub model_id : String,
  /// Human readable model name
  pub display_name : String,
  /// Model description
  pub description : String,
  /// Model version
  pub version : Option< String >,
  /// Release date
  pub release_date : Option< String >,
  /// Model architecture
  pub architecture : Option< String >,
  /// Training data cutoff
  pub training_cutoff : Option< String >,

  /// Pricing information
  pub pricing : Option< ModelPricing >,

  /// Enhanced capabilities
  pub capabilities : EnhancedModelCapabilities,

  /// Context window details
  pub context_window : ContextWindowDetails,

  /// Lifecycle information
  pub lifecycle : ModelLifecycle,
}

/// Enhanced model capabilities with detailed information
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ allow( clippy::struct_excessive_bools ) ]
pub struct EnhancedModelCapabilities
{
  /// Basic capabilities
  pub supports_function_calling : bool,
  /// Vision support
  pub supports_vision : bool,
  /// Multimodal input support
  pub supports_multimodal_input : bool,
  /// Streaming support
  pub supports_streaming : bool,
  /// System prompts support
  pub supports_system_prompts : bool,

  /// Limitations
  pub limitations : HashMap< String, String >,

  /// Performance profile
  pub performance_profile : PerformanceProfile,
}

/// Performance profile for a model
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct PerformanceProfile
{
  /// Latency category
  pub latency_category : Option< String >,
  /// Throughput category
  pub throughput_category : Option< String >,
  /// Cost category
  pub cost_category : Option< String >,
}

/// Context window details with token breakdown
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ContextWindowDetails
{
  /// Maximum context tokens
  pub max_context_tokens : u32,
  /// Maximum output tokens
  pub max_output_tokens : u32,
  /// Token breakdown details
  pub token_breakdown : TokenBreakdown,
}

/// Token allocation breakdown
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct TokenBreakdown
{
  /// System prompt tokens
  pub system_prompt_tokens : u32,
  /// Conversation tokens
  pub conversation_tokens : u32,
  /// Tool definition tokens
  pub tool_definition_tokens : u32,
}

/// Model lifecycle information
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ModelLifecycle
{
  /// Model status
  pub status : String,
  /// Whether model is deprecated
  pub is_deprecated : bool,
  /// Release date
  pub release_date : Option< String >,
  /// Deprecation date
  pub deprecation_date : Option< String >,
  /// End of life date
  pub end_of_life_date : Option< String >,
  /// Replacement model
  pub replacement_model : Option< String >,
  /// Migration guide
  pub migration_guide : Vec< String >,
  /// Version compatibility
  pub version_compatibility : VersionCompatibility,
}

/// Version compatibility information
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct VersionCompatibility
{
  /// Supported API versions
  pub supported_api_versions : Vec< String >,
}

/// Model comparison functionality
#[ derive( Debug, Clone ) ]
pub struct ModelComparison
{
  /// First model for comparison
  pub model_a : String,
  /// Second model for comparison
  pub model_b : String,
  /// Capability differences
  pub capability_differences : Vec< String >,
  /// Cost comparison
  pub cost_comparison : CostComparison,
  /// Performance comparison
  pub performance_comparison : PerformanceComparison,
  /// Use case recommendations
  pub use_case_recommendations : Vec< String >,
}

/// Cost comparison between models
#[ derive( Debug, Clone ) ]
pub struct CostComparison
{
  /// Cost ratio between models
  pub cost_ratio : f64,
  /// Cost analysis details
  pub cost_analysis : Vec< String >,
}

/// Performance comparison between models
#[ derive( Debug, Clone ) ]
pub struct PerformanceComparison
{
  /// Latency ratio between models
  pub latency_ratio : f64,
  /// Quality score difference
  pub quality_score_diff : f64,
}

/// Filtered model result
#[ derive( Debug, Clone ) ]
pub struct FilteredModel
{
  /// Model identifier
  pub model_id : String,
  /// Vision support
  pub supports_vision : bool,
  /// Context length
  pub context_length : u32,
  /// Whether deprecated
  pub is_deprecated : bool,
}

/// Model search result
#[ derive( Debug, Clone ) ]
pub struct ModelSearchResult
{
  /// Model identifier
  pub model_id : String,
  /// Model name
  pub name : String,
  /// Model description
  pub description : String,
  /// Relevance score
  pub relevance_score : f64,
}

}

crate::mod_interface!
{
  exposed use EnhancedModelDetails;
  exposed use EnhancedModelCapabilities;
  exposed use PerformanceProfile;
  exposed use ContextWindowDetails;
  exposed use TokenBreakdown;
  exposed use ModelLifecycle;
  exposed use VersionCompatibility;
  exposed use ModelComparison;
  exposed use CostComparison;
  exposed use PerformanceComparison;
  exposed use FilteredModel;
  exposed use ModelSearchResult;
}
