# Examples

### Purpose

This directory contains interactive examples demonstrating the Hugging Face Inference API client capabilities.

### Organization Principles

Examples are organized by use case and complexity:

- **Basic Usage**: Simple inference and embeddings examples
- **Chat Applications**: Conversational AI and interactive chat
- **Specialized Use Cases**: Domain-specific applications (code assistant, translator, QA system, etc.)
- **Advanced Features**: Cached content, multi-turn conversations

### Responsibility

| File | Responsibility |
|------|----------------|
| `chat.rs` | Minimal chat completion example |
| `chat_cached_interactive.rs` | Interactive chat with request caching |
| `hf_interactive_chat.rs` | Terminal-based interactive chat session |
| `huggingface_inference_create.rs` | Basic text generation via inference API |
| `huggingface_embeddings_create.rs` | Text embeddings generation |
| `providers_api_demo.rs` | Pro plan provider discovery and chat completions |
| `huggingface_developer_code_assistant.rs` | Code generation and assistance |
| `huggingface_intelligent_qa_system.rs` | Question-answering system end-to-end |
| `huggingface_multilingual_translator.rs` | Multi-language translation |
| `huggingface_sentiment_content_analyzer.rs` | Sentiment analysis pipeline |
| `huggingface_document_semantic_search.rs` | Document semantic search and retrieval |
| `huggingface_educational_ai_tutor.rs` | Educational tutoring assistant |
| `huggingface_automated_content_generator.rs` | Automated content creation |
| `huggingface_chat_conversational.rs` | Multi-turn conversational chatbot |
| `huggingface_multi_turn_conversation.rs` | Multi-turn dialogue state management |

### Running Examples

All examples require a Hugging Face API key:

```bash
# Set API key
export HUGGINGFACE_API_KEY="hf_..."

# Or use workspace secrets
source ../../secret/-secrets.sh

# Run example
cargo run --example huggingface_inference_create

# Run with all features
cargo run --all-features --example chat
```

### Navigation Guide

- **New users**: Start with `huggingface_inference_create.rs` and `chat.rs`
- **Chat applications**: See `hf_interactive_chat.rs` and `chat_cached_interactive.rs`
- **Domain-specific needs**: Browse specialized applications by use case
- **Advanced patterns**: Review multi-turn conversation and caching examples

Each example includes inline documentation and usage instructions.
