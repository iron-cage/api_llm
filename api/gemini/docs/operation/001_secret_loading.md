# Operation: Secret Loading

### Scope

- **Purpose**: Define the procedure for loading the Gemini API key for api_gemini development and tests.
- **Responsibility**: api_gemini developers and CI/CD; credential values must never be committed.
- **In Scope**: Local development secret file, environment variable loading, integration test setup.
- **Out of Scope**: Workspace-wide secret policy (see `docs/operation/001_secret_loading.md` at workspace root).

### Procedure

**Trigger**: Starting development or running integration tests that require a real Gemini API key.

1. Ensure `secret/-secrets.sh` exists at the workspace root (copy from `secret/secrets.sh.template`)
2. Set `GEMINI_API_KEY` in `secret/-secrets.sh`:
   ```
   export GEMINI_API_KEY="AIza..."
   ```
3. Source the file: `source secret/-secrets.sh`
4. Run integration tests: `cargo nextest run --all-features`

### Loading in Code

api_gemini loads the API key via `workspace_tools`:

```rust
use workspace_tools as workspace;
let ws = workspace::workspace().expect( "Failed to resolve workspace" );
let api_key = ws.load_secret_key( "GEMINI_API_KEY", "-secrets.sh" )
.or_else( |_| std::env::var( "GEMINI_API_KEY" ) )
.expect( "❌ GEMINI_API_KEY not found in workspace secrets or environment" );
```

### CI/CD Integration

Provide `GEMINI_API_KEY` as an environment variable — no secret file needed in CI.

### Secret File Policy

- `secret/secrets.sh.template` — committed; key names only, no values
- `secret/-secrets.sh` — gitignored; real credentials; `-*` prefix enforces exclusion
