//! CURL diagnostics functionality for Anthropic API client
//!
//! Provides debugging capabilities through cURL command generation,
//! enabling developers to replicate and troubleshoot API requests
//! outside of the Rust environment.

#[ cfg( feature = "curl-diagnostics" ) ]
mod private
{
  use std::collections::HashMap;
  use serde_json::Value;

  /// Trait for converting API requests to equivalent cURL commands
  pub trait AsCurl
  {
    /// Convert the request to a cURL command string
    ///
    /// # Arguments
    ///
    /// * `url` - The target URL for the request
    ///
    /// # Returns
    ///
    /// A formatted cURL command string that replicates the request
    fn as_curl( &self, url : &str ) -> String;
  }

  /// Trait for client-level cURL generation with authentication
  pub trait AsCurlClient
  {
    /// Generate cURL command for a request with client authentication
    ///
    /// # Arguments
    ///
    /// * `request` - The request to convert
    /// * `url` - The target URL
    ///
    /// # Returns
    ///
    /// A complete cURL command with authentication headers
    fn as_curl_for_request< T : AsCurl + serde::Serialize >( &self, request : &T, url : &str ) -> String;
  }

  /// Helper for building cURL commands
  #[ derive( Debug ) ]
  pub struct CurlBuilder
  {
    method : String,
    url : String,
    headers : HashMap< String, String >,
    body : Option< String >,
    options : Vec< String >,
  }

  impl CurlBuilder
  {
    /// Create a new cURL builder
    #[ inline ]
    #[ must_use ]
    pub fn new( url : impl Into< String > ) -> Self
    {
      Self
      {
        method : "GET".to_string(),
        url : url.into(),
        headers : HashMap::new(),
        body : None,
        options : Vec::new(),
      }
    }

    /// Set HTTP method
    #[ inline ]
    #[ must_use ]
    pub fn method( mut self, method : impl Into< String > ) -> Self
    {
      self.method = method.into();
      self
    }

    /// Add a header
    #[ inline ]
    #[ must_use ]
    pub fn header( mut self, key : impl Into< String >, value : impl Into< String > ) -> Self
    {
      self.headers.insert( key.into(), value.into() );
      self
    }

    /// Set request body
    #[ inline ]
    #[ must_use ]
    pub fn body( mut self, body : impl Into< String > ) -> Self
    {
      self.body = Some( body.into() );
      self
    }

    /// Add cURL option
    #[ inline ]
    #[ must_use ]
    pub fn option( mut self, option : impl Into< String > ) -> Self
    {
      self.options.push( option.into() );
      self
    }

    /// Build the cURL command
    #[ must_use ]
    pub fn build( &self ) -> String
    {
      let mut parts = vec![ "curl".to_string() ];

      // Add method
      if self.method != "GET"
      {
        parts.push( format!( "--request {}", self.method ) );
      }

      // Add URL
      parts.push( format!( "\"{}\"", self.url ) );

      // Add headers
      for ( key, value ) in &self.headers
      {
        parts.push( format!( "--header \"{}: {}\"", escape_header_value( key ), escape_header_value( value ) ) );
      }

      // Add body
      if let Some( ref body ) = self.body
      {
        parts.push( format!( "--data '{}'", escape_json_body( body ) ) );
      }

      // Add additional options
      for option in &self.options
      {
        parts.push( option.clone() );
      }

      // Join with line continuations for readability
      if parts.len() > 4 || parts.iter().map( std::string::String::len ).sum::< usize >() > 120
      {
        parts.join( " \\\n  " )
      }
      else
      {
        parts.join( " " )
      }
    }
  }

  /// Escape header value for cURL
  fn escape_header_value( value : &str ) -> String
  {
    value
      .replace( '\\', "\\\\" )
      .replace( '"', "\\\"" )
  }

  /// Escape JSON body for cURL
  fn escape_json_body( body : &str ) -> String
  {
    // For single quotes in shell, we can use the body as-is mostly
    // but need to handle embedded single quotes
    body.replace( '\'', "'\"'\"'" )
  }

  /// Convert `serde_json::Value` to pretty-printed JSON string for cURL
  fn format_json_for_curl( value : &Value ) -> String
  {
    serde_json::to_string( value ).unwrap_or_else( |_| "{}".to_string() )
  }

  // Implement AsCurl for CreateMessageRequest
  impl AsCurl for crate::CreateMessageRequest
  {
    fn as_curl( &self, url : &str ) -> String
    {
      let mut builder = CurlBuilder::new( url )
        .method( "POST" )
        .header( "Content-Type", "application/json" );

      // Add streaming option if enabled
      #[ cfg( feature = "streaming" ) ]
      if self.stream.unwrap_or( false )
      {
        builder = builder.option( "--no-buffer".to_string() );
      }

      // Serialize request to JSON
      if let Ok( json ) = serde_json::to_value( self )
      {
        let json_string = format_json_for_curl( &json );
        builder = builder.body( json_string );
      }

      builder.build()
    }
  }

  // Implement AsCurl for EmbeddingRequest if embeddings feature is enabled
  #[ cfg( feature = "embeddings" ) ]
  impl AsCurl for crate::EmbeddingRequest
  {
    fn as_curl( &self, url : &str ) -> String
    {
      let builder = CurlBuilder::new( url )
        .method( "POST" )
        .header( "Content-Type", "application/json" );

      // Serialize request to JSON
      let json_string = if let Ok( json ) = serde_json::to_value( self )
      {
        format_json_for_curl( &json )
      }
      else
      {
        "{}".to_string()
      };

      builder.body( json_string ).build()
    }
  }

  // Implement AsCurlClient for Client
  impl AsCurlClient for crate::Client
  {
    fn as_curl_for_request< T : AsCurl + serde::Serialize >( &self, request : &T, url : &str ) -> String
    {
      let base_curl = request.as_curl( url );

      // Extract the base command and add authentication
      let mut builder = CurlBuilder::new( url )
        .method( "POST" )
        .header( "Content-Type", "application/json" );

      // Add authentication headers
      if let Some( api_key ) = self.api_key()
      {
        builder = builder.header( "x-api-key", &api_key.ANTHROPIC_API_KEY );
      }

      // Add version header
      builder = builder.header( "anthropic-version", "2023-06-01" );

      // Add user agent
      builder = builder.header( "User-Agent", format!( "api_claude/{}", env!( "CARGO_PKG_VERSION" ) ) );

      // Get the body from the original request
      if let Ok( json ) = serde_json::to_value( request )
      {
        let json_string = format_json_for_curl( &json );
        builder = builder.body( json_string );
      }

      // Add streaming option if this is a streaming request
      if base_curl.contains( "--no-buffer" )
      {
        builder = builder.option( "--no-buffer".to_string() );
      }

      builder.build()
    }
  }

}

#[ cfg( feature = "curl-diagnostics" ) ]
crate::mod_interface!
{
  exposed use
  {
    AsCurl,
    AsCurlClient,
    CurlBuilder,
  };
}