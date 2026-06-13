# Operation: Feature Selection

### Scope

- **Purpose**: Document the step-by-step procedure for selecting, combining, and verifying Cargo feature flags when integrating or building `api_huggingface`.
- **Responsibility**: Cargo feature selection procedure — identification, verification, and rollback steps.
- **In Scope**: Feature selection steps, combination rules, verification commands, rollback procedure.
- **Out of Scope**: Feature flag catalog and tier classification (see `catalog/001_features.md`), source-level implementation, API method signatures.

### Trigger

Adding, removing, or changing any Cargo feature flag in `Cargo.toml`; or selecting features for a new integration of `api_huggingface`.

### Prerequisites

| Condition | Verification |
|-----------|--------------|
| Cargo workspace with `api_huggingface` dependency configured | `Cargo.toml` or `Cargo.lock` present in workspace |
| Feature catalog reviewed — Tier 1 vs. Tier 2 distinction understood | `catalog/001_features.md` read and available |
| HuggingFace API key available if `integration` feature is selected | `HUGGINGFACE_API_KEY` set in environment or `-secrets.sh` |

### Procedure Steps

1. Identify required capability tier: Tier 1 (core APIs) or Tier 2 (enterprise features) — see `catalog/001_features.md` for the full catalog.
2. Select the minimum feature set: use `enabled` for types-only, `basic` for core API access, `full` for all features.
3. Add optional capability features as needed from the catalog.
4. Verify `integration` feature is present if running integration tests.
5. Build with `RUSTFLAGS="-D warnings" cargo build --features <selected>` and confirm zero warnings.
6. Run `cargo nextest run --features <selected>` to verify tests pass for the selected feature combination.

### Expected Outcome

Build succeeds with zero warnings. All tests for the selected feature set pass. No integration test runs without real API credentials.

### Rollback Procedure

Remove the newly added feature flag from `Cargo.toml`. Revert any capability-specific code guarded by the removed feature. Re-run build and tests to confirm clean state.

### Collections

| File | Relationship |
|------|--------------|
| `catalog/001_features.md` | Feature flag catalog referenced in Steps 1–3 |

### Features

| File | Relationship |
|------|--------------|
| `feature/001_enterprise_reliability.md` | Enterprise reliability features selected via this procedure |

### Invariants

| File | Relationship |
|------|--------------|
| `invariant/001_thin_client_principle.md` | Enterprise features must be opt-in via feature flags — never automatic |
| `invariant/002_testing_standards.md` | `integration` feature required for all real-API tests |

### Sources

| File | Relationship |
|------|--------------|
| `Cargo.toml` | Canonical feature flag definitions |
| `src/lib.rs` | Feature-gated `pub mod` declarations |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/operation/01_features.md` | GWT spec scenarios for this procedure |
