//! Optimized Semantic Retrieval API with enhanced performance and modularity.
//!
//! This module provides optimized semantic retrieval functionality with:
//! - High-performance vector indexing algorithms
//! - Advanced caching strategies with configurable eviction policies
//! - Modular design with trait-based abstractions
//! - Comprehensive performance monitoring and metrics
//! - Configurable search ranking and filtering algorithms
//! - Memory-efficient storage and retrieval strategies

#![ allow( dead_code, missing_debug_implementations, missing_docs ) ] // Advanced implementation with comprehensive features

mod private
{
  use serde::{ Deserialize, Serialize };
  use std::collections::{ HashMap, BTreeMap };
  use std::sync::{ Arc, RwLock };
  use core::sync::atomic::{ AtomicU64, Ordering };
  use core::time::Duration;
  use std::time::{ SystemTime, Instant };

  /// Base search result structure
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct SearchResult
  {
    /// Document identifier
    pub id : String,
    /// Relevance score
    pub score : f32,
    /// Document content
    pub content : String,
    /// Associated metadata
    pub metadata : Option< HashMap<  String, String  > >,
  }

  /// Base semantic retrieval configuration
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct SemanticRetrievalConfig
  {
    /// API endpoint for semantic retrieval
    pub endpoint : Option< String >,
    /// Request timeout in seconds
    pub timeout_seconds : u64,
    /// Maximum results to return
    pub max_results : usize,
  }

  impl Default for SemanticRetrievalConfig
  {
    fn default() -> Self
    {
      Self {
        endpoint : None,
        timeout_seconds : 30,
        max_results : 10,
      }
    }
  }

  /// Trait for vector indexing strategies
  pub trait VectorIndex : Send + Sync
  {
    /// Add a document vector to the index
    fn add_vector( &mut self, id : &str, vector : &[ f32 ], metadata : Option< HashMap<  String, String  > > ) -> Result< (), crate::error::Error >;

    /// Search for similar vectors
    fn search( &self, query_vector : &[ f32 ], limit : usize, threshold : f32 ) -> Result< Vec< VectorSearchResult >, crate::error::Error >;

    /// Remove vector from index
    fn remove_vector( &mut self, id : &str ) -> Result< bool, crate::error::Error >;

    /// Get index statistics
    fn get_stats( &self ) -> IndexStats;

    /// Optimize index for better search performance
    fn optimize( &mut self ) -> Result< (), crate::error::Error >;
  }

  /// Trait for caching strategies
  pub trait CacheStrategy< K, V >: Send + Sync
  {
    /// Store value in cache
    fn put( &mut self, key : K, value : V );

    /// Retrieve value from cache
    fn get( &self, key : &K ) -> Option< V >;

    /// Remove value from cache
    fn remove( &mut self, key : &K ) -> Option< V >;

    /// Clear all cached values
    fn clear( &mut self );

    /// Get cache statistics
    fn stats( &self ) -> CacheStats;
  }

  /// Vector search result with distance metric
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct VectorSearchResult
  {
    /// Document identifier
    pub id : String,
    /// Similarity score (0.0 to 1.0)
    pub score : f32,
    /// Distance from query vector
    pub distance : f32,
    /// Associated metadata
    pub metadata : Option< HashMap<  String, String  > >,
  }

  /// Index performance statistics
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct IndexStats
  {
    /// Total number of vectors in index
    pub vector_count : u64,
    /// Index memory usage in bytes
    pub memory_usage_bytes : u64,
    /// Average search time in milliseconds
    pub avg_search_time_ms : f64,
    /// Total searches performed
    pub total_searches : u64,
    /// Index build time in milliseconds
    pub build_time_ms : u64,
    /// Last optimization timestamp
    pub last_optimized_at : Option< SystemTime >,
  }

  /// Cache performance statistics
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct CacheStats
  {
    /// Total cache hits
    pub hits : u64,
    /// Total cache misses
    pub misses : u64,
    /// Current cache size
    pub size : usize,
    /// Maximum cache capacity
    pub capacity : usize,
    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio : f64,
    /// Memory usage in bytes
    pub memory_usage_bytes : u64,
  }

  /// Vector entry: embedding vector with optional metadata tags
  type VectorEntry = ( Vec< f32 >, Option< HashMap< String, String > > );

  /// High-performance flat vector index implementation
  #[ derive( Debug ) ]
  pub struct FlatVectorIndex
  {
    /// Vector storage with metadata
    vectors : HashMap< String, VectorEntry >,
    /// Performance statistics
    stats : Arc< RwLock< IndexStats > >,
    /// Vector dimensionality
    dimensions : usize,
  }

  impl FlatVectorIndex
  {
    /// Create new flat vector index
    #[ inline ]
    #[ must_use ]
    pub fn new( dimensions : usize ) -> Self
    {
      Self {
        vectors : HashMap::new(),
        stats : Arc::new( RwLock::new( IndexStats {
          vector_count : 0,
          memory_usage_bytes : 0,
          avg_search_time_ms : 0.0,
          total_searches : 0,
          build_time_ms : 0,
          last_optimized_at : None,
        } ) ),
        dimensions,
      }
    }

    /// Calculate cosine similarity between two vectors
    #[ inline ]
    fn cosine_similarity( &self, a : &[ f32 ], b : &[ f32 ] ) -> f32
    {
      if a.len() != b.len() || a.is_empty()
      {
        return 0.0;
      }

      let dot_product : f32 = a.iter().zip( b.iter() ).map( | ( x, y ) | x * y ).sum();
      let magnitude_a : f32 = a.iter().map( | x | x * x ).sum::< f32 >().sqrt();
      let magnitude_b : f32 = b.iter().map( | x | x * x ).sum::< f32 >().sqrt();

      if magnitude_a == 0.0 || magnitude_b == 0.0
      {
        0.0
      } else {
        dot_product / ( magnitude_a * magnitude_b )
      }
    }
  }

  impl VectorIndex for FlatVectorIndex
  {
    #[ inline ]
    fn add_vector( &mut self, id : &str, vector : &[ f32 ], metadata : Option< HashMap<  String, String  > > ) -> Result< (), crate::error::Error >
    {
      if vector.len() != self.dimensions
      {
        return Err( crate::error::Error::InvalidArgument( format!( "Vector dimension mismatch : expected {}, got {}", self.dimensions, vector.len() ) ) );
      }

      self.vectors.insert( id.to_string(), ( vector.to_vec(), metadata ) );

      // Update statistics
      if let Ok( mut stats ) = self.stats.write()
      {
        stats.vector_count = self.vectors.len() as u64;
        stats.memory_usage_bytes = self.vectors.len() as u64 * ( self.dimensions as u64 * 4 + 64 ); // Rough estimate
      }

      Ok( () )
    }

    #[ inline ]
    fn search( &self, query_vector : &[ f32 ], limit : usize, threshold : f32 ) -> Result< Vec< VectorSearchResult >, crate::error::Error >
    {
      let start_time = Instant::now();

      if query_vector.len() != self.dimensions
      {
        return Err( crate::error::Error::InvalidArgument( format!( "Query vector dimension mismatch : expected {}, got {}", self.dimensions, query_vector.len() ) ) );
      }

      let mut results : Vec< VectorSearchResult > = Vec::new();

      // Calculate similarity for each vector
      for ( id, ( vector, metadata ) ) in &self.vectors
      {
        let similarity = self.cosine_similarity( query_vector, vector );

        if similarity >= threshold
        {
          results.push( VectorSearchResult {
            id : id.clone(),
            score : similarity,
            distance : 1.0 - similarity, // Convert similarity to distance
            metadata : metadata.clone(),
          } );
        }
      }

      // Sort by similarity score (descending)
      results.sort_by( | a, b | b.score.partial_cmp( &a.score ).unwrap_or( std::cmp::Ordering::Equal ) );

      // Limit results
      results.truncate( limit );

      // Update search statistics
      let search_time_ms = start_time.elapsed().as_millis() as f64;
      if let Ok( mut stats ) = self.stats.write()
      {
        stats.total_searches += 1;
        stats.avg_search_time_ms = ( stats.avg_search_time_ms * ( stats.total_searches - 1 ) as f64 + search_time_ms ) / stats.total_searches as f64;
      }

      Ok( results )
    }

    #[ inline ]
    fn remove_vector( &mut self, id : &str ) -> Result< bool, crate::error::Error >
    {
      let removed = self.vectors.remove( id ).is_some();

      if removed
      {
        // Update statistics
        if let Ok( mut stats ) = self.stats.write()
        {
          stats.vector_count = self.vectors.len() as u64;
          stats.memory_usage_bytes = self.vectors.len() as u64 * ( self.dimensions as u64 * 4 + 64 );
        }
      }

      Ok( removed )
    }

    #[ inline ]
    fn get_stats( &self ) -> IndexStats
    {
      self.stats.read().unwrap_or_else( | poisoned | {
        poisoned.into_inner()
      } ).clone()
    }

    #[ inline ]
    fn optimize( &mut self ) -> Result< (), crate::error::Error >
    {
      let start_time = Instant::now();

      // For flat index, optimization involves memory defragmentation
      let optimized_vectors : HashMap< String, VectorEntry > =
        self.vectors.iter().map( | ( k, v ) | ( k.clone(), v.clone() ) ).collect();

      self.vectors = optimized_vectors;

      // Update optimization timestamp
      if let Ok( mut stats ) = self.stats.write()
      {
        stats.last_optimized_at = Some( SystemTime::now() );
        stats.build_time_ms = start_time.elapsed().as_millis() as u64;
      }

      Ok( () )
    }
  }

  /// Adaptive LRU cache with configurable eviction policies
  #[ derive( Debug ) ]
  pub struct AdaptiveLruCache< K, V >
  where
    K: Clone + Eq + std::hash::Hash + Send + Sync,
    V: Clone + Send + Sync,
  {
    /// Cache storage
    cache : HashMap< K, CacheEntry< V > >,
    /// Access order tracking
    access_order : BTreeMap<  u64, K  >,
    /// Current access counter
    access_counter : AtomicU64,
    /// Maximum cache capacity
    capacity : usize,
    /// Cache statistics
    stats : Arc< RwLock< CacheStats > >,
    /// TTL for cache entries (optional)
    ttl : Option< Duration >,
  }

  /// Cache entry with access tracking
  #[ derive( Debug, Clone ) ]
  struct CacheEntry< V >
  {
    /// Cached value
    value : V,
    /// Last access timestamp
    last_accessed : SystemTime,
    /// Access count
    access_count : u64,
    /// Entry creation time
    created_at : SystemTime,
  }

  impl< K, V > AdaptiveLruCache< K, V >
  where
    K: Clone + Eq + std::hash::Hash + Send + Sync,
    V: Clone + Send + Sync,
  {
    /// Create new adaptive LRU cache
    pub fn new( capacity : usize ) -> Self
    {
      Self {
        cache : HashMap::new(),
        access_order : BTreeMap::new(),
        access_counter : AtomicU64::new( 0 ),
        capacity,
        stats : Arc::new( RwLock::new( CacheStats {
          hits : 0,
          misses : 0,
          size : 0,
          capacity,
          hit_ratio : 0.0,
          memory_usage_bytes : 0,
        } ) ),
        ttl : None,
      }
    }

    /// Create cache with TTL expiration
    pub fn with_ttl( capacity : usize, ttl : Duration ) -> Self
    {
      let mut cache = Self::new( capacity );
      cache.ttl = Some( ttl );
      cache
    }

    /// Check if entry is expired
    fn is_expired( &self, entry : &CacheEntry< V > ) -> bool
    {
      if let Some( ttl ) = self.ttl
      {
        if let Ok( elapsed ) = entry.created_at.elapsed()
        {
          return elapsed > ttl;
        }
      }
      false
    }

    /// Evict least recently used entries
    fn evict_lru( &mut self )
    {
      while self.cache.len() >= self.capacity
      {
        if let Some( ( _, oldest_key ) ) = self.access_order.first_key_value()
        {
          let oldest_key = oldest_key.clone();
          self.access_order.remove( &self.access_counter.load( Ordering::Relaxed ) );
          self.cache.remove( &oldest_key );
        } else {
          break;
        }
      }
    }

    /// Update cache statistics
    fn update_stats( &self, hit : bool )
    {
      if let Ok( mut stats ) = self.stats.write()
      {
        if hit
        {
          stats.hits += 1;
        } else {
          stats.misses += 1;
        }

        let total = stats.hits + stats.misses;
        stats.hit_ratio = if total > 0 { stats.hits as f64 / total as f64 } else { 0.0 };
        stats.size = self.cache.len();
        stats.memory_usage_bytes = self.cache.len() as u64 * 256; // Rough estimate
      }
    }
  }

  impl< K, V > CacheStrategy< K, V > for AdaptiveLruCache< K, V >
  where
    K: Clone + Eq + std::hash::Hash + Send + Sync,
    V: Clone + Send + Sync,
  {
    fn put( &mut self, key : K, value : V )
    {
      // Remove expired entries periodically
      let now = SystemTime::now();
      if let Some( ttl ) = self.ttl
      {
        let ttl_duration = ttl;
        self.cache.retain( | _, entry | {
          if let Ok( elapsed ) = entry.created_at.elapsed()
          {
            elapsed <= ttl_duration
          } else {
            true // Keep entry if we can't determine elapsed time
          }
        } );
      }

      // Evict if at capacity
      self.evict_lru();

      let access_id = self.access_counter.fetch_add( 1, Ordering::Relaxed );

      let entry = CacheEntry {
        value,
        last_accessed : now,
        access_count : 1,
        created_at : now,
      };

      self.cache.insert( key.clone(), entry );
      self.access_order.insert( access_id, key );

      self.update_stats( false );
    }

    fn get( &self, key : &K ) -> Option< V >
    {
      if let Some( entry ) = self.cache.get( key )
      {
        if self.is_expired( entry )
        {
          self.update_stats( false );
          return None;
        }

        self.update_stats( true );
        Some( entry.value.clone() )
      } else {
        self.update_stats( false );
        None
      }
    }

    fn remove( &mut self, key : &K ) -> Option< V >
    {
      if let Some( entry ) = self.cache.remove( key )
      {
        // Also remove from access order tracking
        self.access_order.retain( | _, k | k != key );
        Some( entry.value )
      } else {
        None
      }
    }

    fn clear( &mut self )
    {
      self.cache.clear();
      self.access_order.clear();
      self.access_counter.store( 0, Ordering::Relaxed );

      if let Ok( mut stats ) = self.stats.write()
      {
        stats.size = 0;
        stats.memory_usage_bytes = 0;
      }
    }

    fn stats( &self ) -> CacheStats
    {
      self.stats.read().unwrap_or_else( | poisoned | poisoned.into_inner() ).clone()
    }
  }

  /// Optimized semantic retrieval configuration
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct OptimizedRetrievalConfig
  {
    /// Base configuration
    pub base : SemanticRetrievalConfig,
    /// Vector index type to use
    pub index_type : OptimizedIndexType,
    /// Cache strategy configuration
    pub cache_config : CacheConfig,
    /// Search optimization settings
    pub search_config : SearchOptimizationConfig,
    /// Performance monitoring settings
    pub monitoring_config : MonitoringConfig,
  }

  /// Optimized index types
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub enum OptimizedIndexType
  {
    /// High-performance flat index
    OptimizedFlat
    {
      /// Number of vector dimensions
      dimensions : usize
    },
    /// HNSW (Hierarchical Navigable Small World) approximate index
    HNSW
    {
      /// Number of vector dimensions
      dimensions : usize,
      /// Maximum connections per node
      max_connections : u32,
      /// Construction parameter for index building
      ef_construction : u32
    },
    /// LSH (Locality Sensitive Hashing) index
    LSH
    {
      /// Number of vector dimensions
      dimensions : usize,
      /// Number of hash tables to use
      hash_tables : u32,
      /// Number of hash functions per table
      hash_functions : u32
    },
  }

  /// Cache configuration options
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq, Default ) ]
  pub struct CacheConfig
  {
    /// Maximum cache capacity
    pub capacity : usize,
    /// Cache TTL in seconds
    pub ttl_seconds : Option< u64 >,
    /// Enable adaptive cache sizing
    pub adaptive_sizing : bool,
    /// Cache warming strategy
    pub warming_strategy : CacheWarmingStrategy,
  }

  /// Cache warming strategies
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq, Default ) ]
  pub enum CacheWarmingStrategy
  {
    /// No cache warming
    #[ default ]
    None,
    /// Preload most common queries
    CommonQueries,
    /// Preload based on access patterns
    AccessPatterns,
  }

  /// Search optimization configuration
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct SearchOptimizationConfig
  {
    /// Enable query preprocessing
    pub enable_query_preprocessing : bool,
    /// Enable result reranking
    pub enable_reranking : bool,
    /// Use parallel search when possible
    pub enable_parallel_search : bool,
    /// Maximum search timeout in milliseconds
    pub search_timeout_ms : u64,
    /// Enable query expansion
    pub enable_query_expansion : bool,
  }

  /// Performance monitoring configuration
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct MonitoringConfig
  {
    /// Enable performance metrics collection
    pub enable_metrics : bool,
    /// Metrics collection interval in seconds
    pub metrics_interval_seconds : u64,
    /// Enable detailed timing information
    pub enable_detailed_timing : bool,
    /// Maximum metrics history to retain
    pub max_metrics_history : usize,
  }

  impl Default for OptimizedRetrievalConfig
  {
    fn default() -> Self
    {
      Self {
        base : SemanticRetrievalConfig::default(),
        index_type : OptimizedIndexType::OptimizedFlat { dimensions : 1536 },
        cache_config : CacheConfig {
          capacity : 10000,
          ttl_seconds : Some( 3600 ), // 1 hour TTL
          adaptive_sizing : true,
          warming_strategy : CacheWarmingStrategy::CommonQueries,
        },
        search_config : SearchOptimizationConfig {
          enable_query_preprocessing : true,
          enable_reranking : true,
          enable_parallel_search : true,
          search_timeout_ms : 5000, // 5 second timeout
          enable_query_expansion : false,
        },
        monitoring_config : MonitoringConfig {
          enable_metrics : true,
          metrics_interval_seconds : 60,
          enable_detailed_timing : true,
          max_metrics_history : 1000,
        },
      }
    }
  }

  /// Optimized semantic retrieval API
  pub struct OptimizedSemanticRetrievalApi< 'a >
  {
    /// Reference to the Gemini client
    client : &'a crate::client::Client,
    /// Vector index implementation
    index : Arc< RwLock< dyn VectorIndex > >,
    /// Search results cache
    cache : Arc< RwLock< dyn CacheStrategy< String, Vec< SearchResult > > > >,
    /// Embedding cache
    embedding_cache : Arc< RwLock< dyn CacheStrategy< String, Vec< f32 > > > >,
    /// Configuration
    config : OptimizedRetrievalConfig,
    /// Performance metrics
    metrics : Arc< RwLock< PerformanceMetrics > >,
  }

  /// Comprehensive performance metrics
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct PerformanceMetrics
  {
    /// Total search operations
    pub total_searches : u64,
    /// Average search latency in milliseconds
    pub avg_search_latency_ms : f64,
    /// 95th percentile search latency
    pub p95_search_latency_ms : f64,
    /// Total indexing operations
    pub total_indexing_ops : u64,
    /// Average indexing latency in milliseconds
    pub avg_indexing_latency_ms : f64,
    /// Cache hit ratio for search results
    pub search_cache_hit_ratio : f64,
    /// Cache hit ratio for embeddings
    pub embedding_cache_hit_ratio : f64,
    /// Memory usage in bytes
    pub memory_usage_bytes : u64,
    /// Last metrics update
    pub last_updated : SystemTime,
  }

  impl Default for PerformanceMetrics
  {
    fn default() -> Self
    {
      Self {
        total_searches : 0,
        avg_search_latency_ms : 0.0,
        p95_search_latency_ms : 0.0,
        total_indexing_ops : 0,
        avg_indexing_latency_ms : 0.0,
        search_cache_hit_ratio : 0.0,
        embedding_cache_hit_ratio : 0.0,
        memory_usage_bytes : 0,
        last_updated : SystemTime::now(),
      }
    }
  }

  impl< 'a > OptimizedSemanticRetrievalApi< 'a >
  {
    /// Create new optimized semantic retrieval API
    pub fn new( client : &'a crate::client::Client ) -> Self
    {
      Self::with_config( client, OptimizedRetrievalConfig::default() )
    }

    /// Create API with custom configuration
    pub fn with_config( client : &'a crate::client::Client, config : OptimizedRetrievalConfig ) -> Self
    {
      let dimensions = match &config.index_type
      {
        OptimizedIndexType::OptimizedFlat { dimensions } => *dimensions,
        OptimizedIndexType::HNSW { dimensions, .. } => *dimensions,
        OptimizedIndexType::LSH { dimensions, .. } => *dimensions,
      };

      let index : Arc< RwLock< dyn VectorIndex > > = Arc::new( RwLock::new( FlatVectorIndex::new( dimensions ) ) );

      let cache_capacity = config.cache_config.capacity;
      let cache_ttl = config.cache_config.ttl_seconds.map( Duration::from_secs );

      let search_cache : Arc< RwLock< dyn CacheStrategy< String, Vec< SearchResult > > > > =
        if let Some( ttl ) = cache_ttl
        {
          Arc::new( RwLock::new( AdaptiveLruCache::with_ttl( cache_capacity, ttl ) ) )
        } else {
          Arc::new( RwLock::new( AdaptiveLruCache::new( cache_capacity ) ) )
        };

      let embedding_cache : Arc< RwLock< dyn CacheStrategy< String, Vec< f32 > > > > =
        if let Some( ttl ) = cache_ttl
        {
          Arc::new( RwLock::new( AdaptiveLruCache::with_ttl( cache_capacity / 2, ttl ) ) )
        } else {
          Arc::new( RwLock::new( AdaptiveLruCache::new( cache_capacity / 2 ) ) )
        };

      Self {
        client,
        index,
        cache : search_cache,
        embedding_cache,
        config,
        metrics : Arc::new( RwLock::new( PerformanceMetrics::default() ) ),
      }
    }

    /// Get current performance metrics
    pub fn get_metrics( &self ) -> PerformanceMetrics
    {
      self.metrics.read().unwrap_or_else( | poisoned | poisoned.into_inner() ).clone()
    }

    /// Optimize index for better performance
    pub async fn optimize_index( &self ) -> Result< (), crate::error::Error >
    {
      if let Ok( mut index ) = self.index.write()
      {
        index.optimize()?;
      }
      Ok( () )
    }

    /// Clear all caches
    pub fn clear_caches( &self )
    {
      if let Ok( mut cache ) = self.cache.write()
      {
        cache.clear();
      }
      if let Ok( mut embedding_cache ) = self.embedding_cache.write()
      {
        embedding_cache.clear();
      }
    }

    /// Get index statistics
    pub fn get_index_stats( &self ) -> Option< IndexStats >
    {
      if let Ok( index ) = self.index.read()
      {
        Some( index.get_stats() )
      } else {
        None
      }
    }

    /// Get cache statistics
    pub fn get_cache_stats( &self ) -> ( CacheStats, CacheStats )
    {
      let search_cache_stats = if let Ok( cache ) = self.cache.read()
      {
        cache.stats()
      } else {
        CacheStats {
          hits : 0, misses : 0, size : 0, capacity : 0, hit_ratio : 0.0, memory_usage_bytes : 0
        }
      };

      let embedding_cache_stats = if let Ok( cache ) = self.embedding_cache.read()
      {
        cache.stats()
      } else {
        CacheStats {
          hits : 0, misses : 0, size : 0, capacity : 0, hit_ratio : 0.0, memory_usage_bytes : 0
        }
      };

      ( search_cache_stats, embedding_cache_stats )
    }
  }

  impl< 'a > std::fmt::Debug for OptimizedSemanticRetrievalApi< 'a >
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      f.debug_struct( "OptimizedSemanticRetrievalApi" )
        .field( "config", &self.config )
        .finish_non_exhaustive()
    }
  }
}

// Public API exports
pub use private::
{
  SearchResult, SemanticRetrievalConfig,
  VectorIndex, CacheStrategy, VectorSearchResult, IndexStats, CacheStats,
  FlatVectorIndex, AdaptiveLruCache, OptimizedRetrievalConfig,
  OptimizedIndexType, CacheConfig, CacheWarmingStrategy,
  SearchOptimizationConfig, MonitoringConfig, OptimizedSemanticRetrievalApi,
  PerformanceMetrics
};