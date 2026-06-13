# performance

### Purpose

Request performance metrics — latency, throughput, and error rate tracking.

### Responsibility

| File | Purpose |
|------|---------|
| `mod.rs` | Module root — re-exports `MetricsCollector` type |
| `metrics.rs` | Latency, throughput, and error rate collection with statistical aggregates |
