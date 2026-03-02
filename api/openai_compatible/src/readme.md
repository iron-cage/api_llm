# api_openai_compatible — src

| File | Responsibility |
|------|----------------|
| `lib.rs` | Declare crate root, module hierarchy, and feature gates |
| `client.rs` | Provide async HTTP client generic over environment |
| `environment.rs` | Define environment configuration trait and default implementation |
| `error.rs` | Define error types and Result alias |
| `sync_client.rs` | Wrap async client in blocking tokio runtime |
| `components/` | Contain wire types for chat and streaming completions |
