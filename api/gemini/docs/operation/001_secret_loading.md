# Operation: Secret Loading

### Scope

- **Purpose**: Documents the Secret Loading operation — loading the Gemini API key for development and test environments.
- **Responsibility**: Specifies the secret file setup, environment variable fallback, and workspace_tools loading call for api_gemini.
- **In Scope**: Local development secret file, environment variable loading, integration test setup.
- **Out of Scope**: Workspace-wide secret policy (see `docs/operation/001_secret_loading.md` at workspace root).

### Prerequisites

- The workspace root contains a `secret/` directory named exactly `secret` (no dot prefix).
- `secret/secrets.sh.template` exists as a reference for key names.
- `.cargo/config.toml` sets `WORKSPACE_PATH = { value = ".", relative = true }` so workspace_tools can locate the workspace root.

### Procedure Steps

1. Copy `secret/secrets.sh.template` to `secret/-secrets.sh` if it does not exist. The `-` prefix ensures the file is gitignored by the global `-*` rule.
2. Add `GEMINI_API_KEY` to `secret/-secrets.sh` in shell export format: `export GEMINI_API_KEY="AIza..."`.
3. Source the file in the current shell before running tests: `source secret/-secrets.sh`.
4. Run integration tests: `cargo nextest run --all-features`.
5. In source code, load the key via `workspace_tools::workspace()?.load_secret_key("GEMINI_API_KEY", "-secrets.sh")?`. The loader checks the secret file first and falls back to the `GEMINI_API_KEY` environment variable if the key is absent.
6. For CI/CD, provide `GEMINI_API_KEY` as an environment variable — no secret file is needed in that context.

### Expected Outcome

Integration tests connect to the Gemini API using the loaded key and complete successfully. Tests fail loudly if neither the secret file nor the environment variable provides the key, per the Testing Standards invariant.

### Rollback Procedure

Remove or rename `secret/-secrets.sh`. The next `load_secret_key` call returns an error, causing tests to fail loudly. To restore, recreate the file with the correct key.

### Sources

| File | Relationship |
|------|--------------|
| `secret/secrets.sh.template` | Template showing required key names |
| `secret/-secrets.sh` | Runtime secrets file (gitignored via `-*` prefix) |
| `.cargo/config.toml` | Sets `WORKSPACE_PATH=.` for workspace_tools discovery |

### Tests

| File | Relationship |
|------|--------------|
| `tests/api_key_failure_tests.rs` | Tests missing/invalid API key behavior — loud failure on absent credentials |
| `tests/integration_tests.rs` | Uses loaded secret for real Gemini API calls |
