//! Benchmarks for rate limiting overhead measurement
#![allow(missing_docs)]

use criterion::{ criterion_group, criterion_main, Criterion };
use std::time::{ Duration, Instant };

fn benchmark_token_bucket_check( c: &mut Criterion )
{
  c.bench_function( "check_token_availability", |b|
  {
    let bucket = TokenBucket {
      tokens: 8.0,
      max_tokens: 10.0,
      refill_rate: 10.0,
      last_refill: Instant::now(),
    };

    b.iter( ||
    {
      // Simulate token availability check
      bucket.tokens >= 1.0
    } );
  } );
}

fn benchmark_token_refill_calculation( c: &mut Criterion )
{
  c.bench_function( "calculate_token_refill", |b|
  {
    let mut bucket = TokenBucket {
      tokens: 5.0,
      max_tokens: 10.0,
      refill_rate: 10.0,
      last_refill: Instant::now() - Duration::from_millis( 100 ),
    };

    b.iter( ||
    {
      // Simulate token refill calculation
      let now = Instant::now();
      let elapsed = now.duration_since( bucket.last_refill ).as_secs_f64();
      let tokens_to_add = elapsed * bucket.refill_rate;
      let new_tokens = ( bucket.tokens + tokens_to_add ).min( bucket.max_tokens );

      bucket.tokens = new_tokens;
      bucket.last_refill = now;
      bucket.clone()
    } );
  } );
}

fn benchmark_token_consumption( c: &mut Criterion )
{
  c.bench_function( "consume_token", |b|
  {
    b.iter( ||
    {
      let mut bucket = TokenBucket {
        tokens: 8.0,
        max_tokens: 10.0,
        refill_rate: 10.0,
        last_refill: Instant::now(),
      };

      // Simulate token consumption
      if bucket.tokens >= 1.0
      {
        bucket.tokens -= 1.0;
        // black_box prevents the compiler from treating the modification as dead code
        ::std::hint::black_box( bucket.tokens );
        true
      } else {
        false
      }
    } );
  } );
}

fn benchmark_wait_time_calculation( c: &mut Criterion )
{
  c.bench_function( "calculate_wait_time", |b|
  {
    let bucket = TokenBucket {
      tokens: 0.0,
      max_tokens: 10.0,
      refill_rate: 10.0,
      last_refill: Instant::now(),
    };

    b.iter( ||
    {
      // Simulate wait time calculation when tokens unavailable
      if bucket.tokens < 1.0
      {
        let tokens_needed = 1.0 - bucket.tokens;
        let wait_seconds = tokens_needed / bucket.refill_rate;
        Some( Duration::from_secs_f64( wait_seconds ) )
      } else {
        None
      }
    } );
  } );
}

fn benchmark_sliding_window_check( c: &mut Criterion )
{
  c.bench_function( "check_sliding_window_limit", |b|
  {
    let window = SlidingWindow {
      requests: vec![
      Instant::now() - Duration::from_secs( 5 ),
      Instant::now() - Duration::from_secs( 3 ),
      Instant::now() - Duration::from_secs( 1 ),
      ],
      window_size: Duration::from_secs( 60 ),
      max_requests: 10,
    };

    b.iter( ||
    {
      // Simulate sliding window check
      let now = Instant::now();
      let cutoff = now - window.window_size;
      let recent_requests = window.requests.iter().filter( | &&t | t >= cutoff ).count();
      recent_requests < window.max_requests
    } );
  } );
}

// Token bucket for benchmarking
#[ derive( Debug, Clone ) ]
struct TokenBucket
{
  tokens: f64,
  max_tokens: f64,
  refill_rate: f64,
  last_refill: Instant,
}

// Sliding window for benchmarking
#[ derive( Debug, Clone ) ]
struct SlidingWindow
{
  requests: Vec< Instant >,
  window_size: Duration,
  max_requests: usize,
}

criterion_group!(
benches,
benchmark_token_bucket_check,
benchmark_token_refill_calculation,
benchmark_token_consumption,
benchmark_wait_time_calculation,
benchmark_sliding_window_check
);
criterion_main!( benches );
