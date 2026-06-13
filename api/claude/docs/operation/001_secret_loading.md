# Operation: Secret Loading

### Scope

- **Purpose**: Define the procedure for loading an Anthropic API key into the Claude client at startup so all subsequent API calls authenticate correctly.
- **Responsibility**: Documents the three loading methods, their prerequisites, step sequence, success criteria, and recovery procedure.
- **In Scope**: All three loading paths — environment variable, workspace secrets file, and direct construction — including validation and diagnostic steps.
- **Out of Scope**: API call mechanics, retry/failover configuration, and secret rotation scheduling.

### Prerequisites

| Prerequisite | Details |
|--------------|---------|
| Valid API key | Must be a real Anthropic key starting with `sk-ant-api03-`, obtained from `console.anthropic.com/settings/keys` |
| Cargo workspace | Working directory must be inside a Rust workspace with `Cargo.toml` at root (required for workspace method only) |
| Secrets file (workspace method) | `secret/-secrets.sh` at workspace root containing `export ANTHROPIC_API_KEY="sk-ant-api03-..."` |
| Environment variable (env method) | `ANTHROPIC_API_KEY` exported in the current shell or CI environment |

### Procedure Steps

| # | Action | Command / Code | Expected State |
|---|--------|----------------|----------------|
| 1 | Choose loading method | — | Method selected: (A) env var, (B) workspace file, (C) direct |
| 2A | Set environment variable | `export ANTHROPIC_API_KEY="sk-ant-api03-..."` | Variable visible in `env \| grep ANTHROPIC_API_KEY` |
| 2B | Create workspace secrets file | `echo 'export ANTHROPIC_API_KEY="sk-ant-api03-..."' >> secret/-secrets.sh` | File exists at `<workspace_root>/secret/-secrets.sh` |
| 2C | Construct secret directly | `Secret::new( "sk-ant-api03-...".to_string( ) )?` | `Secret` value in scope |
| 3A | Load from environment | `Client::from_env( )?` | `Client` ready; key sourced from `ANTHROPIC_API_KEY` |
| 3B | Load from workspace | `Client::from_workspace( )?` | `Client` ready; key parsed from `secret/-secrets.sh` |
| 3C | Load from direct secret | `Client::new( secret )` | `Client` ready using the provided `Secret` |
| 4 | Validate loaded key format | `assert!( client.secret( ).ANTHROPIC_API_KEY.starts_with( "sk-ant-" ) )` | Key confirmed as real Anthropic credential |
| 5 | Run diagnostic on failure | `println!( "{}", secret_diagnostic_info( ) )` | Diagnostic report printed with available/missing indicators |

### Expected Outcome

After successful execution the caller holds a `Client` value authenticated with a real Anthropic API key. The key begins with `sk-ant-api03-`, has length greater than 30 characters, and equals the value stored in the chosen secret source. All subsequent API calls on the client use this key for HTTP bearer authentication.

### Rollback Procedure

If loading fails or an invalid key was loaded:

1. Remove the incorrect environment variable: `unset ANTHROPIC_API_KEY`
2. If using workspace method, edit `secret/-secrets.sh` to correct the `ANTHROPIC_API_KEY` value
3. Run `secret_diagnostic_info()` to confirm the corrected source is reachable
4. Re-run the loading step (step 3A, 3B, or 3C above)

If no valid API key is available, the client cannot be constructed; all `from_env()` and `from_workspace()` calls return `Err`. No partial state remains — the `Client` value is never created on failure.

### Sources

| File | Relationship |
|------|--------------|
| `src/secret.rs` | `Secret` type — workspace and environment loading logic |
| `src/client.rs` | `Client` constructors — `from_env()`, `from_workspace()`, `new()` |
| `src/environment.rs` | `validate_anthropic_secret()` — checks non-empty key presence across env and workspace sources; `validate_workspace_structure()` — confirms `secret/-secrets.sh` path is reachable; `secret_diagnostic_info()` — returns human-readable credential availability report |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/operation/01_secret_loading.md` | Behavioral spec — 15 scenarios covering all loading paths, error cases, diagnostics, and rollback |
| `tests/inc/workspace_loading_integration_test.rs` | Integration — tests all three loading paths against real workspace |
| `tests/inc/authentication_test.rs` | Unit tests for `Secret` validation and authentication error handling |
| `tests/inc/spec_verification_integration_test.rs` | Integration — verifies API key format against real Anthropic credential format |
