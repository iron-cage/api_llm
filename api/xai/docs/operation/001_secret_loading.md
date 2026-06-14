# Operation: Secret Loading

### Scope

- **Purpose**: Define the procedure for loading an X.AI API key into the `api_xai` client at startup so all subsequent API calls authenticate correctly.
- **Responsibility**: Documents the loading methods, prerequisites, step sequence, success criteria, and recovery procedure.
- **In Scope**: All loading paths — environment variable, workspace secrets file, and direct construction — including validation.
- **Out of Scope**: API call mechanics, retry/failover configuration, and secret rotation scheduling.

### Prerequisites

| Prerequisite | Details |
|--------------|---------|
| Valid API key | Real X.AI key starting with `xai-`, obtained from the X.AI console |
| Cargo workspace | Working directory inside a Rust workspace with `Cargo.toml` at root (workspace method only) |
| Secrets file (workspace method) | `secret/-secrets.sh` at workspace root containing `export XAI_API_KEY="xai-..."` |
| Environment variable (env method) | `XAI_API_KEY` exported in the current shell or CI environment |

### Procedure Steps

| # | Action | Code | Expected State |
|---|--------|------|----------------|
| 1 | Choose loading method | — | Method selected: (A) workspace file, (B) env var, (C) direct |
| 2A | Create workspace secrets file | `echo 'export XAI_API_KEY="xai-..."' >> secret/-secrets.sh` | File exists at `<workspace_root>/secret/-secrets.sh` |
| 2B | Set environment variable | `export XAI_API_KEY="xai-..."` | Variable visible in current shell |
| 2C | Construct secret directly | `Secret::new( "xai-...".to_string() )?` | `Secret` value validated and in scope |
| 3 | Load via fallback chain | `Secret::load_with_fallbacks( "XAI_API_KEY" )` | Tries workspace file first, falls back to env var |
| 4 | Construct environment | `Environment::build( secret, base_url, timeout )?` | `Environment` with base URL `https://api.x.ai/v1` and timeout |
| 5 | Construct client | `Client::build( env )?` | `Client` ready; all API calls authenticated |

### Expected Outcome

After successful execution the caller holds a `Client< Environment >` value authenticated with a real X.AI API key. The key begins with `xai-`. All subsequent API calls on the client send `Authorization: Bearer <key>` headers.

### Rollback Procedure

If loading fails: (1) verify key format starts with `xai-`, (2) check `secret/-secrets.sh` uses `export KEY="value"` format (not `KEY=value`), (3) confirm environment variable is exported (not just set), (4) use `Secret::load_with_fallbacks("XAI_API_KEY")` — it attempts workspace file first, then env var, returning a diagnostic error if both fail.

### Sources

| File | Relationship |
|------|--------------|
| `src/secret.rs` | `Secret` struct, `load_with_fallbacks()`, format validation (xai- prefix check) |
| `src/environment.rs` | `Environment` struct — wraps `Secret` with base URL and timeout |
| `src/client.rs` | `Client::build(env)` — client construction from environment |

### Tests

| File | Relationship |
|------|--------------|
| `tests/secret_tests.rs` | Tests Secret loading paths — workspace file, env var, and format validation |
