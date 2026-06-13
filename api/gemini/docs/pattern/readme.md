# Pattern Doc Entity

### Scope

- **Purpose**: Document reusable structural patterns for common api_gemini usage scenarios.
- **Responsibility**: Master file listing all pattern instances with ID, name, and pattern scope.
- **In Scope**: Architectural patterns addressing distinct failure modes or operational constraints — each pattern covers one problem/solution/applicability context.
- **Out of Scope**: Step-by-step API call procedures (see `operation/002_usage_examples.md`), enterprise feature configuration (see `feature/002_enterprise_reliability.md`), protocol wire format (see `protocol/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Quick Response](001_quick_response.md) | Single-call generation; all errors propagate to caller as Result | ✅ |
| 002 | [Error-Resilient](002_error_resilient.md) | Generation with error-variant-to-fallback-string mapping; absorbs error signal | ✅ |
| 003 | [Batch Processing](003_batch_processing.md) | Sequential paced loop for rate-limited bulk prompt workloads | ✅ |
