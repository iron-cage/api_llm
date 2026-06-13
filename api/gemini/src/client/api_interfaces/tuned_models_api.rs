//! API handle for tuned model operations.

use crate::error::Error;
use super::super::Client;

/// API handle for tuned model operations.
///
/// Provides access to model tuning and fine-tuning capabilities including
/// creating, listing, and managing custom tuned models.
#[ derive( Debug ) ]

pub struct TunedModelsApi< 'a >
{
    pub( crate ) client : &'a Client,
}

impl TunedModelsApi< '_ >
{
  /// Create a new tuned model.
  ///
  /// This method initiates the creation of a tuned model based on a base model
  /// with custom training data and parameters.
  ///
  /// # Arguments
  ///
  /// * `request` - The create tuned model request containing model configuration and training data
  ///
  /// # Returns
  ///
  /// Returns a [`TunedModel`] with the created model information and training status.
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
  /// let tuned_models_api = client.tuned_models();
  ///
  /// let tuned_model = TunedModel {
  ///   name : "projects/my-project/locations/us-central1/models/my-tuned-model".to_string(),
  ///   display_name : Some("My Custom Model".to_string()),
  ///   description : Some("A model tuned for specific tasks".to_string()),
  ///   base_model : "models/gemini-1.5-pro-002".to_string(),
  ///   state : None,
  ///   create_time : None,
  ///   update_time : None,
  ///   tuning_task : None,
  ///   tuned_model_source : None,
  ///   temperature : Some(0.7),
  ///   top_p : Some(0.9),
  ///   top_k : Some(40),
  /// };
  ///
  /// let request = CreateTunedModelRequest {
  ///   tuned_model,
  ///   tuned_model_id : Some("my-tuned-model".to_string()),
  /// };
  ///
  /// let response = tuned_models_api.create(&request).await?;
  /// println!("Created tuned model : {}", response.name);
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn create(
    &self,
    request : &crate::models::CreateTunedModelRequest
  ) -> Result< crate::models::TunedModel, Error >
  {
    let url = format!( "{}/v1beta/tunedModels", self.client.base_url );

    crate ::internal::http::enterprise::execute_with_optional_retries::< crate::models::CreateTunedModelRequest, crate::models::TunedModel >
    (
      self.client,
      reqwest ::Method::POST,
      &url,
      &self.client.api_key,
      Some( request ),
    )
    .await
  }

  /// List all tuned models.
  ///
  /// This method retrieves a list of tuned models accessible to the current user,
  /// with optional pagination and filtering capabilities.
  ///
  /// # Arguments
  ///
  /// * `request` - The list tuned models request with pagination and filter options
  ///
  /// # Returns
  ///
  /// Returns a [`ListTunedModelsResponse`] containing:
  /// - `tuned_models`: Vector of [`TunedModel`] objects
  /// - `next_page_token`: Optional token for retrieving the next page of results
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
  /// let tuned_models_api = client.tuned_models();
  ///
  /// let request = ListTunedModelsRequest {
  ///   page_size : Some(10),
  ///   page_token : None,
  ///   filter : None,
  /// };
  ///
  /// let response = tuned_models_api.list(&request).await?;
  ///
  /// for model in response.tuned_models {
  ///   println!("Tuned model : {} - {}", model.name, model.display_name.unwrap_or_default());
  /// }
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn list(
    &self,
    request : &crate::models::ListTunedModelsRequest
  ) -> Result< crate::models::ListTunedModelsResponse, Error >
  {
    let mut url = format!( "{}/v1beta/tunedModels", self.client.base_url );
    let mut params = Vec::new();

    if let Some( page_size ) = request.page_size
    {
      params.push( format!( "pageSize={page_size}" ) );
    }

    if let Some( page_token ) = &request.page_token
    {
      params.push( format!( "pageToken={page_token}" ) );
    }

    if let Some( filter ) = &request.filter
    {
      params.push( format!( "filter={}", urlencoding::encode( filter ) ) );
    }

    if !params.is_empty()
    {
      url.push( '?' );
      url.push_str( &params.join( "&" ) );
    }

    crate ::internal::http::enterprise::execute_with_optional_retries::< (), crate::models::ListTunedModelsResponse >
    (
      self.client,
      reqwest ::Method::GET,
      &url,
      &self.client.api_key,
      None,
    )
    .await
  }

  /// Get information about a specific tuned model.
  ///
  /// This method retrieves detailed information about a specific tuned model
  /// including its configuration, training status, and performance metrics.
  ///
  /// # Arguments
  ///
  /// * `name` - The full name of the tuned model to retrieve
  ///
  /// # Returns
  ///
  /// Returns a [`TunedModel`] with complete model information.
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
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let tuned_models_api = client.tuned_models();
  ///
  /// let model = tuned_models_api.get("tunedModels/my-model-id").await?;
  /// println!("Model : {} - Status : {}", model.name, model.state.unwrap_or_default());
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn get( &self, name : &str ) -> Result< crate::models::TunedModel, Error >
  {
    let url = format!( "{}/v1beta/{}", self.client.base_url, name );

    crate ::internal::http::enterprise::execute_with_optional_retries::< (), crate::models::TunedModel >
    (
      self.client,
      reqwest ::Method::GET,
      &url,
      &self.client.api_key,
      None,
    )
    .await
  }

  /// Delete a tuned model.
  ///
  /// This method permanently deletes a tuned model and all associated data.
  /// This action cannot be undone.
  ///
  /// # Arguments
  ///
  /// * `name` - The full name of the tuned model to delete
  ///
  /// # Returns
  ///
  /// Returns `Ok(())` if the deletion was successful.
  ///
  /// # Errors
  ///
  /// This method returns an error in the following cases:
  /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
  /// - [`Error::ApiError`] - Other API-related errors
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let tuned_models_api = client.tuned_models();
  ///
  /// tuned_models_api.delete("tunedModels/my-model-id").await?;
  /// println!("Tuned model deleted successfully");
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn delete( &self, name : &str ) -> Result< (), Error >
  {
    let url = format!( "{}/v1beta/{}", self.client.base_url, name );

    let response = crate::internal::http::execute_raw
    (
      &self.client.http,
      reqwest ::Method::DELETE,
      &url,
      &self.client.api_key,
      None::< &()>,
    )
    .await?;

    if response.status().is_success()
    {
      Ok( () )
    }
    else
    {
      let error_text = response.text().await.unwrap_or_else( |_| "Failed to read error response".to_string() );
      Err( Error::ApiError( format!( "Failed to delete tuned model : {error_text}" ) ) )
    }
  }
}
