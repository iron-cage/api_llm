# api_gemini Examples

Examples demonstrating the `api_gemini` crate, organized by difficulty.

## Quick Start

1. `gemini_dry_run.rs` - Validate setup without API calls
2. `gemini_api_basic.rs` - Basic conversation
3. `gemini_list_models.rs` - Explore available models

## All Examples

### Beginner
- `gemini_dry_run.rs` - Validate patterns without API calls
- `gemini_api_basic.rs` - Basic text generation
- `gemini_list_models.rs` - Model exploration

### Intermediate
- `gemini_embeddings.rs` - Text embeddings and search
- `gemini_multimodal.rs` - Image analysis
- `gemini_safety_settings.rs` - Content filtering
- `gemini_error_handling.rs` - Error handling patterns
- `gemini_code_execution.rs` - Python code generation and execution via Gemini
- `gemini_search_grounding.rs` - Google Search grounding for real-time information
- `gemini_system_instructions.rs` - System prompt and instruction configuration

### Advanced
- `gemini_api_interactive.rs` - Real-time interactive chat with streaming
- `gemini_api_cached_interactive.rs` - Advanced chat with server-side caching (Featured)
- `gemini_function_calling.rs` - AI agents with tools
- `gemini_performance_optimization.rs` - Performance patterns and monitoring

## Setup

```bash
# Set API key
export GEMINI_API_KEY="your-key"

# Run examples
cargo run --example gemini_api_basic
cargo run --example gemini_api_interactive --features streaming
cargo run --example gemini_api_cached_interactive --features streaming
cargo run --example gemini_performance_optimization --features logging
```

## Features

- `streaming` - Real-time responses
- `logging` - Structured logging (used in performance_optimization.rs)
- `full` - All features

See individual example files for detailed documentation and usage instructions.