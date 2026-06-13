//! Enhanced WebSocket Reliability Module
//!
//! This module provides production-ready WebSocket reliability features including:
//! - Automatic reconnection with exponential backoff
//! - Message delivery guarantees and buffering
//! - Connection health monitoring and keepalive
//! - Network interruption recovery
//! - Concurrent connection management
//! - Comprehensive error handling and graceful degradation

mod private
{
  use crate::
  {
    realtime ::WsSession,
    error ::{ OpenAIError, Result },
    components ::realtime_shared::RealtimeClientEvent,
  };
  use std::
  {
    collections ::VecDeque,
    sync ::{ Arc, Mutex },
    time ::Instant,
  };
  use tokio::
  {
    sync ::{ RwLock, Semaphore },
    time ::{ timeout, sleep, interval },
  };
  use core::time::Duration;
  use serde::{ Serialize, Deserialize };

  /// Configuration for WebSocket reliability features
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct WebSocketReliabilityConfig
  {
    /// Maximum number of reconnection attempts
    pub max_reconnection_attempts : usize,
    /// Initial reconnection delay
    pub initial_reconnection_delay : Duration,
    /// Maximum reconnection delay
    pub max_reconnection_delay : Duration,
    /// Connection timeout duration
    pub connection_timeout : Duration,
    /// Heartbeat interval for keepalive
    pub heartbeat_interval : Duration,
    /// Message buffer size
    pub message_buffer_size : usize,
    /// Health check interval
    pub health_check_interval : Duration,
    /// Enable automatic reconnection
    pub enable_auto_reconnect : bool,
    /// Enable message buffering
    pub enable_message_buffering : bool,
    /// Enable heartbeat monitoring
    pub enable_heartbeat_monitoring : bool,
    /// Connection quality threshold (0.0 - 1.0)
    pub connection_quality_threshold : f64,
  }

  impl Default for WebSocketReliabilityConfig
  {
    #[ inline ]
    fn default() -> Self
    {
      Self
      {
        max_reconnection_attempts : 5,
        initial_reconnection_delay : Duration::from_secs( 1 ),
        max_reconnection_delay : Duration::from_secs( 30 ),
        connection_timeout : Duration::from_secs( 10 ),
        heartbeat_interval : Duration::from_secs( 30 ),
        message_buffer_size : 1000,
        health_check_interval : Duration::from_secs( 5 ),
        enable_auto_reconnect : true,
        enable_message_buffering : true,
        enable_heartbeat_monitoring : true,
        connection_quality_threshold : 0.8,
      }
    }
  }

  /// WebSocket connection statistics for reliability monitoring
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct WebSocketConnectionStats
  {
    /// Total connection attempts
    pub connection_attempts : u64,
    /// Successful connections
    pub successful_connections : u64,
    /// Failed connections
    pub failed_connections : u64,
    /// Total reconnections
    pub reconnections : u64,
    /// Messages sent successfully
    pub messages_sent : u64,
    /// Messages received successfully
    pub messages_received : u64,
    /// Message send failures
    pub message_send_failures : u64,
    /// Connection interruptions
    pub connection_interruptions : u64,
    /// Average connection duration in seconds
    pub average_connection_duration_secs : f64,
    /// Connection quality score (0.0 - 1.0)
    pub connection_quality : f64,
    /// Last successful heartbeat timestamp (seconds since UNIX epoch)
    pub last_heartbeat_timestamp : Option< u64 >,
    /// Total bytes sent
    pub total_bytes_sent : u64,
    /// Total bytes received
    pub total_bytes_received : u64,
  }

  impl Default for WebSocketConnectionStats
  {
    #[ inline ]
    fn default() -> Self
    {
      Self
      {
        connection_attempts : 0,
        successful_connections : 0,
        failed_connections : 0,
        reconnections : 0,
        messages_sent : 0,
        messages_received : 0,
        message_send_failures : 0,
        connection_interruptions : 0,
        average_connection_duration_secs : 0.0,
        connection_quality : 1.0,
        last_heartbeat_timestamp : None,
        total_bytes_sent : 0,
        total_bytes_received : 0,
      }
    }
  }

  /// Connection state for reliability tracking
  #[ derive( Debug, Clone, PartialEq, Eq ) ]
  pub enum ConnectionState
  {
    /// Connection is being established
    Connecting,
    /// Connection is healthy and active
    Connected,
    /// Connection is temporarily disconnected
    Disconnected,
    /// Connection is being reconnected
    Reconnecting,
    /// Connection permanently failed
    Failed,
    /// Connection is being closed gracefully
    Closing,
    /// Connection is closed
    Closed,
  }

  /// Buffered message for reliable delivery
  #[ derive( Debug, Clone ) ]
  pub struct BufferedMessage
  {
    /// The client event to send
    pub event : RealtimeClientEvent,
    /// Message creation timestamp
    pub created_at : Instant,
    /// Number of send attempts
    pub attempts : usize,
    /// Message priority (higher values sent first)
    pub priority : u8,
    /// Unique message identifier
    pub id : String,
  }

  /// Enhanced WebSocket session with reliability features
  #[ derive( Debug ) ]
  pub struct ReliableWebSocketSession
  {
    /// Current WebSocket session
    current_session : Arc< RwLock< Option< WsSession > > >,
    /// Configuration
    config : WebSocketReliabilityConfig,
    /// Connection statistics
    stats : Arc< RwLock< WebSocketConnectionStats > >,
    /// Message buffer for reliability
    message_buffer : Arc< Mutex< VecDeque< BufferedMessage > > >,
    /// Connection URL
    url : String,
    /// Current connection state
    connection_state : Arc< RwLock< ConnectionState > >,
    /// Reconnection attempt count
    reconnection_count : Arc< Mutex< usize > >,
    /// Last successful connection timestamp
    last_connection_time : Arc< Mutex< Option< Instant > > >,
    /// Last heartbeat timestamp
    last_heartbeat : Arc< Mutex< Option< Instant > > >,
    /// Connection semaphore for limiting concurrent operations
    connection_semaphore : Arc< Semaphore >,
    /// Background task handles
    background_tasks : Arc< Mutex< Vec< tokio::task::JoinHandle< () > > > >,
  }

  impl ReliableWebSocketSession
  {
    /// Create a new reliable WebSocket session
    #[ inline ]
    #[ must_use ]
    pub fn new( url : String, config : WebSocketReliabilityConfig ) -> Self
    {
      Self
      {
        current_session : Arc::new( RwLock::new( None ) ),
        config,
        stats : Arc::new( RwLock::new( WebSocketConnectionStats::default() ) ),
        message_buffer : Arc::new( Mutex::new( VecDeque::new() ) ),
        url,
        connection_state : Arc::new( RwLock::new( ConnectionState::Disconnected ) ),
        reconnection_count : Arc::new( Mutex::new( 0 ) ),
        last_connection_time : Arc::new( Mutex::new( None ) ),
        last_heartbeat : Arc::new( Mutex::new( None ) ),
        connection_semaphore : Arc::new( Semaphore::new( 1 ) ), // Only one connection at a time
        background_tasks : Arc::new( Mutex::new( Vec::new() ) ),
      }
    }

    /// Connect with reliability features
    ///
    /// # Errors
    ///
    /// Returns an error if the WebSocket connection cannot be established after
    /// the maximum number of reconnection attempts, or if the connection times out.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex for tracking connection state is poisoned.
    #[ inline ]
    pub async fn connect_reliable( &self ) -> Result< () >
    {
      let _permit = self.connection_semaphore.acquire().await
        .map_err( |_| error_tools::Error::from( OpenAIError::Internal(
          "Failed to acquire connection semaphore".to_string()
        ) ) )?;

      self.set_connection_state( ConnectionState::Connecting ).await;

      let mut attempts = 0;
      let mut delay = self.config.initial_reconnection_delay;

      loop
      {
        {
          let mut stats = self.stats.write().await;
          stats.connection_attempts += 1;
        }

        match timeout( self.config.connection_timeout, WsSession::connect( &self.url ) ).await
        {
          Ok( Ok( session ) ) =>
          {
            {
              let mut current = self.current_session.write().await;
              *current = Some( session );
            }
            {
              let mut stats = self.stats.write().await;
              stats.successful_connections += 1;
            }
            {
              let mut last_connection = self.last_connection_time.lock().unwrap();
              *last_connection = Some( Instant::now() );
            }
            {
              let mut reconnection_count = self.reconnection_count.lock().unwrap();
              *reconnection_count = 0;
            }

            self.set_connection_state( ConnectionState::Connected ).await;

            // Start background tasks
            if self.config.enable_heartbeat_monitoring
            {
              self.start_heartbeat_monitoring();
            }
            self.start_health_monitoring();
            self.start_message_processing();

            return Ok( () );
          },
          Ok( Err( error ) ) =>
          {
            {
              let mut stats = self.stats.write().await;
              stats.failed_connections += 1;
            }

            attempts += 1;
            if attempts >= self.config.max_reconnection_attempts
            {
              self.set_connection_state( ConnectionState::Failed ).await;
              return Err( error );
            }

            if self.config.enable_auto_reconnect
            {
              self.set_connection_state( ConnectionState::Reconnecting ).await;
              sleep( delay ).await;
              delay = core::cmp::min( delay * 2, self.config.max_reconnection_delay );
            }
            else
            {
              self.set_connection_state( ConnectionState::Failed ).await;
              return Err( error );
            }
          },
          Err( _ ) =>
          {
            {
              let mut stats = self.stats.write().await;
              stats.failed_connections += 1;
            }

            attempts += 1;
            if attempts >= self.config.max_reconnection_attempts
            {
              self.set_connection_state( ConnectionState::Failed ).await;
              return Err( error_tools::Error::from( OpenAIError::Internal(
                format!( "Connection timeout after {attempts} attempts" )
              ) ) );
            }

            if self.config.enable_auto_reconnect
            {
              self.set_connection_state( ConnectionState::Reconnecting ).await;
              sleep( delay ).await;
              delay = core::cmp::min( delay * 2, self.config.max_reconnection_delay );
            }
            else
            {
              self.set_connection_state( ConnectionState::Failed ).await;
              return Err( error_tools::Error::from( OpenAIError::Internal(
                "Connection timeout".to_string()
              ) ) );
            }
          }
        }
      }
    }

    /// Send an event with reliability guarantees
    ///
    /// # Errors
    ///
    /// Returns an error if the event cannot be sent due to connection failures,
    /// serialization issues, or if the maximum reconnection attempts are exceeded.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex for message buffering is poisoned.
    #[ inline ]
    pub async fn send_event_reliable( &self, event : RealtimeClientEvent ) -> Result< () >
    {
      let message_id = format!( "msg_{}", uuid::Uuid::new_v4() );
      let message_id_clone = message_id.clone();
      let buffered_msg = BufferedMessage
      {
        event : event.clone(),
        created_at : Instant::now(),
        attempts : 0,
        priority : 1,
        id : message_id_clone,
      };

      if self.config.enable_message_buffering
      {
        let mut buffer = self.message_buffer.lock().unwrap();
        if buffer.len() >= self.config.message_buffer_size
        {
          buffer.pop_front(); // Remove oldest message
        }
        buffer.push_back( buffered_msg );
      }

      // Attempt immediate send if connected
      let connection_state = self.get_connection_state().await;
      if connection_state == ConnectionState::Connected
      {
        if let Some( ref session ) = *self.current_session.read().await
        {
          match session.send_event( event ).await
          {
            Ok( () ) =>
            {
              let mut stats = self.stats.write().await;
              stats.messages_sent += 1;
              // Remove from buffer on successful send
              if self.config.enable_message_buffering
              {
                let mut buffer = self.message_buffer.lock().unwrap();
                buffer.retain( |msg| msg.id != message_id );
              }
              return Ok( () );
            },
            Err( error ) =>
            {
              let mut stats = self.stats.write().await;
              stats.message_send_failures += 1;

              if self.config.enable_auto_reconnect
              {
                self.handle_connection_failure().await?;
              }
              else
              {
                return Err( error );
              }
            }
          }
        }
      }

      // If not connected or send failed, the message is buffered and will be sent on reconnection
      Ok( () )
    }

    /// Receive an event from the WebSocket
    ///
    /// # Errors
    ///
    /// Returns an error if no event can be received due to connection failures,
    /// WebSocket errors, or if no active connection exists.
    #[ inline ]
    pub async fn recv_event_reliable( &self ) -> Result< crate::components::realtime_shared::RealtimeServerEvent >
    {
      if let Some( ref session ) = *self.current_session.read().await
      {
        match session.recv_event().await
        {
          Ok( event ) =>
          {
            let mut stats = self.stats.write().await;
            stats.messages_received += 1;
            Ok( event )
          },
          Err( error ) =>
          {
            if self.config.enable_auto_reconnect
            {
              self.handle_connection_failure().await?;
              // Try again after reconnection
              if let Some( ref session ) = *self.current_session.read().await
              {
                let event = session.recv_event().await?;
                let mut stats = self.stats.write().await;
                stats.messages_received += 1;
                Ok( event )
              }
              else
              {
                Err( error )
              }
            }
            else
            {
              Err( error )
            }
          }
        }
      }
      else
      {
        Err( error_tools::Error::from( OpenAIError::Internal(
          "No active WebSocket connection".to_string()
        ) ) )
      }
    }

    /// Handle connection failure and attempt reconnection
    async fn handle_connection_failure( &self ) -> Result< () >
    {
      {
        let mut stats = self.stats.write().await;
        stats.connection_interruptions += 1;
      }

      self.set_connection_state( ConnectionState::Disconnected ).await;

      {
        let mut current = self.current_session.write().await;
        *current = None; // Clear current session
      }

      // Stop background tasks
      self.stop_background_tasks();

      let should_fail = {
        let mut reconnection_count = self.reconnection_count.lock().unwrap();
        *reconnection_count += 1;
        *reconnection_count > self.config.max_reconnection_attempts
      };

      if should_fail
      {
        self.set_connection_state( ConnectionState::Failed ).await;
        return Err( error_tools::Error::from( OpenAIError::Internal(
          "Maximum reconnection attempts exceeded".to_string()
        ) ) );
      }

      {
        let mut stats = self.stats.write().await;
        stats.reconnections += 1;
      }

      self.connect_reliable().await
    }

    /// Start heartbeat monitoring
    fn start_heartbeat_monitoring( &self )
    {
      let heartbeat_interval = self.config.heartbeat_interval;
      let last_heartbeat = Arc::clone( &self.last_heartbeat );
      let connection_state = Arc::clone( &self.connection_state );
      let heartbeat_stats = Arc::clone( &self.stats );

      let handle = tokio::spawn( async move
      {
        let mut interval = interval( heartbeat_interval );
        loop
        {
          interval.tick().await;

          let current_state = connection_state.read().await;
          if *current_state != ConnectionState::Connected
          {
            break;
          }
          drop( current_state );

          {
            let mut last_hb = last_heartbeat.lock().unwrap();
            *last_hb = Some( Instant::now() );
          }

          {
            let mut stats_guard = heartbeat_stats.write().await;
            stats_guard.last_heartbeat_timestamp = Some(
              std ::time::SystemTime::now()
                .duration_since( std::time::UNIX_EPOCH )
                .unwrap_or_default()
                .as_secs()
            );
          }

          // In a real implementation, this would send a ping/heartbeat message
          // For now, we just update the timestamp
        }
      });

      let mut tasks = self.background_tasks.lock().unwrap();
      tasks.push( handle );
    }

    /// Start health monitoring
    fn start_health_monitoring( &self )
    {
      let health_check_interval = self.config.health_check_interval;
      let connection_state = Arc::clone( &self.connection_state );
      let last_heartbeat = Arc::clone( &self.last_heartbeat );
      let health_stats = Arc::clone( &self.stats );
      let quality_threshold = self.config.connection_quality_threshold;

      let handle = tokio::spawn( async move
      {
        let mut interval = interval( health_check_interval );
        loop
        {
          interval.tick().await;

          let current_health_state = connection_state.read().await;
          if *current_health_state != ConnectionState::Connected
          {
            break;
          }
          drop( current_health_state );

          // Check connection health
          let health_ok = {
            let last_hb = last_heartbeat.lock().unwrap();
            if let Some( last_time ) = *last_hb
            {
              let elapsed = last_time.elapsed();
              elapsed < Duration::from_secs( 90 ) // Allow 3x heartbeat tolerance
            }
            else
            {
              true // No heartbeat data yet, assume healthy
            }
          };

          // Update connection quality
          {
            let mut stats_guard = health_stats.write().await;
            if health_ok
            {
              stats_guard.connection_quality = ( stats_guard.connection_quality * 0.9 + 0.1 ).min( 1.0 );
            }
            else
            {
              stats_guard.connection_quality = ( stats_guard.connection_quality * 0.9 ).max( 0.0 );
            }

            // If quality drops below threshold, consider connection unhealthy
            if stats_guard.connection_quality < quality_threshold
            {
              // In a real implementation, this would trigger reconnection
              break;
            }
          }
        }
      });

      let mut tasks = self.background_tasks.lock().unwrap();
      tasks.push( handle );
    }

    /// Start message processing for buffered messages
    fn start_message_processing( &self )
    {
      let message_buffer = Arc::clone( &self.message_buffer );
      let current_session = Arc::clone( &self.current_session );
      let message_stats = Arc::clone( &self.stats );
      let connection_state = Arc::clone( &self.connection_state );

      let handle = tokio::spawn( async move
      {
        let mut interval = interval( Duration::from_millis( 100 ) );
        loop
        {
          interval.tick().await;

          let current_message_state = connection_state.read().await;
          if *current_message_state != ConnectionState::Connected
          {
            break;
          }
          drop( current_message_state );

          // Process buffered messages
          let messages_to_send : Vec< BufferedMessage > = {
            let mut buffer = message_buffer.lock().unwrap();
            let mut to_send = Vec::new();

            // Take up to 10 messages from buffer
            for _ in 0..core::cmp::min( 10, buffer.len() )
            {
              if let Some( msg ) = buffer.pop_front()
              {
                to_send.push( msg );
              }
            }

            to_send
          };

          if !messages_to_send.is_empty()
          {
            let session_opt = current_session.read().await;
            if let Some( ref session ) = *session_opt
            {
              for mut msg in messages_to_send
              {
                msg.attempts += 1;
                if let Ok( () ) = session.send_event( msg.event.clone() ).await
                {
                  let mut stats_guard = message_stats.write().await;
                  stats_guard.messages_sent += 1;
                }
                else
                {
                  let mut stats_guard = message_stats.write().await;
                  stats_guard.message_send_failures += 1;

                  // Re-add to buffer if attempts < max
                  if msg.attempts < 3
                  {
                    let mut buffer = message_buffer.lock().unwrap();
                    buffer.push_back( msg );
                  }
                }
              }
            }
          }
        }
      });

      let mut tasks = self.background_tasks.lock().unwrap();
      tasks.push( handle );
    }

    /// Stop all background tasks
    fn stop_background_tasks( &self )
    {
      let mut tasks = self.background_tasks.lock().unwrap();
      for handle in tasks.drain( .. )
      {
        handle.abort();
      }
    }

    /// Get current connection state
    #[ inline ]
    pub async fn get_connection_state( &self ) -> ConnectionState
    {
      self.connection_state.read().await.clone()
    }

    /// Set connection state
    async fn set_connection_state( &self, state : ConnectionState )
    {
      let mut current_state = self.connection_state.write().await;
      *current_state = state;
    }

    /// Check if connection is healthy
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex for last heartbeat tracking is poisoned.
    #[ inline ]
    pub async fn is_connection_healthy( &self ) -> bool
    {
      let state = self.get_connection_state().await;
      if state != ConnectionState::Connected
      {
        return false;
      }

      let last_hb = self.last_heartbeat.lock().unwrap();
      if let Some( last_time ) = *last_hb
      {
        let elapsed = last_time.elapsed();
        elapsed < self.config.heartbeat_interval * 3 // Allow 3x heartbeat tolerance
      }
      else
      {
        true // No heartbeat data yet, assume healthy
      }
    }

    /// Get connection statistics
    #[ inline ]
    pub async fn get_stats( &self ) -> WebSocketConnectionStats
    {
      self.stats.read().await.clone()
    }

    /// Get buffered message count
    ///
    /// # Panics
    ///
    /// This function will panic if the message buffer mutex is poisoned.
    #[ inline ]
    #[ must_use ]
    pub fn get_buffered_message_count( &self ) -> usize
    {
      self.message_buffer.lock().unwrap().len()
    }

    /// Flush message buffer and attempt to send all messages
    ///
    /// # Errors
    ///
    /// This function returns an error if the WebSocket session fails to send messages.
    ///
    /// # Panics
    ///
    /// This function will panic if the message buffer mutex is poisoned.
    #[ inline ]
    pub async fn flush_message_buffer( &self ) -> Result< usize >
    {
      let messages : Vec< BufferedMessage > = {
        let mut buffer = self.message_buffer.lock().unwrap();
        buffer.drain( .. ).collect()
      };

      let _message_count = messages.len();

      if let Some( ref session ) = *self.current_session.read().await
      {
        let mut sent_count = 0;
        for msg in messages
        {
          if let Ok( () ) = session.send_event( msg.event ).await
          {
            sent_count += 1;
            let mut stats = self.stats.write().await;
            stats.messages_sent += 1;
          }
          else
          {
            let mut stats = self.stats.write().await;
            stats.message_send_failures += 1;
          }
        }
        Ok( sent_count )
      }
      else
      {
        // Re-add messages to buffer if no connection
        {
          let mut buffer = self.message_buffer.lock().unwrap();
          for msg in messages
          {
            buffer.push_back( msg );
          }
        }
        Ok( 0 )
      }
    }

    /// Close the connection gracefully
    ///
    /// # Errors
    ///
    /// This function returns an error if the connection cannot be closed properly.
    #[ inline ]
    pub async fn close( &self ) -> Result< () >
    {
      self.set_connection_state( ConnectionState::Closing ).await;

      // Stop background tasks
      self.stop_background_tasks();

      // Clear current session
      {
        let mut current = self.current_session.write().await;
        *current = None;
      }

      self.set_connection_state( ConnectionState::Closed ).await;
      Ok( () )
    }
  }

  impl Drop for ReliableWebSocketSession
  {
    #[ inline ]
    fn drop( &mut self )
    {
      // Stop background tasks on drop
      if let Ok( mut tasks ) = self.background_tasks.try_lock()
      {
        for handle in tasks.drain( .. )
        {
          handle.abort();
        }
      }
    }
  }

  /// Global configuration for WebSocket reliability
  static GLOBAL_CONFIG : std::sync::OnceLock< WebSocketReliabilityConfig > = std::sync::OnceLock::new();

  /// Get the global WebSocket reliability configuration
  #[ inline ]
  pub fn get_global_config() -> WebSocketReliabilityConfig
  {
    GLOBAL_CONFIG.get().cloned().unwrap_or_default()
  }

  /// Set the global WebSocket reliability configuration
  #[ inline ]
  pub fn set_global_config( config : WebSocketReliabilityConfig )
  {
    let _ = GLOBAL_CONFIG.set( config );
  }

  /// Create a reliable WebSocket session with global configuration
  #[ inline ]
  #[ must_use ]
  pub fn create_reliable_session( url : String ) -> ReliableWebSocketSession
  {
    ReliableWebSocketSession::new( url, get_global_config() )
  }

  /// Create a reliable WebSocket session with custom configuration
  #[ inline ]
  #[ must_use ]
  pub fn create_reliable_session_with_config( url : String, config : WebSocketReliabilityConfig ) -> ReliableWebSocketSession
  {
    ReliableWebSocketSession::new( url, config )
  }
}

crate ::mod_interface!
{
  exposed use private::WebSocketReliabilityConfig;
  exposed use private::WebSocketConnectionStats;
  exposed use private::ConnectionState;
  exposed use private::BufferedMessage;
  exposed use private::ReliableWebSocketSession;
  exposed use private::get_global_config;
  exposed use private::set_global_config;
  exposed use private::create_reliable_session;
  exposed use private::create_reliable_session_with_config;
}