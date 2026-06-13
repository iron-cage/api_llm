use super::*;

mod authentication_test;
#[ cfg( all( feature = "batch-processing", feature = "error-handling" ) ) ]
mod batch_messages_test;
#[ cfg( feature = "buffered-streaming" ) ]
mod buffered_streaming_test;
mod circuit_breaker_test;
#[ cfg( feature = "compression" ) ]
mod compression_test;
mod comprehensive_integration_test;
mod content_generation_test;
mod content_generation_refactor_test;
mod core_client_test;
mod curl_diagnostics_test;
#[ cfg( feature = "dynamic-config" ) ]
mod dynamic_config_test;
#[ cfg( all( feature = "retry-logic", feature = "error-handling" ) ) ]
mod enhanced_retry_logic_test;
mod enterprise_configuration_test;
#[ cfg( feature = "enterprise-quota" ) ]
mod enterprise_quota_test;
mod enhanced_model_details_test;
mod error_handling_integration_test;
#[ cfg( feature = "failover" ) ]
mod failover_test;
mod general_diagnostics_test;
#[ cfg( feature = "health-checks" ) ]
mod health_checks_test;
mod error_handling_test;
mod example_model_validation_test;
mod examples_validation_test;
mod fallback_behavior_integration_test;
mod messages_api_test;
mod model_management_test;
mod performance_test;
mod performance_monitoring_test;
mod prompt_caching_tests;
#[ cfg( feature = "rate-limiting" ) ]
mod rate_limiting_test;
#[ cfg( feature = "request-templates" ) ]
mod request_templates_test;
mod retry_logic_test;
mod simple_integration_test;
mod spec_verification_integration_test;
mod streaming_test;
#[ cfg( feature = "streaming-control" ) ]
mod streaming_control_test;
mod structured_logging_test;
#[ cfg( feature = "sync-api" ) ]
mod sync_api_test;
#[ cfg( all( feature = "sync-api", feature = "streaming" ) ) ]
mod sync_streaming_test;
#[ cfg( feature = "sync-api" ) ]
mod sync_cached_content_test;
#[ cfg( feature = "tools" ) ]
mod enhanced_function_calling_test;
mod input_validation_test;
mod system_instructions_test;
mod token_counting_test;
mod token_validation_test;
mod tool_calling_test;
mod vision_support_test;
mod thin_client_principle_test;
mod testing_standards_test;
mod endpoint_coverage_test;
mod enterprise_reliability_test;
mod module_organization_test;
mod operation_test_specs;
