# API Types

Core data types for all Gemini API domains.

## Responsibility Table

| filename | Responsibility |
|----------|---------------|
| mod.rs | Module re-exports for all API types |
| core.rs | Fundamental shared types (Role, Part, Content) |
| content.rs | Content and multimodal part types |
| generation.rs | GenerationConfig and safety settings |
| embedding.rs | Embedding vector types |
| chat.rs | Chat session and turn types |
| token.rs | Token counting request and response types |
| function.rs | Function calling declaration and response types |
| code_execution.rs | Code execution result types |
| file.rs | File metadata and reference types |
| streaming.rs | Streaming response chunk types |
| search.rs | Grounding and search retrieval types |
| cache.rs | Cached content types |
| tuning.rs | Model tuning dataset and job types |
