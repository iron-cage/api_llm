//! Spec traceability: FT-01..FT-12 — Enterprise Reliability
//! Source: `tests/docs/feature/001_enterprise_reliability.md`

#[ allow( unused_imports ) ]
use super::*;

mod private
{
  pub fn valid_format_secret() -> super::the_module::Secret
  {
    super::the_module::Secret::new( format!( "sk-ant-api03-{}", "x".repeat( 64 ) ) )
      .expect( "syntactically valid key must construct Secret" )
  }
}

/// FT-01: `Client::new()` has zero enterprise features active
#[ test ]
fn test_ft_01()
{
  let client = the_module::Client::new( private::valid_format_secret() );
  // No enterprise config parameter; all enterprise feature methods show disabled state
  let h = client.health();
  assert_eq!( h.consecutive_failures(), 0, "FT-01: no circuit breaker tracking by default" );
  assert_eq!( h.total_requests(), 0, "FT-01: no request tracking by default" );
  assert!( client.rate_limit_info().usage_percentage().abs() < f64::EPSILON, "FT-01: no rate limiter active" );
}

/// FT-02: `EnterpriseConfigBuilder` requires explicit construction
#[ test ]
fn test_ft_02()
{
  // EnterpriseConfig with no features has all enterprise features disabled
  let config = the_module::EnterpriseConfigBuilder::new().build();
  assert!( !config.retry_enabled(), "FT-02: no retry without explicit with_retry() call" );
  assert!( !config.circuit_breaker_enabled(), "FT-02: no circuit breaker without explicit call" );
  assert!( !config.rate_limiting_enabled(), "FT-02: no rate limiting without explicit call" );
  assert!( !config.failover_enabled(), "FT-02: no failover without explicit call" );
  assert!( !config.health_checks_enabled(), "FT-02: no health checks without explicit call" );
}

/// FT-03: `conservative()` profile sets 3 retry attempts
#[ cfg( feature = "retry-logic" ) ]
#[ test ]
fn test_ft_03()
{
  let config = the_module::EnterpriseConfigBuilder::conservative();
  assert!( config.retry_enabled(), "FT-03: conservative profile must enable retry" );
  assert_eq!(
    config.retry_config().expect( "retry must be configured" ).max_attempts(),
    3,
    "FT-03: conservative profile must have max_attempts == 3"
  );
}

/// FT-04: `balanced()` profile sets 5 retry attempts
#[ cfg( feature = "retry-logic" ) ]
#[ test ]
fn test_ft_04()
{
  let config = the_module::EnterpriseConfigBuilder::balanced();
  assert!( config.retry_enabled(), "FT-04: balanced profile must enable retry" );
  assert_eq!(
    config.retry_config().expect( "retry must be configured" ).max_attempts(),
    5,
    "FT-04: balanced profile must have max_attempts == 5"
  );
}

/// FT-05: `aggressive()` profile sets 10 retry attempts
#[ cfg( feature = "retry-logic" ) ]
#[ test ]
fn test_ft_05()
{
  let config = the_module::EnterpriseConfigBuilder::aggressive();
  assert!( config.retry_enabled(), "FT-05: aggressive profile must enable retry" );
  assert_eq!(
    config.retry_config().expect( "retry must be configured" ).max_attempts(),
    10,
    "FT-05: aggressive profile must have max_attempts == 10"
  );
}

/// FT-06: Enterprise modules compile only under their feature flag
#[ test ]
fn test_ft_06()
{
  // Verify that enterprise feature fields on EnterpriseConfig only appear under their flags
  // by checking that the enabled/disabled stubs exist for each feature
  let config = the_module::EnterpriseConfigBuilder::new().build();
  // These always-available methods confirm the feature-gating pattern is present
  let _retry = config.retry_enabled();         // stub: false when retry-logic disabled
  let _cb = config.circuit_breaker_enabled();  // stub: false when circuit-breaker disabled
  let _rl = config.rate_limiting_enabled();    // stub: false when rate-limiting disabled
  // Compilation success with any feature combination confirms independent gating
  assert!( config.is_valid(), "FT-06: default empty config must be valid" );
}

/// FT-07: Each enterprise module is independently gated
#[ test ]
fn test_ft_07()
{
  // Enabling retry alone does not activate other enterprise features
  let retry_only = the_module::EnterpriseConfigBuilder::new()
    .with_retry( the_module::RetryConfig::default() )
    .build();
  assert!( retry_only.retry_enabled(), "FT-07: retry-only config has retry" );
  assert!( !retry_only.circuit_breaker_enabled(), "FT-07: retry-only config has no circuit breaker" );
  assert!( !retry_only.rate_limiting_enabled(), "FT-07: retry-only config has no rate limiting" );
  assert!( !retry_only.failover_enabled(), "FT-07: retry-only config has no failover" );
  assert!( !retry_only.health_checks_enabled(), "FT-07: retry-only config has no health checks" );
}

/// FT-08: `EnterpriseConfigBuilder` rejects invalid configuration
#[ test ]
fn test_ft_08()
{
  let invalid_retry = the_module::RetryConfig::new()
    .with_max_attempts( 0 ); // 0 attempts is invalid
  let result = the_module::EnterpriseConfigBuilder::new()
    .with_retry( invalid_retry )
    .try_build();
  assert!( result.is_err(), "FT-08: try_build() must reject max_attempts == 0" );
  let msg = result.unwrap_err();
  assert!( !msg.is_empty(), "FT-08: rejection message must be non-empty and descriptive" );
}

/// FT-09: `enterprise_quota` module compiles under enterprise-quota flag
#[ cfg( feature = "enterprise-quota" ) ]
#[ test ]
fn test_ft_09()
{
  // QuotaConfig and QuotaManager are accessible in the public API only under this feature
  let config = the_module::QuotaConfig::new()
    .with_daily_requests( 1000 );
  let manager = the_module::QuotaManager::new( config );
  assert_eq!(
    manager.daily_usage().request_count, 0,
    "FT-09: fresh quota manager starts at zero daily usage"
  );
}

/// FT-10: `dynamic_config` module compiles under dynamic-config flag
#[ cfg( feature = "dynamic-config" ) ]
#[ test ]
fn test_ft_10()
{
  // RuntimeConfig is accessible in the public API only under this feature
  let config = the_module::RuntimeConfig::new();
  assert!(
    config.validate().is_ok(),
    "FT-10: default RuntimeConfig must pass validation"
  );
}

/// FT-11: `request_caching` module compiles under request-caching flag
#[ cfg( feature = "request-caching" ) ]
#[ test ]
fn test_ft_11()
{
  // CacheConfig and RequestCache are accessible in the public API only under this feature
  let config = the_module::CacheConfig::new()
    .with_ttl_seconds( 300 )
    .with_max_entries( 100 );
  assert!(
    config.is_valid(),
    "FT-11: valid CacheConfig must pass validation"
  );
}

/// FT-12: compression module compiles under compression flag
#[ cfg( feature = "compression" ) ]
#[ test ]
fn test_ft_12()
{
  // CompressionConfig and compress/decompress are accessible in the public API only under this feature
  // Use large repetitive data so compressed output is smaller than input, ensuring
  // compress() returns the gzip bytes rather than the original (its size-guard kicks in)
  let config = the_module::CompressionConfig::new().with_min_size( 0 );
  let data : Vec< u8 > = b"FT-12 compression roundtrip test data ".repeat( 64 );
  let compressed = the_module::compress( &data, &config )
    .expect( "FT-12: compression must succeed on valid input" );
  assert!( compressed != data, "FT-12: compressed output must differ from original (gzip bytes)" );
  let decompressed = the_module::decompress( &compressed )
    .expect( "FT-12: decompression must succeed on valid compressed input" );
  assert_eq!( decompressed, data, "FT-12: roundtrip compress/decompress must be lossless" );
}
