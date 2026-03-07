//! Type definitions and API operations for the Gemini API.
//!
//! This module provides all types, request/response structures, and API implementations
//! for interacting with Google's Gemini AI models.

/// Type definitions organized by functional domain.
pub mod types;

/// API endpoint implementations for model operations.
pub mod api;

/// Health check functionality for monitoring endpoint availability.
pub mod health;

/// Dynamic configuration management for runtime updates.
pub mod config;

/// Failover management for high availability.
pub mod failover;

/// Streaming control for fine-grained stream management.
pub mod streaming_control;

/// WebSocket streaming for real-time bidirectional communication.
pub mod websocket_streaming;
pub mod websocket_streaming_optimized;

/// Model tuning and fine-tuning capabilities.
pub mod model_tuning;

/// Model deployment and hosting capabilities.
pub mod model_deployment;
pub mod media_optimization;
pub mod semantic_retrieval_optimized;

/// Batch Mode API for async job-based processing with 50% cost discount.
pub mod batch;

mod private
{
  // Re-export all types from the types module
  pub use super::types::core::*;
  pub use super::types::generation::*;
  pub use super::types::embedding::*;
  pub use super::types::file::*;
  pub use super::types::token::*;
  pub use super::types::cache::*;
  pub use super::types::content::*;
  pub use super::types::streaming::*;
  #[ cfg( feature = "chat" ) ]
  pub use super::types::chat::*;
  pub use super::types::comparison::*;
  pub use super::types::search::*;
  pub use super::types::function::*;
  pub use super::types::code_execution::*;
  pub use super::types::tuning::*;
}

::mod_interface::mod_interface!
{
  // Core model types
  exposed use private::Model;
  exposed use private::ListModelsResponse;

  // Content generation types
  exposed use private::GenerateContentRequest;
  exposed use private::GenerateContentResponse;
  exposed use private::GenerationConfig;
  exposed use private::SafetySetting;
  exposed use private::PromptFeedback;
  exposed use private::UsageMetadata;
  exposed use private::BatchGenerateContentRequest;
  exposed use private::BatchGenerateContentResponse;

  // Embedding types
  exposed use private::EmbedContentRequest;
  exposed use private::EmbedContentResponse;
  exposed use private::ContentEmbedding;
  exposed use private::BatchEmbedContentsRequest;
  exposed use private::BatchEmbedContentsResponse;

  // File management types
  exposed use private::FileMetadata;
  exposed use private::VideoMetadata;
  exposed use private::UploadFileRequest;
  exposed use private::UploadFileResponse;
  exposed use private::ListFilesRequest;
  exposed use private::ListFilesResponse;
  exposed use private::DeleteFileRequest;

  // Token operations types
  exposed use private::CountTokensRequest;
  exposed use private::CountTokensResponse;
  exposed use private::BatchCountTokensRequest;
  exposed use private::BatchCountTokensResponse;
  exposed use private::AnalyzeTokensRequest;
  exposed use private::AnalyzeTokensResponse;
  exposed use private::TokenBreakdown;
  exposed use private::CostEstimate;

  // Cache management types
  exposed use private::CreateCachedContentRequest;
  exposed use private::CachedContentResponse;
  exposed use private::ListCachedContentsResponse;
  exposed use private::UpdateCachedContentRequest;

  // Content structure types
  exposed use private::Content;
  exposed use private::Part;
  exposed use private::Blob;
  exposed use private::FileData;
  exposed use private::FunctionCall;
  exposed use private::FunctionResponse;
  exposed use private::Candidate;
  exposed use private::SafetyRating;
  exposed use private::CitationMetadata;
  exposed use private::CitationSource;
  exposed use private::SystemInstruction;

  // Streaming types (feature-gated)
  #[ cfg( feature = "streaming" ) ]
  exposed use private::StreamingResponse;
  #[ cfg( feature = "streaming" ) ]
  exposed use private::StreamingRequestBuilder;

  // Chat types (feature-gated)
  #[ cfg( feature = "chat" ) ]
  exposed use private::ChatCompletionRequest;
  #[ cfg( feature = "chat" ) ]
  exposed use private::ChatCompletionRequestBuilder;
  #[ cfg( feature = "chat" ) ]
  exposed use private::ChatCompletionResponse;
  #[ cfg( feature = "chat" ) ]
  exposed use private::ChatMessage;
  #[ cfg( feature = "chat" ) ]
  exposed use private::ChatChoice;
  #[ cfg( feature = "chat" ) ]
  exposed use private::ChatUsage;

  // Model comparison and recommendation types
  exposed use private::CompareModelsRequest;
  exposed use private::CompareModelsResponse;
  exposed use private::ModelComparison;
  exposed use private::PerformanceMetrics;
  exposed use private::CostAnalysis;
  exposed use private::ModelRecommendation;
  exposed use private::GetRecommendationsRequest;
  exposed use private::GetRecommendationsResponse;
  exposed use private::AdvancedFilterRequest;
  exposed use private::AdvancedFilterResponse;
  exposed use private::ModelStatusRequest;
  exposed use private::ModelStatusResponse;
  exposed use private::ModelStatus;

  // Search and grounding types
  exposed use private::GoogleSearchTool;
  exposed use private::GroundingMetadata;
  exposed use private::GroundingChunk;
  exposed use private::GroundingSupport;
  exposed use private::SearchEntryPoint;

  // Function calling types
  exposed use private::Tool;
  exposed use private::FunctionDeclaration;
  exposed use private::FunctionCallingConfig;
  exposed use private::FunctionCallingMode;
  exposed use private::ToolConfig;
  exposed use private::CodeExecutionTool;

  // Code execution types
  exposed use private::CodeExecution;
  exposed use private::CodeExecutionConfig;
  exposed use private::CodeExecutionResult;

  // Model tuning types
  exposed use private::CreateTunedModelRequest;
  exposed use private::TunedModel;
  exposed use private::TuningTask;
  exposed use private::TuningSnapshot;
  exposed use private::Dataset;
  exposed use private::TuningExamples;
  exposed use private::TuningExample;
  exposed use private::Hyperparameters;
  exposed use private::TunedModelSource;
  exposed use private::ListTunedModelsResponse;
  exposed use private::ListTunedModelsRequest;

  // Re-exports from other modules
  exposed use health::{ HealthStatus, HealthCheckResult, HealthCheckConfig, HealthCheckStrategy, HealthCheckBuilder };
  exposed use config::{ DynamicConfig, DynamicConfigBuilder, ConfigChangeType, ConfigChangeEvent, ConfigHistoryEntry, ConfigUpdate, ConfigManager, ConfigChangeListener };
  exposed use failover::{ FailoverConfig, FailoverConfigBuilder, FailoverStrategy, EndpointHealth, FailoverMetrics, FailoverManager, FailoverBuilder };
  exposed use streaming_control::{ StreamState, StreamControlConfig, StreamControlConfigBuilder, StreamMetrics, StreamMetricsSnapshot, BufferStrategy, MetricsLevel, ControllableStream, ControllableStreamBuilder };
  exposed use websocket_streaming::{ WebSocketConnectionState, WebSocketConfig, WebSocketConfigBuilder, WebSocketPoolConfig, WebSocketPoolConfigBuilder, WebSocketMessage, WebSocketMetrics, WebSocketConnection, WebSocketStreamBuilder };
  exposed use websocket_streaming_optimized::{ ConnectionPool, MessageSerializerType, ConnectionPoolStats, OptimizedWebSocketConfig, ConnectionPoolConfig, MessageOptimizationConfig, WebSocketMonitoringConfig, ResourceManagementConfig, SerializationFormat, OptimizedConnectionPool, OptimizedWebSocketConnection, ConnectionMetrics, ConnectionHealthChecker, OptimizedWebSocketStreamingApi, StreamingMetrics };
  exposed use model_tuning::{ TrainingJobState, HyperparameterConfig, HyperparameterConfigBuilder, LoRAConfig, LoRAConfigBuilder, TrainingObjective, TrainingMetrics, ModelCheckpoint, TrainingProgress, TrainingJob, FineTuningBuilder };
  exposed use model_deployment::{ DeploymentState, DeploymentEnvironment, DeploymentStrategy, ScalingConfig, ScalingConfigBuilder, ResourceConfig, ResourceConfigBuilder, DeploymentHealthCheckConfig, DeploymentHealthCheckConfigBuilder, MonitoringConfig, MonitoringConfigBuilder, ContainerConfig, ContainerConfigBuilder, OrchestrationConfig, DeploymentMetrics, ModelDeployment, DeploymentBuilder, DeploymentSummary, DeploymentCache, IntelligentScaler, ScalingDecision, PerformanceOptimizer, OptimizationRecommendation, OptimizationCategory, OptimizationPriority, ImpactEstimate, ImplementationEffort };
  exposed use media_optimization::{ MediaProcessingConfig, MediaRetryConfig, ThumbnailConfig, ThumbnailFormat, MediaCache, MediaCacheStats, MediaCacheStatsReport, MediaProcessingPipeline, MediaProcessingMetrics, ProcessedMediaResult, ProcessedMediaMetadata, MediaProcessingMetricsReport, ThumbnailGenerator, OptimizedMediaApi };
  exposed use semantic_retrieval_optimized::{ VectorIndex, CacheStrategy, VectorSearchResult, IndexStats, CacheStats, FlatVectorIndex, AdaptiveLruCache, OptimizedRetrievalConfig, OptimizedIndexType, CacheConfig, CacheWarmingStrategy, SearchOptimizationConfig, MonitoringConfig as OptimizedMonitoringConfig, OptimizedSemanticRetrievalApi, PerformanceMetrics as OptimizedPerformanceMetrics };
  exposed use batch::{ BatchJobState, BatchJob, BatchJobStatus, BatchBillingMetadata, BatchJobResults, BatchEmbeddingResults, BatchJobList, CreateBatchJobRequest, CreateBatchEmbeddingRequest };
}
