# Pattern Test Surface

### Scope

- **Purpose**: Define test cases that verify structural conformance with mandated code patterns per `docs/pattern/` instances.
- **Responsibility**: Each spec file maps one-to-one to a pattern instance; all structural assertions in a pattern must have corresponding spec entries.
- **In Scope**: Structural conformance scenarios for `docs/pattern/001_module_organization.md` — mod_interface layer pattern, private namespace, submodule structure.
- **Out of Scope**: Source-level unit tests in `tests/inc/`; runtime behavioral tests (see `tests/docs/invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 01 | [`01_module_organization.md`](01_module_organization.md) | Verify mod_interface layer pattern and private namespace conformance across src/ — 6 scenarios | ✅ |
