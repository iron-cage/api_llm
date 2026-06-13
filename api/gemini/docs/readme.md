# Documentation

### Purpose

This directory contains detailed API documentation, operational guides, and technical references for the api_gemini crate, organized into design collections following documentation.rulebook.md standards.

Documentation is structured by design dimensions (pattern/, api/, operation/, protocol/) with each collection containing:
- Master file (`readme.md`) with collection scope and Overview Table
- Instance files with NNN identifiers (001, 002, etc.)

### Responsibility

This table documents all entities in the docs/ directory, ensuring Complete Entity Coverage.

| Path | Purpose |
|------|---------|
| `readme.md` | Master documentation file with navigation and Complete Entity Coverage |
| `entities.md` | Module index — all entity directories and their instances |
| `invariant/` | Invariant constraint collection — non-negotiable behavioral constraints |
| `feature/` | Feature specification collection — optional feature behavior specs |
| `api/` | API doc entity — comprehensive API coverage, endpoints, test statistics |
| `operation/` | Operation doc entity — procedures for API usage patterns |
| `pattern/` | Pattern doc entity — reusable API usage patterns |
| `protocol/` | Protocol doc entity — streaming format specification |
| `investigations/` | Investigation reports — debugging sessions and format discoveries |

### Collections

#### invariant/

Non-negotiable behavioral constraints governing every implementation decision.

**Master File**: `invariant/readme.md`

**Instances**:
- `001_thin_client_principle.md` — Stateless HTTP transport, no automatic behaviors, explicit caller control
- `002_testing_standards.md` — No-mock mandate, real API calls only, loud failure on missing credentials

#### feature/

Optional feature behavior specifications and configuration contracts.

**Master File**: `feature/readme.md`

**Instances**:
- `001_core_api.md` — Core Gemini endpoints: content generation, embeddings, models, streaming, multimodal
- `002_enterprise_reliability.md` — Enterprise modules: retry, circuit-breaker, rate-limiting, failover, health-checks

#### api/

API design, endpoints, coverage, and implementation status.

**Master File**: `api/readme.md`

**Instances**:
- `001_coverage.md` — Comprehensive API coverage: core endpoints, advanced features, enterprise capabilities, test statistics

#### operation/

Operational procedures for common API usage patterns.

**Master File**: `operation/readme.md`

**Instances**:
- `002_usage_examples.md` — Common API usage operation procedures

#### pattern/

Reusable structural patterns for common Gemini API usage scenarios.

**Master File**: `pattern/readme.md`

**Instances**:
- `001_quick_response.md` — Single-call generation; all errors propagate to caller as Result
- `002_error_resilient.md` — Generation with error-variant-to-fallback-string mapping
- `003_batch_processing.md` — Sequential paced loop for rate-limited bulk prompt workloads

#### protocol/

Protocol specifications for API communication at the wire level.

**Master File**: `protocol/readme.md`

**Instances**:
- `001_streaming_format.md` — Streaming wire protocol: JSON array format specification

#### investigations/

Investigation reports documenting debugging sessions and format discoveries. Not an entity collection — plain reference directory.

**Contents**:
- `001_streaming_format.md` — Gemini streaming format discovery: SSE assumption vs JSON array reality

### Navigation

**For API Usage**:
- Reference `api/001_coverage.md` for comprehensive API coverage
- See `operation/002_usage_examples.md` for usage operation procedures

**For Implementation Patterns**:
- See `pattern/001_quick_response.md` for simple single-call generation pattern
- See `pattern/002_error_resilient.md` for error-absorbing generation pattern
- See `pattern/003_batch_processing.md` for rate-limited bulk processing pattern

**For Protocol**:
- See `protocol/001_streaming_format.md` for streaming wire protocol specification
- See `investigations/001_streaming_format.md` for streaming format discovery history

**For Implementation Details**:
- See `invariant/001_thin_client_principle.md` for governing design principles
- See `invariant/002_testing_standards.md` for testing policy and requirements
- See `feature/001_core_api.md` for core API endpoint specifications
- See `feature/002_enterprise_reliability.md` for enterprise feature configuration
- See source code documentation for implementation decisions
- See `tests/` for coverage details

### Documentation Principles

- **API Coverage**: WHAT functionality is available (endpoints, features, status)
- **Usage Procedures**: HOW to use the library (operation steps, prerequisites, expected outcomes)
- **Protocol**: WIRE specification for API communication (streaming format)
- **Investigations**: HOW bugs were found and resolved (debugging history)
- **Abstract First**: Documentation focuses on concepts, not implementation details
- **Complete Coverage**: All documents listed in Responsibility Tables
