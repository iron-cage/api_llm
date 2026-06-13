//! Example demonstrating text embeddings generation for semantic search and similarity.
//!
//! This example shows:
//! - How to generate embeddings for text content
//! - How to use task types for optimized embeddings
//! - How to batch process multiple texts
//! - How to calculate cosine similarity between embeddings


use api_gemini::{ client::Client, models::* };

/// Calculate cosine similarity between two embedding vectors
fn cosine_similarity( a: &[ f32 ], b: &[ f32 ] ) -> f32
{
  let dot_product: f32 = a
  .iter()
  .zip( b.iter() )
  .map( |( x, y )| x * y )
  .sum();
  let magnitude_a: f32 = a
  .iter()
  .map( |x| x * x )
  .sum::< f32 >()
  .sqrt();
  let magnitude_b: f32 = b
  .iter()
  .map( |x| x * x )
  .sum::< f32 >()
  .sqrt();

  dot_product / ( magnitude_a * magnitude_b )
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  let client = Client::new()?;

  println!( "=== Text Embeddings Example ===" );

  // Example 1: Generate a single embedding
  println!( "\n1. Single Text Embedding" );

  let text_to_embed = "The quick brown fox jumps over the lazy dog";

  let embed_request = EmbedContentRequest
  {
    content: Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( text_to_embed.to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    },
    // Task types help optimize embeddings for specific use cases:
    // - RETRIEVAL_QUERY: For search queries
    // - RETRIEVAL_DOCUMENT: For documents to be searched
    // - SEMANTIC_SIMILARITY: For comparing text similarity
    // - CLASSIFICATION: For text classification tasks
    task_type: Some( "RETRIEVAL_DOCUMENT".to_string() ),
    title: None, // Optional title for document embeddings
    output_dimensionality: None, // Use model's default dimensions
  };

  let response = client
  .models()
  .by_name( "gemini-embedding-001" )
  .embed_content( &embed_request )
  .await?;

println!( "Embedded text : \"{text_to_embed}\"" );
println!( "Embedding dimensions : {}", response.embedding.values.len() );
println!( "First 5 values : {:?}", &response.embedding.values[ ..5 ] );

  // Example 2: Batch embeddings for multiple texts
  println!( "\n2. Batch Text Embeddings for Similarity Comparison" );

  let texts =
  [
  "The weather is beautiful today",
  "It's a sunny and warm day",
  "I love programming in Rust",
  "Rust is a great systems programming language",
  "Pizza is my favorite food",
  ];

  // Create batch request
  let _batch_request = BatchEmbedContentsRequest
  {
    requests: texts
    .iter()
    .map( |text|
    {
      EmbedContentRequest
      {
        content: Content
        {
          role: "user".to_string(),
          parts: vec!
          [
          Part
          {
            text: Some( text.to_string() ),
            inline_data: None,
            function_call: None,
            function_response: None,
            ..Default::default()
          }
          ],
        },
        task_type: Some( "SEMANTIC_SIMILARITY".to_string() ),
        title: None,
        output_dimensionality: None,
      }
    })
    .collect(),
  };

  // Note : The batch endpoint would be called differently in a real implementation
  // For this example, we'll generate individual embeddings
  let mut embeddings = Vec::new();

  for ( i, text ) in texts.iter().enumerate()
  {
    let request = EmbedContentRequest
    {
      content: Content
      {
        role: "user".to_string(),
        parts: vec!
        [
        Part
        {
          text: Some( text.to_string() ),
          inline_data: None,
          function_call: None,
          function_response: None,
          ..Default::default()
        }
        ],
      },
      task_type: Some( "SEMANTIC_SIMILARITY".to_string() ),
      title: None,
      output_dimensionality: None,
    };

    let response = client
    .models()
    .by_name( "gemini-embedding-001" )
    .embed_content( &request )
    .await?;

    embeddings.push( response.embedding.values );
println!( "Embedded text {}: \"{}\"", i + 1, text );
  }

  // Calculate similarity matrix
  println!( "\n3. Similarity Matrix (Cosine Similarity)" );
  println!( "Higher values (closer to 1.0) indicate more similar texts\n" );

  // Print header
  print!( "     " );
  for i in 0..texts.len()
  {
  print!( "  Text{}  ", i + 1 );
  }
  println!();

  // Calculate and display similarities
  for i in 0..texts.len()
  {
  print!( "Text{} ", i + 1 );
    for j in 0..texts.len()
    {
      let similarity = cosine_similarity( &embeddings[ i ], &embeddings[ j ] );
    print!( " {similarity:.3}   " );
    }
    println!();
  }

  // Find most similar pairs
  println!( "\n4. Most Similar Text Pairs:" );
  let mut similarities = Vec::new();

  for i in 0..texts.len()
  {
    for j in i + 1..texts.len()
    {
      let similarity = cosine_similarity( &embeddings[ i ], &embeddings[ j ] );
      similarities.push( ( i, j, similarity ) );
    }
  }

  similarities.sort_by( |a, b| b.2.partial_cmp( &a.2 ).unwrap() );

  for ( i, ( idx1, idx2, sim ) ) in similarities.iter().take( 3 ).enumerate()
  {
println!( "{}. Similarity : {:.3}", i + 1, sim );
println!( "   Text {}: \"{}\"", idx1 + 1, texts[ *idx1 ] );
println!( "   Text {}: \"{}\"", idx2 + 1, texts[ *idx2 ] );
  }

  // Example 3: Query-Document matching
  println!( "\n5. Query-Document Matching Example" );

  let query = "What's the weather like?";
  let query_request = EmbedContentRequest
  {
    content: Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( query.to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    },
    task_type: Some( "RETRIEVAL_QUERY".to_string() ), // Optimized for queries
    title: None,
    output_dimensionality: None,
  };

  let query_response = client
  .models()
  .by_name( "gemini-embedding-001" )
  .embed_content( &query_request )
  .await?;

println!( "Query : \"{query}\"" );
  println!( "\nRelevance scores:" );

  for ( i, text ) in texts.iter().enumerate()
  {
    let similarity = cosine_similarity( &query_response.embedding.values, &embeddings[ i ] );
println!( "  {similarity:.3} - \"{text}\"" );
  }

  println!( "\n=== Key Points About Embeddings ===" );
  println!( "1. Use appropriate task_type for better performance" );
  println!( "2. Batch processing is more efficient for multiple texts" );
  println!( "3. Cosine similarity is the standard metric for comparing embeddings" );
  println!( "4. Store embeddings in a vector database for production use" );

  Ok( () )
}