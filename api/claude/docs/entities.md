# Docs Module Index

Entity Model Document for `api_claude` documentation collections.

### Scope

- **Purpose**: Define the entity model, mutability schema, and instance-naming conventions for the `docs/` documentation collection.
- **Responsibility**: Maintained by crate maintainers; updated whenever a new entity type is added to or removed from `docs/`.
- **In Scope**: All entity directories directly under `docs/` — their type, mutability, allowed operations, and ordering.
- **Out of Scope**: Entity types in other crates; workspace-level documentation structure; test surface entities in `tests/docs/`.

### Entity Tree

```
Entity               Type        Mut?  Instances  Purpose
──────────────────────────────────────────────────────────────────────────────
api/                 Collection  ✓     1          API contracts and endpoint coverage
feature/             Collection  ✓     1          Optional enterprise feature specifications
invariant/           Collection  ✓     2          Non-negotiable behavioral constraints
operation/           Collection  ✓     1          Operational procedures for client users
pattern/             Collection  ✓     1          Reusable design patterns for contributors
```

### Entities

| Entity Path | Type | Mutable? | Operations | Order | Latent? |
|---|---|---|---|---|---|
| `api/` | Collection | ✓ | Create, Read, Archive | 1 | No |
| `feature/` | Collection | ✓ | Create, Read, Archive | 1 | No |
| `invariant/` | Collection | ✓ | Create, Read, Archive | 1 | No |
| `operation/` | Collection | ✓ | Create, Read, Archive | 1 | No |
| `pattern/` | Collection | ✓ | Create, Read, Archive | 1 | No |

### Instance Naming

All entities use independent sequential NNN format (3-digit zero-padded, e.g. `001_`, `002_`). IDs are permanent once assigned — never reuse after deletion.
