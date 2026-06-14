# Documentation

## Purpose

Technical documentation, design collections, and architecture analysis for the api_openai crate.

## Collection Scope

This documentation directory contains:
- **Design Collections**: Dimension-based organization of design documentation
- **General Documentation**: Crate-wide documentation not specific to a collection
- **Master Files**: Collection inventory and navigation

## Responsibility

| Path | Purpose |
|------|---------|
| `readme.md` | Master documentation file with collection inventory and navigation |
| `invariant/` | Non-negotiable behavioral constraints governing all implementation decisions |
| `api/` | OpenAI API endpoint coverage and feature-gating policy |
| `feature/` | Optional enterprise reliability features and feature-gate policy |
| `pattern/` | Design pattern collection (async patterns, builder patterns, etc.) |
| `operation/` | Operational procedure collection (versioning, release management, etc.) |

## Design Collections Overview

### pattern/ - Design Patterns Collection
**Purpose**: Reusable design patterns used throughout the crate
**Instances**: 1
**Master File**: [pattern/readme.md](pattern/readme.md)

**Pattern Instances**:
| ID | Pattern Name | Category | Complexity |
|----|-------------|----------|------------|
| 001 | Async Patterns | Concurrency | Medium |

### operation/ - Operational Procedures Collection
**Purpose**: Operational procedures for maintenance and release management
**Instances**: 1
**Master File**: [operation/readme.md](operation/readme.md)

**Operation Instances**:
| ID | Operation Name | Category | Frequency |
|----|---------------|----------|-----------|
| 001 | Semantic Versioning | Release Management | Per Release |

## Navigation Guide

### By Collection Type
- **Design Patterns**: [pattern/readme.md](pattern/readme.md)
- **Operational Procedures**: [operation/readme.md](operation/readme.md)

### By Topic
- **Async Programming**: [pattern/001_async_patterns.md](pattern/001_async_patterns.md)
- **Versioning Strategy**: [operation/001_semantic_versioning.md](operation/001_semantic_versioning.md)

## Collection Guidelines

All design collections follow the doc.rulebook.md standards:
1. **Abstract-First**: No language-specific code in collection docs
2. **Instance-Level Granularity**: Each instance has unique NNN identifier
3. **Master Files**: Each collection has readme.md with Scope, Responsibility, Overview
4. **Complete Entity Coverage**: All instances listed in master file Responsibility Table

## Adding New Documentation

### Adding to Existing Collection
1. Create new instance file with next NNN identifier (e.g., `002_pattern_name.md`)
2. Update collection master file Responsibility Table
3. Update collection master file Overview Table
4. Update this root readme.md if needed

### Creating New Collection
1. Create new collection directory (e.g., `protocol/`, `api/`)
2. Create collection master file (`readme.md`) with required sections
3. Move/create instance files with NNN identifiers
4. Update this root readme.md Responsibility Table
5. Add collection overview to this root readme.md

## Related Documentation

- **Root readme.md**: Crate overview, quick start, usage examples
- **tests/readme.md**: Testing framework, policies, and organization
- **examples/readme.md**: Example usage and tutorials
