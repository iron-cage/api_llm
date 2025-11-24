//! Comprehensive tests for enhanced model details functionality.
//!
//! This file implements comprehensive failing tests for enhanced model details
//! following TDD principles. Tests cover detailed metadata retrieval, pricing
//! information, capabilities, limitations, context windows, and lifecycle management.

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  environment ::OpenaiEnvironmentImpl,
  secret ::Secret,
  components ::models::
  {
    EnhancedModel,
    EnhancedListModelsResponse,
    ModelPricing,
    ModelCapabilities,
    ModelLimitations,
    ModelLifecycle,
    ModelStatus,
    ResponseMetadata,
    ModelComparison,
  },
};

use core::time::Duration;

/// Helper function to create test client
fn create_test_client() -> Result< Client< OpenaiEnvironmentImpl >, Box< dyn std::error::Error > >
{
  let secret = Secret::load_from_env( "OPENAI_API_KEY" )?;
  let env = OpenaiEnvironmentImpl::build( secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string() )?;
  Ok( Client::build( env )? )
}

/// Helper function to check if we should run integration tests
fn should_run_integration_tests() -> bool
{
  std ::env::var( "OPENAI_API_KEY" ).is_ok()
}

// === UNIT TESTS ===

#[ test ]
fn test_enhanced_model_structure()
{
  // Test that enhanced model structure contains pricing information
  let enhanced_model = EnhancedModel
  {
    id : "gpt-4".to_string(),
    created : 1_678_935_615,
    object : "model".to_string(),
    owned_by : "openai".to_string(),
    pricing : Some( ModelPricing
    {
      input_cost_per_1k_tokens : 0.03,
      output_cost_per_1k_tokens : 0.06,
      currency : "USD".to_string(),
      effective_date : "2024-01-01".to_string(),
    }),
    capabilities : ModelCapabilities
    {
      supports_function_calling : true,
      supports_vision : false,
      supports_streaming : true,
      max_context_window : 8192,
      max_output_tokens : 4096,
      supported_formats : vec![ "text".to_string() ],
    },
    limitations : ModelLimitations
    {
      rate_limit_rpm : Some( 3500 ),
      rate_limit_tpm : Some( 90000 ),
      concurrent_requests : Some( 200 ),
    },
    lifecycle : ModelLifecycle
    {
      status : ModelStatus::Active,
      deprecation_date : None,
      sunset_date : None,
      replacement_model : None,
    },
  };

  assert_eq!(enhanced_model.id, "gpt-4");
  assert!(enhanced_model.pricing.is_some());
  assert_eq!(enhanced_model.capabilities.max_context_window, 8192);
  assert_eq!(enhanced_model.lifecycle.status, ModelStatus::Active);
}

#[ test ]
fn test_model_pricing_structure()
{
  let pricing = ModelPricing
  {
    input_cost_per_1k_tokens : 0.01,
    output_cost_per_1k_tokens : 0.03,
    currency : "USD".to_string(),
    effective_date : "2024-01-01".to_string(),
  };

  assert!( ( pricing.input_cost_per_1k_tokens - 0.01 ).abs() < f64::EPSILON );
  assert!( ( pricing.output_cost_per_1k_tokens - 0.03 ).abs() < f64::EPSILON );
  assert_eq!(pricing.currency, "USD");
}

#[ test ]
fn test_model_capabilities_structure()
{
  let capabilities = ModelCapabilities
  {
    supports_function_calling : true,
    supports_vision : true,
    supports_streaming : true,
    max_context_window : 128_000,
    max_output_tokens : 4096,
    supported_formats : vec![ "text".to_string(), "image".to_string() ],
  };

  assert!(capabilities.supports_function_calling);
  assert!(capabilities.supports_vision);
  assert_eq!(capabilities.max_context_window, 128_000);
  assert_eq!(capabilities.supported_formats.len(), 2);
}

#[ test ]
fn test_model_limitations_structure()
{
  let limitations = ModelLimitations
  {
    rate_limit_rpm : Some( 500 ),
    rate_limit_tpm : Some( 30000 ),
    concurrent_requests : Some( 50 ),
  };

  assert_eq!(limitations.rate_limit_rpm, Some( 500 ));
  assert_eq!(limitations.rate_limit_tpm, Some( 30000 ));
  assert_eq!(limitations.concurrent_requests, Some( 50 ));
}

#[ test ]
fn test_model_lifecycle_structure()
{
  let lifecycle = ModelLifecycle
  {
    status : ModelStatus::Deprecated,
    deprecation_date : Some( "2024-06-01".to_string() ),
    sunset_date : Some( "2024-12-31".to_string() ),
    replacement_model : Some( "gpt-5.1-chat-latest".to_string() ),
  };

  assert_eq!(lifecycle.status, ModelStatus::Deprecated);
  assert!(lifecycle.deprecation_date.is_some());
  assert!(lifecycle.replacement_model.is_some());
}

#[ test ]
fn test_model_status_enum()
{
  assert_eq!(ModelStatus::Active.to_string(), "active");
  assert_eq!(ModelStatus::Deprecated.to_string(), "deprecated");
  assert_eq!(ModelStatus::Sunset.to_string(), "sunset");
  assert_eq!(ModelStatus::Beta.to_string(), "beta");
}

#[ test ]
fn test_enhanced_list_models_response_structure()
{
  let enhanced_response = EnhancedListModelsResponse
  {
    object : "list".to_string(),
    data : vec![ EnhancedModel
    {
      id : "gpt-4".to_string(),
      created : 1_678_935_615,
      object : "model".to_string(),
      owned_by : "openai".to_string(),
      pricing : Some( ModelPricing
      {
        input_cost_per_1k_tokens : 0.03,
        output_cost_per_1k_tokens : 0.06,
        currency : "USD".to_string(),
        effective_date : "2024-01-01".to_string(),
      }),
      capabilities : ModelCapabilities
      {
        supports_function_calling : true,
        supports_vision : false,
        supports_streaming : true,
        max_context_window : 8192,
        max_output_tokens : 4096,
        supported_formats : vec![ "text".to_string() ],
      },
      limitations : ModelLimitations
      {
        rate_limit_rpm : Some( 3500 ),
        rate_limit_tpm : Some( 90000 ),
        concurrent_requests : Some( 200 ),
      },
      lifecycle : ModelLifecycle
      {
        status : ModelStatus::Active,
        deprecation_date : None,
        sunset_date : None,
        replacement_model : None,
      },
    }],
    metadata : ResponseMetadata
    {
      total_models : 1,
      active_models : 1,
      deprecated_models : 0,
      beta_models : 0,
    },
  };

  assert_eq!(enhanced_response.data.len(), 1);
  assert_eq!(enhanced_response.metadata.total_models, 1);
  assert_eq!(enhanced_response.metadata.active_models, 1);
}

// === INTEGRATION TESTS ===

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_enhanced_model_details_retrieval()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: Must have valid API key
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_test_client().expect("Failed to create test client");

  // Test retrieving enhanced details for a specific model
  let result = client.models().retrieve_enhanced("gpt-5-nano").await;

  match result
  {
    Ok(enhanced_model) =>
    {
      assert!(!enhanced_model.id.is_empty());
      assert_eq!(enhanced_model.object, "model");

      // Verify pricing information is present
      assert!(enhanced_model.pricing.is_some());
      let pricing = enhanced_model.pricing.unwrap();
      assert!(pricing.input_cost_per_1k_tokens > 0.0);
      assert_eq!(pricing.currency, "USD");

      // Verify capabilities
      assert!(enhanced_model.capabilities.max_context_window > 0);
      assert!(enhanced_model.capabilities.max_output_tokens > 0);

      // Verify lifecycle information
      assert_ne!(enhanced_model.lifecycle.status, ModelStatus::Sunset);
    },
    Err(e) => panic!("Expected successful enhanced model retrieval, got error : {e:?}"),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_enhanced_model_list_with_metadata()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: Must have valid API key
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_test_client().expect("Failed to create test client");

  let result = client.models().list_enhanced().await;

  match result
  {
    Ok(response) =>
    {
      assert_eq!(response.object, "list");
      assert!(!response.data.is_empty());

      // Verify metadata
      assert!(response.metadata.total_models > 0);
      assert_eq!(response.metadata.total_models, u32::try_from(response.data.len()).unwrap_or(0));

      // Verify at least one model has pricing info
      let models_with_pricing = response.data.iter()
        .filter(|model| model.pricing.is_some())
        .count();
      assert!(models_with_pricing > 0, "At least one model should have pricing information");

      // Verify capabilities are present
      for model in &response.data
      {
        assert!(model.capabilities.max_context_window > 0);
        assert!(!model.capabilities.supported_formats.is_empty());
      }
    },
    Err(e) => panic!("Expected successful enhanced model list, got error : {e:?}"),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_model_pricing_accuracy()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: Must have valid API key
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_test_client().expect("Failed to create test client");

  let result = client.models().retrieve_enhanced("gpt-4").await;

  match result
  {
    Ok(model) =>
    {
      if let Some(pricing) = model.pricing
      {
        // Verify pricing is reasonable (not zero, not negative)
        assert!(pricing.input_cost_per_1k_tokens > 0.0);
        assert!(pricing.output_cost_per_1k_tokens > 0.0);
        assert!(pricing.output_cost_per_1k_tokens >= pricing.input_cost_per_1k_tokens);
        assert_eq!(pricing.currency, "USD");
        assert!(!pricing.effective_date.is_empty());
      }
      else
      {
        panic!("Expected pricing information for gpt-4");
      }
    },
    Err(e) => panic!("Expected successful model retrieval, got error : {e:?}"),
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_model_capabilities_validation()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: Must have valid API key
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_test_client().expect("Failed to create test client");

  let models_to_test = vec![ "gpt-4", "gpt-5-nano", "text-embedding-ada-002" ];

  for model_id in models_to_test
  {
    let result = client.models().retrieve_enhanced(model_id).await;

    match result
    {
      Ok(model) =>
      {
        // Verify context window is reasonable
        assert!(model.capabilities.max_context_window >= 1024,
                "Model {model_id} should have at least 1024 context window");
        assert!(model.capabilities.max_context_window <= 2_000_000,
                "Model {model_id} context window seems unreasonably large");

        // Verify output tokens don't exceed context window
        assert!(model.capabilities.max_output_tokens <= model.capabilities.max_context_window,
                "Model {model_id} output tokens exceed context window");

        // Verify supported formats
        assert!(!model.capabilities.supported_formats.is_empty(),
                "Model {model_id} should support at least one format");
      },
      Err(e) => panic!("Failed to retrieve enhanced details for {model_id}: {e:?}"),
    }
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_model_lifecycle_information()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: Must have valid API key
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_test_client().expect("Failed to create test client");

  let result = client.models().list_enhanced().await;

  match result
  {
    Ok(response) =>
    {
      // Find models with different lifecycle statuses
      let active_models = response.data.iter()
        .filter(|m| m.lifecycle.status == ModelStatus::Active)
        .count();

      let _deprecated_models = response.data.iter()
        .filter(|m| m.lifecycle.status == ModelStatus::Deprecated)
        .count();

      // Should have at least some active models
      assert!(active_models > 0, "Should have at least one active model");

      // Verify deprecated models have proper information
      for model in &response.data
      {
        if model.lifecycle.status == ModelStatus::Deprecated
        {
          assert!(model.lifecycle.deprecation_date.is_some(),
                  "Deprecated model {} should have deprecation date", model.id);

          if model.lifecycle.sunset_date.is_some()
          {
            // If sunset date exists, it should be after deprecation date
            let dep_date = model.lifecycle.deprecation_date.as_ref().unwrap();
            let sun_date = model.lifecycle.sunset_date.as_ref().unwrap();
            assert!(sun_date >= dep_date,
                    "Sunset date should be after deprecation date for model {}", model.id);
          }
        }
      }
    },
    Err(e) => panic!("Expected successful model list, got error : {e:?}"),
  }
}

#[ test ]
fn test_model_comparison_functionality()
{
  let model_a = create_test_enhanced_model("gpt-4", 8192, 0.03, 0.06);
  let model_b = create_test_enhanced_model("gpt-5-nano", 4096, 0.001, 0.002);

  let comparison = ModelComparison::compare(&model_a, &model_b);

  assert!( ( comparison.context_window_ratio - 2.0 ).abs() < f64::EPSILON ); // 8192 / 4096
  assert!(comparison.cost_efficiency_ratio > 10.0); // GPT-4 is more expensive
  assert_eq!(comparison.capability_score_diff, 0); // Both text-only models
}

#[ test ]
fn test_pricing_calculation_utilities()
{
  let pricing = ModelPricing
  {
    input_cost_per_1k_tokens : 0.01,
    output_cost_per_1k_tokens : 0.03,
    currency : "USD".to_string(),
    effective_date : "2024-01-01".to_string(),
  };

  // Test cost calculation for a conversation
  let input_tokens = 1500;
  let output_tokens = 500;
  let expected_cost = (1500.0 / 1000.0) * 0.01 + (500.0 / 1000.0) * 0.03;

  let calculated_cost = pricing.calculate_cost(input_tokens, output_tokens);
  assert!((calculated_cost - expected_cost).abs() < 0.0001);
}

// === PERFORMANCE BENCHMARKS ===

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_enhanced_model_details_performance()
{
  // INTEGRATION TEST - STRICT FAILURE POLICY: Must have valid API key
  if !should_run_integration_tests()
  {
    eprintln!("Skipping integration test : OPENAI_API_KEY not available");
    return;
  }

  let client = create_test_client().expect("Failed to create test client");

  let start = std::time::Instant::now();

  // Test bulk retrieval performance
  let models = vec![ "gpt-4", "gpt-5-nano", "text-embedding-ada-002" ];

  for model_id in models
  {
    let _result = client.models().retrieve_enhanced(model_id).await
      .expect("Model retrieval should succeed");
  }

  let elapsed = start.elapsed();

  // Should complete within reasonable time (5 seconds for 3 models)
  assert!(elapsed < Duration::from_secs(5),
          "Enhanced model details retrieval took too long : {elapsed:?}");
}

#[ test ]
fn test_model_serialization_deserialization()
{
  let enhanced_model = create_test_enhanced_model("gpt-4", 8192, 0.03, 0.06);

  let serialized = serde_json::to_string(&enhanced_model)
    .expect("Enhanced model should serialize");

  assert!(serialized.contains("\"pricing\""));
  assert!(serialized.contains("\"capabilities\""));
  assert!(serialized.contains("\"limitations\""));
  assert!(serialized.contains("\"lifecycle\""));

  let deserialized : EnhancedModel = serde_json::from_str(&serialized)
    .expect("Enhanced model should deserialize");

  assert_eq!(deserialized.id, enhanced_model.id);
  assert_eq!(deserialized.capabilities.max_context_window, 8192);
}

// === HELPER FUNCTIONS ===

/// Create a test enhanced model with specified parameters
fn create_test_enhanced_model(
  id : &str,
  context_window : u32,
  input_cost : f64,
  output_cost : f64
) -> EnhancedModel
{
  EnhancedModel
  {
    id : id.to_string(),
    created : 1_678_935_615,
    object : "model".to_string(),
    owned_by : "openai".to_string(),
    pricing : Some( ModelPricing
    {
      input_cost_per_1k_tokens : input_cost,
      output_cost_per_1k_tokens : output_cost,
      currency : "USD".to_string(),
      effective_date : "2024-01-01".to_string(),
    }),
    capabilities : ModelCapabilities
    {
      supports_function_calling : true,
      supports_vision : false,
      supports_streaming : true,
      max_context_window : context_window,
      max_output_tokens : context_window / 2,
      supported_formats : vec![ "text".to_string() ],
    },
    limitations : ModelLimitations
    {
      rate_limit_rpm : Some( 3500 ),
      rate_limit_tpm : Some( 90000 ),
      concurrent_requests : Some( 200 ),
    },
    lifecycle : ModelLifecycle
    {
      status : ModelStatus::Active,
      deprecation_date : None,
      sunset_date : None,
      replacement_model : None,
    },
  }
}

// Structures are now defined in api_openai::components::models module