# reliability

### Purpose

Enterprise reliability features — circuit breaker, rate limiting, failover, and health monitoring.

### Responsibility

| File | Purpose |
|------|---------|
| `mod.rs` | Module root — re-exports all reliability types |
| `circuit_breaker.rs` | Circuit breaker with open/half-open/closed state machine |
| `rate_limiter.rs` | Token bucket rate limiter with per-second/minute/hour windows |
| `failover.rs` | Multi-endpoint failover with Priority, RoundRobin, Random, Sticky strategies |
| `health_check.rs` | Background endpoint health monitoring with configurable intervals |
