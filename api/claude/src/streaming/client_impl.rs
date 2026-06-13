//! Client implementation for streaming
//!
//! Adds streaming methods to the Client struct.

#[ cfg( feature = "streaming" ) ]
mod private
{
  use super::super::types::orphan::*;
  #[ cfg( feature = "error-handling" ) ]
  use crate::error::{ AnthropicError, AnthropicResult };
  
  #[ cfg( not( feature = "error-handling" ) ) ]
  type AnthropicError = crate::error_tools::Error;
  #[ cfg( not( feature = "error-handling" ) ) ]  
  type AnthropicResult< T > = Result< T, crate::error_tools::Error >;

  use crate::client::CreateMessageRequest;

  /// HTTP streaming client methods (implemented in client.rs)
  #[ cfg( feature = "streaming" ) ]
  impl crate::client::Client
  {
    /// Create a streaming message request
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, network issues occur, or response parsing fails
    ///
    /// # Panics
    ///
    /// Panics if header values are invalid (should not happen with known valid values)
    pub async fn create_message_stream( &self, request : CreateMessageRequest ) -> AnthropicResult< EventStream >
    {
      use tokio_stream::wrappers::UnboundedReceiverStream;
      
      // Validate the request
      request.validate()?;
      
      let url = format!( "{}/v1/messages", self.base_url() );
      
      // Build headers for streaming
      let mut headers = reqwest::header::HeaderMap::new();
      headers.insert( 
        "Content-Type", 
        "application/json".parse().expect( "Valid content type" ) 
      );
      headers.insert( 
        "x-api-key", 
        self.secret().ANTHROPIC_API_KEY.parse().expect( "Valid API key" ) 
      );
      headers.insert( 
        "anthropic-version", 
        self.config().api_version.parse().expect( "Valid API version" ) 
      );
      headers.insert(
        "Accept",
        "text/event-stream".parse().expect( "Valid accept header" )
      );
      headers.insert(
        "Cache-Control",
        "no-cache".parse().expect( "Valid cache control" )
      );
      
      // Create HTTP client with timeout
      let http_client = reqwest::Client::builder()
        .timeout( self.config().request_timeout )
        .build()
        .map_err( | e | AnthropicError::http_error( format!( "Failed to build HTTP client : {e}" ) ) )?;
      
      // Make the streaming request
      let response = http_client
        .post( &url )
        .headers( headers )
        .json( &request )
        .send()
        .await
        .map_err( AnthropicError::from )?;
      
      // Check response status
      if !response.status().is_success()
      {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else( |_| "Unknown error".to_string() );
        
        if let Ok( api_error ) = serde_json::from_str::< crate::error::ApiErrorWrap >( &error_text )
        {
          return Err( AnthropicError::Api( api_error.error ) );
        }
        
        return Err( AnthropicError::http_error_with_status( format!( "HTTP {status}: {error_text}" ), status.as_u16() ) );
      }
      
      // Create a channel for the event stream
      let ( tx, rx ) = tokio::sync::mpsc::unbounded_channel();
      
      // Spawn a task to handle the SSE stream
      let _handle = tokio::spawn( async move
      {
        // Read response text line by line
        let text = match response.text().await
        {
          Ok( text ) => text,
          Err( e ) =>
          {
            let error = AnthropicError::http_error( format!( "Failed to read response : {e}" ) );
            let _ = tx.send( Err( error ) );
            return;
          }
        };
        
        // For now, treat the entire response as SSE data
        // In a real implementation, we'd process the stream incrementally
        match parse_sse_events( &text )
        {
          Ok( events ) =>
          {
            for event in events
            {
              if tx.send( Ok( event ) ).is_err()
              {
                // Receiver dropped, stop processing
                return;
              }
            }
          },
          Err( e ) =>
          {
            let _ = tx.send( Err( e ) );
          }
        }
      } );
      
      // Convert the receiver to a stream
      let stream = UnboundedReceiverStream::new( rx );
      Ok( Box::pin( stream ) )
    }
  }
}


#[ cfg( feature = "streaming" ) ]
crate::mod_interface!
{
  // Client impl only, no exposed types
}
