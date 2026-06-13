# Docs

### Purpose

This directory contains technical documentation for the `api_huggingface` crate, organized into design collections following doc.rulebook.md standards. Documentation is structured by design dimension (invariant/, api/, feature/, operation/, pattern/) with each collection containing a master file (`readme.md`) and NNN-prefixed instance files.

### Responsibility

| Path | Purpose |
|------|---------|
| `readme.md` | Master documentation index with navigation and Complete Entity Coverage |
| `invariant/` | Non-negotiable behavioral constraints — thin client principle, testing standards |
| `api/` | API design collection - comprehensive API reference, endpoints, usage patterns |
| `operation/` | Operational procedures collection - feature management, cargo features, status tracking |
| `pattern/` | Design pattern specifications — module organization (mod_interface) |
| `feature/` | Feature specification collection — capability groups, enterprise feature contracts |
| `entities/` | Module index of all active doc entities and instances |

### Collections

#### invariant/

Non-negotiable behavioral constraints.

**Master File**: `invariant/readme.md`

**Instances**:
- `001_thin_client_principle.md` — No automatic behaviors; all client actions must be explicit
- `002_testing_standards.md` — No-mock mandate and loud-failure requirement for all integration tests

#### api/

API design, endpoints, and usage patterns.

**Master File**: `api/readme.md`

**Instances**:
- `001_reference.md` — Comprehensive API reference covering client operations, models, environment config, error handling

#### operation/

Operational procedures for feature management and configuration.

**Master File**: `operation/readme.md`

**Instances**:
- `001_features.md` — Complete feature tables, cargo features documentation, feature tier classification

#### pattern/

Design pattern specifications.

**Master File**: `pattern/readme.md`

**Instances**:
- `001_module_organization.md` — mod_interface pattern and module structure conventions

#### feature/

Enterprise reliability and capability feature specifications.

**Master File**: `feature/readme.md`

**Instances**:
- `001_enterprise_reliability.md` — Enterprise reliability feature group — circuit breaker, rate limiting, failover, caching, metrics

### Navigation

- Behavioral constraints: see `invariant/`
- API usage and reference: see `api/001_reference.md`
- Feature capability specs: see `feature/001_enterprise_reliability.md`
- Feature availability and cargo flags: see `operation/001_features.md`
- Design patterns: see `pattern/`
- Doc entity index: see `entities/`
- Project overview: see `../readme.md`
- Usage examples: see `../examples/`
