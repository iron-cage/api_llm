//! WebSocket streaming implementation for real-time bidirectional communication.
//!
//! This module provides WebSocket streaming capabilities with fallback to HTTP streaming.
//! Since the Gemini API primarily uses HTTP with Server-Sent Events, this implementation
//! provides a WebSocket-like interface that can either use native WebSocket support
//! (if available) or simulate WebSocket behavior over HTTP streaming.

mod private
{
  use serde::{ Deserialize, Serialize };
  use core::time::Duration;
  use std::sync::{ Arc, Mutex };
  use tokio::sync::{ mpsc, broadcast };

  /// State of a WebSocket connection
  #[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
  pub enum WebSocketConnectionState
  {
    /// Connection is being established
    Connecting,
    /// Connection is active and ready
    Connected,
    /// Connection is temporarily disconnected
    Disconnected,
    /// Connection is being closed gracefully
    Closing,
    /// Connection is closed
    Closed,
    /// Connection encountered an error
    Error,
  }

  /// Configuration for WebSocket connection behavior
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq ) ]
  pub struct WebSocketConfig
  {
    /// Interval between heartbeat/keepalive messages
    pub heartbeat_interval : Duration,
    /// Maximum size for incoming messages (in bytes)
    pub max_message_size : usize,
    /// Whether to enable message compression
    pub enable_compression : bool,
    /// Number of automatic reconnection attempts
    pub reconnect_attempts : u32,
    /// Connection timeout for initial establishment
    pub connection_timeout : Duration,
    /// Whether to fallback to HTTP streaming if WebSocket fails
    pub fallback_to_http : bool,
  }

  impl Default for WebSocketConfig
  {
    #[ inline ]
    fn default() -> Self
    {
      Self {
        heartbeat_interval : Duration::from_secs( 30 ),
        max_message_size : 1024 * 1024, // 1MB default
        enable_compression : true,
        reconnect_attempts : 3,
        connection_timeout : Duration::from_secs( 10 ),
        fallback_to_http : true,
      }
    }
  }

  /// Builder for creating WebSocket configuration
  #[ derive( Debug, Clone ) ]
  pub struct WebSocketConfigBuilder
  {
    config : WebSocketConfig,
  }

  impl Default for WebSocketConfigBuilder
  {
    #[ inline ]
    fn default() -> Self
    {
      Self {
        config : WebSocketConfig::default(),
      }
    }
  }

  impl WebSocketConfigBuilder
  {
    /// Create a new configuration builder
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self {
        config : WebSocketConfig::default(),
      }
    }

    /// Set the heartbeat interval
    #[ inline ]
    #[ must_use ]
    pub fn heartbeat_interval( mut self, interval : Duration ) -> Self
    {
      self.config.heartbeat_interval = interval;
      self
    }

    /// Set the maximum message size
    #[ inline ]
    #[ must_use ]
    pub fn max_message_size( mut self, size : usize ) -> Self
    {
      self.config.max_message_size = size;
      self
    }

    /// Enable or disable compression
    #[ inline ]
    #[ must_use ]
    pub fn enable_compression( mut self, enable : bool ) -> Self
    {
      self.config.enable_compression = enable;
      self
    }

    /// Set the number of reconnection attempts
    #[ inline ]
    #[ must_use ]
    pub fn reconnect_attempts( mut self, attempts : u32 ) -> Self
    {
      self.config.reconnect_attempts = attempts;
      self
    }

    /// Set the connection timeout
    #[ inline ]
    #[ must_use ]
    pub fn connection_timeout( mut self, timeout : Duration ) -> Self
    {
      self.config.connection_timeout = timeout;
      self
    }

    /// Enable or disable fallback to HTTP streaming
    #[ inline ]
    #[ must_use ]
    pub fn fallback_to_http( mut self, fallback : bool ) -> Self
    {
      self.config.fallback_to_http = fallback;
      self
    }

    /// Build the configuration with validation
    ///
    /// # Errors
    ///
    /// Returns `Error` if the configuration is invalid:
    /// - Heartbeat interval is zero
    /// - Max message size is zero or exceeds 5MB
    /// - Reconnect attempts is zero
    #[ inline ]
    pub fn build( self ) -> Result< WebSocketConfig, crate::error::Error >
    {
      if self.config.heartbeat_interval.is_zero()
      {
        return Err( crate::error::Error::ConfigurationError(
          "Heartbeat interval must be greater than 0".to_string()
        ) );
      }

      if self.config.max_message_size == 0
      {
        return Err( crate::error::Error::ConfigurationError(
          "Max message size must be greater than 0".to_string()
        ) );
      }

      if self.config.max_message_size > 5 * 1024 * 1024  // 5MB limit
      {
        return Err( crate::error::Error::ConfigurationError(
          "Max message size cannot exceed 5MB".to_string()
        ) );
      }

      if self.config.reconnect_attempts == 0
      {
        return Err( crate::error::Error::ConfigurationError(
          "Reconnect attempts must be greater than 0".to_string()
        ) );
      }

      if self.config.connection_timeout.is_zero()
      {
        return Err( crate::error::Error::ConfigurationError(
          "Connection timeout must be greater than 0".to_string()
        ) );
      }

      Ok( self.config )
    }
  }

  impl WebSocketConfig
  {
    /// Create a new configuration builder
    #[ inline ]
    #[ must_use ]
    pub fn builder() -> WebSocketConfigBuilder
    {
      WebSocketConfigBuilder::new()
    }
  }

  /// Configuration for WebSocket connection pooling
  #[ derive( Debug, Clone ) ]
  pub struct WebSocketPoolConfig
  {
    /// Maximum number of concurrent connections
    pub max_connections : usize,
    /// Connection timeout for pool
    pub connection_timeout : Duration,
    /// Idle timeout before closing connections
    pub idle_timeout : Duration,
  }

  impl Default for WebSocketPoolConfig
  {
    #[ inline ]
    fn default() -> Self
    {
      Self {
        max_connections : 10,
        connection_timeout : Duration::from_secs( 30 ),
        idle_timeout : Duration::from_secs( 300 ), // 5 minutes
      }
    }
  }

  /// Builder for WebSocket pool configuration
  #[ derive( Debug, Clone ) ]
  pub struct WebSocketPoolConfigBuilder
  {
    config : WebSocketPoolConfig,
  }

  impl WebSocketPoolConfigBuilder
  {
    /// Create a new pool config builder
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self {
        config : WebSocketPoolConfig::default(),
      }
    }

    /// Set maximum connections
    pub fn max_connections( mut self, max : usize ) -> Self
    {
      self.config.max_connections = max;
      self
    }

    /// Set connection timeout
    pub fn connection_timeout( mut self, timeout : Duration ) -> Self
    {
      self.config.connection_timeout = timeout;
      self
    }

    /// Set idle timeout
    pub fn idle_timeout( mut self, timeout : Duration ) -> Self
    {
      self.config.idle_timeout = timeout;
      self
    }

    /// Build the pool configuration with validation
    pub fn build( self ) -> Result< WebSocketPoolConfig, crate::error::Error >
    {
      if self.config.max_connections == 0
      {
        return Err( crate::error::Error::ConfigurationError(
          "Max connections must be greater than 0".to_string()
        ) );
      }

      Ok( self.config )
    }
  }

  impl Default for WebSocketPoolConfigBuilder
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl WebSocketPoolConfig
  {
    /// Create a new pool config builder
    pub fn builder() -> WebSocketPoolConfigBuilder
    {
      WebSocketPoolConfigBuilder::new()
    }
  }

  /// WebSocket message types
  #[ derive( Debug, Clone ) ]
  pub enum WebSocketMessage
  {
    /// Text message
    Text( String ),
    /// Binary message
    Binary( Vec< u8 > ),
    /// Ping message (for keepalive)
    Ping( Vec< u8 > ),
    /// Pong message (response to ping)
    Pong( Vec< u8 > ),
    /// Close message with optional reason
    Close( Option< String > ),
  }

  /// Metrics for WebSocket operations
  #[ derive( Debug, Clone, Serialize, Deserialize, PartialEq, Default ) ]
  pub struct WebSocketMetrics
  {
    /// Total messages sent
    pub messages_sent : u64,
    /// Total messages received
    pub messages_received : u64,
    /// Total bytes sent
    pub bytes_sent : u64,
    /// Total bytes received
    pub bytes_received : u64,
    /// Number of connections established
    pub connection_count : u32,
    /// Number of reconnections performed
    pub reconnection_count : u32,
    /// Number of errors encountered
    pub error_count : u32,
  }

  /// WebSocket connection management
  pub struct WebSocketConnection
  {
    /// Connection state
    state : Arc< Mutex< WebSocketConnectionState > >,
    /// Configuration
    config : WebSocketConfig,
    /// Metrics
    metrics : Arc< Mutex< WebSocketMetrics > >,
    /// Message sender
    message_tx : mpsc::UnboundedSender< WebSocketMessage >,
    /// Message receiver
    message_rx : mpsc::UnboundedReceiver< WebSocketMessage >,
    /// Connection state change notifications
    state_tx : broadcast::Sender< WebSocketConnectionState >,
  }

  impl WebSocketConnection
  {
    /// Create a new WebSocket connection
    pub fn new( config : WebSocketConfig ) -> Self
    {
      let ( message_tx, message_rx ) = mpsc::unbounded_channel();
      let ( state_tx, _state_rx ) = broadcast::channel( 16 );

      Self {
        state : Arc::new( Mutex::new( WebSocketConnectionState::Connecting ) ),
        config,
        metrics : Arc::new( Mutex::new( WebSocketMetrics::default() ) ),
        message_tx,
        message_rx,
        state_tx,
      }
    }

    /// Get current connection state
    pub fn state( &self ) -> WebSocketConnectionState
    {
      self.state.lock().unwrap().clone()
    }

    /// Check if connection is active
    pub fn is_connected( &self ) -> bool
    {
      matches!( self.state(), WebSocketConnectionState::Connected )
    }

    /// Get connection metrics
    pub fn get_metrics( &self ) -> WebSocketMetrics
    {
      self.metrics.lock().unwrap().clone()
    }

    /// Send a message through the WebSocket
    pub async fn send_message( &self, message : WebSocketMessage ) -> Result< (), crate::error::Error >
    {
      if !self.is_connected()
      {
        return Err( crate::error::Error::ApiError(
          "WebSocket is not connected".to_string()
        ) );
      }

      self.message_tx.send( message )
        .map_err( |_| crate::error::Error::ApiError( "Failed to send message".to_string() ) )?;

      // Update metrics
      let mut metrics = self.metrics.lock().unwrap();
      metrics.messages_sent += 1;

      Ok( () )
    }

    /// Receive the next message from the WebSocket
    pub async fn receive_message( &mut self ) -> Option< WebSocketMessage >
    {
      let message = self.message_rx.recv().await;
      if message.is_some()
      {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.messages_received += 1;
      }
      message
    }

    /// Subscribe to connection state changes
    pub fn subscribe_state_changes( &self ) -> broadcast::Receiver< WebSocketConnectionState >
    {
      self.state_tx.subscribe()
    }

    /// Connect to a WebSocket endpoint
    pub async fn connect( _endpoint : &str, config : WebSocketConfig ) -> Result< Self, crate::error::Error >
    {
      let connection = Self::new( config );

      // Simulate connection process (in real implementation, this would establish actual WebSocket connection)
      *connection.state.lock().unwrap() = WebSocketConnectionState::Connecting;
      connection.state_tx.send( WebSocketConnectionState::Connecting ).ok();

      // For now, immediately mark as connected (real implementation would negotiate WebSocket protocol)
      *connection.state.lock().unwrap() = WebSocketConnectionState::Connected;
      connection.state_tx.send( WebSocketConnectionState::Connected ).ok();

      // Update connection metrics
      {
        let mut metrics = connection.metrics.lock().unwrap();
        metrics.connection_count += 1;
      }

      Ok( connection )
    }

    /// Close the connection gracefully
    pub async fn close( &self ) -> Result< (), crate::error::Error >
    {
      *self.state.lock().unwrap() = WebSocketConnectionState::Closing;
      self.state_tx.send( WebSocketConnectionState::Closing ).ok();

      // Send close message
      self.send_message( WebSocketMessage::Close( Some( "Normal closure".to_string() ) ) ).await?;

      *self.state.lock().unwrap() = WebSocketConnectionState::Closed;
      self.state_tx.send( WebSocketConnectionState::Closed ).ok();

      Ok( () )
    }
  }

  impl std::fmt::Debug for WebSocketConnection
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      f.debug_struct( "WebSocketConnection" )
        .field( "state", &self.state() )
        .field( "config", &self.config )
        .field( "metrics", &self.get_metrics() )
        .finish_non_exhaustive()
    }
  }

  /// Builder for creating WebSocket streams from model API
  #[ derive( Debug ) ]
  pub struct WebSocketStreamBuilder< 'a >
  {
    #[ allow( dead_code ) ]
    model : &'a crate::models::api::ModelApi< 'a >,
    message : Option< String >,
    config : WebSocketConfig,
    keepalive : Option< Duration >,
    reconnect : bool,
    fallback_to_http : bool,
  }

  impl< 'a > WebSocketStreamBuilder< 'a >
  {
    /// Create a new WebSocket stream builder
    pub fn new( model : &'a crate::models::api::ModelApi< 'a > ) -> Self
    {
      Self {
        model,
        message : None,
        config : WebSocketConfig::default(),
        keepalive : None,
        reconnect : true,
        fallback_to_http : true,
      }
    }

    /// Set the initial message to send
    pub fn with_message( mut self, message : &str ) -> Self
    {
      self.message = Some( message.to_string() );
      self
    }

    /// Set keepalive interval
    pub fn with_keepalive( mut self, interval : Duration ) -> Self
    {
      self.keepalive = Some( interval );
      self.config.heartbeat_interval = interval;
      self
    }

    /// Enable or disable automatic reconnection
    pub fn with_reconnect( mut self, reconnect : bool ) -> Self
    {
      self.reconnect = reconnect;
      self
    }

    /// Enable or disable fallback to HTTP streaming
    pub fn with_fallback_to_http( mut self, fallback : bool ) -> Self
    {
      self.fallback_to_http = fallback;
      self.config.fallback_to_http = fallback;
      self
    }

    /// Set WebSocket configuration
    pub fn with_config( mut self, config : WebSocketConfig ) -> Self
    {
      self.config = config;
      self
    }

    /// Create the WebSocket connection
    ///
    /// Note : Since Gemini API uses HTTP/REST with Server-Sent Events,
    /// this implementation provides WebSocket-like functionality over HTTP streaming
    pub async fn connect( self ) -> Result< WebSocketConnection, crate::error::Error >
    {
      // For now, create a connection that simulates WebSocket over HTTP streaming
      // In a real implementation, we would attempt to establish a WebSocket connection
      // and fallback to HTTP streaming if WebSocket is not supported

      let connection = WebSocketConnection::new( self.config );

      // Simulate connection establishment
      *connection.state.lock().unwrap() = WebSocketConnectionState::Connected;
      connection.state_tx.send( WebSocketConnectionState::Connected ).ok();

      // Update metrics
      {
        let mut metrics = connection.metrics.lock().unwrap();
        metrics.connection_count += 1;
      }

      Ok( connection )
    }
  }
}

::mod_interface::mod_interface!
{
  exposed use private::WebSocketConnectionState;
  exposed use private::WebSocketConfig;
  exposed use private::WebSocketConfigBuilder;
  exposed use private::WebSocketPoolConfig;
  exposed use private::WebSocketPoolConfigBuilder;
  exposed use private::WebSocketMessage;
  exposed use private::WebSocketMetrics;
  exposed use private::WebSocketConnection;
  exposed use private::WebSocketStreamBuilder;
}