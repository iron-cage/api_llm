# Implement WebSocket Send and Planned Features

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** ❓ (Unverified)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** src/models/
- **Validated By:** N/A
- **Validation Date:** N/A

## Goal

Three `// xxx` markers in `src/models/websocket_streaming_optimized.rs` identify unimplemented WebSocket features. The most critical is `send_message()` which returns `Ok(())` without sending anything — it is currently a mock. Implement the actual WebSocket send logic and the two compression/serialization features, or delete the dead stub if the feature is not needed. Observable: `grep "// xxx" src/models/websocket_streaming_optimized.rs | wc -l` → 0.

## In Scope

- `src/models/websocket_streaming_optimized.rs` — all 3 `// xxx` markers
- `src/models/media_optimization/upload.rs:115` — thumbnail caching marker

## Out of Scope

- Core API changes
- New public API surface
- Tests

## Markers to Resolve

| Location | Marker | Action |
|----------|--------|--------|
| `src/models/websocket_streaming_optimized.rs:73` | Implement compression | Add compression to WebSocket frame sending |
| `src/models/websocket_streaming_optimized.rs:80` | Implement MessagePack, fallback to JSON | Implement MessagePack serialization or remove the branch |
| `src/models/websocket_streaming_optimized.rs:683` | Actual WebSocket sending implementation | Replace `Ok(())` stub with real `sink.send(message)` call using tokio-tungstenite |
| `src/models/media_optimization/upload.rs:115` | Cache thumbnails | Implement thumbnail caching or remove the field |

## History

- **2026-06-13** `CREATED` — L1 hygiene audit identified 3 untracked `// xxx` markers in websocket_streaming_optimized.rs (including a mock send_message) and 1 in upload.rs. The `send_message()` mock is also covered by PRB-003 in the L1 audit report — fix there supersedes the `unimplemented!()` replacement and should produce a real implementation.
