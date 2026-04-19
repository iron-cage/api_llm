//! Document Semantic Search Example
//!
//! This example demonstrates building a semantic document search engine using `HuggingFace` embeddings.
//! Features include document ingestion, embedding generation, similarity-based search, and an interactive CLI.
//!
//! ## Usage
//!
//! ```bash
//! export HUGGINGFACE_API_KEY="your-api-key-here"
//! cargo run --example document_semantic_search --features="full"
//! ```
//!
//! ## Commands
//!
//! - `/add < title > < content >` - Add a document to the search index
//! - `/search < query >` - Search for relevant documents
//! - `/list` - List all indexed documents
//! - `/stats` - Show search engine statistics
//! - `/clear` - Clear all documents from index
//! - `/model < model >` - Change embedding model
//! - `/help` - Show available commands
//! - `/quit` - Exit the search engine

#![allow(clippy::missing_inline_in_public_items)]

use api_huggingface::
{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  components::
  {
  embeddings::EmbeddingResponse,
  models::Models,
  },
  secret::Secret,
};
use std::
{
  collections::HashMap,
  io::{ self, Write as IoWrite },
  time::{ Instant, SystemTime, UNIX_EPOCH },
  fmt,
};

/// Represents a document in the search index
#[ derive( Debug, Clone ) ]
pub struct Document
{
  /// Unique document identifier
  pub id : String,
  /// Document title
  pub title : String,
  /// Document content
  pub content : String,
  /// Document metadata
  pub metadata : HashMap< String, String >,
  /// Document embedding vector (generated from content)
  pub embedding : Option< Vec< f32 > >,
  /// Timestamp when document was added
  pub created_at : SystemTime,
}

/// Document categories for classification
#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub enum DocumentCategory
{
  /// Technical documentation
  Technical,
  /// Academic research papers
  Academic,
  /// News articles
  News,
  /// Creative writing
  Creative,
  /// Legal documents
  Legal,
  /// General content
  General,
}

impl DocumentCategory
{
  /// Parse category from string
  fn from_str( s : &str ) -> Option< Self >
  {
  match s.to_lowercase().as_str()
  {
      "technical" => Some( Self::Technical ),
      "academic" => Some( Self::Academic ),
      "news" => Some( Self::News ),
      "creative" => Some( Self::Creative ),
      "legal" => Some( Self::Legal ),
      "general" => Some( Self::General ),
      _ => None,
  }
  }

  /// Get category as string
  fn as_str( self ) -> &'static str
  {
  match self
  {
      Self::Technical => "technical",
      Self::Academic => "academic",
      Self::News => "news",
      Self::Creative => "creative",
      Self::Legal => "legal",
      Self::General => "general",
  }
  }
}

impl fmt::Display for DocumentCategory
{
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
  write!( f, "{}", self.as_str() )
  }
}

/// Search query with parameters
#[ derive( Debug, Clone ) ]
pub struct SearchQuery
{
  /// Query text
  pub text : String,
  /// Number of results to return
  pub limit : usize,
  /// Minimum similarity threshold (0.0 to 1.0)
  pub threshold : f32,
  /// Optional category filter
  pub category_filter : Option< DocumentCategory >,
  /// Optional metadata filters
  pub metadata_filters : HashMap< String, String >,
}

impl Default for SearchQuery
{
  fn default() -> Self
  {
  Self
  {
      text : String::new(),
      limit : 10,
      threshold : 0.3,
      category_filter : None,
      metadata_filters : HashMap::new(),
  }
  }
}

/// Search result with similarity score
#[ derive( Debug, Clone ) ]
pub struct SearchResult
{
  /// Document reference
  pub document : Document,
  /// Similarity score (0.0 to 1.0, higher is more similar)
  pub similarity : f32,
  /// Ranking position in results
  pub rank : usize,
}

/// Search engine statistics
#[ derive( Debug ) ]
pub struct SearchStats
{
  /// Total number of documents indexed
  pub total_documents : usize,
  /// Total queries performed
  pub total_queries : usize,
  /// Average query processing time (milliseconds)
  pub avg_query_time_ms : f64,
  /// Current embedding model
  pub embedding_model : String,
  /// Total embedding vectors generated
  pub total_embeddings : usize,
}

/// High-performance document search engine with embedding-based similarity
#[ derive( Debug ) ]
pub struct DocumentSearchEngine
{
  client : Client< HuggingFaceEnvironmentImpl >,
  documents : HashMap< String, Document >,
  embedding_model : String,
  stats : SearchStats,
  query_times : Vec< u64 >,
}

impl DocumentSearchEngine
{
  /// Create new search engine with embedding model
  #[ must_use ]
  pub fn new( client : Client< HuggingFaceEnvironmentImpl >, embedding_model : String ) -> Self
  {
  let stats = SearchStats
  {
      total_documents : 0,
      total_queries : 0,
      avg_query_time_ms : 0.0,
      embedding_model : embedding_model.clone(),
      total_embeddings : 0,
  };

  Self
  {
      client,
      documents : HashMap::new(),
      embedding_model,
      stats,
      query_times : Vec::new(),
  }
  }

  /// Add document to search index with embedding generation
  /// 
  /// # Errors
  /// 
  /// Returns error if embedding generation fails or document processing fails
  pub async fn add_document( &mut self, mut document : Document ) -> Result< String, Box< dyn std::error::Error > >
  {
  // Generate embedding for document content
  let response = self.client
      .embeddings()
      .create( document.content.clone(), &self.embedding_model )
      .await?;

  match response
  {
      EmbeddingResponse::Single( embedding_vectors ) =>
      {
  if let Some( embedding ) = embedding_vectors.first()
  {
          document.embedding = Some( embedding.clone() );
  }
  else
  {
          return Err( "No embedding generated".into() );
  }
      },
      EmbeddingResponse::Batch( batch_vectors ) =>
      {
  if let Some( first_doc_vectors ) = batch_vectors.first()
  {
          if let Some( embedding ) = first_doc_vectors.first()
          {
      document.embedding = Some( embedding.clone() );
          }
          else
          {
      return Err( "No embedding in batch result".into() );
          }
  }
  else
  {
          return Err( "No embedding generated".into() );
  }
      },
  }

  let doc_id = document.id.clone();
  self.documents.insert( doc_id.clone(), document );
  self.stats.total_documents += 1;
  self.stats.total_embeddings += 1;
  Ok( doc_id )
  }

  /// Add multiple documents in batch for better performance
  /// 
  /// # Errors
  /// 
  /// Returns error if any document embedding generation fails
  pub async fn add_documents_batch( &mut self, documents : Vec< Document > ) -> Result< Vec< String >, Box< dyn std::error::Error > >
  {
  let mut added_ids = Vec::new();

  // Process in smaller batches to avoid API limits
  let batch_size = 5;
  for batch in documents.chunks( batch_size )
  {
      let batch_contents : Vec< String > = batch.iter().map( |doc| doc.content.clone() ).collect();

      let response = self.client
  .embeddings()
  .create_batch( batch_contents, &self.embedding_model )
  .await?;

      match response
      {
  EmbeddingResponse::Batch( batch_vectors ) =>
  {
          for ( i, doc_vectors ) in batch_vectors.iter().enumerate()
          {
      if let Some( mut doc ) = batch.get( i ).cloned()
      {
              if let Some( embedding ) = doc_vectors.first()
              {
        doc.embedding = Some( embedding.clone() );
        let doc_id = doc.id.clone();
        self.documents.insert( doc_id.clone(), doc );
        added_ids.push( doc_id );
        self.stats.total_embeddings += 1;
              }
      }
          }
  },
  EmbeddingResponse::Single( embedding_vectors ) =>
  {
          // Handle single response for batch (shouldn't happen but be safe)
          if let Some( mut doc ) = batch.first().cloned()
          {
      if let Some( embedding ) = embedding_vectors.first()
      {
              doc.embedding = Some( embedding.clone() );
              let doc_id = doc.id.clone();
              self.documents.insert( doc_id.clone(), doc );
              added_ids.push( doc_id );
              self.stats.total_embeddings += 1;
      }
          }
  },
      }
  }

  self.stats.total_documents += added_ids.len();
  Ok( added_ids )
  }

  /// Search for documents based on semantic similarity
  /// 
  /// # Errors
  /// 
  /// Returns error if query embedding generation fails
  /// 
  /// # Panics
  /// 
  /// Panics if similarity comparison fails (should never happen with valid f32 values)
  pub async fn search( &mut self, query : SearchQuery ) -> Result< Vec< SearchResult >, Box< dyn std::error::Error > >
  {
  let start_time = Instant::now();

  // Generate embedding for query
  let response = self.client
      .embeddings()
      .create( query.text.clone(), &self.embedding_model )
      .await?;
  
  let query_embedding = match response
  {
      EmbeddingResponse::Single( embedding_vectors ) => 
      {
  embedding_vectors.first()
          .ok_or( "No query embedding generated" )?
          .clone()
      },
      EmbeddingResponse::Batch( batch_vectors ) => 
      {
  batch_vectors.first()
          .and_then( |doc_vectors| doc_vectors.first() )
          .ok_or( "No query embedding generated" )?
          .clone()
      },
  };

  // Calculate similarities and filter results
  let mut results = Vec::new();
  for document in self.documents.values()
  {
      if let Some( doc_embedding ) = &document.embedding
      {
  // Apply category filter if specified
  if let Some( category_filter ) = &query.category_filter
  {
          if let Some( doc_category_str ) = document.metadata.get( "category" )
          {
      if let Some( doc_category ) = DocumentCategory::from_str( doc_category_str )
      {
              if doc_category != *category_filter
              {
        continue;
              }
      }
          }
          else
          {
      continue; // Skip documents without category when filter is applied
          }
  }

  // Apply metadata filters
  let mut passes_metadata_filters = true;
  for ( key, value ) in &query.metadata_filters
  {
          if document.metadata.get( key ) != Some( value )
          {
      passes_metadata_filters = false;
      break;
          }
  }
  if !passes_metadata_filters
  {
          continue;
  }

  // Calculate cosine similarity
  let similarity = Self::cosine_similarity( &query_embedding, doc_embedding );
  
  // Apply similarity threshold
  if similarity >= query.threshold
  {
          results.push( SearchResult
          {
      document : document.clone(),
      similarity,
      rank : 0, // Will be set after sorting
          } );
  }
      }
  }

  // Sort by similarity (descending) and limit results
  results.sort_by( |a, b| b.similarity.partial_cmp( &a.similarity ).unwrap() );
  results.truncate( query.limit );

  // Set ranking positions
  for ( i, result ) in results.iter_mut().enumerate()
  {
      result.rank = i + 1;
  }

  // Update statistics
  let query_time = u64::try_from( start_time.elapsed().as_millis() ).unwrap_or( u64::MAX );
  self.query_times.push( query_time );
  self.stats.total_queries += 1;
  
  // Calculate rolling average query time
  let recent_queries = self.query_times.iter().rev().take( 100 ).copied().collect::< Vec< _ > >();
  self.stats.avg_query_time_ms = recent_queries.iter().sum::< u64 >() as f64 / recent_queries.len() as f64;

  Ok( results )
  }

  /// Calculate cosine similarity between two vectors
  fn cosine_similarity( a : &[ f32 ], b : &[ f32 ] ) -> f32
  {
  if a.len() != b.len()
  {
      return 0.0;
  }

  let dot_product : f32 = a.iter().zip( b.iter() ).map( |(x, y)| x * y ).sum();
  let norm_a : f32 = a.iter().map( |x| x * x ).sum::< f32 >().sqrt();
  let norm_b : f32 = b.iter().map( |x| x * x ).sum::< f32 >().sqrt();

  if norm_a == 0.0 || norm_b == 0.0
  {
      return 0.0;
  }

  dot_product / ( norm_a * norm_b )
  }

  /// Get document by ID
  #[ must_use ]
  pub fn get_document( &self, id : &str ) -> Option< &Document >
  {
  self.documents.get( id )
  }

  /// Remove document from index
  pub fn remove_document( &mut self, id : &str ) -> Option< Document >
  {
  if let Some( doc ) = self.documents.remove( id )
  {
      self.stats.total_documents = self.stats.total_documents.saturating_sub( 1 );
      Some( doc )
  }
  else
  {
      None
  }
  }

  /// Clear all documents from index
  pub fn clear( &mut self )
  {
  self.documents.clear();
  self.stats.total_documents = 0;
  self.stats.total_embeddings = 0;
  }

  /// Change embedding model
  pub fn set_embedding_model( &mut self, model : String )
  {
  self.embedding_model.clone_from( &model );
  self.stats.embedding_model = model;
  }

  /// Get search engine statistics
  #[ must_use ]
  pub fn get_stats( &self ) -> &SearchStats
  {
  &self.stats
  }

  /// List all documents in the index
  #[ must_use ]
  pub fn list_documents( &self ) -> Vec< &Document >
  {
  self.documents.values().collect()
  }
}

/// Interactive CLI for document search
#[ derive( Debug ) ]
pub struct SearchCLI
{
  engine : DocumentSearchEngine,
}

impl SearchCLI
{
  /// Create new search CLI
  #[ must_use ]
  pub fn new( client : Client< HuggingFaceEnvironmentImpl > ) -> Self
  {
  Self
  {
      engine : DocumentSearchEngine::new( client, Models::all_minilm_l6_v2().to_string() ),
  }
  }

  /// Start interactive CLI session
  /// 
  /// # Errors
  /// 
  /// Returns error if I/O operations fail or command processing fails
  pub async fn start( &mut self ) -> Result< (), Box< dyn std::error::Error > >
  {
  println!( "🔍 Document Semantic Search Engine" );
  println!( "==================================" );
  println!( "Type '/help' for commands or start adding documents!" );
  println!( "Current model : {}", self.engine.embedding_model );
  println!();

  let stdin = io::stdin();
  let mut stdout = io::stdout();

  // Add some sample documents
  self.add_sample_documents().await?;

  loop
  {
      print!( "search > " );
      stdout.flush()?;

      let mut input = String::new();
      stdin.read_line( &mut input )?;
      let input = input.trim();

      if input.is_empty()
      {
  continue;
      }

      // Handle commands
      if input.starts_with( '/' )
      {
  match self.handle_command( input ).await
  {
          Ok( Some( response ) ) => println!( "{response}" ),
          Ok( None ) => {}, // Command handled without output
          Err( e ) => println!( "❌ Error : {e}" ),
  }
  continue;
      }

      // Handle search query
      let query = SearchQuery
      {
  text : input.to_string(),
  ..SearchQuery::default()
      };

      match self.engine.search( query ).await
      {
  Ok( results ) => 
  {
          if results.is_empty()
          {
      println!( "No documents found matching your query." );
          }
          else
          {
      println!( "\n🎯 Found {} results:", results.len() );
      println!( "=" );
      
      for result in &results
      {
              let created_at = result.document.created_at
        .duration_since( UNIX_EPOCH )
        .unwrap_or_default()
        .as_secs();
        
              println!(
        "{}. {} (similarity : {:.3})\n   Content : {}\n   Created : {} seconds ago\n",
        result.rank,
        result.document.title,
        result.similarity,
        if result.document.content.len() > 100 
        { 
                  format!( "{}...", &result.document.content[ ..100 ] ) 
        } else {
                  result.document.content.clone() 
        },
        created_at
              );
      }
          }
  },
  Err( e ) => println!( "❌ Search failed : {e}" ),
      }

      println!();
  }
  }

  /// Add sample documents for demonstration
  async fn add_sample_documents( &mut self ) -> Result< (), Box< dyn std::error::Error > >
  {
  let sample_docs = vec![
      Document
      {
  id : "1".to_string(),
  title : "Rust Programming Language".to_string(),
  content : "Rust is a systems programming language that focuses on safety, speed, and concurrency. It achieves memory safety without garbage collection.".to_string(),
  metadata : {
          let mut map = HashMap::new();
          map.insert( "category".to_string(), "technical".to_string() );
          map.insert( "language".to_string(), "english".to_string() );
          map
  },
  embedding : None,
  created_at : SystemTime::now(),
      },
      Document
      {
  id : "2".to_string(),
  title : "Machine Learning Basics".to_string(),
  content : "Machine learning is a subset of artificial intelligence that enables computers to learn and improve from experience without being explicitly programmed.".to_string(),
  metadata : {
          let mut map = HashMap::new();
          map.insert( "category".to_string(), "academic".to_string() );
          map.insert( "language".to_string(), "english".to_string() );
          map
  },
  embedding : None,
  created_at : SystemTime::now(),
      },
      Document
      {
  id : "3".to_string(),
  title : "Climate Change Effects".to_string(),
  content : "Climate change refers to long-term shifts in global temperatures and weather patterns. Human activities are the main driver of climate change since the 1800s.".to_string(),
  metadata : {
          let mut map = HashMap::new();
          map.insert( "category".to_string(), "news".to_string() );
          map.insert( "language".to_string(), "english".to_string() );
          map
  },
  embedding : None,
  created_at : SystemTime::now(),
      },
  ];

  println!( "Adding sample documents..." );
  match self.engine.add_documents_batch( sample_docs ).await
  {
      Ok( ids ) => println!( "✅ Added {} sample documents", ids.len() ),
      Err( e ) => println!( "⚠️ Failed to add sample documents : {e}" ),
  }

  Ok( () )
  }

  /// Handle CLI commands
  #[ allow( clippy::too_many_lines ) ]
  async fn handle_command( &mut self, command : &str ) -> Result< Option< String >, Box< dyn std::error::Error > >
  {
  let parts : Vec< &str > = command[ 1.. ].splitn( 2, ' ' ).collect();
  
  if parts.is_empty()
  {
      return Ok( Some( "Invalid command. Type '/help' for available commands.".to_string() ) );
  }

  match parts[ 0 ].to_lowercase().as_str()
  {
      "help" => Ok( Some( self.show_help() ) ),
      
      "quit" | "exit" => 
      {
  println!( "👋 Goodbye!" );
  std::process::exit( 0 );
      },
      
      "add" =>
      {
  if parts.len() < 2
  {
          return Ok( Some( "Usage : /add < title > < content >".to_string() ) );
  }
  
  let parts : Vec< &str > = parts[ 1 ].splitn( 2, ' ' ).collect();
  if parts.len() < 2
  {
          return Ok( Some( "Usage : /add < title > < content >".to_string() ) );
  }
  
  let title = parts[ 0 ];
  let content = parts[ 1 ];
  let doc_id = format!( "user_{}", SystemTime::now().duration_since( UNIX_EPOCH )?.as_secs() );
  
  let document = Document
  {
          id : doc_id.clone(),
          title : title.to_string(),
          content : content.to_string(),
          metadata : {
      let mut map = HashMap::new();
      map.insert( "category".to_string(), "general".to_string() );
      map.insert( "source".to_string(), "user".to_string() );
      map
          },
          embedding : None,
          created_at : SystemTime::now(),
  };
  
  match self.engine.add_document( document ).await
  {
          Ok( _ ) => Ok( Some( format!( "✅ Document '{title}' added successfully" ) ) ),
          Err( e ) => Ok( Some( format!( "❌ Failed to add document : {e}" ) ) ),
  }
      },
      
      "list" =>
      {
  let documents = self.engine.list_documents();
  if documents.is_empty()
  {
          Ok( Some( "No documents in the index.".to_string() ) )
  }
  else
  {
          let mut result = format!( "📚 {} documents in index:\n\n", documents.len() );
          for ( i, doc ) in documents.iter().enumerate()
          {
      let preview = if doc.content.len() > 80 
      { 
              format!( "{}...", &doc.content[ ..80 ] ) 
      } else {
              doc.content.clone() 
      };
      use core::fmt::Write;
      writeln!( &mut result, "{}. {} - {}", i + 1, doc.title, preview ).unwrap();
          }
          Ok( Some( result ) )
  }
      },
      
      "stats" =>
      {
  let stats = self.engine.get_stats();
  let result = format!(
          "📊 Search Engine Statistics:\n\
           Documents : {}\n\
           Queries : {}\n\
           Avg Query Time : {:.2}ms\n\
           Embedding Model : {}\n\
           Total Embeddings : {}",
          stats.total_documents,
          stats.total_queries,
          stats.avg_query_time_ms,
          stats.embedding_model,
          stats.total_embeddings
  );
  Ok( Some( result ) )
      },
      
      "clear" =>
      {
  self.engine.clear();
  Ok( Some( "🗑️ All documents cleared from index.".to_string() ) )
      },
      
      "model" =>
      {
  if parts.len() < 2
  {
          return Ok( Some( format!( "Current model : {}\nUsage : /model < model-name >", self.engine.embedding_model ) ) );
  }
  
  let model = parts[ 1 ].to_string();
  self.engine.set_embedding_model( model.clone() );
  Ok( Some( format!( "🔧 Changed embedding model to : {model}" ) ) )
      },
      
      _ => Ok( Some( format!( "Unknown command : /{}\nType '/help' for available commands.", parts[ 0 ] ) ) ),
  }
  }

  /// Show help information
  fn show_help( &self ) -> String
  {
  r#"Available Commands:
===================

/add < title > < content > - Add a document to the search index
/search < query >        - Search for relevant documents (or just type without /)
/list                  - List all indexed documents
/stats                 - Show search engine statistics
/clear                 - Clear all documents from index
/model < model >         - Change embedding model
/help                  - Show this help message
/quit or /exit         - Exit the search engine

Search Tips:
============

• Use natural language queries for best results
• The engine finds documents based on semantic meaning, not just keywords
• Try queries like:
  - "programming languages for system development"
  - "artificial intelligence and learning"
  - "environmental changes and global warming"

Current embedding model : "#.to_string() + &self.engine.embedding_model
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
  })
  .map_err(|_| "HUGGINGFACE_API_KEY not found in environment or workspace secrets")?;

  // Build client
  let secret_key = Secret::new( api_key );
  let environment = HuggingFaceEnvironmentImpl::build( secret_key, None )?;
  let client = Client::build( environment )?;

  // Start interactive search CLI
  let mut cli = SearchCLI::new( client );
  cli.start().await?;

  Ok( () )
}