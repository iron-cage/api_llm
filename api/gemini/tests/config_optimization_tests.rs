//! Integration test for configuration optimizations

use api_gemini::models::config::*;
use std::time::Duration;

#[ test ]
fn test_configuration_optimizations()
{
  // Test basic configuration building
  let config = DynamicConfig::builder()
  .timeout(Duration::from_secs(30))
  .retry_attempts(3)
  .base_url("https://generativelanguage.googleapis.com".to_string())
  .source_priority(75)
  .tag("environment".to_string(), "test".to_string())
  .build()
  .expect("Config should build successfully");

  assert_eq!(config.timeout, Duration::from_secs(30));
  assert_eq!(config.retry_attempts, 3);
  assert_eq!(config.base_url, "https://generativelanguage.googleapis.com");
  assert_eq!(config.source_priority, Some(75));
  assert_eq!(config.tags.get("environment"), Some(&"test".to_string()));

  // Test configuration comparison
  let config2 = DynamicConfig::builder()
  .timeout(Duration::from_secs(60))
  .retry_attempts(3)
  .base_url("https://generativelanguage.googleapis.com".to_string())
  .source_priority(80)
  .tag("environment".to_string(), "test".to_string())
  .build()
  .expect("Config2 should build successfully");

  assert!(config.has_changes(&config2), "Configurations should be different");

  // Test configuration merging
  let merged = config.merge_with(&config2);
  assert_eq!(merged.timeout, Duration::from_secs(60)); // Should use higher priority

  // Test configuration metrics
  let metrics = ConfigMetrics::default();
  metrics.record_update(1000); // 1ms
  metrics.record_cache_hit();
  metrics.record_cache_miss();

  let report = metrics.generate_report();
  assert_eq!(report.total_updates, 1);
  assert!(report.cache_hit_ratio > 0.0);

  // Test rollback analysis
  let analysis = RollbackAnalysis::analyze_rollback(&config2, &config);
  assert!(!analysis.changed_fields.is_empty());
  assert!(analysis.is_safe); // Low impact change should be safe

  println!("✓ All configuration optimization tests passed!");
}