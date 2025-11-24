mod private
{
  //! Batch processing for multiple chat completion requests.
  //!
  //! This module provides client-side parallel request orchestration
  //! for processing multiple chat completion requests efficiently.
  //!
  //! # Design Decisions
  //!
  //! ## Why Client-Side Batch Processing?
  //!
  //! The XAI Grok API does not provide a native batch processing endpoint.
  //! Client-side batching offers:
  //!
  //! 1. **Parallelism**: Process multiple requests concurrently
  //! 2. **Throughput**: Higher total throughput than sequential processing
  //! 3. **Control**: Fine-grained control over concurrency limits
  //! 4. **Rate Limiting**: Respect API rate limits with semaphore
  //!
  //! ## Concurrency Control
  //!
  //! Uses `tokio::sync::Semaphore` to limit concurrent requests:
  //!
  //! - **Prevents Overload**: Avoids overwhelming the API
  //! - **Rate Limit Compliance**: Respects API rate limits
  //! - **Resource Management**: Prevents excessive memory usage
  //! - **Graceful Degradation**: Continues processing on individual failures
  //!
  //! ## Error Handling Strategy
  //!
  //! - **Partial Success**: Returns all results (success + failures)
  //! - **Non-Blocking**: One failure doesn't stop other requests
  //! - **Transparent**: Each result is individually inspected
  //!
  //! ## Alternatives Considered
  //!
  //! - **Sequential Processing**: Too slow for large batches
  //! - **Unbounded Parallelism**: Risk of rate limit violations
  //! - **External Batch API**: XAI doesn't provide this endpoint

  use crate::{ ChatCompletionRequest, ChatCompletionResponse, Client, XaiEnvironment, ClientApiAccessors };
  use crate::error::Result;
  use std::sync::Arc;

  #[ cfg( feature = "batch_operations" ) ]
  use tokio::sync::Semaphore;

  /// A client wrapper that supports batch processing of requests.
  ///
  /// Processes multiple chat completion requests in parallel with
  /// configurable concurrency limits.
  ///
  /// # Concurrency
  ///
  /// The `max_concurrent` parameter controls how many requests can
  /// be in-flight simultaneously. This helps:
  ///
  /// - Respect API rate limits
  /// - Control resource usage
  /// - Prevent overwhelming the API
  ///
  /// # Error Handling
  ///
  /// Failures are returned individually in the results vector.
  /// One request failing does not stop processing of others.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # #[ cfg( feature = "batch_operations") ]
  /// # {
  /// use api_xai::{ BatchProcessor, Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message };
  ///
  /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let secret = Secret::new( "xai-key".to_string() )?;
  /// let env = XaiEnvironmentImpl::new( secret )?;
  /// let client = Client::build( env )?;
  ///
  /// // Create batch processor (max 5 concurrent requests)
  /// let processor = BatchProcessor::new( client, 5 );
  ///
  /// // Prepare multiple requests
  /// let requests = vec!
  /// [
  ///   ChatCompletionRequest::former()
  ///     .model( "grok-2-1212".to_string() )
  ///     .messages( vec![ Message::user( "Hello!" ) ] )
  ///     .form(),
  ///   ChatCompletionRequest::former()
  ///     .model( "grok-2-1212".to_string() )
  ///     .messages( vec![ Message::user( "Goodbye!" ) ] )
  ///     .form(),
  /// ];
  ///
  /// // Process batch
  /// let results = processor.process_batch( requests ).await;
  ///
  /// // Inspect results
  /// for ( idx, result ) in results.iter().enumerate()
  /// {
  ///   match result
  ///   {
  ///     Ok( response ) => println!( "Request {}: Success", idx ),
  ///     Err( e ) => println!( "Request {}: Failed - {}", idx, e ),
  ///   }
  /// }
  /// # Ok( () )
  /// # }
  /// # }
  /// ```
  #[ cfg( feature = "batch_operations" ) ]
  #[ derive( Debug ) ]
  pub struct BatchProcessor< E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    client : Arc< Client< E > >,
    max_concurrent : usize,
  }

  #[ cfg( feature = "batch_operations" ) ]
  impl< E > BatchProcessor< E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    /// Creates a new batch processor.
    ///
    /// # Arguments
    ///
    /// * `client` - The XAI client to use for requests
    /// * `max_concurrent` - Maximum number of concurrent requests
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "batch_operations") ]
    /// # {
    /// use api_xai::{ BatchProcessor, Client, Secret, XaiEnvironmentImpl };
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    ///
    /// // Allow up to 10 concurrent requests
    /// let processor = BatchProcessor::new( client, 10 );
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    pub fn new( client : Client< E >, max_concurrent : usize ) -> Self
    {
      Self
      {
        client : Arc::new( client ),
        max_concurrent,
      }
    }

    /// Processes a batch of chat completion requests.
    ///
    /// Executes all requests in parallel (up to `max_concurrent` at a time)
    /// and returns results in the same order as input requests.
    ///
    /// # Arguments
    ///
    /// * `requests` - Vector of chat completion requests to process
    ///
    /// # Returns
    ///
    /// Vector of results (one per request, in same order).
    /// Successful requests return `Ok(ChatCompletionResponse)`,
    /// failed requests return `Err`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "batch_operations") ]
    /// # {
    /// use api_xai::{ BatchProcessor, Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message };
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    /// let processor = BatchProcessor::new( client, 5 );
    ///
    /// let requests = vec!
    /// [
    ///   ChatCompletionRequest::former()
    ///     .model( "grok-2-1212".to_string() )
    ///     .messages( vec![ Message::user( "Request 1" ) ] )
    ///     .form(),
    ///   ChatCompletionRequest::former()
    ///     .model( "grok-2-1212".to_string() )
    ///     .messages( vec![ Message::user( "Request 2" ) ] )
    ///     .form(),
    /// ];
    ///
    /// let results = processor.process_batch( requests ).await;
    ///
    /// let successes = results.iter().filter( | r | r.is_ok() ).count();
    /// println!( "Successful : {}/{}", successes, results.len() );
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the semaphore is closed.
    pub async fn process_batch
    (
      &self,
      requests : Vec< ChatCompletionRequest >
    )
    -> Vec< Result< ChatCompletionResponse > >
    {
      let semaphore = Arc::new( Semaphore::new( self.max_concurrent ) );
      let mut handles = Vec::new();

      for request in requests
      {
        let client = Arc::clone( &self.client );
        let semaphore = Arc::clone( &semaphore );

        let handle = tokio::spawn
        (
          async move
          {
            // Acquire permit (blocks if max_concurrent reached)
            let _permit = semaphore.acquire().await.unwrap();

            // Execute request
            client.chat().create( request ).await
          }
        );

        handles.push( handle );
      }

      // Collect results in order
      let mut results = Vec::new();
      for handle in handles
      {
        match handle.await
        {
          Ok( result ) => results.push( result ),
          Err( e ) =>
          {
            // Task join error (very rare)
            results.push
            (
              Err
              (
                crate::error::XaiError::ApiError
                (
                  format!( "Task join error : {e}" )
                ).into()
              )
            );
          }
        }
      }

      results
    }

    /// Processes a batch with progress callback.
    ///
    /// Same as `process_batch` but calls a callback for each completed request,
    /// allowing progress tracking.
    ///
    /// # Arguments
    ///
    /// * `requests` - Vector of chat completion requests to process
    /// * `on_complete` - Callback invoked for each completed request
    ///
    /// # Returns
    ///
    /// Vector of results (one per request, in same order).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "batch_operations") ]
    /// # {
    /// use api_xai::{ BatchProcessor, Client, Secret, XaiEnvironmentImpl, ChatCompletionRequest, Message };
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// let secret = Secret::new( "xai-key".to_string() )?;
    /// let env = XaiEnvironmentImpl::new( secret )?;
    /// let client = Client::build( env )?;
    /// let processor = BatchProcessor::new( client, 5 );
    ///
    /// let requests = vec!
    /// [
    ///   ChatCompletionRequest::former()
    ///     .model( "grok-2-1212".to_string() )
    ///     .messages( vec![ Message::user( "Request 1" ) ] )
    ///     .form(),
    ///   ChatCompletionRequest::former()
    ///     .model( "grok-2-1212".to_string() )
    ///     .messages( vec![ Message::user( "Request 2" ) ] )
    ///     .form(),
    /// ];
    ///
    /// let total = requests.len();
    /// let results = processor.process_batch_with_progress
    /// (
    ///   requests,
    ///   move | idx, result |
    ///   {
    ///     println!
    ///     (
    ///       "Completed {}/{}: {}",
    ///       idx + 1,
    ///       total,
    ///       if result.is_ok() { "Success" } else { "Failed" }
    ///     );
    ///   }
    /// ).await;
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the semaphore is closed.
    pub async fn process_batch_with_progress< F >
    (
      &self,
      requests : Vec< ChatCompletionRequest >,
      on_complete : F
    )
    -> Vec< Result< ChatCompletionResponse > >
    where
      F : Fn( usize, &Result< ChatCompletionResponse > ) + Send + Sync + 'static,
    {
      let semaphore = Arc::new( Semaphore::new( self.max_concurrent ) );
      let callback = Arc::new( on_complete );
      let mut handles = Vec::new();

      for ( idx, request ) in requests.into_iter().enumerate()
      {
        let client = Arc::clone( &self.client );
        let semaphore = Arc::clone( &semaphore );
        let callback = Arc::clone( &callback );

        let handle = tokio::spawn
        (
          async move
          {
            let _permit = semaphore.acquire().await.unwrap();
            let result = client.chat().create( request ).await;

            // Call progress callback
            callback( idx, &result );

            result
          }
        );

        handles.push( handle );
      }

      // Collect results
      let mut results = Vec::new();
      for handle in handles
      {
        match handle.await
        {
          Ok( result ) => results.push( result ),
          Err( e ) =>
          {
            results.push
            (
              Err
              (
                crate::error::XaiError::ApiError
                (
                  format!( "Task join error : {e}" )
                ).into()
              )
            );
          }
        }
      }

      results
    }
  }
}

#[ cfg( feature = "batch_operations" ) ]
crate::mod_interface!
{
  exposed use
  {
    BatchProcessor,
  };
}
