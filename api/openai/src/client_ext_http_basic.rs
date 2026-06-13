// src/client_ext_http_basic.rs
//! Client basic HTTP methods extension.
//!
//! This module extends the `Client` with basic HTTP methods (GET, POST, DELETE, PATCH)
//! and diagnostics access.

mod private
{
  use crate::
  {
    client ::Client,
    environment ::{ OpenaiEnvironment, EnvironmentInterface },
    error ::{ OpenAIError, Result, map_deserialization_error },
    diagnostics ::{ DiagnosticsCollector, RequestMetrics, ResponseMetrics, ErrorMetrics },
  };

  use reqwest::Method;
  use serde::{ de::DeserializeOwned, Serialize };
  use std::{ sync::Arc, time::Instant };

  impl< E > Client< E >
  where
    E : OpenaiEnvironment + EnvironmentInterface + Send + Sync + 'static,
  {
    /// Returns the diagnostics collector for monitoring requests.
    ///
    /// # Returns
    /// - `Some(DiagnosticsCollector)` if diagnostics are enabled
    /// - `None` if diagnostics are disabled
    #[ inline ]
    pub fn get_diagnostics( &self ) -> Option< Arc< DiagnosticsCollector > >
    {
      self.diagnostics.clone()
    }

    /// Lists all registered custom tools.
    ///
    /// # Returns
    /// An empty vector as custom tool registration is not yet implemented.
    #[ inline ]
    pub fn list_registered_tools( &self ) -> Vec< &str >
    {
      Vec::new()
    }

    /// Sends a GET request to the specified path with query parameters.
    #[ inline ]
    pub(in crate) async fn get_with_query< Q, O >( &self, path : &str, query : &Q ) -> Result< O >
    where
      Q : Serialize + ?Sized + Sync,
      O : DeserializeOwned,
    {
      let url = self.environment.join_base_url( path )?;
      let http_client = &self.http_client;

      let response = self.execute_request_with_retry( || {
        http_client.request( Method::GET, url.clone() ).query( query ).send()
      }).await?;

      let bytes = response.bytes().await?.to_vec(); // Convert to Vec< u8 >
      let result = serde_json::from_slice( &bytes )
      .map_err( | error | map_deserialization_error( &error ) )?;
      Ok( result )
    }

    /// Sends a GET request to the specified path.
    #[ inline ]
    pub(in crate) async fn get< O >( &self, path : &str ) -> Result< O >
    where
      O : DeserializeOwned,
    {
      let url = self.environment.join_base_url( path )?;
      let http_client = &self.http_client;

      let response = self.execute_request_with_retry( || {
        http_client.request( Method::GET, url.clone() ).send()
      }).await?;

      let bytes = response.bytes().await?.to_vec(); // Convert to Vec< u8 >
      let result = serde_json::from_slice( &bytes )
        .map_err( |e| { let body = String::from_utf8_lossy(&bytes); OpenAIError::Internal( format!( "Failed to parse JSON response : {e}. Response body : {body}" ) ) } )?;
      Ok( result )
    }


    /// Sends a POST request to the specified path with a JSON body.
    #[ inline ]
    pub(in crate) async fn post< I, O >( &self, path : &str, body : &I ) -> Result< O >
    where
      I : Serialize + Sync,
      O : DeserializeOwned,
    {
      let url = self.environment.join_base_url( path )?;
      let http_client = &self.http_client;
      let start_time = Instant::now();

      // Record request metrics if diagnostics are enabled
      if let Some( diagnostics ) = &self.diagnostics
      {
        let request_body_size = serde_json::to_vec( body ).map_or( 0, |v| v.len() );
        let request_metrics = RequestMetrics
        {
          timestamp : start_time,
          method : "POST".to_string(),
          endpoint : path.to_string(),
          headers : if diagnostics.config.collection.request_headers
          {
            // Collect headers safely
            vec![ ( "Content-Type".to_string(), "application/json".to_string() ) ]
          }
          else
          {
            vec![]
          },
          body_size : request_body_size,
          user_agent : "api_openai/0.2.0".to_string(),
        };
        diagnostics.record_request( &request_metrics );
      }

      let response = self.execute_request_with_retry( || {
        http_client.request( Method::POST, url.clone() ).json( body ).send()
      }).await;

      // Handle response and record metrics
      match response
      {
        Ok( response ) =>
        {
          let status_code = response.status().as_u16();
          let response_time = start_time.elapsed();

          let bytes = response.bytes().await?.to_vec();

          // Record successful response metrics if diagnostics are enabled
          if let Some( diagnostics ) = &self.diagnostics
          {
            let response_metrics = ResponseMetrics
            {
              timestamp : Instant::now(),
              status_code,
              headers : if diagnostics.config.collection.response_headers
              {
                vec![ ( "Content-Type".to_string(), "application/json".to_string() ) ]
              }
              else
              {
                vec![]
              },
              body_size : bytes.len(),
              response_time,
              tokens_used : None, // Will be extracted from response if available
            };
            diagnostics.record_response( &response_metrics );
          }

          let result = serde_json::from_slice( &bytes )
            .map_err( |e| { let body = String::from_utf8_lossy(&bytes); OpenAIError::Internal( format!( "Failed to parse JSON response : {e}. Response body : {body}" ) ) } )?;
          Ok( result )
        },
        Err( error ) =>
        {
          // Record error metrics if diagnostics are enabled
          if let Some( diagnostics ) = &self.diagnostics
          {
            let error_metrics = ErrorMetrics
            {
              timestamp : Instant::now(),
              error_type : "RequestError".to_string(),
              error_code : None,
              error_message : error.to_string(),
              retry_count : 0, // This would need to be tracked in execute_request
              final_failure : true,
            };
            diagnostics.record_error( &error_metrics );
          }
          Err( error )?
        }
      }
    }


    /// Sends a DELETE request to the specified path.
    #[ inline ]
    pub(in crate) async fn delete< O >( &self, path : &str ) -> Result< O >
    where
      O : DeserializeOwned,
    {
      let url = self.environment.join_base_url( path )?;
      let http_client = &self.http_client;

      let response = self.execute_request_with_retry( || {
        http_client.request( Method::DELETE, url.clone() ).send()
      }).await?;

      let bytes = response.bytes().await?.to_vec(); // Convert to Vec< u8 >
      let result = serde_json::from_slice( &bytes )
        .map_err( |e| { let body = String::from_utf8_lossy(&bytes); OpenAIError::Internal( format!( "Failed to parse JSON response : {e}. Response body : {body}" ) ) } )?;
      Ok( result )
    }

    /// Sends a PATCH request to the specified path with a JSON body.
    #[ inline ]
    pub(in crate) async fn patch< I, O >( &self, path : &str, body : &I ) -> Result< O >
    where
      I : Serialize + Sync,
      O : DeserializeOwned,
    {
      let url = self.environment.join_base_url( path )?;
      let http_client = &self.http_client;
      let response = self.execute_request_with_retry( || {
        http_client.request( Method::PATCH, url.clone() ).json( body ).send()
      }).await?;

      let bytes = response.bytes().await?.to_vec();
      let result = serde_json::from_slice( &bytes )?;
      Ok( result )
    }

    /// Sends a POST request to the specified path without a body.
    #[ inline ]
    pub(in crate) async fn post_no_body< O >( &self, path : &str ) -> Result< O >
    where
      O : DeserializeOwned,
    {
      let url = self.environment.join_base_url( path )?;
      let http_client = &self.http_client;
      let response = self.execute_request_with_retry( || {
        http_client.request( Method::POST, url.clone() ).send()
      }).await?;

      let bytes = response.bytes().await?.to_vec();
      let result = serde_json::from_slice( &bytes )?;
      Ok( result )
    }
  }

} // end mod private
