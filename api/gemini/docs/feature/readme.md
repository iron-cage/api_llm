# Feature Doc Entity

### Scope

- **Purpose**: Document optional feature behavior specifications for `api_gemini`.
- **Responsibility**: Master file listing all feature specification instances with ID, name, and implementation status.
- **In Scope**: Core API features, enterprise reliability features, their configuration contracts, and Cargo feature gates.
- **Out of Scope**: Always-on baseline client behavior (see invariant/ and api/), implementation internals (see pattern/).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Core API](001_core_api.md) | Core Gemini API endpoints — content generation, embeddings, models, streaming, multimodal | ✅ |
| 002 | [Enterprise Reliability](002_enterprise_reliability.md) | Enterprise reliability modules — retry, circuit breaker, rate limiting, failover, health checks, and extensions | ✅ |
