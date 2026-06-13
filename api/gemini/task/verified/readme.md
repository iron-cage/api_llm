# Verified Tasks

Holds task files that have passed the VERIFY gate. Tasks here are ready for execution. Completed tasks move to `completed/` on SUBMIT + validation pass.

### Responsibility Table

| File | Purpose |
|------|---------|
| `readme.md` | This file — directory responsibility and lifecycle note |
| `002_remove_fabricated_gemini_api_endpoints.md` | Remove 4 methods calling non-existent API endpoints (404) |
| `003_fix_circuit_breaker_per_call_reset.md` | Fix fresh CircuitBreaker per call — feature inoperative |
| `004_remove_batch_api_mock_data.md` | Replace 7 mock Ok() returns with Err(NotImplemented) |
| `005_resolve_clippy_allow_overrides.md` | Eliminate 27 TDD-cleanup clippy suppression overrides |
