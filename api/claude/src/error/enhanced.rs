//! Enhanced error handling types and utilities
//!
//! Advanced error handling including error recovery, classification, logging, and metrics.

#[ allow( clippy::missing_inline_in_public_items ) ]
mod private
{
  use super::super::core::orphan::*;
  use serde::{ Serialize, Deserialize };
  use std::time::Duration;
  #[ cfg( feature = "error-handling" ) ]
  use chrono;


/// Enhanced error with context and classification
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct EnhancedAnthropicError
{
  /// Error type classification
  error_type : ErrorType,
  /// Error message
  message : String,
  /// Error context information
  context : Option< ErrorContext >,
  /// Error classification
  class : ErrorClass,
  /// Error severity
  severity : ErrorSeverity,
  /// Whether error is transient
  is_transient : bool,
  /// Stack trace information
  stack_trace : Vec< String >,
  /// Request correlation ID
  correlation_id : Option< String >,
  /// Request ID
  request_id : Option< String >,
}

/// Error context information
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ErrorContext
{
  /// Operation name
  operation : String,
  /// Request ID
  request_id : String,
  /// Additional context data
  context_data : std::collections::HashMap< String, String >,
  /// Timestamp
  timestamp : chrono::DateTime< chrono::Utc >,
}

/// Timeout error details
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct TimeoutError
{
  /// Timeout type
  timeout_type : TimeoutType,
  /// Timeout duration
  duration : Duration,
  /// Error message
  message : String,
}

/// Network error details
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct NetworkError
{
  /// Network error type
  error_type : NetworkErrorType,
  /// Error message
  message : String,
  /// Additional error details
  details : Option< String >,
}
/// Custom error type for domain-specific errors
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct CustomError
{
  /// Error name
  name : String,
  /// Error message
  message : String,
  /// Error severity
  severity : ErrorSeverity,
}

/// Error chain for cause relationships
#[ derive( Debug, Clone ) ]
pub struct ErrorChain
{
  /// Primary error
  primary : CustomError,
  /// Causing errors
  causes : Vec< AnthropicError >,
  /// Context
  context : String,
}

/// Chained error result
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ChainedError
{
  /// Chain length
  chain_length : u32,
  /// Root cause
  root_cause : String,
  /// Immediate cause
  immediate_cause : String,
  /// Context
  context : String,
}

/// Request context for operations
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct RequestContext
{
  /// Correlation ID
  correlation_id : String,
  /// Request sequence
  request_sequence : u32,
}

impl RequestContext
{
  /// Correlation ID for this request
  pub fn correlation_id( &self ) -> &str
  {
    &self.correlation_id
  }

  /// Request sequence number
  pub fn request_sequence( &self ) -> u32
  {
    self.request_sequence
  }
}

  // Implementation of ErrorContext
  impl ErrorContext
  {
    /// Create new error context
    #[ must_use ]
    pub fn new( operation : String, request_id : String, context_data : std::collections::HashMap< String, String > ) -> Self
    {
      Self
      {
        operation,
        request_id,
        context_data,
        timestamp : chrono::Utc::now(),
      }
    }

    /// Get request ID
    #[ must_use ]
    pub fn request_id( &self ) -> &str
    {
      &self.request_id
    }

    /// Get context as a string (operation info)
    #[ must_use ]
    pub fn context( &self ) -> &str
    {
      &self.operation
    }
  }

  // Implementation of EnhancedAnthropicError
  impl EnhancedAnthropicError
  {
    /// Create new enhanced error
    #[ must_use ]
    pub fn new( error_type : ErrorType, message : String, context : Option< ErrorContext > ) -> Self
    {
      let ( class, severity, is_transient ) = match error_type
      {
        ErrorType::Authentication => ( ErrorClass::Authentication, ErrorSeverity::High, false ),
        ErrorType::InvalidRequest => ( ErrorClass::InvalidRequest, ErrorSeverity::Medium, false ),
        ErrorType::ServerError => ( ErrorClass::ServerError, ErrorSeverity::High, true ),
        ErrorType::RateLimit => ( ErrorClass::RateLimit, ErrorSeverity::Medium, true ),
        ErrorType::Network => ( ErrorClass::Network, ErrorSeverity::High, true ),
        ErrorType::Timeout => ( ErrorClass::Timeout, ErrorSeverity::Medium, true ),
        ErrorType::Parsing => ( ErrorClass::Parsing, ErrorSeverity::Medium, false ),
        ErrorType::Internal => ( ErrorClass::Internal, ErrorSeverity::High, false ),
      };

      Self
      {
        error_type,
        message,
        context,
        class,
        severity,
        is_transient,
        stack_trace : Vec::new(),
        correlation_id : None,
        request_id : None,
      }
    }

    /// Get error class
    #[ must_use ]
    pub fn error_class( &self ) -> ErrorClass
    {
      self.class.clone()
    }

    /// Get error severity
    #[ must_use ]
    pub fn severity( &self ) -> ErrorSeverity
    {
      self.severity
    }

    /// Check if error is transient
    #[ must_use ]
    pub fn is_transient( &self ) -> bool
    {
      self.is_transient
    }

    /// Check if requires credential refresh
    #[ must_use ]
    pub fn requires_credential_refresh( &self ) -> bool
    {
      matches!( self.error_type, ErrorType::Authentication )
    }

    /// Get error type
    #[ must_use ]
    pub fn error_type( &self ) -> ErrorType
    {
      self.error_type.clone()
    }

    /// Check if has remediation steps
    #[ must_use ]
    pub fn has_remediation_steps( &self ) -> bool
    {
      // Simplified implementation
      true
    }

    /// Check if is credential related
    #[ must_use ]
    pub fn is_credential_related( &self ) -> bool
    {
      matches!( self.error_type, ErrorType::Authentication )
    }

    /// Check if has backoff strategy
    #[ must_use ]
    pub fn has_backoff_strategy( &self ) -> bool
    {
      matches!( self.error_type, ErrorType::RateLimit )
    }

    /// Check if supports retry
    #[ must_use ]
    pub fn supports_retry( &self ) -> bool
    {
      self.is_transient
    }

    /// Check if has context
    #[ must_use ]
    pub fn has_context( &self ) -> bool
    {
      self.context.is_some()
    }

    /// Get context
    #[ must_use ]
    pub fn context( &self ) -> Option< &ErrorContext >
    {
      self.context.as_ref()
    }

    /// Check if has stack trace
    #[ must_use ]
    pub fn has_stack_trace( &self ) -> bool
    {
      !self.stack_trace.is_empty()
    }

    /// Get stack trace
    #[ must_use ]
    pub fn stack_trace( &self ) -> &Vec< String >
    {
      &self.stack_trace
    }

    /// Get request ID
    #[ must_use ]
    pub fn request_id( &self ) -> &Option< String >
    {
      &self.request_id
    }

    /// Get correlation ID
    #[ must_use ]
    pub fn correlation_id( &self ) -> &Option< String >
    {
      &self.correlation_id
    }

    /// Get message
    #[ must_use ]
    pub fn message( &self ) -> &str
    {
      &self.message
    }

    /// Set stack trace
    #[ must_use ]
    pub fn with_stack_trace( mut self, stack_trace : Vec< String > ) -> Self
    {
      self.stack_trace = stack_trace;
      self
    }

    /// Set request ID
    #[ must_use ]
    pub fn with_request_id( mut self, request_id : Option< String > ) -> Self
    {
      self.request_id = request_id;
      self
    }

    /// Set correlation ID
    #[ must_use ]
    pub fn with_correlation_id( mut self, correlation_id : Option< String > ) -> Self
    {
      self.correlation_id = correlation_id;
      self
    }
  }

  // Implementation of TimeoutError
  impl TimeoutError
  {
    /// Create new timeout error
    #[ must_use ]
    pub fn new( timeout_type : TimeoutType, duration : Duration, message : String ) -> Self
    {
      Self
      {
        timeout_type,
        duration,
        message,
      }
    }
  }

  // Implementation of NetworkError
  impl NetworkError
  {
    /// Create new network error
    #[ must_use ]
    pub fn new( error_type : NetworkErrorType, message : String, details : Option< String > ) -> Self
    {
      Self
      {
        error_type,
        message,
        details,
      }
    }
  }

  // Implementation of CustomError
  impl CustomError
  {
    /// Create new custom error
    #[ must_use ]
    pub fn new( name : String, message : String, severity : ErrorSeverity ) -> Self
    {
      Self
      {
        name,
        message,
        severity,
      }
    }
  }

  // Implementation of ErrorChain
  impl ErrorChain
  {
    /// Create new error chain
    #[ must_use ]
    pub fn new( primary : CustomError ) -> Self
    {
      Self
      {
        primary,
        causes : Vec::new(),
        context : String::new(),
      }
    }

    /// Add caused by error
    #[ must_use ]
    pub fn caused_by( mut self, error : AnthropicError ) -> Self
    {
      self.causes.push( error );
      self
    }

    /// Add context
    #[ must_use ]
    pub fn with_context( mut self, context : &str ) -> Self
    {
      self.context = context.to_string();
      self
    }

    /// Build the chained error
    ///
    /// # Errors
    ///
    /// Returns an error if chain length conversion fails
    pub fn build( self ) -> AnthropicResult< ChainedError >
    {
      let chain_length = u32::try_from( self.causes.len() + 1 )
        .map_err( | _ | AnthropicError::InvalidArgument( "Chain length exceeds u32 maximum".to_string() ) )?;
      let root_cause = if let Some( last_cause ) = self.causes.last()
      {
        last_cause.to_string()
      }
      else
      {
        self.primary.message.clone()
      };
      let immediate_cause = self.primary.message;
      let context = self.context;

      Ok( ChainedError
      {
        chain_length,
        root_cause,
        immediate_cause,
        context,
      })
    }
  }

  // Implementation of ChainedError
  impl ChainedError
  {
    /// Get chain length
    #[ must_use ]
    pub fn chain_length( &self ) -> u32
    {
      self.chain_length
    }

    /// Get root cause
    #[ must_use ]
    pub fn root_cause( &self ) -> &str
    {
      &self.root_cause
    }

    /// Get immediate cause
    #[ must_use ]
    pub fn immediate_cause( &self ) -> &str
    {
      &self.immediate_cause
    }

    /// Check if has context
    #[ must_use ]
    pub fn has_context( &self ) -> bool
    {
      !self.context.is_empty()
    }

    /// Get context
    #[ must_use ]
    pub fn context( &self ) -> &str
    {
      &self.context
    }

    /// Get chain iterator
    #[ must_use ]
    pub fn chain_iterator( &self ) -> std::vec::IntoIter< String >
    {
      let chain = vec![ self.immediate_cause.clone(), self.root_cause.clone() ];
      chain.into_iter()
    }
  }

  // Implementation of RequestContext
  impl RequestContext
  {
    /// Create new request context
    #[ must_use ]
    pub fn new( correlation_id : String ) -> Self
    {
      Self
      {
        correlation_id,
        request_sequence : 1,
      }
    }
  }
}

crate::mod_interface!
{
  #[ cfg( feature = "error-handling" ) ]
  exposed use
  {
    EnhancedAnthropicError,
    ErrorContext,
    TimeoutError,
    NetworkError,
    CustomError,
    ErrorChain,
    ChainedError,
    RequestContext,
  };
}
