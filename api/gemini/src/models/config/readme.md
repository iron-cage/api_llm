# Dynamic Configuration

Runtime configuration updates with hot-reload and history.

## Responsibility Table

| filename | Responsibility |
|----------|---------------|
| mod.rs | DynamicConfig types, builder, and client integration |
| sources.rs | Configuration loading from env, files, and remote |
| hot_reload.rs | File-watch-based automatic configuration reload |
| propagation.rs | Configuration change event propagation |
| rollback.rs | Configuration rollback to previous version |
| versioning.rs | Configuration version tracking and history |
