//! Real-time Web Search Grounding Example
//!
//! This example demonstrates Google Search grounding capabilities including:
//! - Real-time web search integration with Gemini models
//! - Citation and source attribution from search results
//! - Grounding metadata extraction and analysis
//! - Search query optimization and result filtering
//! - Current events and news queries with source verification
//! - Multi-query search aggregation and synthesis
//!
//! Usage:
//! ```bash
//! # Basic search grounding query
//! cargo run --example gemini_search_grounding
//!
//! # Custom query with search grounding
//! cargo run --example gemini_search_grounding -- --query "What are the latest AI developments in 2024?"
//!
//! # News search with source analysis
//! cargo run --example gemini_search_grounding -- --mode news --topic "climate change research"
//!
//! # Multi-query synthesis
//! cargo run --example gemini_search_grounding -- --mode multi-query --queries "AI safety,machine learning ethics,AGI timeline"
//! ```

use api_gemini::{ client::Client, models::* };
use std::env;
use std::collections::HashMap;
use tokio::time::{ timeout, Duration };

/// Configuration for search grounding examples
#[ derive( Debug, Clone ) ]
pub struct SearchConfig
{
  /// Search mode (basic, news, multi-query)
  pub mode: SearchMode,
  /// Primary search query
  pub query: Option< String >,
  /// Topic for news mode
  pub topic: Option< String >,
  /// Multiple queries for synthesis
  pub queries: Vec< String >,
  /// Maximum results to process
  pub max_results: usize,
  /// Enable detailed source analysis
  pub analyze_sources: bool,
}

/// Search grounding execution modes
#[ derive( Debug, Clone ) ]
pub enum SearchMode
{
  /// Basic search grounding with general queries
  Basic,
  /// News-focused search with current events
  News,
  /// Multi-query synthesis and aggregation
  MultiQuery,
}

impl Default for SearchConfig
{
  fn default() -> Self
  {
    Self
    {
      mode: SearchMode::Basic,
      query: None,
      topic: None,
      queries: Vec::new(),
      max_results: 10,
      analyze_sources: true,
    }
  }
}

/// Create a test client using the API key from environment or file.
fn create_client() -> Result< Client, Box< dyn std::error::Error > >
{
  match std::env::var( "GEMINI_API_KEY" )
  {
    Ok( key ) if !key.is_empty() =>
    {
      Ok( Client::builder().api_key( key ).build()? )
    },
    _ => {
      // Try to read from secret file
      let secret_paths = vec![
      "secret/-secret.sh",
      "secret/gemini_api_key",
      ".env",
      ];

      for path in secret_paths
      {
        if let Ok( content ) = std::fs::read_to_string( path )
        {
          // Parse different formats
          for line in content.lines()
          {
            if line.starts_with( "GEMINI_API_KEY" )
            {
              if let Some( key ) = line.split( '=' ).nth( 1 )
              {
                let key = key.trim().trim_matches( '"' ).trim_matches( '\'' );
                if !key.is_empty()
                {
                  return Ok( Client::builder().api_key( key.to_string() ).build()? );
                }
              }
            }
          }
        }
      }

      Err( "No API key found. Set GEMINI_API_KEY environment variable or create secret file".into() )
    }
  }
}

/// Parse command line arguments
fn parse_args() -> SearchConfig
{
  let args: Vec< String > = env::args().collect();
  let mut config = SearchConfig::default();

  let mut i = 1;
  while i < args.len()
  {
    match args[ i ].as_str()
    {
      "--query" => {
        if i + 1 < args.len()
        {
          config.query = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      },
      "--mode" => {
        if i + 1 < args.len()
        {
          config.mode = match args[ i + 1 ].as_str()
          {
            "news" => SearchMode::News,
            "multi-query" => SearchMode::MultiQuery,
            _ => SearchMode::Basic,
          };
          i += 1;
        }
      },
      "--topic" => {
        if i + 1 < args.len()
        {
          config.topic = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      },
      "--queries" => {
        if i + 1 < args.len()
        {
          config.queries = args[ i + 1 ].split( ',' ).map( |s| s.trim().to_string() ).collect();
          i += 1;
        }
      },
    _ => {}
    }
    i += 1;
  }

  config
}

/// Perform a basic search grounding query
async fn basic_search_grounding(
client: &Client,
query: &str,
) -> Result< (), Box< dyn std::error::Error > >
{
println!( "🔍 Basic Search Grounding : {}", query );
println!( "{}", "=".repeat( 80 ) );

  // Configure Google Search tool
  let search_tool = Tool {
    function_declarations: None,
    code_execution: None,
    google_search_retrieval : Some( GoogleSearchTool {
      config: None, // Use default search configuration
    } ),
    code_execution_tool: None,
  };

  let request = GenerateContentRequest {
    contents : vec![ Content {
      parts : vec![ Part {
        text: Some( query.to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        file_data: None,
        video_metadata: None,
      } ],
      role: "user".to_string(),
    } ],
    generation_config : Some( GenerationConfig {
      temperature: Some( 0.7 ),
      top_k: Some( 40 ),
      top_p: Some( 0.95 ),
      candidate_count: Some( 1 ),
      max_output_tokens: Some( 2048 ),
      stop_sequences: None,
    } ),
    safety_settings: None,
    tools: Some( vec![ search_tool ] ),
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  println!( "📡 Sending search grounding request..." );
  let start_time = std::time::Instant::now();

  let response = timeout(
  Duration::from_secs( 45 ), // Search grounding may take longer
  client.models().by_name( "gemini-2.5-flash" ).generate_content( &request )
  ).await??;

  let duration = start_time.elapsed();
println!( "⚡ Response received in {:.2}s", duration.as_secs_f64() );

  // Display the main response
  if let Some( candidate ) = response.candidates.first()
  {
    if let Some( part ) = candidate.content.parts.first()
    {
      if let Some( text ) = &part.text
      {
        println!( "\n📝 Generated Response:" );
      println!( "{}", "-".repeat( 40 ) );
      println!( "{}", text );
      }
    }
  }

  // Analyze grounding metadata
  if let Some( grounding_metadata ) = &response.grounding_metadata
  {
    println!( "\n🔗 Grounding Analysis:" );
  println!( "{}", "-".repeat( 40 ) );

    // Display search queries used
    if let Some( web_search_queries ) = &grounding_metadata.web_search_queries
    {
      println!( "🔍 Search queries used:" );
      for ( i, query ) in web_search_queries.iter().enumerate()
      {
    println!( "  {}. {}", i + 1, query );
      }
      println!();
    }

    // Display grounding chunks (sources)
    if let Some( grounding_chunks ) = &grounding_metadata.grounding_chunks
    {
    println!( "📚 Sources found ({} total):", grounding_chunks.len() );
      for ( i, chunk ) in grounding_chunks.iter().enumerate()
      {
      println!( "\n  Source {}:", i + 1 );
        if let Some( title ) = &chunk.title
        {
        println!( "    📄 Title : {}", title );
        }
        if let Some( uri ) = &chunk.uri
        {
        println!( "    🔗 URL: {}", uri );
        }
        if let Some( domain ) = &chunk.domain
        {
        println!( "    🌐 Domain : {}", domain );
        }
        if let Some( published_date ) = &chunk.published_date
        {
        println!( "    📅 Published : {}", published_date );
        }
        if let Some( content ) = &chunk.content
        {
          let preview = if content.len() > 150
          {
          format!( "{}...", &content[ ..150 ] )
          } else {
            content.clone()
          };
        println!( "    📖 Content : {}", preview );
        }
      }
    }

    // Display grounding supports (which parts of response are grounded)
    if let Some( grounding_supports ) = &grounding_metadata.grounding_supports
    {
    println!( "\n🎯 Grounding Support ({} segments):", grounding_supports.len() );
      for ( i, support ) in grounding_supports.iter().enumerate()
      {
      println!( "  Segment {}:", i + 1 );
        if let ( Some( start ), Some( end ) ) = ( support.start_index, support.end_index )
        {
      println!( "    📍 Position : characters {} to {}", start, end );
        }
      println!( "    📊 Supported by {} sources", support.grounding_chunk_indices.len() );
        if let Some( confidence ) = support.confidence_score
        {
        println!( "    🎯 Confidence : {:.2}%", confidence * 100.0 );
        }
      }
    }

    // Display search entry point if available
    if let Some( search_entry_point ) = &grounding_metadata.search_entry_point
    {
      println!( "\n🚪 Search Entry Point:" );
      if let Some( rendered_content ) = &search_entry_point.rendered_content
      {
      println!( "  📄 Rendered content available ({} chars)", rendered_content.len() );
      }
      if search_entry_point.sdk_blob.is_some()
      {
        println!( "  🔧 SDK blob available" );
      }
    }
  } else {
    println!( "\n⚠️  No grounding metadata received" );
  }

  // Display usage metadata
  if let Some( usage ) = &response.usage_metadata
  {
    println!( "\n📊 Token Usage:" );
  println!( "{}", "-".repeat( 40 ) );
    if let Some( prompt_tokens ) = usage.prompt_token_count
    {
    println!( "📥 Prompt tokens : {}", prompt_tokens );
    }
    if let Some( candidates_tokens ) = usage.candidates_token_count
    {
    println!( "📤 Response tokens : {}", candidates_tokens );
    }
    if let Some( total_tokens ) = usage.total_token_count
    {
    println!( "🔢 Total tokens : {}", total_tokens );
    }
  }

  Ok( () )
}

/// Perform news search with detailed source analysis
async fn news_search_grounding(
client: &Client,
topic: &str,
) -> Result< (), Box< dyn std::error::Error > >
{
let query = format!( "What are the latest news and developments about {}? Please provide current information with sources.", topic );

println!( "📰 News Search Grounding: {}", topic );
println!( "{}", "=".repeat( 80 ) );

  basic_search_grounding( client, &query ).await?;

  Ok( () )
}

/// Perform multi-query synthesis with search grounding
async fn multi_query_synthesis(
client: &Client,
queries: &[ String ],
) -> Result< (), Box< dyn std::error::Error > >
{
  println!( "🔍 Multi-Query Search Synthesis" );
println!( "{}", "=".repeat( 80 ) );

  let combined_query = format!(
"Please research and synthesize information about the following topics : {}. \
  Provide a comprehensive analysis with current information and sources for each topic.",
  queries.join( ", " )
  );

  println!( "🎯 Queries to synthesize:" );
  for ( i, query ) in queries.iter().enumerate()
  {
println!( "  {}. {}", i + 1, query );
  }
  println!();

  basic_search_grounding( client, &combined_query ).await?;

  Ok( () )
}

/// Demonstrate search result quality analysis
async fn analyze_search_quality(
client: &Client,
) -> Result< (), Box< dyn std::error::Error > >
{
  println!( "🔬 Search Quality Analysis" );
println!( "{}", "=".repeat( 80 ) );

  let test_queries = vec![
  "What is the current state of quantum computing research?",
  "Latest developments in renewable energy technology 2024",
  "Recent breakthroughs in artificial intelligence safety",
  "Current economic indicators and market trends",
  ];

  for ( i, query ) in test_queries.iter().enumerate()
  {
println!( "\n🧪 Test Query {} of {}", i + 1, test_queries.len() );
  println!( "Query : {}", query );
  println!( "{}", "-".repeat( 60 ) );

    let search_tool = Tool {
      function_declarations: None,
      code_execution: None,
      google_search_retrieval : Some( GoogleSearchTool {
        config: None,
      } ),
      code_execution_tool: None,
    };

    let request = GenerateContentRequest {
      contents : vec![ Content {
        parts : vec![ Part {
          text: Some( query.to_string() ),
          inline_data: None,
          function_call: None,
          function_response: None,
          file_data: None,
          video_metadata: None,
        } ],
        role: "user".to_string(),
      } ],
      generation_config : Some( GenerationConfig {
        temperature: Some( 0.3 ), // Lower temperature for more factual responses
        max_output_tokens: Some( 1024 ),
        ..Default::default()
      } ),
      tools: Some( vec![ search_tool ] ),
      ..Default::default()
    };

    let start_time = std::time::Instant::now();

    match timeout(
    Duration::from_secs( 30 ),
    client.models().by_name( "gemini-2.5-flash" ).generate_content( &request )
    ).await {
      Ok( Ok( response ) ) => {
        let duration = start_time.elapsed();

        // Analyze response quality
        let mut quality_metrics = HashMap::new();

        if let Some( grounding_metadata ) = &response.grounding_metadata
        {
          if let Some( chunks ) = &grounding_metadata.grounding_chunks
          {
            quality_metrics.insert( "source_count", chunks.len() );

            let unique_domains: std::collections::HashSet<  _  > = chunks
            .iter()
            .filter_map( |c| c.domain.as_ref() )
            .collect();
            quality_metrics.insert( "unique_domains", unique_domains.len() );

            let has_recent_content = chunks.iter().any( |c|
            c.published_date.as_ref().map_or( false, |d| d.contains( "2024" ) )
            );
        quality_metrics.insert( "recent_content", if has_recent_content { 1 } else { 0 } );
          }

          if let Some( supports ) = &grounding_metadata.grounding_supports
          {
            let avg_confidence = supports
            .iter()
            .filter_map( |s| s.confidence_score )
            .sum::< f64 >() / supports.len() as f64;
            quality_metrics.insert( "avg_confidence", ( avg_confidence * 100.0 ) as usize );
          }
        }

      println!( "✅ Success ({:.2}s)", duration.as_secs_f64() );
        println!( "📊 Quality Metrics:" );
        for ( metric, value ) in quality_metrics
        {
      println!( "   {}: {}", metric, value );
        }

        if let Some( candidate ) = response.candidates.first()
        {
          if let Some( part ) = candidate.content.parts.first()
          {
            if let Some( text ) = &part.text
            {
              let word_count = text.split_whitespace().count();
            println!( "   response_words : {}", word_count );
            }
          }
        }
      },
      Ok( Err( e ) ) => {
      println!( "❌ API Error : {:?}", e );
      },
      Err( _ ) => {
        println!( "⏰ Timeout after 30s" );
      }
    }

    // Brief pause between queries to avoid rate limiting
    tokio ::time::sleep( Duration::from_millis( 500 ) ).await;
  }

  Ok( () )
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  println!( "🚀 Google Search Grounding Example" );
  println!( "====================================" );

  let client = create_client()?;
  let config = parse_args();

  match config.mode
  {
    SearchMode::Basic => {
      let query = config.query.unwrap_or_else( ||
      "What are the latest developments in artificial intelligence and machine learning in 2024?".to_string()
      );
      basic_search_grounding( &client, &query ).await?;
    },
    SearchMode::News => {
      let topic = config.topic.unwrap_or_else( || "artificial intelligence".to_string() );
      news_search_grounding( &client, &topic ).await?;
    },
    SearchMode::MultiQuery => {
      let queries = if config.queries.is_empty()
      {
        vec![
        "quantum computing progress".to_string(),
        "renewable energy breakthroughs".to_string(),
        "space exploration missions".to_string(),
        ]
      } else {
        config.queries
      };
      multi_query_synthesis( &client, &queries ).await?;
    },
  }

  // Run quality analysis if enabled
  if config.analyze_sources
  {
    println!( "\n" );
    analyze_search_quality( &client ).await?;
  }

  println!( "\n✅ Search grounding examples completed successfully!" );
  println!( "\n💡 Tips:" );
  println!( "   • Use specific, current topics for better grounding results" );
  println!( "   • Check grounding metadata for source attribution" );
  println!( "   • Lower temperature values provide more factual responses" );
  println!( "   • Search grounding works best with factual queries" );

  Ok( () )
}