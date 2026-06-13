//! API handle for file management operations.

use crate::error::Error;
use super::super::Client;

/// API handle for file management operations.
///
/// Provides direct access to server-side file upload, listing, and deletion functionality
/// without client-side logic. All file management decisions are explicit developer calls.
#[ derive( Debug ) ]

pub struct FilesApi< 'a >
{
    pub( crate ) client : &'a Client,
}

impl FilesApi< '_ >
{
  /// Upload a file to the Gemini API.
  ///
  /// This method uploads a file to the Gemini API for use in content generation
  /// and other operations. The file is stored server-side and can be referenced
  /// in subsequent API calls.
  ///
  /// # Arguments
  ///
  /// * `request` - The upload file request containing file data and metadata
  ///
  /// # Returns
  ///
  /// Returns an [`UploadFileResponse`] containing:
  /// - `file`: [`FileMetadata`] with file information including URI and processing state
  ///
  /// # Errors
  ///
  /// This method returns an error in the following cases:
  /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
  /// - [`Error::ApiError`] - File size limits exceeded or unsupported file type
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # use api_gemini::models::*;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let files_api = client.files();
  ///
  /// // Read file data
  /// let file_data = std::fs::read("example.png")?;
  ///
  /// let request = UploadFileRequest {
  ///   file_data,
  ///   mime_type : "image/png".to_string(),
  ///   display_name : Some("Example Image".to_string()),
  /// };
  ///
  /// let response = files_api.upload(&request).await?;
  /// println!("Uploaded file : {}", response.file.name);
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn upload( &self, request : &crate::models::UploadFileRequest ) -> Result< crate::models::UploadFileResponse, Error >
  {
    let url = format!( "{}/upload/v1beta/files", self.client.base_url );

    // Create multipart form for file upload
    let form = reqwest::multipart::Form::new()
      .part( "file", reqwest::multipart::Part::bytes( request.file_data.clone() )
        .mime_str( &request.mime_type.clone() )?
        .file_name( request.display_name.as_deref().unwrap_or( "file" ).to_string() ) );

    let response = self.client.http
      .post( &url )
      .header( "X-Goog-Api-Key", &self.client.api_key )
      .multipart( form )
      .send()
      .await
      .map_err( Error::from )?;

    if response.status().is_success()
    {
      let upload_response : crate::models::UploadFileResponse = response
        .json()
        .await
        .map_err( |e| Error::DeserializationError( e.to_string() ) )?;
      Ok( upload_response )
    }
    else
    {
      let status = response.status();
      let text = response.text().await.unwrap_or_else( |_| "Failed to read error response".to_string() );
      Err( Error::ApiError( format!( "HTTP {status}: {text}" ) ) )
    }
  }

  /// List all files uploaded to the Gemini API.
  ///
  /// This method retrieves a list of all files that have been uploaded to the Gemini API,
  /// including their metadata and processing status. Supports pagination for large file lists.
  ///
  /// # Arguments
  ///
  /// * `request` - Optional list files request for pagination and filtering
  ///
  /// # Returns
  ///
  /// Returns a [`ListFilesResponse`] containing:
  /// - `files`: Vector of [`FileMetadata`] objects with file information
  /// - `next_page_token`: Token for retrieving the next page of results
  ///
  /// # Errors
  ///
  /// This method returns an error in the following cases:
  /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
  /// - [`Error::DeserializationError`] - Failed to parse the API response
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # use api_gemini::models::*;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let files_api = client.files();
  ///
  /// // List all files
  /// let request = ListFilesRequest::default();
  /// let response = files_api.list(&request).await?;
  ///
  /// for file in &response.files {
  ///   println!("File : {} ({})", file.name, file.mime_type);
  /// }
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn list( &self, request : &crate::models::ListFilesRequest ) -> Result< crate::models::ListFilesResponse, Error >
  {
    let mut url = format!( "{}/v1beta/files", self.client.base_url );

    // Add query parameters if provided
    let mut query_params = Vec::new();
    if let Some( page_size ) = request.page_size
    {
      query_params.push( format!( "pageSize={page_size}" ) );
    }
    if let Some( ref page_token ) = request.page_token
    {
      query_params.push( format!( "pageToken={page_token}" ) );
    }

    if !query_params.is_empty()
    {
      url.push( '?' );
      url.push_str( &query_params.join( "&" ) );
    }

    crate ::internal::http::enterprise::execute_with_optional_retries::< (), crate::models::ListFilesResponse >
    (
      self.client,
      reqwest ::Method::GET,
      &url,
      &self.client.api_key,
      None,
    )
    .await
  }

  /// Get metadata for a specific file.
  ///
  /// This method retrieves detailed metadata for a specific file that has been
  /// uploaded to the Gemini API, including processing status and download URI.
  ///
  /// # Arguments
  ///
  /// * `file_name` - The name/ID of the file to retrieve
  ///
  /// # Returns
  ///
  /// Returns [`FileMetadata`] containing detailed file information
  ///
  /// # Errors
  ///
  /// This method returns an error in the following cases:
  /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
  /// - [`Error::ApiError`] - File not found (404)
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # use api_gemini::models::*;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let files_api = client.files();
  ///
  /// let file_metadata = files_api.get("files/abc123").await?;
  /// println!("File : {} ({})", file_metadata.name, file_metadata.mime_type);
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn get( &self, file_name : &str ) -> Result< crate::models::FileMetadata, Error >
  {
    let url = format!( "{}/v1beta/{}", self.client.base_url, file_name );

    crate ::internal::http::enterprise::execute_with_optional_retries::< (), crate::models::FileMetadata >
    (
      self.client,
      reqwest ::Method::GET,
      &url,
      &self.client.api_key,
      None,
    )
    .await
  }

  /// Delete a file from the Gemini API.
  ///
  /// This method permanently deletes a file that has been uploaded to the Gemini API.
  /// Once deleted, the file cannot be recovered and can no longer be used in API calls.
  ///
  /// # Arguments
  ///
  /// * `file_name` - The name/ID of the file to delete
  ///
  /// # Returns
  ///
  /// Returns `Ok(())` if the file was successfully deleted
  ///
  /// # Errors
  ///
  /// This method returns an error in the following cases:
  /// - [`Error::NetworkError`] - Network connectivity issues or request timeout
  /// - [`Error::AuthenticationError`] - Invalid or missing API key
  /// - [`Error::ServerError`] - Gemini API server-side errors (5xx status codes)
  /// - [`Error::ApiError`] - File not found (404) or deletion failed
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// # use api_gemini::client::Client;
  /// # #[ tokio::main ]
  /// # async fn main() -> Result< (), Box< dyn std::error::Error > > {
  /// let client = Client::new()?;
  /// let files_api = client.files();
  ///
  /// files_api.delete("files/abc123").await?;
  /// println!("File deleted successfully");
  /// # Ok( () )
  /// # }
  /// ```
  #[ inline ]
  pub async fn delete( &self, file_name : &str ) -> Result< (), Error >
  {
    let url = format!( "{}/v1beta/{}", self.client.base_url, file_name );

    let response = self.client.http
      .delete( &url )
      .header( "X-Goog-Api-Key", &self.client.api_key )
      .send()
      .await
      .map_err( Error::from )?;

    if response.status().is_success()
    {
      Ok( () )
    }
    else
    {
      let status = response.status();
      let text = response.text().await.unwrap_or_else( |_| "Failed to read error response".to_string() );
      Err( Error::ApiError( format!( "HTTP {status}: {text}" ) ) )
    }
  }
}
