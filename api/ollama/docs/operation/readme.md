# Operational Procedures Collection

## Collection Scope

This collection documents operational procedures for the api_ollama crate.

## Responsibility

| Instance | Purpose |
|----------|---------|
| `001_secret_loading.md` | Secret directory layout, host URL configuration, and workspace_tools loading procedure |

## Overview

| ID | Operation Name | Category | Complexity |
|----|----------------|----------|------------|
| 001 | Secret Loading | Configuration | Low |

## Collection Principles

- **Abstract First**: Documentation focuses on operational procedures and workflows, not implementation details
- **Instance Granularity**: Each operational aspect documented in separate NNN-prefixed file
- **Complete Coverage**: All operational documents listed in Responsibility Table
- **Dimension Focus**: operation/ dimension documents "how we operate" not "how code works"

## Navigation

- For host URL configuration and workspace_tools loading: see `001_secret_loading.md`

## Related Collections

- See `../readme.md` for docs/ directory organization
- See `../invariant/` for behavioral constraints and governing principles
- See `../../tests/readme.md` for test documentation
