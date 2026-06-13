# api_huggingface API Reference

Complete API reference for the HuggingFace client library.

## Client Operations

The `Client` provides access to all API endpoints through specialized interfaces:

### Text Generation

Generate text using large language models:

```rust
// Basic generation
let response = client.inference()
  .create( "What is the capital of France?", Models::llama_3_1_8b_instruct() )
  .await?;

// With parameters
let params = InferenceParameters::new()
  .with_temperature( 0.7 )
  .with_max_new_tokens( 100 );

let response = client.inference()
  .create_with_parameters( "Tell me a story", Models::llama_3_1_8b_instruct(), params )
  .await?;

// Streaming
let mut stream = client.inference()
  .create_stream( "Generate a long response", Models::llama_3_1_8b_instruct() )
  .await?;

while let Some( chunk ) = stream.next().await
{
  println!( "{:?}", chunk );
}
```

**Methods:**
- `create(prompt, model)` - Basic text generation
- `create_with_parameters(prompt, model, params)` - Generation with custom parameters
- `create_stream(prompt, model)` - Streaming generation

### Embeddings

Generate vector embeddings for text:

```rust
// Single embedding
let response = client.embeddings()
  .create( "Hello, world!", Models::all_minilm_l6_v2() )
  .await?;

// Batch embeddings
let texts = vec![ "First text", "Second text" ];
let response = client.embeddings()
  .create_batch( &texts, Models::all_minilm_l6_v2() )
  .await?;

// Similarity calculation
let similarity = client.embeddings()
  .similarity( "I love programming", "Coding is fun", Models::all_minilm_l6_v2() )
  .await?;
```

**Methods:**
- `create(text, model)` - Generate single embedding
- `create_batch(texts, model)` - Generate multiple embeddings
- `similarity(text1, text2, model)` - Calculate cosine similarity

### Model Management

Query model information and availability:

```rust
// Get model info
let info = client.models()
  .get( Models::llama_3_1_8b_instruct() )
  .await?;

// Check availability
let available = client.models()
  .is_available( Models::llama_3_1_8b_instruct() )
  .await?;

// Get status
let status = client.models()
  .status( Models::llama_3_1_8b_instruct() )
  .await?;

// Wait for model to load
client.models()
  .wait_for_model( Models::llama_3_1_8b_instruct(), Duration::from_secs( 30 ) )
  .await?;
```

**Methods:**
- `get(model)` - Retrieve model information
- `is_available(model)` - Check if model is ready
- `status(model)` - Get current model status
- `wait_for_model(model, timeout)` - Wait for model to become available

## Popular Models

The library provides convenient constants for commonly used models:

### Text Generation Models

```rust
use api_huggingface::components::models::Models;

// Llama models
Models::llama_3_1_8b_instruct()      // Meta Llama 3.1 8B
Models::llama_3_2_1b_instruct()      // Meta Llama 3.2 1B
Models::llama_2_7b_chat()            // Meta Llama 2 7B Chat

// Mistral models
Models::mistral_7b_instruct()        // Mistral 7B Instruct
Models::mixtral_8x7b_instruct()      // Mixtral 8x7B

// Other models
Models::kimi_k2_instruct()           // Moonshot Kimi K2
```

### Embedding Models

```rust
// Sentence transformers
Models::all_minilm_l6_v2()           // All-MiniLM-L6-v2 (fast, 384 dims)
Models::bge_large_en_v1_5()          // BGE Large English (1024 dims)
Models::gte_large()                  // GTE Large (1024 dims)

// Specialized embeddings
Models::nomic_embed_text_v1_5()      // Nomic Embed Text
```

## Environment Configuration

### Required Environment Variables

```bash
# API key (required)
export HUGGINGFACE_API_KEY="hf_..."
```

### Optional Environment Variables

```bash
# Custom API endpoint (optional)
export HUGGINGFACE_BASE_URL="https://api-inference.huggingface.co"
```

### Loading from Environment

```rust
use api_huggingface::{ Client, environment::HuggingFaceEnvironmentImpl, secret::Secret };

// Load from environment variable
let api_key = Secret::load_from_env( "HUGGINGFACE_API_KEY" )?;

// Build environment with default URL
let env = HuggingFaceEnvironmentImpl::build( api_key, None )?;

// Build environment with custom URL
let custom_url = Some( "https://custom-endpoint.com".to_string() );
let env = HuggingFaceEnvironmentImpl::build( api_key, custom_url )?;

// Create client
let client = Client::build( env )?;
```

## Error Handling

The library provides comprehensive error types for precise error handling:

### Error Types

```rust
use api_huggingface::error::HuggingFaceError;

match result
{
  Ok( response ) => { /* handle success */ }
  Err( HuggingFaceError::Api( msg ) ) =>
  {
    // API-specific errors (invalid request, model errors)
    eprintln!( "API error: {}", msg );
  }
  Err( HuggingFaceError::Authentication( msg ) ) =>
  {
    // Authentication failures (invalid key, expired token)
    eprintln!( "Auth error: {}", msg );
  }
  Err( HuggingFaceError::RateLimit( msg ) ) =>
  {
    // Rate limiting errors
    eprintln!( "Rate limit: {}", msg );
  }
  Err( HuggingFaceError::ModelUnavailable( msg ) ) =>
  {
    // Model loading or availability errors
    eprintln!( "Model unavailable: {}", msg );
  }
  Err( e ) =>
  {
    // Other errors
    eprintln!( "Error: {:?}", e );
  }
}
```

### Error Categories

- **`Api`** - Invalid requests, model errors, parameter validation
- **`Authentication`** - Invalid API key, expired tokens, permission errors
- **`RateLimit`** - Request rate exceeded, quota limits
- **`ModelUnavailable`** - Model loading, cold start, not found
- **`Network`** - Connection errors, timeouts, DNS failures
- **`Serialization`** - JSON parsing errors, invalid response formats

## Parameters and Options

### InferenceParameters

Control text generation behavior:

```rust
use api_huggingface::components::input::InferenceParameters;

let params = InferenceParameters::new()
  .with_temperature( 0.7 )        // Randomness (0.0-2.0)
  .with_max_new_tokens( 100 )     // Maximum tokens to generate
  .with_top_p( 0.95 )             // Nucleus sampling
  .with_top_k( 50 )               // Top-k sampling
  .with_repetition_penalty( 1.2 ); // Penalize repetition
```

**Common Parameters:**
- `temperature` - Controls randomness (0.0 = deterministic, 2.0 = very random)
- `max_new_tokens` - Maximum tokens to generate
- `top_p` - Nucleus sampling threshold
- `top_k` - Top-k sampling threshold
- `repetition_penalty` - Penalty for repeating tokens

### EmbeddingOptions

Configure embedding generation:

```rust
use api_huggingface::components::embeddings::EmbeddingOptions;

let options = EmbeddingOptions::new()
  .with_normalize( true )       // Normalize vectors
  .with_truncate( true );       // Truncate long inputs
```

## Response Types

### InferenceResponse

Text generation response:

```rust
pub struct InferenceResponse
{
  pub generated_text : Option< String >,
  pub details : Option< ResponseDetails >,
}

// Extract text with fallback
let text = response.extract_text_or_default( "Default text" );
```

### EmbeddingResponse

Embedding generation response:

```rust
pub struct EmbeddingResponse
{
  pub embeddings : Vec< Vec< f32 > >,
  pub model : String,
}

// Access embeddings
let vector = &response.embeddings[ 0 ];
```

### StreamingChunk

Streaming response chunk:

```rust
pub struct StreamingChunk
{
  pub token : Option< String >,
  pub details : Option< ChunkDetails >,
}
```

## Advanced Features

### Synchronous API

For blocking/synchronous contexts:

```rust
use api_huggingface::sync::SyncClient;

// Create sync client (requires "sync" feature)
let client = SyncClient::new( "hf_...".to_string() )?;

// Blocking calls
let response = client.inference()
  .create( "What is 2+2?", "meta-llama/Llama-3.2-1B-Instruct" )?;
```

### CURL Diagnostics

Generate curl commands for debugging:

```rust
// Generate equivalent curl command for debugging
let curl_command = client.generate_curl_command( request )?;
println!( "Debug: {}", curl_command );
```

## See Also

- [Features Overview](features.md) - Complete feature list and status
- [Examples](../examples/readme.md) - Working code examples
