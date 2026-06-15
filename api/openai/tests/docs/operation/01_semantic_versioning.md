# Operation Spec: Semantic Versioning

**Source:** [`docs/operation/001_semantic_versioning.md`](../../../docs/operation/001_semantic_versioning.md)

### Overview Table

| ID | Name | Category | Status |
|----|------|----------|--------|
| OP-01 | Cargo.toml version field parses as valid semver triple | version-format | ✅ |
| OP-02 | Version string has exactly three numeric components | version-components | ✅ |
| OP-03 | Crate compiles with zero warnings at RUSTFLAGS="-D warnings" | clean-compilation | ✅ |
| OP-04 | All public types have documentation | doc-coverage | ✅ |

---

### OP-01: Cargo.toml version field parses as valid semver triple

- **Given:** The `version` field in `api/openai/Cargo.toml`
- **When:** The version string is parsed with a semver parser
- **Then:** Parsing succeeds; the result has `major`, `minor`, and `patch` components; all three are non-negative integers

---

### OP-02: Version string has exactly three numeric components

- **Given:** The `version` field from `api/openai/Cargo.toml`
- **When:** The string is split on `.`
- **Then:** Exactly three components result; each component parses as a non-negative integer; no pre-release or build metadata suffixes are present

---

### OP-03: Crate compiles with zero warnings at RUSTFLAGS="-D warnings"

- **Given:** The api_openai crate with all features enabled
- **When:** `RUSTFLAGS="-D warnings" cargo check --all-features` is executed
- **Then:** The command exits with status 0; no compiler warnings are emitted; denied warnings do not cause compilation failure

---

### OP-04: All public types have documentation

- **Given:** The api_openai crate with all features enabled
- **When:** `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` is executed
- **Then:** The command exits with status 0; all public types, functions, and modules have documentation comments; missing-docs warnings are treated as errors
