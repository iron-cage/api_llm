# Documentation Directory

This directory contains detailed documentation, roadmaps, and architectural guides for the api_ollama crate.

## Purpose

The `docs/` directory serves as the central location for:
- Feature roadmaps and planning documents
- Implementation roadmaps and technical guides
- Architectural documentation
- Integration guides and tutorials
- Any documentation beyond inline code documentation

## Responsibility Table

| Entity | Responsibility | Reason |
|--------|---------------|--------|
| `readme.md` | Document docs/ organization | Directory structure, collection overview, navigation |
| `invariant/` | Non-negotiable behavioral constraints | Thin client principle, testing standards |
| `api/` | Ollama API endpoint coverage | Endpoint table, feature-gating policy |
| `feature/` | Optional feature behaviors | Enterprise reliability feature table |
| `pattern/` | Recurring design patterns | Module organization, client_ext_*.rs pattern |
| `operation/` | Operational procedures collection | Feature roadmaps, implementation strategies, workspace integration - see operation/readme.md |

## Documentation Collections

### operation/
Operational procedures and workflows for api_ollama development:
- Feature planning and roadmaps
- Implementation strategies and guidelines
- Workspace integration patterns
- Development best practices

See `operation/readme.md` for complete collection details.

## Documentation Guidelines

- **Feature Documentation**: New features should document their design decisions here
- **Architectural Documentation**: Major architectural changes should be documented with rationale
- **Integration Guides**: Complex integration scenarios should have dedicated documentation files
- **API Documentation**: High-level API overviews and usage patterns belong here; detailed API docs belong in source code
- **Collection Organization**: All documentation must be organized into appropriate design collections (operation/, pattern/, api/, etc.)

## Related Documentation

- See `../tests/readme.md` for test documentation and organization
- See source code (`../src/`) for inline API documentation
