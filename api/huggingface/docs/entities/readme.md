# Doc Entities

## Master Doc Entities Table

| Type | Master File | Purpose | Instance Count |
|------|-------------|---------|----------------|
| `api/` | [readme.md](../api/readme.md) | API contracts, endpoint mapping, error conditions, versioning | 1 |
| `catalog/` | [readme.md](../catalog/readme.md) | Enumerated item catalogs — feature flag tables, tier classifications | 1 |
| `feature/` | [readme.md](../feature/readme.md) | Cargo feature flags, capability groups, enterprise feature contracts | 1 |
| `invariant/` | [readme.md](../invariant/readme.md) | Non-negotiable behavioral constraints | 2 |
| `operation/` | [readme.md](../operation/readme.md) | Operational procedures — feature selection, verification, rollback | 1 |
| `pattern/` | [readme.md](../pattern/readme.md) | Design pattern specifications | 1 |
| `pitfall/` | [readme.md](../pitfall/readme.md) | Confirmed design pitfalls with root cause and avoidance guidance | 1 |

## Master Doc Instances Table

| ID | Type | Name | File | Status |
|----|------|------|------|--------|
| 001 | api | Reference | [api/001_reference.md](../api/001_reference.md) | ✅ |
| 001 | catalog | Features | [catalog/001_features.md](../catalog/001_features.md) | ✅ |
| 001 | feature | Enterprise Reliability | [feature/001_enterprise_reliability.md](../feature/001_enterprise_reliability.md) | ✅ |
| 001 | invariant | Thin Client Principle | [invariant/001_thin_client_principle.md](../invariant/001_thin_client_principle.md) | ✅ |
| 002 | invariant | Testing Standards | [invariant/002_testing_standards.md](../invariant/002_testing_standards.md) | ✅ |
| 001 | operation | Feature Selection | [operation/001_features.md](../operation/001_features.md) | ✅ |
| 001 | pattern | Module Organization | [pattern/001_module_organization.md](../pattern/001_module_organization.md) | ✅ |
| 001 | pitfall | URL Join Absolute Path | [pitfall/001_url_join_absolute_path.md](../pitfall/001_url_join_absolute_path.md) | ✅ |
