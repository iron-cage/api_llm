# Performance Benchmarks

## Responsibility Table

| filename | Responsibility |
|----------|---------------|
| client_overhead.rs | Request building and response parsing overhead |
| retry_logic_overhead.rs | Retry decision and backoff calculation overhead |
| circuit_breaker_overhead.rs | Circuit breaker state check overhead |
| rate_limiting_overhead.rs | Token bucket and sliding window overhead |
| streaming_overhead.rs | Stream buffer management overhead |

This directory contains comprehensive performance benchmarks for the Gemini API client to validate the <50ms overhead claim specified in the requirements.

## Running Benchmarks

Run all benchmarks:
```bash
cargo bench
```

Run a specific benchmark:
```bash
cargo bench --bench client_overhead
cargo bench --bench retry_logic_overhead
cargo bench --bench circuit_breaker_overhead
cargo bench --bench rate_limiting_overhead
cargo bench --bench streaming_overhead
```

Run a specific test within a benchmark:
```bash
cargo bench --bench client_overhead build_generate_content_request
```

## Benchmark Categories

### client_overhead.rs
Measures core client-side processing overhead:
- **build_generate_content_request**: Request building time (~0.5μs target)
- **create_client_builder**: Client initialization time (~5μs target)
- **parse_generate_content_response**: Response parsing time (~10μs target)

**Expected Total**: <20μs for basic request/response cycle

### retry_logic_overhead.rs
Measures retry mechanism processing overhead:
- **create_retry_config**: Retry configuration creation (~0.1μs target)
- **retry_decision_logic**: Decision if retry needed (~0.05μs target)
- **calculate_backoff_delay**: Exponential backoff calculation (~0.2μs target)
- **classify_retryable_error**: Error classification logic (~0.05μs target)

**Expected Total**: <0.5μs per retry decision

### circuit_breaker_overhead.rs
Measures circuit breaker state management overhead:
- **check_circuit_breaker_state**: State checking (~0.05μs target)
- **record_failure**: Failure recording and state transition (~0.1μs target)
- **record_success**: Success recording (~0.1μs target)
- **check_timeout_for_half_open**: Timeout checking for state transition (~0.1μs target)

**Expected Total**: <0.5μs per circuit breaker check

### rate_limiting_overhead.rs
Measures rate limiting algorithm overhead:
- **check_token_availability**: Token bucket availability check (~0.05μs target)
- **calculate_token_refill**: Token refill calculation (~0.2μs target)
- **consume_token**: Token consumption (~0.1μs target)
- **calculate_wait_time**: Wait time calculation (~0.1μs target)
- **check_sliding_window_limit**: Sliding window algorithm check (~1μs target)

**Expected Total**: <2μs per rate limit check

### streaming_overhead.rs
Measures streaming buffer management overhead:
- **allocate_stream_buffer**: Buffer allocation (~0.5μs target)
- **push_to_stream_buffer**: Buffer push operation (~0.1μs target)
- **pop_from_stream_buffer**: Buffer pop operation (~0.1μs target)
- **parse_sse_line**: Server-Sent Events parsing (~0.5μs target)
- **parse_json_chunk**: JSON chunk parsing (~2μs target)
- **accumulate_partial_chunks**: Partial chunk accumulation (~0.2μs target)
- **check_buffer_size_limit**: Buffer size checking (~0.1μs target)
- **classify_stream_event_type**: Event type classification (~0.05μs target)

**Expected Total**: <5μs per streaming chunk

## Performance Target

The client aims for **<50ms total overhead** for complete API operations, broken down as:

- Request building: <1ms
- Response parsing: <5ms
- Retry logic: <0.5ms (per retry attempt)
- Circuit breaker: <0.5ms
- Rate limiting: <2ms
- Streaming buffers: <5ms (per chunk)
- Network I/O overhead: <40ms

**Total client overhead budget: <50ms** (excluding actual network latency to API servers)

## Interpreting Results

Criterion outputs results in the format:
```
benchmark_name     time:   [lower_bound mean upper_bound]
```

Focus on the **mean** value. The benchmarks measure:
- Microseconds (μs) for most operations
- Nanoseconds (ns) for very fast operations
- Milliseconds (ms) for complex operations

### What to Look For

✅ **Good**: Mean times at or below targets listed above
⚠️  **Warning**: Mean times 2x-5x above targets (investigate)
❌ **Critical**: Mean times >5x above targets (requires optimization)

### Example Output

```
build_generate_content_request
                        time:   [450.12 ns 455.23 ns 461.45 ns]
```

This shows ~0.45μs mean time, well under the 0.5μs target.

## Baseline Results

Run `cargo bench` to establish baseline metrics for your system. Results are stored in `target/criterion/` and include:
- HTML reports for visualization
- Historical comparison data
- Statistical analysis (mean, median, std dev)

## Continuous Monitoring

These benchmarks should be run:
1. Before each release to validate performance targets
2. After any changes to core processing logic
3. When adding new features that affect request/response flow
4. As part of CI/CD pipeline for performance regression detection

## Troubleshooting

If benchmarks fail to compile:
1. Ensure criterion is in dev-dependencies: `cargo add --dev criterion`
2. Check that all features are enabled: `cargo bench --all-features`
3. Verify Rust toolchain is up to date: `rustup update`

If results are inconsistent:
1. Close other applications to reduce system noise
2. Run with `--warm-up-time=5` for longer warm-up
3. Check CPU frequency scaling: `cat /proc/cpuinfo | grep MHz`
4. Consider running on a dedicated benchmark machine
