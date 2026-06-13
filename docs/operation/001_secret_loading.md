# Operation: Secret Loading

### Scope

- **Purpose**: Define the workspace-wide procedure for loading provider API keys in development and CI/CD.
- **Responsibility**: All developers and CI/CD maintainers; secrets must never be committed.
- **In Scope**: Local development secret file, environment variable loading, CI/CD integration.
- **Out of Scope**: Provider-specific credential formats (see `api/*/docs/operation/` per crate).

### Procedure

**Trigger**: Starting development or running integration tests that require real API credentials.

1. Copy `secret/secrets.sh.template` to `secret/-secrets.sh` (the `-` prefix ensures gitignore)
2. Populate `secret/-secrets.sh` with real API keys:
   ```
   export OPENAI_API_KEY="sk-..."
   export ANTHROPIC_API_KEY="sk-ant-..."
   export GEMINI_API_KEY="AIza..."
   export HUGGINGFACE_API_KEY="hf_..."
   export XAI_API_KEY="xai-..."
   export OLLAMA_BASE_URL="http://localhost:11434"
   ```
3. Source the file: `source secret/-secrets.sh`
4. Run tests: `cargo nextest run --all-features`

### Secret File Policy

- `secret/secrets.sh.template` — committed; contains key names only, no values
- `secret/-secrets.sh` — gitignored via `-*` global rule; contains real credentials
- Individual credential files inside `secret/` must use `-*` prefix (gitignored)
- CI/CD provides credentials via environment variables — no secret file needed

### Loading in Code

Each crate loads its secret via `workspace_tools`:

```rust
workspace_tools::workspace()?.load_secret_key( "PROVIDER_API_KEY", "-secrets.sh" )
```

This checks environment variables first, then falls back to `secret/-secrets.sh` in the workspace root. Integration tests `panic!` loudly if neither source has the key.

### Sources

| File | Relationship |
|------|--------------|
| `secret/secrets.sh.template` | Template showing required key names |
| `secret/-secrets.sh` | Runtime secrets file (gitignored) |
| `.cargo/config.toml` | Sets `WORKSPACE_PATH=.` for workspace_tools discovery |

### Tests

| File | Relationship |
|------|--------------|
| `api/*/tests/` | Every crate's integration tests demonstrate loud-failure on missing credentials |
