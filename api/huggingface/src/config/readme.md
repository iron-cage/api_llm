# config

### Purpose

Dynamic runtime configuration with hot-reload and watcher callbacks.

### Responsibility

| File | Purpose |
|------|---------|
| `mod.rs` | Module root — re-exports `DynamicConfig` type |
| `dynamic.rs` | Hot-reload configuration with watcher registration and rollback |
