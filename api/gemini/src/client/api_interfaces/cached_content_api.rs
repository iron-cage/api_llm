//! API handle for cached content management operations.

use crate::error::Error;
use crate::models::{ CreateCachedContentRequest, CachedContentResponse, ListCachedContentsResponse, UpdateCachedContentRequest };
use super::super::Client;

/// API handle for cached content management operations.
///
/// Provides direct access to server-side caching functionality without
/// client-side logic. All cache management decisions are explicit developer calls.
#[ derive( Debug ) ]

pub struct CachedContentApi< 'a >
{
    pub( crate ) client : &'a Client,
}

impl CachedContentApi< '_ >
{
  /// Create new cached content on the server.
  ///
  /// Direct server-side cache creation with no client-side logic or optimization.
  ///
  /// # Arguments
  ///
  /// * `request` - The create cached content request
  ///
  /// # Returns
  ///
  /// Returns the cached content response with details of the created cache
  ///
  /// # Errors
  ///
  /// Returns an error if the cached content creation fails
  #[ inline ]
  pub async fn create( &self, request : &CreateCachedContentRequest ) -> Result< CachedContentResponse, Error >
  {
    let url = format!( "{}/v1beta/cachedContents", self.client.base_url );

    crate ::internal::http::enterprise::execute_with_optional_retries::< CreateCachedContentRequest, CachedContentResponse >
    (
      self.client,
      reqwest ::Method::POST,
      &url,
      &self.client.api_key,
      Some( request ),
    )
    .await
  }

  /// List all cached contents
  ///
  /// # Arguments
  ///
  /// * `page_size` - Optional maximum number of cached contents to return per page
  /// * `page_token` - Optional token for retrieving subsequent pages
  ///
  /// # Returns
  ///
  /// Returns the list cached contents response with the available cache entries
  ///
  /// # Errors
  ///
  /// Returns an error if the listing operation fails
  #[ inline ]
  pub async fn list( &self, page_size : Option< i32 >, page_token : Option< &str > ) -> Result< ListCachedContentsResponse, Error >
  {
    let mut url = format!( "{}/v1beta/cachedContents", self.client.base_url );
    let mut query_params = Vec::new();

    if let Some( size ) = page_size
    {
      query_params.push( format!( "pageSize={size}" ) );
    }

    if let Some( token ) = page_token
    {
      query_params.push( format!( "pageToken={}", urlencoding::encode( token ) ) );
    }

    if !query_params.is_empty()
    {
      url.push( '?' );
      url.push_str( &query_params.join( "&" ) );
    }

    crate ::internal::http::enterprise::execute_with_optional_retries::< (), ListCachedContentsResponse >
    (
      self.client,
      reqwest ::Method::GET,
      &url,
      &self.client.api_key,
      None,
    )
    .await
  }

  /// Get a specific cached content by ID
  ///
  /// # Arguments
  ///
  /// * `cache_id` - The unique identifier of the cached content to retrieve
  ///
  /// # Returns
  ///
  /// Returns the cached content response with the requested cache details
  ///
  /// # Errors
  ///
  /// Returns an error if the cached content is not found or the request fails
  #[ inline ]
  pub async fn get( &self, cache_id : &str ) -> Result< CachedContentResponse, Error >
  {
    let url = format!( "{}/v1beta/cachedContents/{}", self.client.base_url, urlencoding::encode( cache_id ) );

    crate ::internal::http::enterprise::execute_with_optional_retries::< (), CachedContentResponse >
    (
      self.client,
      reqwest ::Method::GET,
      &url,
      &self.client.api_key,
      None,
    )
    .await
  }

  /// Update cached content properties
  ///
  /// # Arguments
  ///
  /// * `cache_id` - The unique identifier of the cached content to update
  /// * `request` - The update cached content request with the changes
  ///
  /// # Returns
  ///
  /// Returns the updated cached content response
  ///
  /// # Errors
  ///
  /// Returns an error if the update operation fails or the cache is not found
  #[ inline ]
  pub async fn update( &self, cache_id : &str, request : &UpdateCachedContentRequest ) -> Result< CachedContentResponse, Error >
  {
    let url = format!( "{}/v1beta/cachedContents/{}", self.client.base_url, urlencoding::encode( cache_id ) );

    crate ::internal::http::enterprise::execute_with_optional_retries::< UpdateCachedContentRequest, CachedContentResponse >
    (
      self.client,
      reqwest ::Method::PATCH,
      &url,
      &self.client.api_key,
      Some( request ),
    )
    .await
  }

  /// Delete cached content
  ///
  /// # Arguments
  ///
  /// * `cache_id` - The unique identifier of the cached content to delete
  ///
  /// # Returns
  ///
  /// Returns `Ok(())` if the cached content was successfully deleted
  ///
  /// # Errors
  ///
  /// Returns an error if the deletion fails or the cache is not found
  #[ inline ]
  pub async fn delete( &self, cache_id : &str ) -> Result< (), Error >
  {
    let url = format!( "{}/v1beta/cachedContents/{}", self.client.base_url, urlencoding::encode( cache_id ) );

    let _response : serde_json::Value = crate::internal::http::enterprise::execute_with_optional_retries
    (
      self.client,
      reqwest ::Method::DELETE,
      &url,
      &self.client.api_key,
      None::< &() >,
    )
    .await?;

    Ok( () )
  }
}
