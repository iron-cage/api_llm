//! Enterprise Quota Management Tests
//!
//! Unit tests for the enterprise-quota feature that provides usage tracking
//! and quota enforcement for production deployments.

#[ allow( unused_imports ) ]
use super::*;

#[ cfg( feature = "enterprise-quota" ) ]
mod enterprise_quota_tests
{
  use super::*;
  use the_module::{ UsageMetrics, QuotaConfig, CostCalculator, QuotaManager };

  #[ test ]
  fn test_usage_metrics_new()
  {
    let metrics = UsageMetrics::new();
    assert_eq!( metrics.request_count, 0 );
    assert_eq!( metrics.input_tokens, 0 );
    assert_eq!( metrics.output_tokens, 0 );
    assert!( metrics.total_cost.abs() < f64::EPSILON );
    assert_eq!( metrics.total_tokens(), 0 );
  }

  #[ test ]
  fn test_usage_metrics_record()
  {
    let mut metrics = UsageMetrics::new();
    metrics.record_request( 100, 50, 0.5 );

    assert_eq!( metrics.request_count, 1 );
    assert_eq!( metrics.input_tokens, 100 );
    assert_eq!( metrics.output_tokens, 50 );
    assert!( ( metrics.total_cost - 0.5_f64 ).abs() < f64::EPSILON );
    assert_eq!( metrics.total_tokens(), 150 );

    // Record another
    metrics.record_request( 200, 100, 1.0 );
    assert_eq!( metrics.request_count, 2 );
    assert_eq!( metrics.input_tokens, 300 );
    assert_eq!( metrics.output_tokens, 150 );
    assert!( ( metrics.total_cost - 1.5_f64 ).abs() < f64::EPSILON );
  }

  #[ test ]
  fn test_quota_config_defaults()
  {
    let config = QuotaConfig::new();
    assert_eq!( config.daily_request_limit, None );
    assert_eq!( config.daily_token_limit, None );
    assert_eq!( config.daily_cost_limit, None );
    assert_eq!( config.monthly_request_limit, None );
    assert_eq!( config.monthly_token_limit, None );
    assert_eq!( config.monthly_cost_limit, None );
  }

  #[ test ]
  fn test_quota_config_builder()
  {
    let config = QuotaConfig::new()
      .with_daily_requests( 100 )
      .with_daily_tokens( 10_000 )
      .with_daily_cost( 5.0 )
      .with_monthly_requests( 3_000 )
      .with_monthly_tokens( 300_000 )
      .with_monthly_cost( 150.0 );

    assert_eq!( config.daily_request_limit, Some( 100 ) );
    assert_eq!( config.daily_token_limit, Some( 10_000 ) );
    assert_eq!( config.daily_cost_limit, Some( 5.0 ) );
    assert_eq!( config.monthly_request_limit, Some( 3_000 ) );
    assert_eq!( config.monthly_token_limit, Some( 300_000 ) );
    assert_eq!( config.monthly_cost_limit, Some( 150.0 ) );
  }

  #[ test ]
  fn test_cost_calculator_sonnet()
  {
    let pricing = CostCalculator::for_model( "claude-3-5-sonnet-latest" );
    assert!( ( pricing.input_cost_per_million - 3.0_f64 ).abs() < f64::EPSILON );
    assert!( ( pricing.output_cost_per_million - 15.0_f64 ).abs() < f64::EPSILON );
  }

  #[ test ]
  fn test_cost_calculator_opus()
  {
    let pricing = CostCalculator::for_model( "claude-3-opus-20240229" );
    assert!( ( pricing.input_cost_per_million - 15.0_f64 ).abs() < f64::EPSILON );
    assert!( ( pricing.output_cost_per_million - 75.0_f64 ).abs() < f64::EPSILON );
  }

  #[ test ]
  fn test_cost_calculator_haiku()
  {
    let pricing = CostCalculator::for_model( "claude-3-haiku-latest" );
    assert!( ( pricing.input_cost_per_million - 0.25_f64 ).abs() < f64::EPSILON );
    assert!( ( pricing.output_cost_per_million - 1.25_f64 ).abs() < f64::EPSILON );
  }

  #[ test ]
  fn test_cost_calculator_calculate_cost()
  {
    let pricing = CostCalculator::for_model( "claude-3-5-sonnet-latest" );

    // 1000 input tokens, 500 output tokens
    let cost = pricing.calculate_cost( 1_000, 500 );

    // Expected : (1000/1_000_000)*3.0 + (500/1_000_000)*15.0
    //         = 0.003 + 0.0075 = 0.0105
    assert!( ( cost - 0.0105 ).abs() < 0.0001 );
  }

  #[ test ]
  fn test_quota_manager_no_limits()
  {
    let config = QuotaConfig::new();
    let manager = QuotaManager::new( config );

    // Should succeed without limits
    let result = manager.record_usage( "claude-3-5-sonnet-latest", 1_000, 500 );
    assert!( result.is_ok() );

    let daily = manager.daily_usage();
    assert_eq!( daily.request_count, 1 );
    assert_eq!( daily.input_tokens, 1_000 );
    assert_eq!( daily.output_tokens, 500 );
  }

  #[ test ]
  fn test_quota_manager_daily_request_limit()
  {
    let config = QuotaConfig::new().with_daily_requests( 2 );
    let manager = QuotaManager::new( config );

    // First two should succeed
    assert!( manager.record_usage( "claude-3-5-sonnet-latest", 100, 50 ).is_ok() );
    assert!( manager.record_usage( "claude-3-5-sonnet-latest", 100, 50 ).is_ok() );

    // Third should fail
    let result = manager.record_usage( "claude-3-5-sonnet-latest", 100, 50 );
    assert!( result.is_err() );
    assert!( result.unwrap_err().message.contains( "Daily request limit" ) );
  }

  #[ test ]
  fn test_quota_manager_daily_token_limit()
  {
    let config = QuotaConfig::new().with_daily_tokens( 1_000 );
    let manager = QuotaManager::new( config );

    // 800 tokens should succeed
    assert!( manager.record_usage( "claude-3-5-sonnet-latest", 500, 300 ).is_ok() );

    // Another 300 tokens should fail (would exceed 1000)
    let result = manager.record_usage( "claude-3-5-sonnet-latest", 200, 100 );
    assert!( result.is_err() );
    assert!( result.unwrap_err().message.contains( "Daily token limit" ) );
  }

  #[ test ]
  fn test_quota_manager_daily_cost_limit()
  {
    let config = QuotaConfig::new().with_daily_cost( 0.015 );
    let manager = QuotaManager::new( config );

    // First request within budget (costs ~$0.0105)
    assert!( manager.record_usage( "claude-3-5-sonnet-latest", 1_000, 500 ).is_ok() );

    // Second request would exceed budget (would be ~$0.021 total)
    let result = manager.record_usage( "claude-3-5-sonnet-latest", 1_000, 500 );
    assert!( result.is_err() );
    assert!( result.unwrap_err().message.contains( "Daily cost limit" ) );
  }

  #[ test ]
  fn test_quota_manager_per_model_tracking()
  {
    let config = QuotaConfig::new();
    let manager = QuotaManager::new( config );

    // Record for different models
    manager.record_usage( "claude-3-5-sonnet-latest", 1_000, 500 ).unwrap();
    manager.record_usage( "claude-3-opus-latest", 2_000, 1_000 ).unwrap();
    manager.record_usage( "claude-3-5-sonnet-latest", 500, 250 ).unwrap();

    // Check per-model metrics
    let sonnet_metrics = manager.model_usage( "claude-3-5-sonnet-latest" ).unwrap();
    assert_eq!( sonnet_metrics.request_count, 2 );
    assert_eq!( sonnet_metrics.input_tokens, 1_500 );
    assert_eq!( sonnet_metrics.output_tokens, 750 );

    let opus_metrics = manager.model_usage( "claude-3-opus-latest" ).unwrap();
    assert_eq!( opus_metrics.request_count, 1 );
    assert_eq!( opus_metrics.input_tokens, 2_000 );
    assert_eq!( opus_metrics.output_tokens, 1_000 );
  }

  #[ test ]
  fn test_quota_manager_export_json()
  {
    let config = QuotaConfig::new();
    let manager = QuotaManager::new( config );

    manager.record_usage( "claude-3-5-sonnet-latest", 1_000, 500 ).unwrap();

    let json = manager.export_json().unwrap();
    assert!( json.contains( "\"daily\"" ) );
    assert!( json.contains( "\"monthly\"" ) );
    assert!( json.contains( "\"per_model\"" ) );
    assert!( json.contains( "request_count" ) );
  }

  #[ test ]
  fn test_quota_manager_reset_daily()
  {
    let config = QuotaConfig::new();
    let mut manager = QuotaManager::new( config );

    manager.record_usage( "claude-3-5-sonnet-latest", 1_000, 500 ).unwrap();
    assert_eq!( manager.daily_usage().request_count, 1 );

    manager.reset_daily();
    assert_eq!( manager.daily_usage().request_count, 0 );
  }

  #[ test ]
  fn test_quota_manager_reset_monthly()
  {
    let config = QuotaConfig::new();
    let mut manager = QuotaManager::new( config );

    manager.record_usage( "claude-3-5-sonnet-latest", 1_000, 500 ).unwrap();
    assert_eq!( manager.monthly_usage().request_count, 1 );

    manager.reset_monthly();
    assert_eq!( manager.monthly_usage().request_count, 0 );
  }
}

#[ cfg( not( feature = "enterprise-quota" ) ) ]
mod enterprise_quota_feature_disabled
{
  #[ test ]
  fn test_enterprise_quota_feature_disabled()
  {
    // When enterprise-quota feature is disabled, this test verifies
    // that compilation succeeds without the feature
  }
}
