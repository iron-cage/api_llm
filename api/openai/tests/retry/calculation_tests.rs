//! Retry Delay Calculation Tests
//!
//! Tests for retry delay calculation including:
//! - Exponential backoff progression
//! - Jitter application
//! - Maximum delay enforcement

#[ cfg( feature = "retry" ) ]
mod retry_calculation_tests
{
  use crate::enhanced_retry_helpers::*;
  use core::time::Duration;

  #[ tokio::test ]
  async fn test_exponential_backoff_calculation()
  {
    let config = EnhancedRetryConfig::new()
      .with_base_delay( 1000 )
      .with_max_delay( 30000 )
      .with_jitter( 0 ) // No jitter for predictable testing
      .with_backoff_multiplier( 2.0 );

    // Test exponential backoff progression
    let delay_0 = config.calculate_delay( 0 );
    let delay_1 = config.calculate_delay( 1 );
    let delay_2 = config.calculate_delay( 2 );
    let delay_3 = config.calculate_delay( 3 );

    // Attempt 0: 1000 * 2^0 = 1000ms
    assert_eq!( delay_0, Duration::from_secs( 1 ) );

    // Attempt 1: 1000 * 2^1 = 2000ms
    assert_eq!( delay_1, Duration::from_secs( 2 ) );

    // Attempt 2: 1000 * 2^2 = 4000ms
    assert_eq!( delay_2, Duration::from_secs( 4 ) );

    // Attempt 3: 1000 * 2^3 = 8000ms
    assert_eq!( delay_3, Duration::from_secs( 8 ) );
  }

  #[ tokio::test ]
  async fn test_delay_calculation_with_jitter()
  {
    let config = EnhancedRetryConfig::new()
      .with_base_delay( 1000 )
      .with_jitter( 500 ); // 500ms jitter

    // Test that delay includes jitter (should be between base and base+jitter)
    let delay = config.calculate_delay( 0 );

    // Should be at least base_delay (1000ms)
    assert!( delay >= Duration::from_secs( 1 ) );

    // Should be at most base_delay + jitter (1500ms)
    assert!( delay <= Duration::from_millis( 1500 ) );
  }

  #[ tokio::test ]
  async fn test_delay_calculation_respects_max_delay()
  {
    let config = EnhancedRetryConfig::new()
      .with_base_delay( 1000 )
      .with_max_delay( 5000 )
      .with_jitter( 0 )
      .with_backoff_multiplier( 2.0 );

    // Calculate delay for high attempt number that would exceed max_delay
    let delay = config.calculate_delay( 10 ); // 1000 * 2^10 = 1024000ms

    // Should be capped at max_delay
    assert_eq!( delay, Duration::from_secs( 5 ) );
  }
}
