# components

### Purpose

Shared request/response types used across all API domains.

### Responsibility

| File | Purpose |
|------|---------|
| `mod.rs` | Module root — re-exports all component types |
| `wire_types.rs` | Wire-format types — `ErrorResponse`, `ResponseMetadata`, `TaskType` |
| `embeddings.rs` | Embedding request/response types and options |
| `inference_shared.rs` | Shared inference request/response types |
| `input.rs` | Input parameter types — `InferenceParameters`, `BinaryClassificationInput` |
| `models.rs` | Model identifier constants and `Models` struct |
| `output.rs` | Output response types — `InferenceResponse`, `EmbeddingResponse` |
| `tools.rs` | Tool/function calling types |
