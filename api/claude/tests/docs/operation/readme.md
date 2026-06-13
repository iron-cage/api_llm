# Operation Test Surface

### Scope

- **Purpose**: Define test cases covering all behavioral scenarios in `docs/operation/` doc instances.
- **Responsibility**: Each spec file maps one-to-one to a doc instance; all scenarios in a doc instance must have a corresponding spec entry.
- **In Scope**: Test scenarios for `docs/operation/001_secret_loading.md` — all loading paths, error cases, diagnostic behaviors, and rollback procedure.
- **Out of Scope**: Source-level unit tests in `tests/inc/`; enterprise feature tests not in `docs/operation/`.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 01 | [`01_secret_loading.md`](01_secret_loading.md) | Test spec for secret loading operation — 15 scenarios across all loading paths, error cases, diagnostics, and rollback | ✅ |
