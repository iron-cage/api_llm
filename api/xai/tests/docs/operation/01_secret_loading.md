# Operation Spec: Secret Loading
**Source:** `../../docs/operation/001_secret_loading.md`

## Test Cases

### OP-01: API key loaded from environment variable

- **Given:** `XAI_API_KEY` environment variable set to a valid key starting with `xai-`
- **When:** `Secret::load_with_fallbacks("XAI_API_KEY")` is called
- **Then:** A `Secret` is returned containing the key from the environment variable

### OP-02: Missing key with no secrets file returns descriptive error

- **Given:** No `XAI_API_KEY` environment variable and no `secret/-secrets.sh` workspace file
- **When:** `Secret::load_with_fallbacks("XAI_API_KEY")` is called
- **Then:** The call fails with an error message identifying `XAI_API_KEY` as the missing credential — not a generic "key not found"

### OP-03: Key format validation rejects invalid prefix

- **Given:** A key string that does not start with `xai-` (e.g., `sk-abc123`)
- **When:** The key is used to construct a `Secret`
- **Then:** Construction fails with an error indicating the expected `xai-` prefix format

### OP-04: Fallback chain tries workspace file first then env var

- **Given:** Both `secret/-secrets.sh` (containing one key) and `XAI_API_KEY` env var (containing a different key) are present
- **When:** `Secret::load_with_fallbacks("XAI_API_KEY")` is called
- **Then:** The key from the workspace secrets file is used, not the environment variable

### OP-05: Environment constructed with correct base URL

- **Given:** A valid `Secret` loaded successfully
- **When:** `Environment::build(secret, base_url, timeout)?` is called with default settings
- **Then:** The environment's base URL is `https://api.x.ai/v1`
