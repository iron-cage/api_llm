#![ allow( clippy::missing_errors_doc ) ]
#![ allow( clippy::missing_panics_doc ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::double_must_use ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::must_use_candidate ) ]
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_lossless ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::map_unwrap_or ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::redundant_closure_for_method_calls ) ]
#![ allow( clippy::match_same_arms ) ]
#![ allow( clippy::await_holding_lock ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::new_without_default ) ]
#![ allow( clippy::missing_fields_in_debug ) ]
#![ allow( clippy::type_complexity ) ]
#![ allow( clippy::struct_excessive_bools ) ]
#![ allow( clippy::manual_strip ) ]
#![ allow( clippy::unused_self ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::unnecessary_map_or ) ]
#![ allow( clippy::unchecked_time_subtraction ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::single_match ) ]
#![ allow( clippy::should_implement_trait ) ]
#![ allow( clippy::return_self_not_must_use ) ]
#![ allow( clippy::redundant_else ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::format_in_format_args ) ]
#![ allow( clippy::for_kv_map ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::assertions_on_constants ) ]
#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::bool_assert_comparison ) ]
#![ allow( clippy::manual_string_new ) ]
#![ allow( clippy::len_zero ) ]
#![ allow( clippy::needless_borrows_for_generic_args ) ]
#![ allow( clippy::useless_format ) ]
#![ allow( clippy::no_effect_underscore_binding ) ]
#![ allow( clippy::useless_vec ) ]

//! Ollama local LLM runtime API client.
//!
//! This crate provides HTTP client functionality for Ollama's local LLM runtime API,
//! following the **"Thin Client, Rich API"** governing principle.
//!
//! ## Governing Principle : "Thin Client, Rich API"
//!
//! This library exposes all server-side functionality transparently while maintaining
//! zero client-side intelligence or **automatic** behaviors. This ensures:
//!
//! ### 1. **API Transparency**
//! - Every method directly corresponds to an Ollama API endpoint
//! - No hidden transformations or side effects
//! - Method names clearly indicate exact server calls
//!
//! ### 2. **Zero Client Intelligence**
//! - No automatic decision-making or behavior inference
//! - No automatic configuration-driven actions without explicit enabling
//! - All behaviors are explicitly requested by developers
//!
//! ### 3. **Explicit Control**
//! - Developers have complete control over when and how API calls are made
//! - No background operations without explicit configuration
//! - Clear separation between information retrieval and action methods
//!
//! ### 4. **Information vs Action**
//! - Information methods (like `list_models()`) only retrieve data
//! - Action methods (like `chat()`) only perform requested operations
//! - No methods that implicitly combine information gathering with actions
//!
//! ## Enterprise Reliability Features
//!
//! The following enterprise reliability features are **explicitly allowed** when implemented
//! with explicit configuration and transparent operation:
//!
//! - **Configurable Retry Logic**: Exponential backoff with explicit configuration
//! - **Circuit Breaker Pattern**: Failure threshold management with transparent state
//! - **Rate Limiting**: Request throttling with explicit rate configuration
//! - **Failover Support**: Multi-endpoint configuration and automatic switching
//! - **Health Checks**: Periodic endpoint health verification and monitoring
//!
//! ## State Management Policy
//!
//! **✅ ALLOWED: Runtime-Stateful, Process-Stateless**
//! - Connection pools, circuit breaker state, rate limiting buckets
//! - Retry logic state, failover state, health check state
//! - Runtime state that dies with the process
//! - No persistent storage or cross-process state
//!
//! **❌ PROHIBITED: Process-Persistent State**
//! - File storage, databases, configuration accumulation
//! - State that survives process restarts
//!
//! **Implementation Requirements**:
//! - Feature gating behind cargo features (`retry`, `circuit_breaker`, `rate_limiting`, `failover`, `health_checks`)
//! - Explicit configuration required (no automatic enabling)
//! - Transparent method naming (e.g., `execute_with_retries()`, `execute_with_circuit_breaker()`)
//! - Zero overhead when features disabled
//!
//! This principle ensures predictable, explicit, and transparent behavior while supporting
//! production-ready reliability features when explicitly requested.
//!
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ doc( html_root_url = "https://docs.rs/api_ollama/latest/api_ollama/" ) ]

// This attribute prevents "unused" warnings when the feature is disabled.
// The entire module is gated by the "enabled" feature.
#[ cfg( feature = "enabled" ) ]
use mod_interface::mod_interface;
#[ cfg( feature = "enabled" ) ]
pub mod websocket;
#[ cfg( feature = "websocket_streaming" ) ]
mod websocket_config;
#[ cfg( feature = "enabled" ) ]
pub mod tuning;
#[ cfg( feature = "enabled" ) ]
pub mod auth;
#[ cfg( feature = "enabled" ) ]
pub mod workspace;
#[ cfg( feature = "enabled" ) ]
pub mod enhanced_retry;
#[ cfg( feature = "audio_processing" ) ]
pub mod audio;
#[ cfg( feature = "count_tokens" ) ]
pub mod tokens;
#[ cfg( feature = "cached_content" ) ]
pub mod cached_content;
#[ cfg( feature = "batch_operations" ) ]
pub mod batch_operations;
#[ cfg( feature = "safety_settings" ) ]
pub mod safety_settings;
#[ cfg( feature = "circuit_breaker" ) ]
pub mod circuit_breaker;
#[ cfg( feature = "rate_limiting" ) ]
pub mod rate_limiter;
#[ cfg( feature = "request_caching" ) ]
pub mod request_cache;
#[ cfg( any( feature = "failover", feature = "health_checks" ) ) ]
pub mod failover;
#[ cfg( feature = "sync_api" ) ]
pub mod sync_api;
#[ cfg( feature = "dynamic_config" ) ]
pub mod dynamic_config;
#[ cfg( feature = "general_diagnostics" ) ]
pub mod diagnostics;
#[ cfg( feature = "structured_logging" ) ]
pub mod logging;
#[ cfg( feature = "input_validation" ) ]
pub mod input_validation;
#[ cfg( feature = "enhanced_function_calling" ) ]
pub mod enhanced_function_calling;
#[ cfg( feature = "model_comparison" ) ]
pub mod model_comparison;
#[ cfg( feature = "request_templates" ) ]
pub mod request_templates;
#[ cfg( all( feature = "buffered_streaming", feature = "streaming" ) ) ]
pub mod buffered_streaming;
#[ cfg( feature = "compression" ) ]
pub mod compression;
#[ cfg( feature = "enterprise_quota" ) ]
pub mod enterprise_quota;
#[ cfg( feature = "curl_diagnostics" ) ]
pub mod curl_diagnostics;

// Client extension modules (impl blocks for OllamaClient)
#[ cfg( feature = "count_tokens" ) ]
mod client_ext_count_tokens;
#[ cfg( feature = "audio_processing" ) ]
mod client_ext_audio;
#[ cfg( any( feature = "failover", feature = "health_checks" ) ) ]
mod client_ext_resilience;
#[ cfg( any( feature = "circuit_breaker", feature = "rate_limiting", feature = "request_caching", feature = "general_diagnostics" ) ) ]
mod client_ext_features;
#[ cfg( all( feature = "streaming", feature = "streaming_control" ) ) ]
mod client_ext_streaming_control;
#[ cfg( feature = "model_tuning" ) ]
mod client_ext_tuning;
#[ cfg( any( feature = "workspace", all( feature = "workspace", feature = "secret_management" ) ) ) ]
mod client_ext_workspace;
#[ cfg( feature = "safety_settings" ) ]
mod client_ext_safety;
#[ cfg( feature = "retry" ) ]
mod client_ext_retry;
#[ cfg( feature = "secret_management" ) ]
mod client_ext_auth;
#[ cfg( feature = "model_details" ) ]
mod client_ext_model_details;
#[ cfg( feature = "streaming" ) ]
mod client_ext_streaming;
#[ cfg( feature = "cached_content" ) ]
mod client_ext_cached_content;
// NOTE: client_ext_batch.rs has syntax errors - temporarily disabled
// #[ cfg( feature = "batch_operations" ) ]
// mod client_ext_batch;
// #[ cfg( feature = "enabled" ) ]
// mod client_builders;
#[ cfg( feature = "enabled" ) ]
pub mod messages;
#[ cfg( feature = "enabled" ) ]
pub mod chat;
#[ cfg( feature = "enabled" ) ]
pub mod generate;
#[ cfg( feature = "embeddings" ) ]
pub mod embeddings;
#[ cfg( feature = "enabled" ) ]
pub mod models_info;
#[ cfg( feature = "model_details" ) ]
pub mod models_operations;
#[ cfg( feature = "model_details" ) ]
pub mod models_enhanced;
#[ cfg( feature = "model_details" ) ]
pub mod models_additional;
#[ cfg( feature = "health_checks" ) ]
pub mod health_checks;
#[ cfg( feature = "enabled" ) ]
pub mod client;
#[ cfg( feature = "enabled" ) ]
pub mod builders;
#[ cfg( feature = "streaming_control" ) ]
pub mod stream_control;
// Public exports for count tokens feature
#[ cfg( feature = "count_tokens" ) ]
pub use crate::tokens::{
  TokenCountRequest, TokenCountResponse, CostEstimation,
  BatchTokenRequest, BatchTokenResponse, TokenValidationConfig,
  ModelTokenCapabilities
};
// Public exports for cached content feature
#[ cfg( feature = "cached_content" ) ]
pub use crate::cached_content::{
  CachedContentRequest, CachedContentResponse, ContentCacheConfig,
  CacheInvalidationRequest, CacheInvalidationResponse, CachePerformanceMetrics,
  IntelligentCacheManager
};
// Public exports for batch operations feature
#[ cfg( feature = "batch_operations" ) ]
pub use crate::batch_operations::{
  BatchChatRequest, BatchChatResponse, BatchGenerateRequest, BatchGenerateResponse,
  BatchOperationConfig, BatchResult, BatchError
};
// Public exports for safety settings feature
#[ cfg( feature = "safety_settings" ) ]
pub use crate::safety_settings::{
  SafetyConfiguration, HarmPreventionLevel, ContentType, ComplianceMode,
  ContentFilterRequest, ContentFilterResponse, FilterCategory, SeverityLevel, SafetyAction,
  HarmClassificationRequest, HarmClassificationResponse, HarmType, HarmCategory,
  SafetyPolicyEnforcement, EnforcementLevel, EscalationRule, EscalationTrigger, EscalationAction,
  ComplianceReporting, ReportFrequency, ComplianceAuditTrail, SafetyAssessment, ComplianceStatus,
  ComplianceReportRequest, ReportType, ReportFormat, ComplianceReportResponse,
  SafetyStatus, SafetyPerformanceMetrics, validate_safety_configuration
};
#[ cfg( feature = "enabled" ) ]
mod private
{
  use error_tools::untyped::Result;
  /// Result type for Ollama API operations
  pub type OllamaResult< T > = Result< T >;
}
#[ cfg( feature = "enabled" ) ]
crate ::mod_interface!
{
  exposed use
  {
    client ::OllamaClient,
    private ::OllamaResult,
    messages ::Message,
    messages ::MessageRole,
    messages ::ChatMessage,
    chat ::ChatRequest,
    chat ::ChatResponse,
    generate ::GenerateRequest,
    generate ::GenerateResponse,
    models_info ::ModelInfo,
    models_info ::ModelDetails,
    models_info ::ModelEntry,
    models_info ::TagsResponse,
  };
  #[ cfg( feature = "websocket_streaming" ) ]
  use websocket;
  #[ cfg( feature = "model_tuning" ) ]
  use tuning;
  #[ cfg( any( feature = "secret_management", feature = "workspace" ) ) ]
  use auth;
  #[ cfg( feature = "workspace" ) ]
  exposed use
  {
    auth ::WorkspaceConfig,
  };
  #[ cfg( feature = "failover" ) ]
  exposed use
  {
    failover ::FailoverPolicy,
    failover ::FailoverStats,
    failover ::EndpointHealth,
    failover ::EndpointInfo,
    failover ::FailoverManager,
  };
  #[ cfg( feature = "secret_management" ) ]
  exposed use
  {
    auth ::SecretStore,
    auth ::SecretConfig,
  };
  #[ cfg( all( feature = "workspace", feature = "secret_management" ) ) ]
  exposed use
  {
    workspace ::WorkspaceSecretStore,
  };
  #[ cfg( feature = "embeddings" ) ]
  exposed use
  {
    embeddings ::EmbeddingsRequest,
    embeddings ::EmbeddingsResponse,
  };
  #[ cfg( feature = "builder_patterns" ) ]
  exposed use
  {
    builders ::ChatRequestBuilder,
    builders ::GenerateRequestBuilder,
  };
  #[ cfg( all( feature = "builder_patterns", feature = "embeddings" ) ) ]
  exposed use
  {
    builders ::EmbeddingsRequestBuilder,
  };
  // Enhanced retry logic exports (feature-gated)
  #[ cfg( feature = "retry" ) ]
  exposed use crate::enhanced_retry::
  {
    RetryConfig,
    ErrorClassification,
    RetryMetrics,
    RetryStats,
    ErrorClassifier,
    RetryableHttpClient,
    execute_with_retries,
    calculate_retry_delay,
    retry_operation,
  };

  #[ cfg( feature = "audio_processing" ) ]
  exposed use crate::audio::
  {
    AudioFormat,
    SpeechToTextRequest,
    SpeechToTextResponse,
    TextToSpeechRequest,
    TextToSpeechResponse,
    AudioStreamRequest,
    AudioStreamChunk,
    VoiceChatRequest,
    VoiceChatResponse,
    AudioProcessingConfig,
    AudioStreamReceiver,
  };

  // MessageRole and ChatMessage are now exported in the base module
  // #[ cfg( feature = "vision_support" ) ]
  // exposed use
  // {
  //   private::MessageRole,
  //   private::ChatMessage,
  // };

  #[ cfg( feature = "tool_calling" ) ]
  exposed use
  {
    messages ::ToolDefinition,
    messages ::ToolCall,
    messages ::ToolMessage,
  };

  #[ cfg( feature = "enhanced_function_calling" ) ]
  exposed use
  {
    enhanced_function_calling ::ToolExecutor,
    enhanced_function_calling ::ToolRegistry,
    enhanced_function_calling ::ToolResult,
    enhanced_function_calling ::helpers,
    enhanced_function_calling ::orchestration,
  };

  #[ cfg( feature = "circuit_breaker" ) ]
  exposed use
  {
    circuit_breaker ::CircuitBreaker,
    circuit_breaker ::CircuitBreakerConfig,
    circuit_breaker ::CircuitBreakerState,
  };

  #[ cfg( feature = "rate_limiting" ) ]
  exposed use
  {
    rate_limiter ::RateLimiter,
    rate_limiter ::RateLimitingConfig,
    rate_limiter ::RateLimitingAlgorithm,
  };

  #[ cfg( feature = "request_caching" ) ]
  exposed use
  {
    request_cache ::RequestCache,
    request_cache ::RequestCacheConfig,
    request_cache ::CacheEntry,
    request_cache ::CacheStats,
  };

  #[ cfg( feature = "general_diagnostics" ) ]
  exposed use
  {
    diagnostics ::DiagnosticsConfig,
    diagnostics ::DiagnosticsCollector,
    diagnostics ::RequestMetrics,
    diagnostics ::ErrorAnalysis,
    diagnostics ::PerformanceReport,
    diagnostics ::ComprehensiveReport,
    diagnostics ::WindowedMetrics,
    diagnostics ::WindowMetrics,
    diagnostics ::ThroughputReport,
  };

  #[ cfg( feature = "model_details" ) ]
  exposed use
  {
    models_enhanced ::EnhancedModelDetails,
    models_enhanced ::ModelMetadata,
    models_enhanced ::ModelLifecycle,
    models_enhanced ::ModelOperation,
    models_enhanced ::ModelPerformanceMetrics,
    models_operations ::ShowModelRequest,
    models_operations ::PullModelRequest,
    models_operations ::PushModelRequest,
    models_operations ::DeleteModelRequest,
    models_operations ::ModelProgressUpdate,
    models_operations ::ModelProgressStream,
    models_additional ::ComprehensiveModelInfo,
    models_additional ::ModelRecommendation,
    models_additional ::ModelLifecycleStatus,
    models_additional ::ModelOperationHistoryEntry,
    models_additional ::ModelHealthCheck,
    models_additional ::LocalModelStorageInfo,
    models_additional ::ModelDiagnostics,
  };

  #[ cfg( feature = "sync_api" ) ]
  exposed use
  {
    sync_api ::SyncOllamaClient,
    sync_api ::SyncApiConfig,
    sync_api ::SyncRuntimeManager,
    sync_api ::SyncApiConfigBuilder,
  };

  #[ cfg( feature = "health_checks" ) ]
  exposed use
  {
    health_checks ::HealthCheckStrategy,
    health_checks ::HealthCheckConfig,
    health_checks ::HealthStatus,
    health_checks ::HealthMetrics,
  };

  #[ cfg( feature = "dynamic_config" ) ]
  exposed use
  {
    dynamic_config ::DynamicConfig,
    dynamic_config ::DynamicConfigManager,
    dynamic_config ::ConfigDiff,
    dynamic_config ::ConfigBackup,
    dynamic_config ::ConfigVersion,
  };

  #[ cfg( feature = "streaming_control" ) ]
  exposed use
  {
    stream_control ::StreamState,
    stream_control ::StreamControlError,
    stream_control ::StreamMetrics,
    stream_control ::StreamBuffer,
    stream_control ::StreamControl,
  };
  #[ cfg( all( feature = "streaming", feature = "streaming_control" ) ) ]
  exposed use
  {
    stream_control ::ControlledStream,
  };
  // EmbeddingsRequestBuilder is already exported above
}