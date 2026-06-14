# docs

### Purpose

This directory contains supplementary technical documentation for the Claude API client, organized into design collections following doc.rulebook.md standards.

Documentation is structured by design dimensions (operation/, pattern/, protocol/, etc.) with each collection containing:
- Master file (`readme.md`) with Collection Scope and Responsibility Table
- Instance files with NNN identifiers (001, 002, etc.)

### Responsibility

This table documents all entities in the docs/ directory, ensuring Complete Entity Coverage.

| Path | Purpose |
|------|---------|
| `readme.md` | Master documentation file with navigation and Complete Entity Coverage |
| `api/` | API contracts and endpoint coverage requirements |
| `feature/` | Optional feature behavior specifications |
| `invariant/` | Non-negotiable behavioral constraints |
| `operation/` | Operational procedures collection - authentication, configuration, secret management |
| `pattern/` | Design pattern specifications |

### Collections

#### api/

API contracts and endpoint coverage for Claude API client.

**Master File**: `api/readme.md`

**Instances**:
- `001_endpoint_coverage.md` - Required API endpoint coverage and feature-gating policy

#### feature/

Optional feature behavior specifications.

**Master File**: `feature/readme.md`

**Instances**:
- `001_enterprise_reliability.md` - Enterprise reliability feature set and configuration contract

#### invariant/

Non-negotiable behavioral constraints.

**Master File**: `invariant/readme.md`

**Instances**:
- `001_thin_client_principle.md` - Thin Client governing principle and its boundaries
- `002_testing_standards.md` - No-mock and loud-failure testing mandates

#### operation/

Operational procedures for Claude API client.

**Master File**: `operation/readme.md`

**Instances**:
- `001_secret_loading.md` - Secret loading, authentication, and credential management procedures

#### pattern/

Design pattern specifications.

**Master File**: `pattern/readme.md`

**Instances**:
- `001_module_organization.md` - mod_interface pattern and module structure conventions

### Navigation

- API contracts and endpoint coverage: see `api/`
- Feature behavior specifications: see `feature/`
- Behavioral constraints: see `invariant/`
- Operational procedures (authentication, secret loading): see `operation/`
- Design patterns: see `pattern/`

### Collection Organization Principles

Per doc.rulebook.md:
- **Dimension-based structure**: Collections organized by design dimension (operation, pattern, protocol, etc.)
- **Instance granularity**: Each design concept in separate NNN-prefixed file
- **Master files required**: Each collection has readme.md with Scope, Responsibility, Overview
- **Complete Entity Coverage**: All files and directories documented in Responsibility Tables
