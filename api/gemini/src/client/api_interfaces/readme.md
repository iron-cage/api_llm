# Client API Interfaces

API method implementations organized by endpoint family.

## Responsibility Table

| filename | Responsibility |
|----------|---------------|
| mod.rs | Module re-exports for all API interfaces |
| models_api.rs | Models list and get endpoint implementations |
| chat_api.rs | Chat conversation endpoint implementations |
| conversation_builder.rs | Builder for multi-turn conversation requests |
| files_api.rs | File upload and management endpoint implementations |
| cached_content_api.rs | Cached content endpoint implementations |
| tuned_models_api.rs | Tuned model management endpoint implementations |
