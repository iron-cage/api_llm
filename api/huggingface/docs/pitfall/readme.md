# Pitfall Doc Entity

### Scope

- **Purpose**: Document confirmed design pitfalls for `api_huggingface` — causes, consequences, and avoidance patterns.
- **Responsibility**: Each pitfall/ instance covers one confirmed design pitfall with root cause and avoidance guidance.
- **In Scope**: Pitfalls confirmed through task investigations, code review, or production incidents.
- **Out of Scope**: Hypothetical risks, best-practice suggestions, unconfirmed design concerns.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [URL Join Absolute Path](001_url_join_absolute_path.md) | `Url::join` with leading-slash paths silently strips base URL prefix | ✅ |
