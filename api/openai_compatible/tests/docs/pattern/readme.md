# Pattern Test Surface

### Scope

- **Purpose**: Define test cases that verify behavioral claims from pattern doc instances in `docs/pattern/`.
- **Responsibility**: Each spec file maps one-to-one to a pattern doc instance; all structural and behavioral assertions must have corresponding spec entries.
- **In Scope**: Module organization patterns, generic client design, feature-gated module layers.
- **Out of Scope**: Wire contract scenarios (see `tests/docs/api/`); invariant compliance tests (see `tests/docs/invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [`001_module_organization.md`](001_module_organization.md) | Verify mod_interface layers, Client<E> generic, feature-gated compilation — PT-01..PT-03 (3 scenarios) | ✅ |
