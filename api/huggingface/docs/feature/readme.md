# Feature Doc Entity

### Scope

- **Purpose**: Document Cargo feature flags, optional capability groups, and enterprise feature contracts for `api_huggingface`.
- **Responsibility**: Master file listing all feature doc instances with ID, name, and status.
- **In Scope**: Feature behavior specifications, capability groupings, dependency contracts, activation requirements.
- **Out of Scope**: Operational procedures, API method signatures, implementation source details.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Enterprise Reliability](001_enterprise_reliability.md) | Enterprise reliability feature group — circuit breaker, rate limiting, failover, health checks, caching, metrics, token counting, dynamic config | ✅ |
