//! API accessor methods for Client.
//!
//! This module contains all methods that return API handle instances for
//! accessing different Gemini API endpoints.

use super::Client;
use super::api_interfaces::{ ModelsApi, TunedModelsApi, FilesApi, CachedContentApi };

#[ cfg( feature = "chat" ) ]
use super::api_interfaces::ChatApi;

#[ cfg( feature = "websocket_streaming" ) ]
use crate::websocket::WebSocketStreamingApi;

#[ cfg( feature = "streaming_control" ) ]
use crate::models::streaming_control::StreamingControlApi;

use crate::models::{ OptimizedMediaApi, OptimizedSemanticRetrievalApi, OptimizedWebSocketStreamingApi };
use crate::models::{ OptimizedRetrievalConfig, OptimizedWebSocketConfig };

impl Client
{
    /// Get a models API instance for interacting with model-related endpoints
    #[ must_use ]
    #[ inline ]
    pub fn models( &self ) -> ModelsApi< '_ >
    {
        ModelsApi { client : self }
    }

      /// Get a chat API instance for chat completion functionality
    #[ cfg( feature = "chat" ) ]
    #[ must_use ]
    #[ inline ]
    pub fn chat( &self ) -> ChatApi< '_ >
    {
        ChatApi { client : self }
    }

    /// Get a files API instance for file management operations
    #[ must_use ]
    #[ inline ]
    pub fn files( &self ) -> FilesApi< '_ >
    {
        FilesApi { client : self }
    }

    /// Get an optimized media API instance with advanced processing capabilities
    ///
    /// This creates an enhanced media processing system with intelligent caching,
    /// compression, memory optimization, and performance monitoring.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # use api_gemini::models::MediaProcessingConfig;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let media_api = client.optimized_media();
    ///
    /// // Access optimization capabilities
    /// let config = media_api.config();
    /// let metrics = media_api.get_metrics();
    /// let cache_stats = media_api.get_cache_stats();
    ///
    /// println!("Media optimization enabled with {} max cache size",
    ///     config.max_cache_size_bytes);
    /// # Ok( () )
    /// # }
    /// ```
    #[ must_use ]
    #[ inline ]
    pub fn optimized_media( &self ) -> OptimizedMediaApi< '_ >
    {
        OptimizedMediaApi::new( self )
    }

    /// Get an optimized media API instance with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Custom media processing configuration
    #[ must_use ]
    #[ inline ]
    pub fn optimized_media_with_config( &self, config : crate::models::MediaProcessingConfig ) -> OptimizedMediaApi< '_ >
    {
        OptimizedMediaApi::with_config( self, config )
    }

    /// Get an optimized semantic retrieval API instance with enhanced performance
    ///
    /// This provides access to the optimized semantic retrieval functionality with:
    /// - High-performance vector indexing algorithms
    /// - Advanced caching strategies with configurable eviction policies
    /// - Comprehensive performance monitoring and metrics
    /// - Modular design with trait-based abstractions
    ///
    /// # Returns
    ///
    /// Returns a new `OptimizedSemanticRetrievalApi` instance with default configuration
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let optimized_semantic = client.optimized_semantic_retrieval();
    /// // Use optimized semantic API for high-performance document operations
    /// # Ok( () )
    /// # }
    /// ```
    #[ must_use ]
    #[ inline ]
    pub fn optimized_semantic_retrieval( &self ) -> OptimizedSemanticRetrievalApi< '_ >
    {
        OptimizedSemanticRetrievalApi::new( self )
    }

    /// Get an optimized semantic retrieval API instance with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Custom optimized retrieval configuration with advanced options
    ///
    /// # Returns
    ///
    /// Returns a new `OptimizedSemanticRetrievalApi` instance with the specified configuration
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # use api_gemini::models::{ OptimizedRetrievalConfig, OptimizedIndexType, CacheConfig };
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let config = OptimizedRetrievalConfig {
    ///     index_type : OptimizedIndexType::OptimizedFlat { dimensions : 1536 },
    ///     cache_config : CacheConfig {
    ///         capacity : 50000,
    ///         ttl_seconds : Some( 7200 ), // 2 hour TTL
    ///         adaptive_sizing : true,
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    /// let optimized_semantic = client.optimized_semantic_retrieval_with_config( config );
    /// # Ok( () )
    /// # }
    /// ```
    #[ must_use ]
    #[ inline ]
    pub fn optimized_semantic_retrieval_with_config( &self, config : OptimizedRetrievalConfig ) -> OptimizedSemanticRetrievalApi< '_ >
    {
        OptimizedSemanticRetrievalApi::with_config( self, config )
    }

    /// Get an optimized WebSocket streaming API instance with enhanced performance
    ///
    /// This provides access to the optimized WebSocket streaming functionality with:
    /// - High-performance connection pooling and reuse strategies
    /// - Advanced message serialization with binary protocols
    /// - Comprehensive performance monitoring and metrics
    /// - Resource-efficient connection management
    /// - Sophisticated error handling and recovery mechanisms
    ///
    /// # Returns
    ///
    /// Returns a new `OptimizedWebSocketStreamingApi` instance with default configuration
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let optimized_ws = client.optimized_websocket_streaming();
    /// // Use optimized WebSocket API for high-performance streaming
    /// # Ok( () )
    /// # }
    /// ```
    #[ must_use ]
    #[ inline ]
    pub fn optimized_websocket_streaming( &self ) -> OptimizedWebSocketStreamingApi< '_ >
    {
        OptimizedWebSocketStreamingApi::new( self )
    }

    /// Get an optimized WebSocket streaming API instance with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Custom optimized WebSocket configuration with advanced options
    ///
    /// # Returns
    ///
    /// Returns a new `OptimizedWebSocketStreamingApi` instance with the specified configuration
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # use api_gemini::models::{ OptimizedWebSocketConfig, ConnectionPoolConfig, MessageOptimizationConfig, SerializationFormat };
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let config = OptimizedWebSocketConfig {
    ///     pool_config : ConnectionPoolConfig {
    ///         max_connections_per_endpoint : 20,
    ///         max_total_connections : 200,
    ///         ..Default::default()
    ///     },
    ///     message_config : MessageOptimizationConfig {
    ///         serialization_format : SerializationFormat::MessagePack,
    ///         enable_compression : true,
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// };
    /// let optimized_ws = client.optimized_websocket_streaming_with_config( config );
    /// # Ok( () )
    /// # }
    /// ```
    #[ must_use ]
    #[ inline ]
    pub fn optimized_websocket_streaming_with_config( &self, config : OptimizedWebSocketConfig ) -> OptimizedWebSocketStreamingApi< '_ >
    {
        OptimizedWebSocketStreamingApi::with_config( self, config )
    }

    /// Get a WebSocket streaming API instance for real-time bidirectional communication
    ///
    /// This provides access to core WebSocket streaming functionality including:
    /// - Connection lifecycle management with automatic reconnection
    /// - Bidirectional message streaming with proper serialization
    /// - Stream control mechanisms (start, pause, resume, stop)
    /// - Session management and metrics collection
    ///
    /// # Returns
    ///
    /// Returns a new `WebSocketStreamingApi` instance
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let ws_api = client.websocket_streaming();
    /// // Create and manage WebSocket streaming sessions
    /// # Ok( () )
    /// # }
    /// ```
    #[ cfg( feature = "websocket_streaming" ) ]
    #[ must_use ]
    #[ inline ]
    pub fn websocket_streaming( &self ) -> WebSocketStreamingApi< '_ >
    {
        WebSocketStreamingApi::new( self )
    }

    /// Get a streaming control API instance for managing stream lifecycle
    ///
    /// Provides unified streaming control interface that works with both SSE and
    /// WebSocket streams. Enables pause, resume, and cancel operations for active streams.
    ///
    /// # Features
    ///
    /// - Unified control interface for both SSE and WebSocket streaming
    /// - Pause/resume functionality with configurable buffering
    /// - Immediate stream cancellation with proper cleanup
    /// - Thread-safe operations for concurrent stream management
    /// - State tracking and metrics reporting
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let control_api = client.streaming_control();
    /// // Create controllable streams and manage their lifecycle
    /// # Ok( () )
    /// # }
    /// ```
    #[ cfg( feature = "streaming_control" ) ]
    #[ must_use ]
    #[ inline ]
    pub fn streaming_control( &self ) -> StreamingControlApi< '_ >
    {
        StreamingControlApi::new( self )
    }

    /// Get a tuned models API instance for model tuning operations
    #[ must_use ]
    #[ inline ]
    pub fn tuned_models( &self ) -> TunedModelsApi< '_ >
    {
        TunedModelsApi { client : self }
    }

    /// Get a cached content API instance for cache management operations
    #[ must_use ]
    #[ inline ]
    pub fn cached_content( &self ) -> CachedContentApi< '_ >
    {
        CachedContentApi { client : self }
    }

    /// Access the Batch Mode API for async job-based processing with 50% cost discount.
    ///
    /// Batch Mode provides:
    /// - 50% cost discount compared to standard API
    /// - 24-hour Service Level Objective (SLO)
    /// - Async job processing with status polling
    /// - Support for content generation and embeddings
    ///
    /// # Returns
    ///
    /// Returns a `BatchApi` instance for managing batch jobs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use api_gemini::client::Client;
    /// # use api_gemini::models::*;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    ///
    /// // Create batch job
    /// let requests = vec![ /* GenerateContentRequest instances */ ];
    /// let batch_job = client.batches().create_inline( "gemini-2.5-flash", requests ).await?;
    ///
    /// // Poll for completion and retrieve results
    /// let results = client.batches().wait_and_retrieve( &batch_job.job_id, std::time::Duration::from_secs( 300 ) ).await?;
    /// # Ok( () )
    /// # }
    /// ```
    #[ cfg( feature = "batch_operations" ) ]
    #[ must_use ]
    #[ inline ]
    pub fn batches( &self ) -> crate::batch_api::BatchApi< '_ >
    {
        crate ::batch_api::BatchApi::new( self )
    }

    /// Get a health check builder for explicit endpoint monitoring
    ///
    /// This method provides explicit, on-demand health checking functionality
    /// following the "Thin Client, Rich API" principle. No automatic background
    /// monitoring is performed - all health checks are explicit operations.
    #[ must_use ]
    #[ inline ]
    pub fn health( &self ) -> crate::models::health::HealthCheckBuilder
    {
        crate ::models::health::HealthCheckBuilder::new( self.clone() )
    }

    /// Get the base URL for this client
    #[ must_use ]
    #[ inline ]
    pub fn base_url( &self ) -> &str
    {
        &self.base_url
    }
}
