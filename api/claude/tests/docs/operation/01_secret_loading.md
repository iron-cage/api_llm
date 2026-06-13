# Operation Spec: Secret Loading

**Source:** [`docs/operation/001_secret_loading.md`](../../docs/operation/001_secret_loading.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| OP-01 | env var success path | success path | ✅ |
| OP-02 | env var absent error | error path | ✅ |
| OP-03 | workspace success path | success path | ✅ |
| OP-04 | workspace secrets file absent | error path | ✅ |
| OP-05 | secrets file missing key | error path | ✅ |
| OP-06 | direct construction valid key | success path | ✅ |
| OP-07 | direct construction invalid key | error path | ✅ |
| OP-08 | key format invariant | format invariant | ✅ |
| OP-09 | env and workspace keys identical | consistency | ✅ |
| OP-10 | validate secret key present | diagnostic: success | ✅ |
| OP-11 | validate secret key absent | diagnostic: error | ✅ |
| OP-12 | diagnostic info always callable | diagnostic: smoke | ✅ |
| OP-13 | validate workspace structure valid | diagnostic: success | ✅ |
| OP-14 | rollback: unset env var then reload succeeds | rollback | ✅ |
| OP-15 | from_workspace() fails when no Cargo workspace reachable | error path | ✅ |

---

### OP-01: env var success path

- **Given:** `ANTHROPIC_API_KEY` is exported in the process environment with a value that starts with `sk-ant-api03-` and is at least 30 characters long
- **When:** `Client::from_env()` is called
- **Then:** Returns `Ok(Client)`; the client's secret holds the exact key string that was set in the env var

---

### OP-02: env var absent error

- **Given:** `ANTHROPIC_API_KEY` is not set in the process environment (unset or empty)
- **When:** `Client::from_env()` is called
- **Then:** Returns `Err`; the error is not a panic; the error message references `ANTHROPIC_API_KEY` or indicates a missing credential source

---

### OP-03: workspace success path

- **Given:** The current process is inside a Rust workspace; `secret/-secrets.sh` exists at the workspace root and contains `export ANTHROPIC_API_KEY="sk-ant-api03-..."` with a valid key
- **When:** `Client::from_workspace()` is called
- **Then:** Returns `Ok(Client)`; the client's secret holds a key that starts with `sk-ant-`

---

### OP-04: workspace secrets file absent

- **Given:** The current process is inside a valid Rust workspace (Cargo.toml reachable), but `secret/-secrets.sh` does not exist at the workspace root
- **When:** `Client::from_workspace()` is called
- **Then:** Returns `Err`; no panic; the error is descriptive (references secrets file or secret loading failure)

---

### OP-05: secrets file missing key

- **Given:** `secret/-secrets.sh` exists at the workspace root but does not contain the `ANTHROPIC_API_KEY` variable (file exists, key absent)
- **When:** `Client::from_workspace()` is called
- **Then:** Returns `Err`; no panic; the error indicates that the key could not be found in the secrets file

---

### OP-06: direct construction valid key

- **Given:** A key string that starts with `sk-ant-api03-` and has length ≥ 30 characters
- **When:** `Secret::new(key_string)` succeeds and `Client::new(secret)` is called
- **Then:** The resulting `Client` holds the exact key; no error is returned; the client is usable for subsequent API calls

---

### OP-07: direct construction invalid key

- **Given:** A key string that is clearly invalid (e.g., `"bad"`, `""`, or a string that does not begin with `sk-ant-`)
- **When:** `Secret::new(invalid_key)` is called (or equivalent validation step during client construction)
- **Then:** The operation returns `Err(...)` with a descriptive message; no panic occurs; it never silently accepts the invalid key and returns a ready `Client`

---

### OP-08: key format invariant

- **Given:** A `Client` was successfully constructed via either `from_env()` or `from_workspace()` in a test environment with real credentials
- **When:** The client's internal secret key is inspected
- **Then:** The key string starts with `sk-ant-`; its length is greater than 30 characters; both conditions hold simultaneously

---

### OP-09: env and workspace keys identical

- **Given:** Both `ANTHROPIC_API_KEY` in the process environment and `secret/-secrets.sh` at the workspace root contain the same valid API key
- **When:** `Client::from_env()` and `Client::from_workspace()` are both called in the same test
- **Then:** The key strings held by both clients are byte-for-byte identical

---

### OP-10: validate secret key present

- **Given:** At least one secret source (env var or workspace secrets file) contains a non-empty value for `ANTHROPIC_API_KEY` (the value need not be a valid Anthropic key format — `validate_anthropic_secret()` only checks that the value is non-empty)
- **When:** `validate_anthropic_secret()` is called
- **Then:** Returns `Ok(source_string)` where `source_string` is a non-empty string identifying which source provided the key

---

### OP-11: validate secret key absent

- **Given:** Neither `ANTHROPIC_API_KEY` in the environment nor `secret/-secrets.sh` in the workspace contains a valid API key
- **When:** `validate_anthropic_secret()` is called
- **Then:** Returns `Err`; the error contains a descriptive message; it does not panic

---

### OP-12: diagnostic info always callable

- **Given:** Any state — API key may or may not be available; workspace may or may not be valid
- **When:** `secret_diagnostic_info()` is called
- **Then:** Returns a non-empty `String`; it never panics regardless of environment state; the returned string contains at least one human-readable indicator about credential availability

---

### OP-13: validate workspace structure valid

- **Given:** The current process is inside a valid Rust workspace and `secret/-secrets.sh` exists at the workspace root
- **When:** `validate_workspace_structure()` is called
- **Then:** Returns `Ok(path)` where `path` points to the `secret/-secrets.sh` file; the path is non-empty and resolvable

---

### OP-14: rollback: unset env var then reload succeeds

- **Given:** `ANTHROPIC_API_KEY` was set to an incorrect value in the process environment; `secret/-secrets.sh` at the workspace root contains a valid non-empty key
- **When:** The env var is unset and `Client::from_workspace()` is called as part of the rollback procedure
- **Then:** Returns `Ok(Client)` using the workspace secret file as the source; no residual state from the prior incorrect env var affects the result

---

### OP-15: from_workspace() fails when no Cargo workspace reachable

- **Given:** The current working directory is not inside any Rust workspace (no `Cargo.toml` with `[workspace]` reachable by walking up the directory tree)
- **When:** `Client::from_workspace()` is called
- **Then:** Returns `Err`; no panic; the error message references the workspace root detection failure or the absence of the secrets file; no partial `Client` is constructed
