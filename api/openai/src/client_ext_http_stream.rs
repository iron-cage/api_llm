// src/client_ext_http_stream.rs
//! Client streaming and binary HTTP methods extension.
//!
//! This module extends the `Client` with streaming HTTP methods (POST streaming,
//! multipart uploads, binary POST/GET).

mod private
{
  use crate::
  {
    client ::Client,
    environment ::{ OpenaiEnvironment, EnvironmentInterface },
    error ::{ OpenAIError, Result },
    diagnostics ::{ RequestMetrics, ResponseMetrics },
  };

  use reqwest::Method;
  use serde::{ de::DeserializeOwned, Serialize };
  use futures_util::StreamExt;
  use tokio::sync::mpsc;
  use std::{ sync::Arc, time::Instant };
  use eventsource_stream::Eventsource;

  impl< E > Client< E >
  where
    E : OpenaiEnvironment + EnvironmentInterface + Send + Sync + 'static,
  {
    /// Sends a POST request to the specified path with a JSON body and streams the response.
    #[ allow( clippy::unused_async ) ]
    #[ inline ]
    pub(in crate) async fn post_stream< I, O >( &self, path : &str, body : &I ) -> Result< mpsc::Receiver< Result< O > > >
    where
      I : Serialize,
      O : DeserializeOwned + Send + 'static, // Add Send + 'static
    {
      let url = self.environment.join_base_url( path )?;
      let request = self.http_client.request( Method::POST, url ).json( body );
      let ( tx, rx ) = mpsc::channel( 100 );
      let tx_arc = Arc::new( tx ); // Wrap tx in Arc

      tokio ::spawn( async move
      {
        let tx_clone = Arc::< _ >::clone( &tx_arc ); // Clone the Arc< Sender >
        let response = match request.send().await
        {
          Ok( res ) => res,
          Err( e ) =>
          {
            let _ = tx_clone.send( Err( OpenAIError::Stream( e.to_string() ).into() ) ).await;
            return;
          }
        };

        // Use robust eventsource-stream parser instead of manual parsing
        let mut event_stream = response.bytes_stream().eventsource();

        while let Some( event_result ) = event_stream.next().await
        {
          match event_result
          {
            Ok( event ) =>
            {
              // Handle different event types based on the actual Event structure
              let data = &event.data;
              if !data.is_empty()
              {
                // Handle data messages with robust SSE parsing
                if data == "[DONE]"
                {
                  return; // Exit the entire task
                }

                // Parse JSON with better error handling
                match serde_json::from_str::< O >( data )
                {
                  Ok( obj ) =>
                  {
                    let _ = tx_clone.send( Ok( obj ) ).await;
                  },
                  Err( e ) =>
                  {
                    // More informative error with context
                    let error_msg = format!( "Failed to parse JSON from SSE data '{data}': {e}" );
                    let _ = tx_clone.send( Err( OpenAIError::Stream( error_msg ).into() ) ).await;
                  },
                }
              }
              // Skip events without data (like connection open events)
            },
            Err( e ) =>
            {
              // Handle any streaming errors
              let error_msg = format!( "SSE streaming error : {e}" );
              let _ = tx_clone.send( Err( OpenAIError::Stream( error_msg ).into() ) ).await;
              break;
            },
          }
        }
      });

      Ok( rx )
    }

    /// Sends a POST request with multipart form data
    #[ inline ]
    pub(in crate) async fn post_multipart< O >( &self, path : &str, form : reqwest::multipart::Form ) -> Result< O >
    where
      O : DeserializeOwned,
    {
      let url = self.environment.join_base_url( path )?;
      let http_client = &self.http_client;
      let start_time = Instant::now();

      // Record request metrics if diagnostics are enabled
      if let Some( diagnostics ) = &self.diagnostics
      {
        let request_metrics = RequestMetrics
        {
          timestamp : start_time,
          method : "POST".to_string(),
          endpoint : path.to_string(),
          headers : if diagnostics.config.collection.request_headers
          {
            vec![ ( "Content-Type".to_string(), "multipart/form-data".to_string() ) ]
          }
          else
          {
            vec![]
          },
          body_size : 0, // Cannot easily calculate multipart form size
          user_agent : "api_openai/0.2.0".to_string(),
        };
        diagnostics.record_request( &request_metrics );
      }

      // For multipart requests, don't use retry logic due to form consumption
      let response = http_client.request( Method::POST, url ).multipart( form ).send().await;

      // Handle response
      let response = response.map_err( | e | OpenAIError::Network( e.to_string() ) )?;

      if response.status().is_success()
      {
        let result : O = response.json().await.map_err( | e | OpenAIError::Internal( e.to_string() ) )?;
        Ok( result )
      }
      else
      {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else( | _ | "Unknown error".to_string() );
        Err( OpenAIError::Api( crate::error::ApiError {
          code : Some( status.as_u16().to_string() ),
          message : error_text,
          param : None,
          r#type : Some( "http_error".to_string() ),
        } ).into() )
      }
    }

    /// Sends a POST request expecting a binary response
    #[ inline ]
    pub(in crate) async fn post_binary< I >( &self, path : &str, body : &I ) -> Result< Vec< u8 > >
    where
      I: serde::Serialize + Sync,
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

      // Send request using execute_request_with_retry but extract bytes
      let response = self.execute_request_with_retry( || {
        http_client.request( Method::POST, url.clone() ).json( body ).send()
      }).await;

      // Handle response
      match response
      {
        Ok( response ) =>
        {
          let status_code = response.status().as_u16();
          let response_time = start_time.elapsed();

          if response.status().is_success()
          {
            let bytes = response.bytes().await
              .map_err( | e | OpenAIError::Internal( format!( "Failed to read response bytes : {e}" ) ) )?;

            // Record response metrics if diagnostics are enabled
            if let Some( diagnostics ) = &self.diagnostics
            {
              let response_metrics = ResponseMetrics
              {
                timestamp : start_time,
                status_code,
                response_time,
                body_size : bytes.len(),
                headers : if diagnostics.config.collection.response_headers
                {
                  vec![ ( "Content-Type".to_string(), "application/octet-stream".to_string() ) ]
                }
                else
                {
                  vec![]
                },
                tokens_used : None,
              };
              diagnostics.record_response( &response_metrics );
            }

            Ok( bytes.to_vec() )
          }
          else
          {
            let error_text = response.text().await.unwrap_or_else( | _ | "Unknown error".to_string() );
            Err( OpenAIError::Api( crate::error::ApiError {
              code : Some( status_code.to_string() ),
              message : error_text,
              param : None,
              r#type : Some( "http_error".to_string() ),
            } ).into() )
          }
        }
        Err( e ) => Err( e )
      }
    }

    /// Sends a GET request expecting a binary response
    #[ inline ]
    pub(in crate) async fn get_bytes( &self, path : &str ) -> Result< Vec< u8 > >
    {
      let url = self.environment.join_base_url( path )?;
      let http_client = &self.http_client;
      let start_time = Instant::now();

      // Record request metrics if diagnostics are enabled
      if let Some( diagnostics ) = &self.diagnostics
      {
        let request_metrics = RequestMetrics
        {
          timestamp : start_time,
          method : "GET".to_string(),
          endpoint : path.to_string(),
          headers : if diagnostics.config.collection.request_headers
          {
            vec![ ( "Accept".to_string(), "application/octet-stream".to_string() ) ]
          }
          else
          {
            vec![]
          },
          body_size : 0,
          user_agent : "api_openai/0.2.0".to_string(),
        };
        diagnostics.record_request( &request_metrics );
      }

      // Send request using execute_request_with_retry but extract bytes
      let response = self.execute_request_with_retry( || {
        http_client.request( Method::GET, url.clone() ).send()
      }).await;

      // Handle response
      match response
      {
        Ok( response ) =>
        {
          let status_code = response.status().as_u16();
          let response_time = start_time.elapsed();

          if response.status().is_success()
          {
            let bytes = response.bytes().await
              .map_err( | e | OpenAIError::Internal( format!( "Failed to read response bytes : {e}" ) ) )?;

            // Record response metrics if diagnostics are enabled
            if let Some( diagnostics ) = &self.diagnostics
            {
              let response_metrics = ResponseMetrics
              {
                timestamp : start_time,
                status_code,
                response_time,
                body_size : bytes.len(),
                headers : if diagnostics.config.collection.response_headers
                {
                  vec![ ( "Content-Type".to_string(), "application/octet-stream".to_string() ) ]
                }
                else
                {
                  vec![]
                },
                tokens_used : None,
              };
              diagnostics.record_response( &response_metrics );
            }

            Ok( bytes.to_vec() )
          }
          else
          {
            let error_text = response.text().await.unwrap_or_else( | _ | "Unknown error".to_string() );
            Err( OpenAIError::Api( crate::error::ApiError {
              code : Some( status_code.to_string() ),
              message : error_text,
              param : None,
              r#type : Some( "http_error".to_string() ),
            } ).into() )
          }
        }
        Err( e ) => Err( e )
      }
    }
  }

} // end mod private
