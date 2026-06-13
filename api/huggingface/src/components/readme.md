# components

### Purpose

Shared request/response types used across all API domains.

### Responsibility

| File | Purpose |
|------|---------|
| `mod.rs` | Module root — re-exports all component types |
| `common.rs` | Common shared types used across multiple API domains |
| `embeddings.rs` | Embedding request/response types and options |
| `inference_shared.rs` | Shared inference request/response types |
| `input.rs` | Input parameter types — `InferenceParameters` |
| `models.rs` | Model identifier constants and `Models` struct |
| `output.rs` | Output response types — `InferenceResponse`, `EmbeddingResponse` |
| `tools.rs` | Tool/function calling types |
