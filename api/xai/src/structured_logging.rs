mod private
{
  //! Structured logging support using the `tracing` crate.
  //!
  //! This module provides instrumentation helpers for tracking
  //! API requests, responses, and errors with structured logging.
  //!
  //! # Design Decisions
  //!
  //! ## Why Macros with #[ cfg ] Guards?
  //!
  //! This module uses `macro_rules!` with conditional compilation (`#[ cfg( feature = "structured_logging") ]`)
  //! instead of regular functions for several reasons:
  //!
  //! 1. **Zero Runtime Overhead**: When the `structured_logging` feature is disabled,
  //!    all logging code is completely eliminated at compile time. No function calls,
  //!    no string formatting, no performance impact.
  //!
  //! 2. **Ergonomic API**: Macros allow flexible argument patterns without trait bounds
  //!    or generic constraints that would complicate the API.
  //!
  //! 3. **Compile-Time Feature Detection**: The `#[ cfg ]` guards ensure that tracing
  //!    dependency and code only exist when explicitly enabled via cargo features.
  //!
  //! ## Alternatives Considered
  //!
  //! - **Regular functions**: Would require runtime checks or always-compiled code
  //! - **Trait-based logging**: Would add complexity and generic constraints
  //! - **Direct tracing calls**: Would scatter logging logic across the codebase
  //!
  //! ## Known Pitfalls
  //!
  //! ### Macro Invocation Syntax (Fix : issue-doctest-macro-syntax)
  //!
  //! **Root Cause**: Rust macros require the `!` suffix for invocation. Documentation
  //! examples that call macros without `!` will fail during doc tests with error
  //! "expected function, found macro".
  //!
  //! **Pitfall**: When writing documentation examples for macros, it's easy to forget
  //! the `!` suffix because the syntax looks like a function call. The compiler
  //! helpfully suggests adding `!`, but this only appears during doc test runs.
  //!
  //! **Prevention**: Always use `cargo test --doc` to validate documentation examples
  //! during development. The macro invocation syntax is not optional - without `!`,
  //! the code won't compile.
  //!
  //! Example of correct syntax:
  //! ```rust,ignore
  //! log_request!( "POST", "/chat", Some("grok-2-1212") );  // Correct
  //! log_request( "POST", "/chat", Some("grok-2-1212") );   // Wrong - won't compile
  //! ```

  /// Logs an API request.
  ///
  /// Records request details including method, path, and model.
  ///
  /// # Arguments
  ///
  /// * `method` - HTTP method (GET, POST, etc.)
  /// * `path` - API endpoint path
  /// * `model` - Optional model name
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "structured_logging") ]
  /// # {
  /// use api_xai::log_request;
  ///
  /// log_request!( "POST", "/chat/completions", Some( "grok-2-1212" ) );
  /// # }
  /// ```
  #[ macro_export ]
  macro_rules! log_request
  {
    ( $method:expr, $path:expr, $model:expr ) =>
    {
      #[ cfg( feature = "structured_logging" ) ]
      {
        tracing::info!(
          method = $method,
          path = $path,
          model = $model,
          "API request"
        );
      }
    };
  }

  /// Logs an API response.
  ///
  /// Records response details including status code and duration.
  ///
  /// # Arguments
  ///
  /// * `status` - HTTP status code
  /// * `duration_ms` - Request duration in milliseconds
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "structured_logging") ]
  /// # {
  /// use api_xai::log_response;
  ///
  /// log_response!( 200, 145 );
  /// # }
  /// ```
  #[ macro_export ]
  macro_rules! log_response
  {
    ( $status:expr, $duration_ms:expr ) =>
    {
      #[ cfg( feature = "structured_logging" ) ]
      {
        tracing::info!(
          status = $status,
          duration_ms = $duration_ms,
          "API response"
        );
      }
    };
  }

  /// Logs an API error.
  ///
  /// Records error details including error type and message.
  ///
  /// # Arguments
  ///
  /// * `error_type` - Type of error
  /// * `message` - Error message
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "structured_logging") ]
  /// # {
  /// use api_xai::log_error;
  ///
  /// log_error!( "RateLimit", "Rate limit exceeded" );
  /// # }
  /// ```
  #[ macro_export ]
  macro_rules! log_error
  {
    ( $error_type:expr, $message:expr ) =>
    {
      #[ cfg( feature = "structured_logging" ) ]
      {
        tracing::error!(
          error_type = $error_type,
          message = $message,
          "API error"
        );
      }
    };
  }

  /// Logs a retry attempt.
  ///
  /// Records retry details including attempt number and delay.
  ///
  /// # Arguments
  ///
  /// * `attempt` - Current attempt number
  /// * `max_attempts` - Maximum number of attempts
  /// * `delay_ms` - Delay before retry in milliseconds
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "structured_logging") ]
  /// # {
  /// use api_xai::log_retry;
  ///
  /// log_retry!( 2, 5, 1000 );
  /// # }
  /// ```
  #[ macro_export ]
  macro_rules! log_retry
  {
    ( $attempt:expr, $max_attempts:expr, $delay_ms:expr ) =>
    {
      #[ cfg( feature = "structured_logging" ) ]
      {
        tracing::warn!(
          attempt = $attempt,
          max_attempts = $max_attempts,
          delay_ms = $delay_ms,
          "Retrying request"
        );
      }
    };
  }

  /// Logs a circuit breaker state change.
  ///
  /// Records state transition details.
  ///
  /// # Arguments
  ///
  /// * `from_state` - Previous state
  /// * `to_state` - New state
  /// * `reason` - Reason for state change
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "structured_logging") ]
  /// # {
  /// use api_xai::log_circuit_breaker_state;
  ///
  /// log_circuit_breaker_state!( "Closed", "Open", "Failure threshold reached" );
  /// # }
  /// ```
  #[ macro_export ]
  macro_rules! log_circuit_breaker_state
  {
    ( $from_state:expr, $to_state:expr, $reason:expr ) =>
    {
      #[ cfg( feature = "structured_logging" ) ]
      {
        tracing::warn!(
          from_state = $from_state,
          to_state = $to_state,
          reason = $reason,
          "Circuit breaker state change"
        );
      }
    };
  }

  /// Logs a failover event.
  ///
  /// Records endpoint rotation details.
  ///
  /// # Arguments
  ///
  /// * `from_endpoint` - Previous endpoint
  /// * `to_endpoint` - New endpoint
  /// * `reason` - Reason for failover
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "structured_logging") ]
  /// # {
  /// use api_xai::log_failover;
  ///
  /// log_failover!(
  ///   "https://api.x.ai/v1/",
  ///   "https://api-backup.x.ai/v1/",
  ///   "Primary endpoint unhealthy"
  /// );
  /// # }
  /// ```
  #[ macro_export ]
  macro_rules! log_failover
  {
    ( $from_endpoint:expr, $to_endpoint:expr, $reason:expr ) =>
    {
      #[ cfg( feature = "structured_logging" ) ]
      {
        tracing::warn!(
          from_endpoint = $from_endpoint,
          to_endpoint = $to_endpoint,
          reason = $reason,
          "Failover to backup endpoint"
        );
      }
    };
  }

  /// Logs rate limiting information.
  ///
  /// Records rate limit details.
  ///
  /// # Arguments
  ///
  /// * `tokens_available` - Tokens available
  /// * `tokens_requested` - Tokens requested
  ///
  /// # Examples
  ///
  /// ```
  /// # #[ cfg( feature = "structured_logging") ]
  /// # {
  /// use api_xai::log_rate_limit;
  ///
  /// log_rate_limit!( 50, 10 );
  /// # }
  /// ```
  #[ macro_export ]
  macro_rules! log_rate_limit
  {
    ( $tokens_available:expr, $tokens_requested:expr ) =>
    {
      #[ cfg( feature = "structured_logging" ) ]
      {
        tracing::debug!(
          tokens_available = $tokens_available,
          tokens_requested = $tokens_requested,
          "Rate limit check"
        );
      }
    };
  }

  /// Creates a tracing span for tracking an API operation.
  ///
  /// This helper creates a named span for tracking the lifecycle
  /// of an API operation from start to finish.
  ///
  /// # Arguments
  ///
  /// * `name` - Span name (e.g., `chat_completion`)
  /// * `model` - Optional model name
  ///
  /// # Returns
  ///
  /// Returns a tracing span that can be entered.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # #[ cfg( feature = "structured_logging") ]
  /// # {
  /// use api_xai::create_operation_span;
  ///
  /// let span = create_operation_span( "chat_completion", Some( "grok-2-1212" ) );
  /// let _guard = span.enter();
  /// // Your operation here
  /// # }
  /// ```
  #[ cfg( feature = "structured_logging" ) ]
  pub fn create_operation_span(
    name : &str,
    model : Option< &str >
  ) -> tracing::Span
  {
    tracing::info_span!(
      "api_operation",
      operation = name,
      model = model,
    )
  }
}

#[ cfg( feature = "structured_logging" ) ]
crate::mod_interface!
{
  exposed use
  {
    create_operation_span,
  };
}
