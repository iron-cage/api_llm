# Operational Procedures Collection

## Collection Scope

This collection documents operational procedures for the api_ollama crate, including:
- Feature planning and implementation roadmaps
- Technical implementation strategies and guidelines
- Workspace integration and tooling procedures
- Development workflows and best practices

## Responsibility

This table documents all instances in this collection, ensuring Complete Entity Coverage.

| Instance | Purpose |
|----------|---------|
| `001_feature_roadmap.md` | Feature planning and implementation roadmap - status tracking, priorities, effort estimates for all features |
| `002_implementation_roadmap.md` | Technical implementation roadmap - strategies, priorities, guidelines for feature development and rollout |
| `003_workspace_integration.md` | workspace_tools integration guide - complete reference for secret loading functionality and workspace patterns |

## Overview

| ID | Operation Name | Category | Complexity | Status |
|----|----------------|----------|------------|--------|
| 001 | Feature Roadmap | Planning | Medium | Active |
| 002 | Implementation Roadmap | Planning | Medium | Active |
| 003 | Workspace Integration | Integration | Low | Active |

## Collection Principles

- **Abstract First**: Documentation focuses on operational procedures and workflows, not implementation details
- **Instance Granularity**: Each operational aspect documented in separate NNN-prefixed file
- **Complete Coverage**: All operational documents listed in Responsibility Table
- **Dimension Focus**: operation/ dimension documents "how we operate" not "how code works"

## Navigation

- For feature planning and status tracking: see `001_feature_roadmap.md`
- For implementation strategies and guidelines: see `002_implementation_roadmap.md`
- For workspace integration patterns: see `003_workspace_integration.md`

## Related Collections

- See `../readme.md` for docs/ directory organization
- See `../invariant/` for behavioral constraints and governing principles
- See `../../tests/readme.md` for test documentation
