//! API handle for model-related operations.

use crate::error::Error;
use super::super::Client;

  /// API handle for model-related operations.
  #[ derive( Debug ) ]
  pub struct ModelsApi< 'a >
  {
      pub( crate ) client : &'a Client,
  }

  impl ModelsApi< '_ >
  {
    /// Batch generate content for multiple requests.
    ///
    /// This method allows you to send multiple content generation requests in a single API call,
    /// improving efficiency when processing multiple inputs. Each request in the batch is processed
    /// independently and returns its own response.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The name of the model to use for generation
    /// * `request` - The batch request containing multiple generation requests
    ///
    /// # Returns
    ///
    /// Returns a [`BatchGenerateContentResponse`] containing:
    /// - `responses`: Vector of [`GenerateContentResponse`] objects, one for each input request
    ///
    /// # Errors
    ///
    /// This method returns an error in the following cases:
    /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
    /// - [`Error::AuthenticationError`] - Invalid or missing API key
    /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
    /// - [`Error::DeserializationError`] - Failed to parse the API response
    /// - [`Error::ApiError`] - Other API-related errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # use api_gemini::models::*;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let models_api = client.models();
    ///
    /// // Create multiple generation requests
    /// let requests = vec![
    ///   GenerateContentRequest {
    ///     contents : vec![Content {
    ///       role : "user".to_string(),
    ///       parts : vec![Part { text : Some("Explain AI".to_string()), ..Default::default() }],
    ///     }],
    ///     ..Default::default()
    ///   },
    ///   GenerateContentRequest {
    ///     contents : vec![Content {
    ///       role : "user".to_string(),
    ///       parts : vec![Part { text : Some("Explain ML".to_string()), ..Default::default() }],
    ///     }],
    ///     ..Default::default()
    ///   },
    /// ];
    ///
    /// let batch_request = BatchGenerateContentRequest { requests };
    /// let response = models_api.batch_generate_content("gemini-2.5-flash", &batch_request).await?;
    ///
    /// for (i, response) in response.responses.iter().enumerate() {
    ///   println!("Response {}: {:?}", i, response);
    /// }
    /// # Ok( () )
    /// # }
    /// ```
    #[ inline ]
    pub async fn batch_generate_content(
      &self,
      model_name : &str,
      request : &crate::models::BatchGenerateContentRequest
    ) -> Result< crate::models::BatchGenerateContentResponse, Error >
    {
      let url = format!( "{}/v1beta/models/{model_name}:batchGenerateContent", self.client.base_url );

      crate ::internal::http::execute_legacy::< crate::models::BatchGenerateContentRequest, crate::models::BatchGenerateContentResponse >
      (
        &self.client.http,
        reqwest ::Method::POST,
        &url,
        &self.client.api_key,
        Some( request ),
      )
      .await
    }

    /// Batch embed content for multiple requests.
    ///
    /// This method allows you to generate embeddings for multiple pieces of content in a single API call,
    /// improving efficiency when processing multiple texts. Each request in the batch is processed
    /// independently and returns its own embedding.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The name of the model to use for embeddings
    /// * `request` - The batch request containing multiple embedding requests
    ///
    /// # Returns
    ///
    /// Returns a [`BatchEmbedContentsResponse`] containing:
    /// - `embeddings`: Vector of [`ContentEmbedding`] objects, one for each input request
    ///
    /// # Errors
    ///
    /// This method returns an error in the following cases:
    /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
    /// - [`Error::AuthenticationError`] - Invalid or missing API key
    /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
    /// - [`Error::DeserializationError`] - Failed to parse the API response
    /// - [`Error::ApiError`] - Other API-related errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # use api_gemini::models::*;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let models_api = client.models();
    ///
    /// // Create multiple embedding requests
    /// let requests = vec![
    ///   EmbedContentRequest {
    ///     content : Content {
    ///       role : "user".to_string(),
    ///       parts : vec![Part { text : Some("Machine learning fundamentals".to_string()), ..Default::default() }],
    ///     },
    ///     ..Default::default()
    ///   },
    ///   EmbedContentRequest {
    ///     content : Content {
    ///       role : "user".to_string(),
    ///       parts : vec![Part { text : Some("Deep learning concepts".to_string()), ..Default::default() }],
    ///     },
    ///     ..Default::default()
    ///   },
    /// ];
    ///
    /// let batch_request = BatchEmbedContentsRequest { requests };
    /// let response = models_api.batch_embed_contents("text-embedding-004", &batch_request).await?;
    ///
    /// for (i, embedding) in response.embeddings.iter().enumerate() {
    ///   println!("Embedding {}: {} values", i, embedding.values.len());
    /// }
    /// # Ok( () )
    /// # }
    /// ```
    #[ inline ]
    pub async fn batch_embed_contents(
      &self,
      model_name : &str,
      request : &crate::models::BatchEmbedContentsRequest
    ) -> Result< crate::models::BatchEmbedContentsResponse, Error >
    {
      let url = format!( "{}/v1beta/models/{model_name}:batchEmbedContents", self.client.base_url );

      crate ::internal::http::execute_legacy::< crate::models::BatchEmbedContentsRequest, crate::models::BatchEmbedContentsResponse >
      (
        &self.client.http,
        reqwest ::Method::POST,
        &url,
        &self.client.api_key,
        Some( request ),
      )
      .await
    }

    /// Batch count tokens for multiple requests.
    ///
    /// This method allows you to count tokens for multiple pieces of content in a single API call,
    /// improving efficiency when analyzing token usage across multiple inputs. Each request in the batch
    /// is processed independently and returns its own token count.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The name of the model to use for token counting
    /// * `request` - The batch request containing multiple token counting requests
    ///
    /// # Returns
    ///
    /// Returns a [`BatchCountTokensResponse`] containing:
    /// - `responses`: Vector of [`CountTokensResponse`] objects, one for each input request
    ///
    /// # Errors
    ///
    /// This method returns an error in the following cases:
    /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
    /// - [`Error::AuthenticationError`] - Invalid or missing API key
    /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
    /// - [`Error::DeserializationError`] - Failed to parse the API response
    /// - [`Error::ApiError`] - Other API-related errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # use api_gemini::models::*;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let models_api = client.models();
    ///
    /// // Create multiple token counting requests
    /// let requests = vec![
    ///   CountTokensRequest {
    ///     contents : vec![Content {
    ///       role : "user".to_string(),
    ///       parts : vec![Part { text : Some("Explain machine learning".to_string()), ..Default::default() }],
    ///     }],
    ///     ..Default::default()
    ///   },
    ///   CountTokensRequest {
    ///     contents : vec![Content {
    ///       role : "user".to_string(),
    ///       parts : vec![Part { text : Some("What is deep learning?".to_string()), ..Default::default() }],
    ///     }],
    ///     ..Default::default()
    ///   },
    /// ];
    ///
    /// let batch_request = BatchCountTokensRequest { requests };
    /// let response = models_api.batch_count_tokens("gemini-2.5-flash", &batch_request).await?;
    ///
    /// for (i, response) in response.responses.iter().enumerate() {
    ///   println!("Request {}: {} tokens", i, response.total_tokens);
    /// }
    /// # Ok( () )
    /// # }
    /// ```
    #[ inline ]
    pub async fn batch_count_tokens(
      &self,
      model_name : &str,
      request : &crate::models::BatchCountTokensRequest
    ) -> Result< crate::models::BatchCountTokensResponse, Error >
    {
      // Validate input parameters
      if model_name.trim().is_empty()
      {
        return Err( Error::InvalidArgument( "Model name cannot be empty".to_string() ) );
      }

      // Validate the request structure
      if let Err( validation_error ) = crate::validation::validate_batch_count_tokens_request( request )
      {
        return Err( Error::InvalidArgument( format!( "Invalid request : {validation_error}" ) ) );
      }

      let url = format!( "{}/v1beta/models/{model_name}:batchCountTokens", self.client.base_url );

      crate ::internal::http::execute_legacy::< crate::models::BatchCountTokensRequest, crate::models::BatchCountTokensResponse >
      (
        &self.client.http,
        reqwest ::Method::POST,
        &url,
        &self.client.api_key,
        Some( request ),
      )
      .await
    }

    /// Analyze tokens with detailed breakdown and cost estimation.
    ///
    /// This method provides enhanced token analysis including detailed breakdown by content type,
    /// cost estimation, and optimization suggestions. It's useful for understanding token usage
    /// patterns and estimating API costs.
    ///
    /// # Arguments
    ///
    /// * `model_name` - The name of the model to use for token analysis
    /// * `request` - The analyze tokens request containing content and analysis options
    ///
    /// # Returns
    ///
    /// Returns an [`AnalyzeTokensResponse`] containing:
    /// - `total_tokens`: Total token count across all content
    /// - `breakdown`: Optional detailed breakdown by content type
    /// - `cost_estimate`: Optional cost estimation based on token usage
    /// - `optimization_suggestions`: Optional suggestions for reducing token usage
    ///
    /// # Errors
    ///
    /// This method returns an error in the following cases:
    /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
    /// - [`Error::AuthenticationError`] - Invalid or missing API key
    /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
    /// - [`Error::DeserializationError`] - Failed to parse the API response
    /// - [`Error::ApiError`] - Other API-related errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # use api_gemini::models::*;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let models_api = client.models();
    ///
    /// let request = AnalyzeTokensRequest {
    ///   contents : vec![Content {
    ///     role : "user".to_string(),
    ///     parts : vec![Part { text : Some("Analyze this complex prompt with detailed explanations".to_string()), ..Default::default() }],
    ///   }],
    ///   generate_content_request : None,
    ///   include_breakdown : Some(true),
    ///   estimate_generation_tokens : Some(true),
    /// };
    ///
    /// let response = models_api.analyze_tokens("gemini-2.5-flash", &request).await?;
    ///
    /// println!("Total tokens : {}", response.total_tokens);
    /// if let Some(breakdown) = response.token_breakdown {
    ///   if let Some(text_tokens) = breakdown.text_tokens {
    ///     println!("Text tokens : {}", text_tokens);
    ///   }
    ///   if let Some(image_tokens) = breakdown.image_tokens {
    ///     println!("Image tokens : {}", image_tokens);
    ///   }
    /// }
    /// if let Some(cost) = response.cost_estimate {
    ///   if let Some(total) = cost.total_cost {
    ///     println!("Estimated cost : ${:.4}", total);
    ///   }
    /// }
    /// # Ok( () )
    /// # }
    /// ```
    #[ inline ]
    pub async fn analyze_tokens(
      &self,
      model_name : &str,
      request : &crate::models::AnalyzeTokensRequest
    ) -> Result< crate::models::AnalyzeTokensResponse, Error >
    {
      // Validate input parameters
      if model_name.trim().is_empty()
      {
        return Err( Error::InvalidArgument( "Model name cannot be empty".to_string() ) );
      }

      // Validate the request structure
      if let Err( validation_error ) = crate::validation::validate_analyze_tokens_request( request )
      {
        return Err( Error::InvalidArgument( format!( "Invalid request : {validation_error}" ) ) );
      }

      let url = format!( "{}/v1beta/models/{model_name}:analyzeTokens", self.client.base_url );

      crate ::internal::http::execute_legacy::< crate::models::AnalyzeTokensRequest, crate::models::AnalyzeTokensResponse >
      (
        &self.client.http,
        reqwest ::Method::POST,
        &url,
        &self.client.api_key,
        Some( request ),
      )
      .await
    }

    /// Compare multiple models across various criteria.
    ///
    /// This method allows you to compare multiple models side-by-side based on performance metrics,
    /// cost analysis, and suitability for specific use cases. It provides comprehensive analysis
    /// to help you choose the best model for your needs.
    ///
    /// # Arguments
    ///
    /// * `request` - The compare models request containing model names and comparison criteria
    ///
    /// # Returns
    ///
    /// Returns a [`CompareModelsResponse`] containing:
    /// - `comparisons`: Vector of [`ModelComparison`] objects with detailed analysis for each model
    /// - `recommendation`: Optional overall recommendation based on the comparison
    ///
    /// # Errors
    ///
    /// This method returns an error in the following cases:
    /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
    /// - [`Error::AuthenticationError`] - Invalid or missing API key
    /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
    /// - [`Error::DeserializationError`] - Failed to parse the API response
    /// - [`Error::ApiError`] - Other API-related errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # use api_gemini::models::*;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let models_api = client.models();
    ///
    /// let request = CompareModelsRequest {
    ///   model_names : vec![
    ///     "gemini-2.5-flash".to_string(),
    ///     "gemini-1.5-pro-002".to_string(),
    ///     "text-embedding-004".to_string(),
    ///   ],
    ///   criteria : Some(vec!["performance".to_string(), "cost".to_string()]),
    ///   include_benchmarks : Some(true),
    ///   include_cost_analysis : Some(true),
    /// };
    ///
    /// let response = models_api.compare_models(&request).await?;
    ///
    /// for comparison in response.comparisons {
    ///   println!("Model : {}", comparison.model.name);
    ///   if let Some(metrics) = comparison.performance_metrics {
    ///     println!("  Quality Score : {:?}", metrics.quality_score);
    ///   }
    /// }
    /// # Ok( () )
    /// # }
    /// ```
    #[ inline ]
    pub async fn compare_models(
      &self,
      request : &crate::models::CompareModelsRequest
    ) -> Result< crate::models::CompareModelsResponse, Error >
    {
      // Validate the request structure
      if let Err( validation_error ) = crate::validation::validate_compare_models_request( request )
      {
        return Err( Error::InvalidArgument( format!( "Invalid request : {validation_error}" ) ) );
      }

      let url = format!( "{}/v1beta/models:compare", self.client.base_url );

      crate ::internal::http::execute_legacy::< crate::models::CompareModelsRequest, crate::models::CompareModelsResponse >
      (
        &self.client.http,
        reqwest ::Method::POST,
        &url,
        &self.client.api_key,
        Some( request ),
      )
      .await
    }

    /// Get model recommendations based on use case and requirements.
    ///
    /// This method analyzes your specific use case and requirements to recommend the most
    /// suitable models. It considers factors like performance needs, budget constraints,
    /// and real-time requirements to provide tailored recommendations.
    ///
    /// # Arguments
    ///
    /// * `request` - The recommendations request containing use case and requirements
    ///
    /// # Returns
    ///
    /// Returns a [`GetRecommendationsResponse`] containing:
    /// - `recommendations`: Vector of [`ModelRecommendation`] objects in priority order
    /// - `use_case_analysis`: Optional analysis of the use case requirements
    ///
    /// # Errors
    ///
    /// This method returns an error in the following cases:
    /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
    /// - [`Error::AuthenticationError`] - Invalid or missing API key
    /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
    /// - [`Error::DeserializationError`] - Failed to parse the API response
    /// - [`Error::ApiError`] - Other API-related errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # use api_gemini::models::*;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let models_api = client.models();
    ///
    /// let request = GetRecommendationsRequest {
    ///   use_case : "Real-time chatbot for customer support".to_string(),
    ///   input_size_range : Some("short".to_string()),
    ///   performance_requirements : Some(vec!["low-latency".to_string(), "high-quality".to_string()]),
    ///   budget_constraints : Some(100.0),
    ///   real_time_required : Some(true),
    /// };
    ///
    /// let response = models_api.get_recommendations(&request).await?;
    ///
    /// for recommendation in response.recommendations {
    ///   println!("Recommended : {} (confidence : {:.2})",
    ///     recommendation.recommended_model,
    ///     recommendation.confidence_score
    ///   );
    ///   println!("  Reasoning : {}", recommendation.reasoning);
    /// }
    /// # Ok( () )
    /// # }
    /// ```
    #[ inline ]
    pub async fn get_recommendations(
      &self,
      request : &crate::models::GetRecommendationsRequest
    ) -> Result< crate::models::GetRecommendationsResponse, Error >
    {
      // Validate the request structure
      if let Err( validation_error ) = crate::validation::validate_get_recommendations_request( request )
      {
        return Err( Error::InvalidArgument( format!( "Invalid request : {validation_error}" ) ) );
      }

      let url = format!( "{}/v1beta/models:recommend", self.client.base_url );

      crate ::internal::http::execute_legacy::< crate::models::GetRecommendationsRequest, crate::models::GetRecommendationsResponse >
      (
        &self.client.http,
        reqwest ::Method::POST,
        &url,
        &self.client.api_key,
        Some( request ),
      )
      .await
    }

    /// Filter models using advanced criteria.
    ///
    /// This method provides advanced filtering capabilities to find models that match
    /// specific criteria such as performance thresholds, cost limits, and feature requirements.
    /// Results can be sorted by various criteria for optimal selection.
    ///
    /// # Arguments
    ///
    /// * `request` - The advanced filter request containing filtering and sorting criteria
    ///
    /// # Returns
    ///
    /// Returns an [`AdvancedFilterResponse`] containing:
    /// - `models`: Filtered and sorted vector of [`Model`] objects
    /// - `total_matches`: Total number of models that matched the criteria
    /// - `applied_filters`: Optional summary of applied filters
    ///
    /// # Errors
    ///
    /// This method returns an error in the following cases:
    /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
    /// - [`Error::AuthenticationError`] - Invalid or missing API key
    /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
    /// - [`Error::DeserializationError`] - Failed to parse the API response
    /// - [`Error::ApiError`] - Other API-related errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # use api_gemini::models::*;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let models_api = client.models();
    ///
    /// let request = AdvancedFilterRequest {
    ///   capabilities : Some(vec!["generateContent".to_string()]),
    ///   max_cost_per_1k : Some(0.001),
    ///   min_quality_score : Some(0.8),
    ///   max_response_time : Some(1000.0),
    ///   supports_streaming : Some(true),
    ///   sort_by : Some("cost".to_string()),
    /// };
    ///
    /// let response = models_api.advanced_filter(&request).await?;
    ///
    /// println!("Found {} models matching criteria", response.total_matches);
    /// for model in response.models {
    ///   println!("  {}: {}", model.name, model.description.unwrap_or_default());
    /// }
    /// # Ok( () )
    /// # }
    /// ```
    #[ inline ]
    pub async fn advanced_filter(
      &self,
      request : &crate::models::AdvancedFilterRequest
    ) -> Result< crate::models::AdvancedFilterResponse, Error >
    {
      // Validate the request structure
      if let Err( validation_error ) = crate::validation::validate_advanced_filter_request( request )
      {
        return Err( Error::InvalidArgument( format!( "Invalid request : {validation_error}" ) ) );
      }

      let url = format!( "{}/v1beta/models:filter", self.client.base_url );

      crate ::internal::http::execute_legacy::< crate::models::AdvancedFilterRequest, crate::models::AdvancedFilterResponse >
      (
        &self.client.http,
        reqwest ::Method::POST,
        &url,
        &self.client.api_key,
        Some( request ),
      )
      .await
    }

    /// Get status and availability information for models.
    ///
    /// This method provides real-time status information for models including availability,
    /// health metrics, and any ongoing issues or maintenance windows. It's useful for
    /// monitoring model availability and planning usage accordingly.
    ///
    /// # Arguments
    ///
    /// * `request` - The model status request containing model names and options
    ///
    /// # Returns
    ///
    /// Returns a [`ModelStatusResponse`] containing:
    /// - `model_statuses`: Vector of [`ModelStatus`] objects with status for each model
    /// - `service_health`: Optional overall service health information
    ///
    /// # Errors
    ///
    /// This method returns an error in the following cases:
    /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
    /// - [`Error::AuthenticationError`] - Invalid or missing API key
    /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
    /// - [`Error::DeserializationError`] - Failed to parse the API response
    /// - [`Error::ApiError`] - Other API-related errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use api_gemini::client::Client;
    /// # use api_gemini::models::*;
    /// # #[ tokio::main ]
    /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
    /// let client = Client::new()?;
    /// let models_api = client.models();
    ///
    /// let request = ModelStatusRequest {
    ///   model_names : vec![
    ///     "gemini-2.5-flash".to_string(),
    ///     "gemini-1.5-pro-002".to_string(),
    ///   ],
    ///   include_health_metrics : Some(true),
    /// };
    ///
    /// let response = models_api.get_model_status(&request).await?;
    ///
    /// for status in response.model_statuses {
    ///   println!("Model : {} - Status : {}", status.model_name, status.status);
    ///   if let Some(health) = status.health_percentage {
    ///     println!("  Health : {:.1}%", health);
    ///   }
    /// }
    /// # Ok( () )
    /// # }
    /// ```
    #[ inline ]
    pub async fn get_model_status(
      &self,
      request : &crate::models::ModelStatusRequest
    ) -> Result< crate::models::ModelStatusResponse, Error >
    {
      // Validate the request structure
      if let Err( validation_error ) = crate::validation::validate_model_status_request( request )
      {
        return Err( Error::InvalidArgument( format!( "Invalid request : {validation_error}" ) ) );
      }

      let url = format!( "{}/v1beta/models:status", self.client.base_url );

      crate ::internal::http::execute_legacy::< crate::models::ModelStatusRequest, crate::models::ModelStatusResponse >
      (
        &self.client.http,
        reqwest ::Method::POST,
        &url,
        &self.client.api_key,
        Some( request ),
      )
      .await
    }
  }
