# Operation: Secret Loading

### Scope

- **Purpose**: Documents the Secret Loading operation — loading provider API keys for development and CI/CD across the api_llm workspace.
- **Responsibility**: Specifies the secret directory layout, file format, and workspace-wide loading procedure.
- **In Scope**: Local development secret file, environment variable loading, CI/CD integration.
- **Out of Scope**: Provider-specific credential formats (see `api/*/docs/operation/` per crate).

### Prerequisites

- A `secret/` directory exists at the workspace root, named exactly `secret` (no dot prefix).
- `secret/secrets.sh.template` exists with the required key names.
- `.cargo/config.toml` sets `WORKSPACE_PATH = { value = ".", relative = true }` so workspace_tools can locate the workspace root.

### Procedure Steps

1. Copy `secret/secrets.sh.template` to `secret/-secrets.sh` if it does not exist. The `-` prefix ensures gitignore via the global `-*` rule.
2. Populate `secret/-secrets.sh` with real API keys in shell export format. Required keys: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GEMINI_API_KEY`, `HUGGINGFACE_API_KEY`, `XAI_API_KEY`, `OLLAMA_BASE_URL`.
3. Source the file in the current shell: `source secret/-secrets.sh`.
4. Run all tests: `cargo nextest run --all-features`.
5. Each crate loads its key via `workspace_tools::workspace()?.load_secret_key("PROVIDER_API_KEY", "-secrets.sh")?`. The loader checks the secret file first and falls back to the environment variable of the same name.
6. For CI/CD, provide keys as environment variables — no secret file is needed in that context.

### Expected Outcome

All integration tests connect to their respective provider APIs using the loaded keys. Tests fail loudly if neither the secret file nor the environment variable provides a required key.

### Rollback Procedure

Remove or rename `secret/-secrets.sh`. All crates return errors from `load_secret_key` on their next run, causing integration tests to fail loudly. To restore, recreate the file with the correct key-value pairs.

### Sources

| File | Relationship |
|------|--------------|
| `secret/secrets.sh.template` | Template showing required key names |
| `secret/-secrets.sh` | Runtime secrets file (gitignored via `-*` prefix) |
| `.cargo/config.toml` | Sets `WORKSPACE_PATH=.` for workspace_tools discovery |

### Tests

| File | Relationship |
|------|--------------|
| `api/*/tests/` | Every crate's integration tests demonstrate loud-failure on missing credentials |
