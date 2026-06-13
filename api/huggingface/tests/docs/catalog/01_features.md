# Collection Spec: Features

Spec scenarios for `docs/catalog/001_features.md`. Verifies that the Cargo feature flag catalog is complete, correctly classified, and consistent with `Cargo.toml`.

### CL-01: integration feature exists in the catalog

- **Given:** `docs/catalog/001_features.md` Testing Features table
- **When:** the table is inspected for the `integration` feature entry
- **Then:** `integration` appears with description "Enables real-API integration tests (requires `HUGGINGFACE_API_KEY`)"

### CL-02: full convenience bundle includes basic and enterprise features

- **Given:** `docs/catalog/001_features.md` Convenience Bundles table
- **When:** the `full` row is inspected
- **Then:** it documents that `full` includes `basic` + all enterprise features + `integration`

### CL-03: Tier 1 features do not include enterprise reliability features

- **Given:** `docs/catalog/001_features.md` Tier 1 Core API Features table
- **When:** the Tier 1 table is inspected for enterprise feature entries
- **Then:** `circuit-breaker`, `rate-limiting`, `failover`, `health-checks`, `caching`, `performance-metrics`, `token-counting`, `dynamic-config` are absent from Tier 1; they appear only in the Tier 2 table

### CL-04: enabled bundle provides types without HTTP client

- **Given:** `docs/catalog/001_features.md` Convenience Bundles table
- **When:** the `enabled` row is inspected
- **Then:** it documents that `enabled` provides "core serialization dependencies only" — no HTTP client, no API methods

### CL-05: Classification section describes Tier 1 vs Tier 2 semantics

- **Given:** `docs/catalog/001_features.md` Classification section
- **When:** the section is read
- **Then:** Tier 1 is described as mapping to HuggingFace API endpoints with no runtime state beyond the HTTP client; Tier 2 is described as requiring explicit construction at the call site with runtime state per feature

### CL-06: basic bundle composes exactly inference, embeddings, models, and env-config

- **Given:** `Cargo.toml` feature definitions for `api_huggingface`
- **When:** the `basic` feature entry is inspected
- **Then:** `basic` includes `inference`, `embeddings`, `models`, and `env-config` — and no enterprise or testing features

### CL-07: default feature is an alias for full

- **Given:** `Cargo.toml` feature definitions for `api_huggingface`
- **When:** the `default` feature entry is inspected
- **Then:** `default` lists only `"full"` as its single member, making it a complete alias that activates all features including enterprise and integration
