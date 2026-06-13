//! WebSocket connection management and lifecycle

use super::protocol::*;
use crate::error::Error;
use crate::models::websocket_streaming::*;
use std::collections::HashMap;
use std::sync::{ Arc, RwLock };
use std::sync::atomic::{ AtomicU64, AtomicBool, Ordering };
use std::time::Instant;
use tokio::sync::{ mpsc, broadcast };

/// WebSocket connection manager for lifecycle management
#[ derive( Debug ) ]
pub struct WebSocketConnectionManager
{
  /// Active connections mapped by session ID
  connections : Arc< RwLock< HashMap< String, Arc< WebSocketStreamSession > > > >,
  /// Connection pool configuration
  pool_config : WebSocketPoolConfig,
  /// Global connection metrics
  global_metrics : Arc< RwLock< WebSocketMetrics > >,
  /// Connection counter for generating unique IDs
  connection_counter : Arc< AtomicU64 >,
  /// Manager status
  is_running : Arc< AtomicBool >,
}

/// Individual WebSocket streaming session
#[ derive( Debug ) ]
pub struct WebSocketStreamSession
{
  /// Unique session identifier
  pub session_id : String,
  /// Session state
  state : Arc< RwLock< StreamSessionState > >,
  /// WebSocket connection
  connection : WebSocketConnection,
  /// Message sender for outbound messages
  message_sender : mpsc::UnboundedSender< WebSocketStreamMessage >,
  /// Message receiver for inbound messages
  message_receiver : Arc< RwLock< Option< mpsc::UnboundedReceiver< WebSocketStreamMessage > > > >,
  /// Broadcast sender for publishing messages to multiple listeners
  broadcast_sender : broadcast::Sender< WebSocketStreamMessage >,
  /// Session configuration
  config : WebSocketConfig,
  /// Session metrics
  metrics : Arc< RwLock< SessionMetrics > >,
  /// Creation timestamp
  created_at : Instant,
}

/// Streaming control interface for managing active streams
#[ derive( Debug ) ]
pub struct StreamController
{
  /// Reference to session
  session : Arc< WebSocketStreamSession >,
  /// Control message sender
  control_sender : mpsc::UnboundedSender< StreamControl >,
}

impl WebSocketConnectionManager
{
  /// Create a new WebSocket connection manager
  pub fn new( pool_config : WebSocketPoolConfig ) -> Self
  {
    Self {
      connections : Arc::new( RwLock::new( HashMap::new() ) ),
      pool_config,
      global_metrics : Arc::new( RwLock::new( WebSocketMetrics::default() ) ),
      connection_counter : Arc::new( AtomicU64::new( 0 ) ),
      is_running : Arc::new( AtomicBool::new( false ) ),
    }
  }

  /// Start the connection manager
  pub async fn start( &self ) -> Result< (), Error >
  {
    self.is_running.store( true, Ordering::Relaxed );
    Ok( () )
  }

  /// Stop the connection manager and close all connections
  // await_holding_lock: write guard must be held across loop to drain all connections atomically
  #[ allow( clippy::await_holding_lock ) ]
  pub async fn stop( &self ) -> Result< (), Error >
  {
    self.is_running.store( false, Ordering::Relaxed );

    // Close all active connections
    if let Ok( mut connections ) = self.connections.write()
    {
      for ( _session_id, session ) in connections.drain()
      {
        let _ = session.close().await;
      }
    }

    Ok( () )
  }

  /// Create a new WebSocket streaming session
  pub async fn create_session( &self, endpoint : &str, config : WebSocketConfig ) -> Result< String, Error >
  {
    let session_id = format!( "ws_session_{}", self.connection_counter.fetch_add( 1, Ordering::Relaxed ) );

    // Create WebSocket connection
    let connection = WebSocketConnection::connect( endpoint, config.clone() ).await?;

    // Create message channels
    let ( message_sender, message_receiver ) = mpsc::unbounded_channel();
    let ( broadcast_sender, _broadcast_receiver ) = broadcast::channel( 1000 );

    let session = Arc::new( WebSocketStreamSession {
      session_id : session_id.clone(),
      state : Arc::new( RwLock::new( StreamSessionState::Initializing ) ),
      connection,
      message_sender,
      message_receiver : Arc::new( RwLock::new( Some( message_receiver ) ) ),
      broadcast_sender,
      config,
      metrics : Arc::new( RwLock::new( SessionMetrics::default() ) ),
      created_at : Instant::now(),
    } );

    // Store session
    if let Ok( mut connections ) = self.connections.write()
    {
      connections.insert( session_id.clone(), session );
    }

    Ok( session_id )
  }

  /// Get a session by ID
  pub fn get_session( &self, session_id : &str ) -> Option< Arc< WebSocketStreamSession > >
  {
    if let Ok( connections ) = self.connections.read()
    {
      connections.get( session_id ).cloned()
    } else {
      None
    }
  }

  /// Remove a session
  // await_holding_lock: write guard held while awaiting close() to prevent double-remove race
  #[ allow( clippy::await_holding_lock ) ]
  pub async fn remove_session( &self, session_id : &str ) -> Result< (), Error >
  {
    if let Ok( mut connections ) = self.connections.write()
    {
      if let Some( session ) = connections.remove( session_id )
      {
        session.close().await?;
      }
    }
    Ok( () )
  }

  /// Get global metrics
  pub fn get_metrics( &self ) -> WebSocketMetrics
  {
    self.global_metrics.read().unwrap_or_else( | poisoned | poisoned.into_inner() ).clone()
  }

  /// List active session IDs
  pub fn list_sessions( &self ) -> Vec< String >
  {
    if let Ok( connections ) = self.connections.read()
    {
      connections.keys().cloned().collect()
    } else {
      Vec::new()
    }
  }

  /// Get pool configuration
  pub fn get_pool_config( &self ) -> &WebSocketPoolConfig
  {
    &self.pool_config
  }
}

impl WebSocketStreamSession
{
  /// Send a message through the stream
  pub async fn send_message( &self, message : WebSocketStreamMessage ) -> Result< (), Error >
  {
    self.message_sender.send( message )
      .map_err( | e | Error::ServerError( format!( "Failed to send message : {}", e ) ) )?;

    // Update metrics
    if let Ok( mut metrics ) = self.metrics.write()
    {
      metrics.messages_sent += 1;
      metrics.last_activity = Some( std::time::SystemTime::now().duration_since( std::time::UNIX_EPOCH ).unwrap().as_secs() );
    }

    Ok( () )
  }

  /// Subscribe to incoming messages
  pub fn subscribe( &self ) -> broadcast::Receiver< WebSocketStreamMessage >
  {
    self.broadcast_sender.subscribe()
  }

  /// Get session state
  pub fn get_state( &self ) -> StreamSessionState
  {
    self.state.read().unwrap_or_else( | poisoned | poisoned.into_inner() ).clone()
  }

  /// Set session state
  pub fn set_state( &self, new_state : StreamSessionState )
  {
    if let Ok( mut state ) = self.state.write()
    {
      *state = new_state;
    }
  }

  /// Get session metrics
  pub fn get_metrics( &self ) -> SessionMetrics
  {
    self.metrics.read().unwrap_or_else( | poisoned | poisoned.into_inner() ).clone()
  }

  /// Get session configuration
  pub fn get_config( &self ) -> &WebSocketConfig
  {
    &self.config
  }

  /// Get session creation time
  pub fn get_created_at( &self ) -> Instant
  {
    self.created_at
  }

  /// Get message receiver (for advanced usage)
  pub fn get_message_receiver( &self ) -> &Arc< RwLock< Option< mpsc::UnboundedReceiver< WebSocketStreamMessage > > > >
  {
    &self.message_receiver
  }

  /// Close the session
  pub async fn close( &self ) -> Result< (), Error >
  {
    self.set_state( StreamSessionState::Terminated );
    self.connection.close().await
  }
}

impl StreamController
{
  /// Create a new stream controller
  pub fn new( session : Arc< WebSocketStreamSession > ) -> Self
  {
    let ( control_sender, _control_receiver ) = mpsc::unbounded_channel();

    Self {
      session,
      control_sender,
    }
  }

  /// Start streaming
  pub async fn start( &self ) -> Result< (), Error >
  {
    self.session.set_state( StreamSessionState::Active );
    self.send_control( StreamControl::Start ).await
  }

  /// Pause streaming
  pub async fn pause( &self ) -> Result< (), Error >
  {
    self.session.set_state( StreamSessionState::Paused );
    self.send_control( StreamControl::Pause ).await
  }

  /// Resume streaming
  pub async fn resume( &self ) -> Result< (), Error >
  {
    self.session.set_state( StreamSessionState::Active );
    self.send_control( StreamControl::Resume ).await
  }

  /// Stop streaming
  pub async fn stop( &self ) -> Result< (), Error >
  {
    self.session.set_state( StreamSessionState::Terminated );
    self.send_control( StreamControl::Stop ).await
  }

  /// Reset streaming
  pub async fn reset( &self ) -> Result< (), Error >
  {
    self.session.set_state( StreamSessionState::Initializing );
    self.send_control( StreamControl::Reset ).await
  }

  /// Send control command
  async fn send_control( &self, command : StreamControl ) -> Result< (), Error >
  {
    // Send control command through control channel
    self.control_sender.send( command.clone() )
      .map_err( | e | Error::ServerError( format!( "Failed to send control command : {}", e ) ) )?;

    // Also send as WebSocket message
    let control_message = WebSocketStreamMessage::Control {
      command,
      metadata : None,
    };

    self.session.send_message( control_message ).await
  }

  /// Get session metrics
  pub fn get_session_metrics( &self ) -> SessionMetrics
  {
    self.session.get_metrics()
  }
}
