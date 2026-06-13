# Operation Spec: Feature Flag Management

Spec scenarios for `docs/operation/001_features.md`. Verifies that Cargo feature flags behave as documented.

## OP-01: Streaming unavailable without inference-streaming feature

**Given:** `api_huggingface` compiled without the `inference-streaming` feature
**When:** user code references `client.inference().create_stream()`
**Then:** compilation fails with "method not found" or equivalent; the `create_stream` symbol does not exist in the compiled output

## OP-02: Similarity utility unavailable without embeddings-similarity feature

**Given:** `api_huggingface` compiled without the `embeddings-similarity` feature
**When:** user code references `client.embeddings().similarity()`
**Then:** compilation fails; the `similarity` method is not available in the compiled output

## OP-03: Sync API unavailable without sync feature

**Given:** `api_huggingface` compiled without the `sync` feature
**When:** user code references `SyncClient`
**Then:** compilation fails; `SyncClient` is not available in the compiled output

## OP-04: full feature activates all documented capabilities

**Given:** `api_huggingface` compiled with `--features full`
**When:** all tier-1 and tier-2 feature modules are referenced
**Then:** all capabilities listed in the Implemented Features table compile and link without errors

## OP-05: Minimal build compiles without optional features

**Given:** `api_huggingface` compiled with `--no-default-features --features enabled`
**When:** `cargo build` runs
**Then:** compilation succeeds with 0 errors and 0 warnings; only core types are available

## OP-06: integration feature enables real API test execution

**Given:** `cargo nextest run --all-features` with valid `HUGGINGFACE_API_KEY` in environment
**When:** integration-gated tests execute
**Then:** integration tests run (not skipped); they call real HuggingFace endpoints and validate actual API responses
