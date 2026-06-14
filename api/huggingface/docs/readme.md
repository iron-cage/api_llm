# Docs

### Purpose

This directory contains technical documentation for the `api_huggingface` crate, organized into design collections following doc.rulebook.md standards. Documentation is structured by design dimension (invariant/, api/, catalog/, feature/, operation/, pattern/, pitfall/) with each entity directory containing a master file (`readme.md`) and NNN-prefixed instance files.

### Responsibility

| Path | Purpose |
|------|---------|
| `readme.md` | Master documentation index with navigation and Complete Entity Coverage |
| `invariant/` | Non-negotiable behavioral constraints — thin client principle, testing standards |
| `api/` | API contract reference — endpoint mapping, operations, error conditions |
| `catalog/` | Enumerated item catalogs — feature flag tables, tier classifications |
| `operation/` | Operational procedures — feature selection, build verification, rollback |
| `pattern/` | Design pattern specifications — module organization (mod_interface) |
| `feature/` | Feature specification collection — capability groups, enterprise feature contracts |
| `pitfall/` | Confirmed design pitfalls — root cause, avoidance, and detection |

### Collections

#### invariant/

Non-negotiable behavioral constraints.

**Master File**: `invariant/readme.md`

**Instances**:
- `001_thin_client_principle.md` — No automatic behaviors; all client actions must be explicit
- `002_testing_standards.md` — No-mock mandate and loud-failure requirement for all integration tests

#### api/

API contract reference — endpoint mapping, operations, and versioning.

**Master File**: `api/readme.md`

**Instances**:
- `001_reference.md` — Complete public API contract: operations, error conditions, compatibility guarantees

#### catalog/

Enumerated item catalogs — feature flag tables and tier classifications.

**Master File**: `catalog/readme.md`

**Instances**:
- `001_features.md` — Cargo feature flag catalog: convenience bundles, Tier 1 core features, Tier 2 enterprise features

#### operation/

Operational procedures for building and verifying `api_huggingface` integrations.

**Master File**: `operation/readme.md`

**Instances**:
- `001_features.md` — Feature selection procedure: trigger, steps, verification, and rollback

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

#### pitfall/

Confirmed design pitfalls discovered through task investigations and code review.

**Master File**: `pitfall/readme.md`

**Instances**:
- `001_url_join_absolute_path.md` — `Url::join` silently strips base URL prefix when path has a leading slash

### Navigation

- Behavioral constraints: see `invariant/`
- API usage and reference: see `api/001_reference.md`
- Feature capability specs: see `feature/001_enterprise_reliability.md`
- Feature flag catalog: see `catalog/001_features.md`
- Feature selection procedure: see `operation/001_features.md`
- Design patterns: see `pattern/`
- Confirmed design pitfalls: see `pitfall/`
- Project overview: see `../readme.md`
- Usage examples: see `../examples/`
