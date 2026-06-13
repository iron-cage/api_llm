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

  }
