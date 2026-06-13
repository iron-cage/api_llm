//! Enhanced `OpenAI` Client with Advanced Connection Management
//!
//! This module provides an enhanced version of the `OpenAI` client that uses
//! sophisticated connection pooling and management for optimal performance.

use mod_interface::mod_interface;

mod private
{
  use crate::
  {
    client ::{ Client, ClientApiAccessors },
    environment ::{ OpenaiEnvironment, EnvironmentInterface },
    connection_manager ::{ ConnectionManager, ConnectionConfig },
    metrics_framework ::{ MetricsCollector, MetricsConfig, MetricsSnapshot, MetricsAnalysisReport },
    error ::{ Result, OpenAIError },
    chat ::Chat,
    embeddings ::Embeddings,
    models ::Models,
    assistants ::Assistants,
    files ::Files,
    fine_tuning ::FineTuning,
    images ::Images,
    responses ::Responses,
    vector_stores ::VectorStores,
    enhanced_client_performance ::{ ConnectionPerformanceReport, UnifiedPerformanceDashboard },
  };

  // Feature-gated imports
  #[ cfg( feature = "websocket" ) ]
  use crate::realtime ::Realtime;
  #[ cfg( feature = "audio" ) ]
  use crate::audio::Audio;

  #[ cfg( feature = "moderation" ) ]
  use crate::moderations::Moderations;

  #[ cfg( feature = "circuit_breaker" ) ]
  use crate::enhanced_circuit_breaker::{ EnhancedCircuitBreaker, EnhancedCircuitBreakerConfig };

  #[ cfg( feature = "caching" ) ]
  use crate::response_cache::{ ResponseCache, CacheConfig, CacheKey };
  use core::time::Duration;
  use std::{ sync::Arc, time::Instant };
  use tokio::sync::RwLock;
  use reqwest::Method;

  /// Enhanced `OpenAI` client with comprehensive reliability features
  #[ derive( Debug ) ]
  pub struct EnhancedClient< E >
  where
    E : OpenaiEnvironment + EnvironmentInterface + Send + Sync + 'static,
  {
    /// Base client for API operations
    base_client : Client< E >,
    /// Advanced connection manager
    connection_manager : Arc< RwLock< ConnectionManager > >,
    /// Connection configuration
    config : ConnectionConfig,
    /// Optional response cache for improved performance
    #[ cfg( feature = "caching" ) ]
    response_cache : Option< ResponseCache >,
    /// Placeholder for response cache when feature is disabled
    #[ cfg( not( feature = "caching" ) ) ]
    response_cache : Option< () >,
    /// Optional circuit breaker configuration
    #[ cfg( feature = "circuit_breaker" ) ]
    circuit_breaker_config : Option< EnhancedCircuitBreakerConfig >,
    /// Placeholder for circuit breaker config when feature is disabled
    #[ cfg( not( feature = "circuit_breaker" ) ) ]
    circuit_breaker_config : Option< () >,
    /// Optional circuit breaker instance for fault tolerance (only when feature is enabled)
    #[ cfg( feature = "circuit_breaker" ) ]
    circuit_breaker : Option< EnhancedCircuitBreaker >,
    /// Placeholder for circuit breaker when feature is disabled
    #[ cfg( not( feature = "circuit_breaker" ) ) ]
    circuit_breaker : Option< () >,
    /// Comprehensive metrics collector
    metrics_collector : Option< Arc< RwLock< MetricsCollector > > >,
  }

  impl< E > EnhancedClient< E >
  where
    E : OpenaiEnvironment + EnvironmentInterface + Send + Sync + 'static,
  {
    /// Create new enhanced client with default connection config
    ///
    /// # Errors
    ///
    /// Returns an error if the base client cannot be created.
    #[ inline ]
    pub fn build( environment : E ) -> Result< Self >
    {
      Self::build_with_config( environment, ConnectionConfig::default() )
    }

    /// Create new enhanced client with custom connection config
    ///
    /// # Errors
    ///
    /// Returns an error if the base client cannot be created.
    #[ inline ]
    pub fn build_with_config( environment : E, config : ConnectionConfig ) -> Result< Self >
    {
      let base_client = Client::build( environment )?;
      let mut connection_manager = ConnectionManager::new( config.clone() );

      // Start background cleanup task
      connection_manager.start_background_cleanup();

      Ok( Self
      {
        base_client,
        connection_manager : Arc::new( RwLock::new( connection_manager ) ),
        config,
        response_cache : None,
        circuit_breaker_config : None,
        circuit_breaker : None,
        metrics_collector : None,
      } )
    }

    /// Create new enhanced client with both connection management and response caching
    ///
    /// # Errors
    ///
    /// Returns an error if the base client cannot be created.
    #[ cfg( feature = "caching" ) ]
    #[ inline ]
    pub fn build_with_caching(
      environment : E,
      connection_config : ConnectionConfig,
      cache_config : CacheConfig
    ) -> Result< Self >
    {
      let base_client = Client::build( environment )?;
      let mut connection_manager = ConnectionManager::new( connection_config.clone() );

      // Start background cleanup task
      connection_manager.start_background_cleanup();

      // Create response cache
      let response_cache = ResponseCache::with_config( cache_config );

      Ok( Self
      {
        base_client,
        connection_manager : Arc::new( RwLock::new( connection_manager ) ),
        config : connection_config,
        response_cache : Some( response_cache ),
        circuit_breaker_config : None,
        circuit_breaker : None,
        metrics_collector : None,
      } )
    }

    /// Enable response caching on existing client
    #[ cfg( feature = "caching" ) ]
    #[ inline ]
    pub fn enable_caching( &mut self, cache_config : CacheConfig )
    {
      self.response_cache = Some( ResponseCache::with_config( cache_config ) );
    }

    /// Disable response caching
    #[ cfg( feature = "caching" ) ]
    #[ inline ]
    pub fn disable_caching( &mut self )
    {
      self.response_cache = None;
    }

    /// Create new enhanced client with full configuration (connection, caching, circuit breaker, and metrics)
    ///
    /// # Errors
    ///
    /// Returns an error if the base client cannot be created.
    #[ cfg( all( feature = "caching", feature = "circuit_breaker" ) ) ]
    #[ inline ]
    pub fn build_with_full_config(
      environment : E,
      connection_config : ConnectionConfig,
      cache_config : Option< CacheConfig >,
      circuit_breaker_config : Option< EnhancedCircuitBreakerConfig >,
      metrics_config : Option< MetricsConfig >
    ) -> Result< Self >
    {
      let base_client = Client::build( environment )?;
      let mut connection_manager = ConnectionManager::new( connection_config.clone() );

      // Start background cleanup task
      connection_manager.start_background_cleanup();

      // Create response cache if configured
      let response_cache = cache_config.map( ResponseCache::with_config );

      // Create circuit breaker if configured
      #[ cfg( feature = "circuit_breaker" ) ]
      let circuit_breaker = if let Some( ref cb_config ) = circuit_breaker_config
      {
        EnhancedCircuitBreaker::new( cb_config.clone() ).ok()
      }
      else
      {
        None
      };

      #[ cfg( not( feature = "circuit_breaker" ) ) ]
      let circuit_breaker = None;

      // Create metrics collector if configured
      let metrics_collector = metrics_config.map( | config |
      {
        let collector = MetricsCollector::with_config( config );
        Arc::new( RwLock::new( collector ) )
      } );

      Ok( Self
      {
        base_client,
        connection_manager : Arc::new( RwLock::new( connection_manager ) ),
        config : connection_config,
        response_cache,
        circuit_breaker_config,
        circuit_breaker,
        metrics_collector,
      } )
    }

    /// Enable circuit breaker with default configuration
    #[ cfg( feature = "circuit_breaker" ) ]
    #[ inline ]
    pub fn enable_circuit_breaker( &mut self )
    {
      let config = EnhancedCircuitBreakerConfig::default();
      self.circuit_breaker_config = Some( config.clone() );

      self.circuit_breaker = EnhancedCircuitBreaker::new( config ).ok();
    }

    /// Enable circuit breaker with custom configuration
    #[ cfg( feature = "circuit_breaker" ) ]
    #[ inline ]
    pub fn enable_circuit_breaker_with_config( &mut self, config : EnhancedCircuitBreakerConfig )
    {
      self.circuit_breaker_config = Some( config.clone() );
      self.circuit_breaker = EnhancedCircuitBreaker::new( config ).ok();
    }

    /// Disable circuit breaker
    #[ inline ]
    pub fn disable_circuit_breaker( &mut self )
    {
      self.circuit_breaker_config = None;
      self.circuit_breaker = None;
    }

    /// Check if circuit breaker is enabled
    #[ inline ]
    pub fn is_circuit_breaker_enabled( &self ) -> bool
    {
      self.circuit_breaker_config.is_some()
    }

    /// Get circuit breaker configuration if enabled
    #[ cfg( feature = "circuit_breaker" ) ]
    #[ inline ]
    pub fn circuit_breaker_config( &self ) -> Option< &EnhancedCircuitBreakerConfig >
    {
      self.circuit_breaker_config.as_ref()
    }

    /// Enable metrics collection with default configuration
    ///
    /// # Errors
    ///
    /// Returns an error if metrics collection cannot be enabled.
    #[ inline ]
    pub fn enable_metrics( &mut self ) -> Result< () >
    {
      let config = MetricsConfig::default();
      self.enable_metrics_with_config( config )
    }

    /// Enable metrics collection with custom configuration
    ///
    /// # Errors
    ///
    /// Returns an error if metrics collection cannot be enabled.
    #[ inline ]
    pub fn enable_metrics_with_config( &mut self, config : MetricsConfig ) -> Result< () >
    {
      let mut collector = MetricsCollector::with_config( config );
      collector.start_collection();
      self.metrics_collector = Some( Arc::new( RwLock::new( collector ) ) );
      Ok( () )
    }

    /// Disable metrics collection
    #[ inline ]
    pub fn disable_metrics( &mut self )
    {
      self.metrics_collector = None;
    }

    /// Check if metrics collection is enabled
    #[ inline ]
    pub fn is_metrics_enabled( &self ) -> bool
    {
      self.metrics_collector.is_some()
    }

    /// Get current metrics snapshot
    #[ inline ]
    pub async fn get_metrics_snapshot( &self ) -> Option< MetricsSnapshot >
    {
      if let Some( ref metrics_collector ) = self.metrics_collector
      {
        let collector = metrics_collector.read().await;

        // Gather data from all components
        let connection_metrics = self.get_connection_stats().await;
        let pool_stats = self.get_pool_stats().await;
        #[ cfg( feature = "caching" ) ]
        let cache_stats = self.get_cache_statistics().await;
        #[ cfg( not( feature = "caching" ) ) ]
        let cache_stats = None;

        // Circuit breaker stats are temporarily disabled until the circuit breaker module provides stats
        #[ cfg( feature = "circuit_breaker" ) ]
        let circuit_breaker_stats : Option< &() > = None;

        #[ cfg( not( feature = "circuit_breaker" ) ) ]
        let circuit_breaker_stats : Option< &() > = None;

        Some( collector.collect_snapshot(
          Some( &connection_metrics ),
          Some( &pool_stats ),
          cache_stats.as_ref(),
          circuit_breaker_stats,
        ).await )
      }
      else
      {
        None
      }
    }

    /// Get comprehensive metrics analysis report
    #[ inline ]
    pub async fn get_metrics_analysis( &self ) -> Option< MetricsAnalysisReport >
    {
      if let Some( ref metrics_collector ) = self.metrics_collector
      {
        let collector = metrics_collector.read().await;
        Some( collector.generate_analysis_report().await )
      }
      else
      {
        None
      }
    }

    /// Export metrics to JSON format
    ///
    /// # Errors
    ///
    /// Returns an error if metrics cannot be serialized to JSON format.
    #[ inline ]
    pub async fn export_metrics_json( &self ) -> Result< String >
    {
      if let Some( ref metrics_collector ) = self.metrics_collector
      {
        let collector = metrics_collector.read().await;
        collector.export_json().await
      }
      else
      {
        Ok( "[]".to_string() )
      }
    }

    /// Export metrics to Prometheus format
    #[ inline ]
    pub async fn export_metrics_prometheus( &self ) -> String
    {
      if let Some( ref metrics_collector ) = self.metrics_collector
      {
        let collector = metrics_collector.read().await;
        collector.export_prometheus().await
      }
      else
      {
        String::new()
      }
    }

    /// Get assistants API with enhanced connection management
    #[ inline ]
    pub fn assistants( &self ) -> Assistants< '_, E >
    {
      self.base_client.assistants()
    }

    /// Get audio API with enhanced connection management
    #[ inline ]
    #[ cfg( feature = "audio" ) ]
    pub fn audio( &self ) -> Audio< '_, E >
    {
      self.base_client.audio()
    }

    /// Get chat API with enhanced connection management
    #[ inline ]
    pub fn chat( &self ) -> Chat< '_, E >
    {
      self.base_client.chat()
    }

    /// Get embeddings API with enhanced connection management
    #[ inline ]
    pub fn embeddings( &self ) -> Embeddings< '_, E >
    {
      self.base_client.embeddings()
    }

    /// Get files API with enhanced connection management
    #[ inline ]
    pub fn files( &self ) -> Files< '_, E >
    {
      self.base_client.files()
    }

    /// Get fine tuning API with enhanced connection management
    #[ inline ]
    pub fn fine_tuning( &self ) -> FineTuning< '_, E >
    {
      self.base_client.fine_tuning()
    }

    /// Get images API with enhanced connection management
    #[ inline ]
    pub fn images( &self ) -> Images< '_, E >
    {
      self.base_client.images()
    }

    /// Get models API with enhanced connection management
    #[ inline ]
    pub fn models( &self ) -> Models< '_, E >
    {
      self.base_client.models()
    }

    /// Get moderations API with enhanced connection management
    #[ inline ]
    #[ cfg( feature = "moderation" ) ]
    pub fn moderations( &self ) -> Moderations< '_, E >
    {
      self.base_client.moderations()
    }

    /// Get realtime API with enhanced connection management
    #[ inline ]
    #[ cfg( feature = "websocket" ) ]
    pub fn realtime( &self ) -> Realtime< '_, E >
    {
      self.base_client.realtime()
    }

    /// Get responses API with enhanced connection management
    #[ inline ]
    pub fn responses( &self ) -> Responses< '_, E >
    {
      self.base_client.responses()
    }

    /// Get vector stores API with enhanced connection management
    #[ inline ]
    pub fn vector_stores( &self ) -> VectorStores< '_, E >
    {
      self.base_client.vector_stores()
    }

    /// Execute HTTP request with advanced connection management and circuit breaker protection
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the circuit breaker is open, or serialization/deserialization fails.
    #[ inline ]
    pub async fn execute_managed_request< I, O >(
      &self,
      method : Method,
      path : &str,
      body : Option< &I >,
    ) -> Result< O >
    where
      I : serde::Serialize + Send + Sync,
      O : serde::de::DeserializeOwned,
    {
      // If circuit breaker is enabled, wrap the request execution
      #[ cfg( feature = "circuit_breaker" ) ]
      {
        if let Some( ref circuit_breaker ) = self.circuit_breaker
        {
          return circuit_breaker.execute( || self.execute_request_internal( method.clone(), path, body ) ).await;
        }
      }

      // Execute without circuit breaker protection
      self.execute_request_internal( method, path, body ).await
    }

    /// Internal request execution method (used by circuit breaker)
    async fn execute_request_internal< I, O >(
      &self,
      method : Method,
      path : &str,
      body : Option< &I >,
    ) -> Result< O >
    where
      I : serde::Serialize + Send + Sync,
      O : serde::de::DeserializeOwned,
    {
      let url = self.base_client.environment.join_base_url( path )?;
      let host = url.host_str().unwrap_or( "api.openai.com" );

      let start_time = Instant::now();

      // Get managed connection
      let connection = {
        let manager = self.connection_manager.read().await;
        manager.get_connection( host ).await
          .map_err( | e | OpenAIError::Internal( format!( "Failed to get connection : {e}" ) ) )?
      };

      // Build and execute request
      let request_builder = connection.client.request( method, url );
      let request_builder = if let Some( body ) = body
      {
        request_builder.json( body )
      }
      else
      {
        request_builder
      };

      let response = request_builder.send().await;

      match response
      {
        Ok( resp ) =>
        {
          let response_time = start_time.elapsed();

          // Record successful request
          connection.record_success( response_time ).await;

          // Record timing metrics
          if let Some( ref metrics_collector ) = self.metrics_collector
          {
            let collector = metrics_collector.read().await;
            collector.record_timing( response_time ).await;
          }

          // Parse response
          let bytes = resp.bytes().await
            .map_err( | e | OpenAIError::Internal( format!( "Failed to read response : {e}" ) ) )?;

          let result : O = serde_json::from_slice( &bytes )
            .map_err( | e | OpenAIError::Internal( format!( "Failed to parse JSON: {e}" ) ) )?;

          // Return connection to pool
          {
            let manager = self.connection_manager.read().await;
            manager.return_connection( connection ).await;
          }

          Ok( result )
        },
        Err( e ) =>
        {
          let response_time = start_time.elapsed();

          // Record failed request
          connection.record_failure().await;

          // Record error metrics
          if let Some( ref metrics_collector ) = self.metrics_collector
          {
            let collector = metrics_collector.read().await;
            collector.record_timing( response_time ).await;
            collector.record_error( "request_failed" ).await;
          }

          // Return connection to pool (it will be health-checked)
          {
            let manager = self.connection_manager.read().await;
            manager.return_connection( connection ).await;
          }

          Err( OpenAIError::Internal( format!( "Request failed : {e}" ) ).into() )
        }
      }
    }

    /// Execute HTTP GET request with intelligent caching
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or caching operations fail.
    #[ allow( unused_variables ) ]
    #[ inline ]
    pub async fn get_cached< O >( &self, path : &str, ttl : Option< Duration > ) -> Result< O >
    where
      O: serde::de::DeserializeOwned + serde::Serialize,
    {
      #[ cfg( feature = "caching" ) ]
      {
        // Check cache first if caching is enabled
        if let Some( ref cache ) = self.response_cache
        {
          let cache_key = CacheKey::new( "GET", path, None, None );
          if let Some( cached_data ) = cache.get( &cache_key ).await
          {
            let result : O = serde_json::from_slice( &cached_data )
              .map_err( | e | OpenAIError::Internal( format!( "Failed to deserialize cached response : {e}" ) ) )?;
            return Ok( result );
          }

          // Cache miss - make request and cache result
          let response : O = self.execute_managed_request( Method::GET, path, None::< &() > ).await?;

          // Cache the response
          if let Ok( serialized ) = serde_json::to_vec( &response )
          {
            let _ = cache.put( &cache_key, serialized, ttl ).await;
          }

          return Ok( response );
        }

        // No caching - use regular request
        self.execute_managed_request( Method::GET, path, None::< &() > ).await
      }

      #[ cfg( not( feature = "caching" ) ) ]
      {
        // No caching - use regular request
        self.execute_managed_request( Method::GET, path, None::< &() > ).await
      }
    }

    /// Execute HTTP POST request with optional caching
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or caching operations fail.
    #[ allow( unused_variables ) ]
    #[ inline ]
    pub async fn post_cached< I, O >( &self, path : &str, body : &I, ttl : Option< Duration > ) -> Result< O >
    where
      I: serde::Serialize + Send + Sync,
      O: serde::de::DeserializeOwned + serde::Serialize,
    {
      #[ cfg( feature = "caching" ) ]
      {
        // Check cache first if caching is enabled and TTL is specified
        if let Some( ref cache ) = &self.response_cache
        {
          if ttl.is_some()
          {
          let body_bytes = serde_json::to_vec( body )
            .map_err( | e | OpenAIError::Internal( format!( "Failed to serialize request body : {e}" ) ) )?;

          let cache_key = CacheKey::new( "POST", path, Some( &body_bytes ), None );
          if let Some( cached_data ) = cache.get( &cache_key ).await
          {
            let result : O = serde_json::from_slice( &cached_data )
              .map_err( | e | OpenAIError::Internal( format!( "Failed to deserialize cached response : {e}" ) ) )?;
            return Ok( result );
          }

          // Cache miss - make request and cache result
          let response : O = self.execute_managed_request( Method::POST, path, Some( body ) ).await?;

          // Cache the response
          if let Ok( serialized ) = serde_json::to_vec( &response )
          {
            let _ = cache.put( &cache_key, serialized, ttl ).await;
          }

          return Ok( response );
          }

          // No TTL - use regular request without caching
          self.execute_managed_request( Method::POST, path, Some( body ) ).await
        }
        else
        {
          // No caching - use regular request
          self.execute_managed_request( Method::POST, path, Some( body ) ).await
        }
      }

      #[ cfg( not( feature = "caching" ) ) ]
      {
        // No caching - use regular request
        self.execute_managed_request( Method::POST, path, Some( body ) ).await
      }
    }

    /// Get cache statistics if caching is enabled
    #[ cfg( feature = "caching" ) ]
    #[ inline ]
    pub async fn get_cache_statistics( &self ) -> Option< crate::response_cache::CacheStatistics >
    {
      if let Some( ref cache ) = self.response_cache
      {
        Some( cache.get_statistics().await )
      }
      else
      {
        None
      }
    }

    /// Clear response cache if caching is enabled
    #[ cfg( feature = "caching" ) ]
    #[ inline ]
    pub async fn clear_cache( &self )
    {
      if let Some( ref cache ) = self.response_cache
      {
        cache.clear().await;
      }
    }

    /// Check if response caching is enabled
    #[ inline ]
    pub fn is_caching_enabled( &self ) -> bool
    {
      self.response_cache.is_some()
    }

    // Circuit breaker statistics methods are commented out until the circuit breaker module
    // provides the necessary stats types and methods

    /// Get connection manager statistics
    #[ inline ]
    pub async fn get_connection_stats( &self ) -> crate::connection_manager::ConnectionEfficiencyMetrics
    {
      let manager = self.connection_manager.read().await;
      manager.get_efficiency_metrics().await
    }

    /// Get detailed pool statistics
    #[ inline ]
    pub async fn get_pool_stats( &self ) -> Vec< crate::connection_manager::PoolStatistics >
    {
      let manager = self.connection_manager.read().await;
      manager.get_all_stats().await
    }

    /// Warm up connections for frequently used endpoints
    ///
    /// # Errors
    ///
    /// Returns an error if connection warm-up operations fail.
    #[ inline ]
    pub async fn warm_up_connections( &self, hosts : Vec< &str >, connections_per_host : usize ) -> Result< () >
    {
      let manager = self.connection_manager.read().await;

      for host in hosts
      {
        for _ in 0..connections_per_host
        {
          // Create and immediately return connections to warm up the pool
          match manager.get_connection( host ).await
          {
            Ok( conn ) =>
            {
              manager.return_connection( conn ).await;
            },
            Err( e ) =>
            {
              eprintln!( "Failed to warm up connection for {host}: {e}" );
            }
          }
        }
      }

      Ok( () )
    }

    /// Get base client for operations that don't need enhanced connection management
    #[ inline ]
    pub fn base_client( &self ) -> &Client< E >
    {
      &self.base_client
    }

    /// Get connection configuration
    #[ inline ]
    pub fn connection_config( &self ) -> &ConnectionConfig
    {
      &self.config
    }

    /// Update connection configuration
    #[ inline ]
    pub async fn update_connection_config( &self, new_config : ConnectionConfig )
    {
      let mut manager = self.connection_manager.write().await;
      *manager = ConnectionManager::new( new_config.clone() );
      manager.start_background_cleanup();
    }

    /// Get unified performance dashboard combining all components
    ///
    /// # Errors
    ///
    /// Returns an error if performance metrics collection fails.
    #[ inline ]
    pub async fn get_unified_performance_dashboard( &self ) -> Result< UnifiedPerformanceDashboard >
    {
      // Collect connection performance metrics
      let connection_report = self.generate_performance_report().await;

      // Collect cache statistics if available
      #[ cfg( feature = "caching" ) ]
      let cache_stats : Option< crate::response_cache::CacheStatistics > = if let Some( ref cache ) = self.response_cache
      {
        Some( cache.get_statistics().await )
      }
      else
      {
        None
      };

      #[ cfg( not( feature = "caching" ) ) ]
      let cache_stats : Option< () > = None;

      // Collect metrics if available (create a snapshot)
      let metrics_summary : Option< crate::metrics_framework::MetricsSnapshot > = if let Some( ref metrics_collector ) = self.metrics_collector
      {
        let collector = metrics_collector.read().await;
        Some( collector.collect_snapshot(
          Some( &connection_report.efficiency_metrics ),
          Some( &connection_report.pool_stats ),
          cache_stats.as_ref(),
          None
        ).await )
      }
      else
      {
        None
      };

      // Aggregate performance score
      let mut performance_scores = Vec::new();

      // Connection performance (0-100)
      let connection_score = connection_report.efficiency_metrics.efficiency_score * 100.0;
      performance_scores.push( connection_score );

      // Cache performance (if available)
      #[ cfg( feature = "caching" ) ]
      if let Some( ref cache_stats ) = cache_stats
      {
        let cache_hit_rate = cache_stats.hit_ratio;
        let cache_score = cache_hit_rate * 100.0; // Hit rate as percentage
        performance_scores.push( cache_score );
      }

      // Overall performance score (average of available scores)
      let overall_score = if performance_scores.is_empty()
      {
        0.0
      }
      else
      {
        performance_scores.iter().sum::< f64 >() / performance_scores.len() as f64
      };

      // Generate recommendations based on all components
      let mut recommendations = connection_report.analysis.recommendations.clone();

      #[ cfg( feature = "caching" ) ]
      if let Some( ref cache_stats ) = cache_stats
      {
        if cache_stats.hit_ratio < 0.3
        {
          recommendations.push( "Low cache hit rate - consider increasing cache TTL or size".to_string() );
        }
      }

      if self.response_cache.is_none()
      {
        recommendations.push( "Response caching is disabled - enable for better performance".to_string() );
      }

      if self.metrics_collector.is_none()
      {
        recommendations.push( "Metrics collection is disabled - enable for better monitoring".to_string() );
      }

      Ok( UnifiedPerformanceDashboard
      {
        overall_performance_score : overall_score,
        connection_performance : connection_report,
        cache_performance : cache_stats,
        metrics_summary,
        recommendations,
      } )
    }
  }

  impl< E > EnhancedClient< E >
  where
    E : OpenaiEnvironment + EnvironmentInterface + Send + Sync + 'static,
  {
    /// Generate comprehensive connection performance report
    #[ inline ]
    pub async fn generate_performance_report( &self ) -> ConnectionPerformanceReport
    {
      let efficiency_metrics = self.get_connection_stats().await;
      let pool_stats = self.get_pool_stats().await;
      let analysis = crate::enhanced_client_performance::analyze_performance( &efficiency_metrics, &pool_stats );

      ConnectionPerformanceReport
      {
        efficiency_metrics,
        pool_stats,
        analysis,
      }
    }
  }
}

// Re-export types from separate modules for convenience
pub use crate::enhanced_client_builder::EnhancedClientBuilder;
pub use crate::enhanced_client_performance::
{
  ConnectionPerformanceReport,
  PerformanceAnalysis,
  UnifiedPerformanceDashboard,
};

mod_interface!
{
  exposed use
  {
    EnhancedClient,
  };
}
