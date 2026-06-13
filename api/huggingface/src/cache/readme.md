# cache

### Purpose

LRU response cache with TTL and eviction statistics.

### Responsibility

| File | Purpose |
|------|---------|
| `mod.rs` | Module root — re-exports `Cache` type and configuration |
| `implementation.rs` | LRU cache implementation with TTL, eviction, and statistics |
