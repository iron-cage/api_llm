//! Cost Management Module Tests
//!
//! Tests for the enterprise cost tracking, budget management, and cost analytics functionality.

use api_openai::enterprise::
{
  CostTracker,
  BudgetLimits,
  CostAlert,
  AlertType,
  AlertSeverity,
  UsageBreakdown,
  TimeUsage,
  TokenUsage,
  CostOptimizationSettings,
};

#[ tokio::test ]
async fn test_cost_tracker_creation()
{
  let tracker = CostTracker::new();

  assert!( (tracker.daily_spend - 0.0).abs() < f64::EPSILON );
  assert!( (tracker.monthly_spend - 0.0).abs() < f64::EPSILON );
  assert!( tracker.cost_alerts.is_empty() );
  assert!( tracker.usage_breakdown.time_usage.is_empty() );
}

#[ tokio::test ]
async fn test_cost_tracker_with_limits()
{
  let tracker = CostTracker::with_limits( Some( 100.0 ), Some( 1000.0 ) );

  assert_eq!( tracker.budget_limits.daily_limit, Some( 100.0 ) );
  assert_eq!( tracker.budget_limits.monthly_limit, Some( 1000.0 ) );
  assert!( (tracker.budget_limits.alert_threshold - 0.8).abs() < f64::EPSILON ); // Default threshold
  assert!( !tracker.budget_limits.enforce_hard_limit ); // Default setting
}

#[ tokio::test ]
async fn test_budget_limits_defaults()
{
  let limits = BudgetLimits::default();

  assert_eq!( limits.daily_limit, None );
  assert_eq!( limits.monthly_limit, None );
  assert!( (limits.alert_threshold - 0.8).abs() < f64::EPSILON );
  assert!( !limits.enforce_hard_limit );
  assert!( !limits.optimization_settings.enabled );
}

#[ tokio::test ]
async fn test_cost_alert_generation()
{
  let mut tracker = CostTracker::with_limits( Some( 100.0 ), Some( 1000.0 ) );

  // Test warning alert at 80% threshold - only daily should trigger
  let alerts = tracker.update_spending( 85.0, 85.0 );
  assert_eq!( alerts.len(), 1 ); // Only daily warning (85% of 100 = 85% > 80%)

  // Verify alert types
  assert!( alerts.iter().any( | a | matches!( a.alert_type, AlertType::DailyLimitApproaching ) ) );

  // Test both alerts by triggering monthly as well with fresh tracker
  let mut tracker2 = CostTracker::with_limits( Some( 100.0 ), Some( 1000.0 ) );
  let alerts2 = tracker2.update_spending( 85.0, 850.0 ); // Both at 85% threshold
  assert_eq!( alerts2.len(), 2 ); // Both daily and monthly warnings
  assert!( alerts2.iter().any( | a | matches!( a.alert_type, AlertType::DailyLimitApproaching ) ) );
  assert!( alerts2.iter().any( | a | matches!( a.alert_type, AlertType::MonthlyLimitApproaching ) ) );

  // Verify alert severity
  assert!( alerts.iter().all( | a | a.severity == AlertSeverity::Warning ) );
}

#[ tokio::test ]
async fn test_cost_limit_exceeded()
{
  let mut tracker = CostTracker::with_limits( Some( 50.0 ), Some( 500.0 ) );

  // Exceed both limits
  let alerts = tracker.update_spending( 60.0, 600.0 );
  assert_eq!( alerts.len(), 2 ); // Daily and monthly exceeded

  // Verify alert types
  assert!( alerts.iter().any( | a | matches!( a.alert_type, AlertType::DailyLimitExceeded ) ) );
  assert!( alerts.iter().any( | a | matches!( a.alert_type, AlertType::MonthlyLimitExceeded ) ) );

  // Verify alert severity
  assert!( alerts.iter().all( | a | a.severity == AlertSeverity::Critical ) );
}

#[ tokio::test ]
async fn test_spending_reset()
{
  let mut tracker = CostTracker::with_limits( Some( 100.0 ), Some( 1000.0 ) );

  // Generate alerts
  tracker.update_spending( 120.0, 1200.0 );
  assert!( !tracker.cost_alerts.is_empty() );

  // Reset daily spending
  tracker.reset_daily_spending();
  assert!( (tracker.daily_spend - 0.0).abs() < f64::EPSILON );

  // Daily alerts should be removed
  assert!( !tracker.cost_alerts.iter().any( | alert |
    matches!( alert.alert_type, AlertType::DailyLimitApproaching | AlertType::DailyLimitExceeded )
  ) );

  // Monthly alerts should remain
  assert!( tracker.cost_alerts.iter().any( | alert |
    matches!( alert.alert_type, AlertType::MonthlyLimitApproaching | AlertType::MonthlyLimitExceeded )
  ) );
}

#[ tokio::test ]
async fn test_usage_breakdown_structure()
{
  let mut breakdown = UsageBreakdown::default();

  // Add time usage
  breakdown.time_usage.push( TimeUsage
  {
    start_time : 1000,
    end_time : 2000,
    request_count : 50,
    cost : 25.0,
  } );

  // Add token usage
  breakdown.token_usage.push( TokenUsage
  {
    token_type : "input".to_string(),
    count : 1000,
    cost_per_token : 0.01,
    total_cost : 10.0,
  } );

  // Add model usage
  breakdown.model_usage.insert( "gpt-4".to_string(), 15.0 );
  breakdown.model_usage.insert( "gpt-5-nano".to_string(), 5.0 );

  assert_eq!( breakdown.time_usage.len(), 1 );
  assert_eq!( breakdown.token_usage.len(), 1 );
  assert_eq!( breakdown.model_usage.len(), 2 );

  // Verify data integrity
  assert_eq!( breakdown.time_usage[ 0 ].request_count, 50 );
  assert_eq!( breakdown.token_usage[ 0 ].token_type, "input" );
  assert!( (breakdown.model_usage[ "gpt-4" ] - 15.0).abs() < f64::EPSILON );
}

#[ tokio::test ]
async fn test_cost_efficiency_ratio()
{
  let mut tracker = CostTracker::new();
  tracker.daily_spend = 50.0;

  // Add usage data
  tracker.usage_breakdown.time_usage.push( TimeUsage
  {
    start_time : 0,
    end_time : 1000,
    request_count : 100,
    cost : 50.0,
  } );

  let efficiency = tracker.get_cost_efficiency_ratio();
  assert!( efficiency > 0.0 );
  assert!( efficiency <= 10.0 ); // Reasonable upper bound
}

#[ tokio::test ]
async fn test_cost_optimization_settings()
{
  let settings = CostOptimizationSettings
  {
    enabled : true,
    prefer_cheaper_models : true,
    max_latency_increase_ms : 500,
    enable_request_batching : true,
    enable_response_caching : true,
  };

  assert!( settings.enabled );
  assert!( settings.prefer_cheaper_models );
  assert_eq!( settings.max_latency_increase_ms, 500 );
  assert!( settings.enable_request_batching );
  assert!( settings.enable_response_caching );
}

#[ tokio::test ]
async fn test_cost_alert_serialization()
{
  let alert = CostAlert
  {
    alert_type : AlertType::DailyLimitExceeded,
    severity : AlertSeverity::Critical,
    message : "Daily limit exceeded".to_string(),
    timestamp : 1_234_567_890,
    current_spend : 150.0,
    limit : 100.0,
  };

  // Test serialization
  let json = serde_json::to_string( &alert ).expect( "Serialization should work" );
  let deserialized : CostAlert = serde_json::from_str( &json ).expect( "Deserialization should work" );

  assert!( (alert.current_spend - deserialized.current_spend).abs() < f64::EPSILON );
  assert!( (alert.limit - deserialized.limit).abs() < f64::EPSILON );
  assert_eq!( alert.message, deserialized.message );
}

#[ tokio::test ]
async fn test_multiple_spending_updates()
{
  let mut tracker = CostTracker::with_limits( Some( 100.0 ), Some( 1000.0 ) );

  // Multiple small updates
  tracker.update_spending( 20.0, 20.0 );
  tracker.update_spending( 30.0, 30.0 );
  tracker.update_spending( 25.0, 25.0 );

  assert!( (tracker.daily_spend - 75.0).abs() < f64::EPSILON );
  assert!( ( tracker.monthly_spend - 75.0 ).abs() < f64::EPSILON );
  assert!( tracker.cost_alerts.is_empty() ); // Should be under threshold

  // Push over threshold
  let alerts = tracker.update_spending( 10.0, 10.0 );
  assert!( !alerts.is_empty() ); // Should generate warnings at 85% of limits
}

#[ tokio::test ]
async fn test_token_usage_calculations()
{
  let token_usage = TokenUsage
  {
    token_type : "output".to_string(),
    count : 500,
    cost_per_token : 0.002,
    total_cost : 1.0,
  };

  // Verify calculation consistency
  assert!( ( token_usage.count as f64 * token_usage.cost_per_token - token_usage.total_cost ).abs() < f64::EPSILON );
}

#[ tokio::test ]
async fn test_edge_cases()
{
  let mut tracker = CostTracker::with_limits( Some( 0.01 ), Some( 0.01 ) ); // Very low limits

  // Spending at 10% should NOT trigger alerts (below 80% threshold)
  let alerts = tracker.update_spending( 0.001, 0.001 );
  assert!( alerts.is_empty() );

  // Spending at 90% should trigger alerts
  let alerts2 = tracker.update_spending( 0.008, 0.008 ); // Total : 0.009, which is 90% of 0.01
  assert!( !alerts2.is_empty() );

  // Test with zero limits
  let mut zero_tracker = CostTracker::with_limits( Some( 0.0 ), Some( 0.0 ) );
  let zero_alerts = zero_tracker.update_spending( 0.001, 0.001 );
  assert_eq!( zero_alerts.len(), 2 ); // Should immediately exceed
}