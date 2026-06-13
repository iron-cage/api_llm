# Anthropic API Crate Examples

This directory contains practical examples demonstrating the most useful use cases of the Anthropic API crate. All examples use real API calls - no mocking.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `readme.md` | Directory index and quick-start guide |
| `claude_api_basic.rs` | Minimal API call: single message and response |
| `claude_api_interactive.rs` | Interactive multi-turn conversation loop |
| `claude_chat_cached_interactive.rs` | Interactive chat with prompt caching enabled |
| `claude_chat_streaming.rs` | Real-time streaming response display |
| `claude_code_review.rs` | Automated code review via the messages API |
| `claude_content_generation.rs` | Long-form content generation with structured output |
| `claude_function_calling.rs` | Tool use and function calling patterns |
| `claude_multi_turn_conversation.rs` | Stateful multi-turn conversation management |
| `claude_vision_analysis.rs` | Image input and visual analysis |


## 🚀 Quick Start

### Prerequisites

1. **API Key Setup** (choose one):
   ```bash
   # Option 1: Environment variable
   export ANTHROPIC_API_KEY="sk-ant-your-key-here"
   
   # Option 2: Workspace secrets (recommended)
   mkdir -p ../../secret
   echo 'ANTHROPIC_API_KEY="sk-ant-your-key-here"' > ../../secret/-secrets.sh
   ```

2. **Features**: Examples require the `integration` feature flag.

## 📚 Examples

### 1. Content Generation (`content_generation.rs`)
**Use Case**: AI-powered writing assistant for blogs, documentation, marketing copy.

```bash
cargo run --example content_generation --features integration
```

**Perfect for**: 
- Technical blog posts
- Documentation generation  
- Marketing content
- Creative writing
- Product descriptions

### 2. Code Review (`code_review.rs`)  
**Use Case**: Intelligent code analysis and review with specific improvement suggestions.

```bash
cargo run --example code_review --features integration
```

**Perfect for**:
- Automated code reviews
- Security audits
- Performance optimization
- Learning best practices
- Refactoring guidance

### 3. Function Calling (`anthropic_function_calling.rs`)
**Use Case**: Advanced tool integration for complex workflows and API interactions.

```bash
cargo run --example anthropic_function_calling --features integration  
```

**Perfect for**:
- API integrations
- Database queries
- Multi-step calculations
- Workflow automation
- External service calls

### 4. Vision Analysis (`vision_analysis.rs`)
**Use Case**: Multi-modal document and image analysis capabilities.

```bash
cargo run --example vision_analysis --features "integration,vision"
```

**Perfect for**:
- OCR and document parsing
- UI/UX analysis
- Chart interpretation  
- Image captioning
- Accessibility descriptions

### 5. Streaming Chat (`anthropic_streaming_chat.rs`)
**Use Case**: Real-time conversational AI with interactive chat interface.

```bash
cargo run --example anthropic_streaming_chat --features integration
```

**Perfect for**:
- Interactive chatbots
- CLI assistants
- Customer support
- Educational tools
- Coding helpers

## 🔧 Technical Features Demonstrated

- **🔐 Secure Authentication**: Workspace secret management with environment fallback
- **🛡️ Error Handling**: Comprehensive error handling and recovery
- **🚀 Async Operations**: Non-blocking API calls with tokio
- **🎯 Type Safety**: Full Rust type system integration
- **📊 Usage Tracking**: Token usage and performance monitoring
- **🛠️ Tool Integration**: Function calling capabilities
- **👁️ Vision Support**: Multi-modal image analysis
- **💬 Streaming**: Real-time conversation capabilities

## 🧪 Testing

All examples include integration tests and can be run with:

```bash
# Run specific example
cargo run --example EXAMPLE_NAME --features integration

# Test all examples compile
cargo check --examples --features "integration,vision"

# Run integration tests
cargo test --features integration
```

## 📊 Performance Notes

- **Haiku Model**: Fast responses (~1-2s) for chat and simple tasks
- **Sonnet Model**: Detailed analysis (~3-5s) for complex reasoning
- **Token Limits**: Examples use 500-1200 token limits for practical responses
- **Rate Limits**: Claude has usage limits - examples include error handling

## 🔍 Troubleshooting

**Missing API Key**:
```
Must have valid ANTHROPIC_API_KEY in ../../secret/-secrets.sh or environment
```
→ Set up your API key using one of the methods above.

**Feature Errors**:
```
error: target `vision_analysis` requires `vision` feature
```
→ Add the required feature: `--features "integration,vision"`

**Network Issues**:
All examples include proper error handling for network failures and API errors.

## 🎯 Next Steps

1. **Customize Examples**: Modify prompts and parameters for your use case
2. **Add Tools**: Extend function calling with your own tools
3. **Error Handling**: Adapt error handling for production use
4. **Performance**: Adjust models and limits for your requirements
5. **Integration**: Build these patterns into your applications

## 📖 Documentation

- [Anthropic API Docs](https://docs.anthropic.com/)
- [Claude Models Guide](https://docs.anthropic.com/claude/docs/models-overview)  
- [Function Calling](https://docs.anthropic.com/claude/docs/functions-external-tools)
- [Vision Capabilities](https://docs.anthropic.com/claude/docs/vision)