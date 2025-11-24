# Usage Examples

This document contains comprehensive usage examples for the api_gemini crate.

## Text Generation

```rust,no_run
use api_gemini::{ client::Client, models::* };

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::new()?;

  let request = GenerateContentRequest
  {
    contents: vec!
    [
      Content
      {
        parts: vec!
        [
          Part
          {
            text: Some( "Explain quantum computing in simple terms".to_string() ),
            ..Default::default()
          }
        ],
        role: "user".to_string(),
      }
    ],
    generation_config: Some
    (
      GenerationConfig
      {
        temperature: Some( 0.7 ),
        top_k: Some( 40 ),
        top_p: Some( 0.95 ),
        max_output_tokens: Some( 1024 ),
        ..Default::default()
      }
    ),
    ..Default::default()
  };

  let response = client
    .models()
    .by_name( "gemini-1.5-pro-latest" )
    .generate_content( &request )
    .await?;
  Ok( () )
}
```

## Multi-turn Conversations

```rust,no_run
use api_gemini::{ client::Client, models::* };

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::new()?;

  let conversation = vec!
  [
    Content
    {
      role: "user".to_string(),
      parts: vec![ Part { text: Some( "What is the capital of France?".to_string() ), ..Default::default() } ],
    },
    Content
    {
      role: "model".to_string(),
      parts: vec![ Part { text: Some( "The capital of France is Paris.".to_string() ), ..Default::default() } ],
    },
    Content
    {
      role: "user".to_string(),
      parts: vec![ Part { text: Some( "What's the population?".to_string() ), ..Default::default() } ],
    },
  ];

  let request = GenerateContentRequest { contents: conversation, ..Default::default() };
  Ok( () )
}
```

## Vision (Multimodal)

```rust,no_run
use api_gemini::{ client::Client, models::* };
use base64::Engine;

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::new()?;

  let image_data = std::fs::read( "image.jpg" )?;
  let base64_image = base64::engine::general_purpose::STANDARD.encode( image_data );

  let request = GenerateContentRequest
  {
    contents: vec!
    [
      Content
      {
        parts: vec!
        [
          Part { text: Some( "What's in this image?".to_string() ), ..Default::default() },
          Part { inline_data: Some( Blob { mime_type: "image/jpeg".to_string(), data: base64_image } ), ..Default::default() },
        ],
        role: "user".to_string(),
      }
    ],
    ..Default::default()
  };
  Ok( () )
}
```

## Function Calling

```rust,no_run
use api_gemini::{ client::Client, models::* };
use serde_json::json;

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::new()?;

  let tools = vec!
  [
    Tool
    {
      function_declarations: Some
      (
        vec!
        [
          FunctionDeclaration
          {
            name: "get_weather".to_string(),
            description: "Get weather in a location".to_string(),
            parameters: Some
            (
              json!
              (
                {
                  "type": "object",
                  "properties": { "location": { "type": "string", "description": "City name" } },
                  "required": ["location"]
                }
              )
            ),
          }
        ]
      ),
      code_execution: None,
      google_search_retrieval: None,
      code_execution_tool: None,
    }
  ];

  let request = GenerateContentRequest
  {
    contents: vec!
    [
      Content
      {
        parts: vec![ Part { text: Some( "What's the weather in Tokyo?".to_string() ), ..Default::default() } ],
        role: "user".to_string(),
      }
    ],
    tools: Some( tools ),
    ..Default::default()
  };
  Ok( () )
}
```

## Google Search Grounding

Real-time web search integration with attribution:

```rust,no_run
use api_gemini::{ client::Client, models::* };

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::new()?;

  let tools = vec!
  [
    Tool
    {
      function_declarations: None,
      code_execution: None,
      google_search_retrieval: Some( GoogleSearchTool { config: None } ),
      code_execution_tool: None,
    }
  ];

  let request = GenerateContentRequest
  {
    contents: vec!
    [
      Content
      {
        parts: vec![ Part { text: Some( "What are the latest developments in AI technology?".to_string() ), ..Default::default() } ],
        role: "user".to_string(),
      }
    ],
    tools: Some( tools ),
    ..Default::default()
  };

  let response = client.models().by_name( "gemini-2.5-flash" ).generate_content( &request ).await?;

  // Check for grounding metadata and citations
  if let Some( grounding_metadata ) = &response.grounding_metadata
  {
    if let Some( grounding_chunks ) = &grounding_metadata.grounding_chunks
    {
      println!( "Sources used:" );
      for chunk in grounding_chunks
      {
        if let Some( uri ) = &chunk.uri { println!( "  - {}", uri ); }
      }
    }
  }
  Ok( () )
}
```

## System Instructions

```rust,no_run
use api_gemini::{ client::Client, models::* };

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::new()?;

  let system_instruction = SystemInstruction
  {
    role: "system".to_string(),
    parts: vec!
    [
      Part
      {
        text: Some( "You are a helpful technical assistant. Always provide code examples.".to_string() ),
        ..Default::default()
      }
    ],
  };

  let request = GenerateContentRequest
  {
    contents: vec!
    [
      Content
      {
        parts: vec![ Part { text: Some( "How do I implement error handling in Rust?".to_string() ), ..Default::default() } ],
        role: "user".to_string(),
      }
    ],
    system_instruction: Some( system_instruction ),
    ..Default::default()
  };

  let response = client.models().by_name( "gemini-2.5-flash" ).generate_content( &request ).await?;
  Ok( () )
}
```

## Code Execution

```rust,no_run
use api_gemini::{ client::Client, models::* };

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::new()?;

  let code_execution_tool = CodeExecutionTool
  {
    config: Some
    (
      CodeExecutionConfig
      {
        timeout: Some( 30 ),
        enable_network: Some( false ),
      }
    ),
  };

  let tools = vec!
  [
    Tool
    {
      function_declarations: None,
      code_execution: None,
      google_search_retrieval: None,
      code_execution_tool: Some( code_execution_tool ),
    }
  ];

  let request = GenerateContentRequest
  {
    contents: vec!
    [
      Content
      {
        parts: vec![ Part { text: Some( "Calculate the factorial of 10 using Python".to_string() ), ..Default::default() } ],
        role: "user".to_string(),
      }
    ],
    tools: Some( tools ),
    ..Default::default()
  };

  let response = client.models().by_name( "gemini-2.5-flash" ).generate_content( &request ).await?;
  Ok( () )
}
```

## Embeddings

```rust,no_run
use api_gemini::{ client::Client, models::* };

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::new()?;

  let embed_request = EmbedContentRequest
  {
    content: Content
    {
      parts: vec![ Part { text: Some( "The quick brown fox".to_string() ), ..Default::default() } ],
      role: "user".to_string(),
    },
    task_type: Some( "RETRIEVAL_DOCUMENT".to_string() ),
    title: None,
    output_dimensionality: None,
  };

  let response = client.models().by_name( "models/text-embedding-004" ).embed_content( &embed_request ).await?;
  println!( "Embedding dimensions: {}", response.embedding.values.len() );
  Ok( () )
}
```

## Model Information

```rust,no_run
use api_gemini::{ client::Client, models::* };

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::new()?;

  // List available models
  let models = client.models().list().await?;
  for model in models.models
  {
    println!( "Model: {}", model.name );
  }

  // Get specific model details
  let model = client.models().get( "models/gemini-1.5-pro-latest" ).await?;
  println!( "Token limit: {:?}", model.input_token_limit );
  Ok( () )
}
```

## Synchronous API

```rust,no_run
use api_gemini::{ client::Client, models::* };
use std::time::Duration;

fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let sync_client = Client::sync_builder()
    .api_key( "your-api-key".to_string() )
    .timeout( Duration::from_secs( 30 ) )
    .build()?;

  let request = GenerateContentRequest
  {
    contents: vec!
    [
      Content
      {
        parts: vec![ Part { text: Some( "Hello, Gemini!".to_string() ), ..Default::default() } ],
        role: "user".to_string(),
      }
    ],
    ..Default::default()
  };

  let response = sync_client.models().by_name( "gemini-1.5-pro-latest" )?.generate_content( &request )?;

  if let Some( text ) = response.candidates.first()
    .and_then( |c| c.content.parts.first() )
    .and_then( |p| p.text.as_ref() )
  {
    println!( "Response: {}", text );
  }
  Ok( () )
}
```

## Safety Settings

```rust,no_run
use api_gemini::{ client::Client, models::* };

let safety_settings = vec!
[
  SafetySetting
  {
    category: HarmCategory::HarmCategoryHarassment,
    threshold: HarmBlockThreshold::BlockMediumAndAbove,
  },
  SafetySetting
  {
    category: HarmCategory::HarmCategoryHateSpeech,
    threshold: HarmBlockThreshold::BlockOnlyHigh,
  },
];

let request = GenerateContentRequest
{
  safety_settings: Some( safety_settings ),
  ..Default::default()
};
```

## Server-side Cached Content

```rust,no_run
use api_gemini::{ client::Client, models::* };

#[tokio::main]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let client = Client::new()?;

  let cache_request = CreateCachedContentRequest
  {
    model: "gemini-1.5-pro-latest".to_string(),
    contents: vec![/* conversation context */],
    ttl: Some( "3600s".to_string() ),
    expire_time: None,
    display_name: Some( "My Conversation Cache".to_string() ),
    system_instruction: Some
    (
      Content
      {
        parts: vec![ Part { text: Some( "You are a helpful assistant".to_string() ), ..Default::default() } ],
        role: "system".to_string(),
      }
    ),
    tools: None,
    tool_config: None,
  };

  let cache = client.cached_content().create( &cache_request ).await?;
  println!( "Created cache: {}", cache.name );

  // Use cached content in conversations
  let request = GenerateContentRequest
  {
    contents: vec![/* new messages only */],
    cached_content: Some( cache.name ),
    ..Default::default()
  };

  let response = client.models().by_name( "gemini-1.5-pro-latest" ).generate_content( &request ).await?;
  Ok( () )
}
```

## Related Documentation

- **[Cookbook](cookbook.md)** - Recipe patterns for common use cases
- **[Testing](testing.md)** - Test organization and coverage
- **[API Coverage](api_coverage.md)** - Complete API endpoint documentation
