# docs

### Purpose

This directory contains technical documentation for the `api_xai` crate, organized into design collections following doc.rulebook.md standards. Documentation is structured by design dimension (invariant/, pattern/, operation/, etc.) with each collection containing a master file (`readme.md`) and NNN-prefixed instance files.

### Responsibility

| Path | Purpose |
|------|---------|
| `readme.md` | Master documentation index with navigation and Complete Entity Coverage |
| `invariant/` | Non-negotiable behavioral constraints — thin client principle, testing standards |
| `api/` | API contracts and endpoint coverage requirements |
| `feature/` | Optional feature behavior specifications — enterprise reliability modules |
| `operation/` | Operational procedures — secret loading, credential management |
| `pattern/` | Design pattern specifications — module organization (mod_interface) |

### Collections

#### invariant/

Non-negotiable behavioral constraints.

**Master File**: `invariant/readme.md`

**Instances**:
- `001_thin_client_principle.md` — No automatic behaviors; all client actions must be explicit
- `002_testing_standards.md` — No-mock mandate and loud-failure requirement for all integration tests

#### api/

API contracts and endpoint coverage.

**Master File**: `api/readme.md`

**Instances**:
- `001_endpoint_coverage.md` — Required X.AI API endpoint coverage and feature-gating policy

#### feature/

Optional feature behavior specifications.

**Master File**: `feature/readme.md`

**Instances**:
- `001_enterprise_reliability.md` — Enterprise reliability feature set and configuration contract

#### operation/

Operational procedures for `api_xai`.

**Master File**: `operation/readme.md`

**Instances**:
- `001_secret_loading.md` — Load XAI_API_KEY via workspace secrets file or environment variable

#### pattern/

Design pattern specifications.

**Master File**: `pattern/readme.md`

**Instances**:
- `001_module_organization.md` — mod_interface pattern and module structure conventions

### Navigation

- Behavioral constraints: see `invariant/`
- API contracts and endpoint coverage: see `api/`
- Feature behavior specifications: see `feature/`
- Operational procedures (authentication, secret loading): see `operation/`
- Design patterns: see `pattern/`
- Project overview: see `../readme.md`
- Usage examples: see `../examples/`
