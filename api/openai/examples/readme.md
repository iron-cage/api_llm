# OpenAI API Examples

This directory contains comprehensive examples demonstrating how to use the OpenAI API client library. Each example is a standalone Rust program that showcases different aspects of the API.

## Prerequisites

Before running any examples, make sure you have:

1. **API Key**: Set your OpenAI API key in one of these ways:
   - Environment variable: `export OPENAI_API_KEY="your-api-key"`
   - Secret file: Create `secret/-secret.sh` with `export OPENAI_API_KEY="your-api-key"`

2. **Dependencies**: Run `cargo build` to install all required dependencies.

## Running Examples

To run any example:

```bash
cargo run --example <example_name>
```

For example:
```bash
cargo run --example responses_create
```

## Examples Index

### 🎯 Responses API

| Example | Description | Key Features | API Endpoints |
|---------|-------------|-------------|---------------|
| `responses_create.rs` | Basic response creation *(planned)* | Simple text input, response parsing | `POST /responses` |
| [`openai_responses_create_image_input.rs`](openai_responses_create_image_input.rs) | Multimodal response with image input | Image + text processing, vision capabilities | `POST /responses` |
| `responses_create_stream.rs` | Streaming response generation *(planned)* | Real-time text streaming, event handling | `POST /responses` (stream) |
| [`openai_responses_create_with_tools.rs`](openai_responses_create_with_tools.rs) | Response with function calling | Tool definitions, function execution | `POST /responses` |
| [`openai_responses_get.rs`](openai_responses_get.rs) | Retrieve existing response | Response retrieval by ID | `GET /responses/{id}` |
| [`openai_responses_list_input_items.rs`](openai_responses_list_input_items.rs) | List response input items | Input item enumeration, pagination | `GET /responses/{id}/input_items` |
| [`openai_responses_delete.rs`](openai_responses_delete.rs) | Delete a response | Response cleanup, deletion verification | `DELETE /responses/{id}` |
| [`openai_responses_update.rs`](openai_responses_update.rs) | Update response metadata | Metadata modification, response updates | `PATCH /responses/{id}` |
| [`openai_responses_cancel.rs`](openai_responses_cancel.rs) | Cancel in-progress response | Stream cancellation, cleanup | `POST /responses/{id}/cancel` |

### 🔄 Realtime API

| Example | Description | Key Features | API Endpoints |
|---------|-------------|-------------|---------------|
| [`openai_realtime_response_create.rs`](openai_realtime_response_create.rs) | Create realtime response | WebSocket connection, realtime communication | WebSocket `/realtime` |
| [`openai_realtime_response_cancel.rs`](openai_realtime_response_cancel.rs) | Cancel realtime response | Realtime cancellation, connection cleanup | WebSocket `/realtime` |
| [`openai_realtime_conversation_item_create.rs`](openai_realtime_conversation_item_create.rs) | Create conversation item | Conversation management, item creation | WebSocket `/realtime` |
| [`openai_realtime_conversation_item_delete.rs`](openai_realtime_conversation_item_delete.rs) | Delete conversation item | Item cleanup, conversation management | WebSocket `/realtime` |
| [`openai_realtime_conversation_item_retrieve.rs`](openai_realtime_conversation_item_retrieve.rs) | Retrieve conversation item | Item retrieval, conversation access | WebSocket `/realtime` |
| [`openai_realtime_conversation_item_truncate.rs`](openai_realtime_conversation_item_truncate.rs) | Truncate conversation item | Item modification, content truncation | WebSocket `/realtime` |
| [`openai_realtime_input_audio_buffer_append.rs`](openai_realtime_input_audio_buffer_append.rs) | Append audio buffer | Audio streaming, buffer management | WebSocket `/realtime` |
| [`openai_realtime_input_audio_buffer_clear.rs`](openai_realtime_input_audio_buffer_clear.rs) | Clear audio buffer | Buffer cleanup, audio management | WebSocket `/realtime` |
| [`openai_realtime_input_audio_buffer_commit.rs`](openai_realtime_input_audio_buffer_commit.rs) | Commit audio buffer | Buffer processing, audio finalization | WebSocket `/realtime` |
| [`openai_realtime_session_update.rs`](openai_realtime_session_update.rs) | Update realtime session | Session configuration, parameter updates | WebSocket `/realtime` |
| [`openai_realtime_transcription_session_update.rs`](openai_realtime_transcription_session_update.rs) | Update transcription session | Transcription settings, session management | WebSocket `/realtime` |

### 💬 Chat Completions API

*Note: Chat completion examples are planned but not yet implemented. Use responses API examples for similar functionality.*

### 🎵 Audio API

*Note: Audio API examples are planned but not yet implemented.*

### 🖼️ Images API

*Note: Images API examples are planned but not yet implemented.*

### 📄 Files API

*Note: Files API examples are planned but not yet implemented.*

### 🔧 Fine-tuning API

*Note: Fine-tuning API examples are planned but not yet implemented.*

### 🤖 Assistants API

*Note: Assistants API examples are planned but not yet implemented.*

### 🔍 Vector Stores API

*Note: Vector Stores API examples are planned but not yet implemented.*

### 🛡️ Moderations API

*Note: Moderations API examples are planned but not yet implemented.*

### 📊 Models API

*Note: Models API examples are planned but not yet implemented.*

### 🔗 Embeddings API

*Note: Embeddings API examples are planned but not yet implemented.*

## Example Categories

### 🚀 **Beginner Examples**
Start with these if you're new to the OpenAI API:
- `responses_create.rs` - Basic text generation *(planned)*
- `openai_responses_get.rs` - Retrieve responses by ID
- `openai_realtime_response_create.rs` - Real-time communication basics

### 🔥 **Advanced Examples**
These showcase more complex functionality:
- `openai_responses_create_with_tools.rs` - Function calling
- `responses_create_stream.rs` - Real-time streaming *(planned)*
- `openai_responses_create_image_input.rs` - Multimodal processing
- `openai_realtime_input_audio_buffer_append.rs` - Audio streaming

### 🛠️ **Management Examples**
Learn how to manage API resources:
- `openai_responses_update.rs` & `openai_responses_delete.rs` - Response management
- `openai_responses_cancel.rs` - Cancel in-progress operations
- `openai_realtime_session_update.rs` - Session configuration

## Common Patterns

### Error Handling
All examples include proper error handling:
```rust
match result {
    Ok(response) => {
        // Handle success
        println!("Success: {:?}", response);
    },
    Err(e) => {
        // Handle errors gracefully
        eprintln!("Error: {:?}", e);
    }
}
```

### Authentication
Examples use environment-based authentication:
```rust
let secret = api_openai::exposed::Secret::load_from_env("OPENAI_API_KEY")
    .unwrap_or_else(|_| api_openai::exposed::Secret::new("dummy_key".to_string()));
```

### Client Initialization
Standard client setup pattern:
```rust
let env = api_openai::exposed::environment::OpenaiEnvironmentImpl::build(
    secret, None, None, None, None
).expect("Failed to create environment");
let client = Client::build(env).expect("Failed to create client");
```

## Contributing

When adding new examples:

1. **Naming**: Use descriptive names following the pattern `{api}_{action}.rs`
2. **Documentation**: Include comprehensive comments and docstrings
3. **Error Handling**: Always handle errors gracefully
4. **Output**: Provide clear, informative output
5. **Update Index**: Add your example to this README table

## Support

For issues or questions:
- Check the [API documentation](https://platform.openai.com/docs)
- Review existing examples for patterns
- Open an issue in the repository

---

**Note**: All examples require a valid OpenAI API key and may incur API usage costs. Please review OpenAI's pricing before running examples extensively.