//! Streaming control operations including pause, resume, and cancel functionality.

use core::time::Duration;
use std::time::Instant;
use std::sync::{ Arc, Mutex };
use core::sync::atomic::{ AtomicU8, Ordering };
use tokio::sync::{ mpsc, oneshot, Notify };
use futures::StreamExt;

use super::{ StreamState, StreamControlConfig, StreamMetrics, StreamMetricsSnapshot, MetricsLevel };
use super::buffer::StreamBuffer;

/// Control commands for stream management with timing information
#[ derive( Debug ) ]
#[ allow( dead_code ) ]
pub( crate ) enum StreamCommand
{
  /// Pause the stream
  Pause { response_tx : oneshot::Sender< Result< (), crate::error::Error > >, start_time : Instant },
  /// Resume the stream
  Resume { response_tx : oneshot::Sender< Result< (), crate::error::Error > >, start_time : Instant },
  /// Cancel the stream
  Cancel { response_tx : oneshot::Sender< Result< (), crate::error::Error > >, start_time : Instant },
  /// Get current state
  GetState( oneshot::Sender< StreamState > ),
  /// Get metrics snapshot
  GetMetrics( oneshot::Sender< StreamMetricsSnapshot > ),
  /// Update buffer configuration at runtime
  UpdateConfig { new_config : StreamControlConfig, response_tx : oneshot::Sender< Result< (), crate::error::Error > > },
}

/// A controllable stream that can be paused, resumed, and cancelled
pub struct ControllableStream< T >
{
  /// Channel for sending control commands
  control_tx : mpsc::UnboundedSender< StreamCommand >,
  /// Channel for receiving stream data
  data_rx : mpsc::UnboundedReceiver< Result< T, crate::error::Error > >,
  /// Current stream state (atomic for efficient access)
  state : Arc< AtomicU8 >, // StreamState as u8 for atomic operations
  /// Stream configuration (can be updated at runtime)
  config : Arc< Mutex< StreamControlConfig > >,
  /// Stream metrics with atomic updates
  metrics : Arc< StreamMetrics >,
  /// Notification for timeout events (more efficient than polling)
  #[ allow( dead_code ) ]
  timeout_notify : Arc< Notify >,
}

impl< T > std::fmt::Debug for ControllableStream< T >
where
  T: Clone + Send + 'static,
{
  fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    let current_state = StreamState::from_u8( self.state.load( Ordering::Relaxed ) );
    f.debug_struct( "ControllableStream" )
      .field( "state", &current_state )
      .field( "config", &"StreamControlConfig{..}" ) // Avoid locking in debug
      .field( "metrics", &self.metrics.snapshot() )
      .finish_non_exhaustive()
  }
}

impl< T > ControllableStream< T >
where
  T: Clone + Send + 'static,
{
  /// Create a new optimized controllable stream from a boxed stream
  pub fn new(
    stream : std::pin::Pin< Box< dyn futures::Stream< Item = Result< T, crate::error::Error > > + Send > >,
    config : StreamControlConfig
  ) -> Self
  {
    let ( control_tx, control_rx ) = mpsc::unbounded_channel();
    let ( data_tx, data_rx ) = mpsc::unbounded_channel();

    // Use atomic operations for better performance
    let state = Arc::new( AtomicU8::new( StreamState::Running.to_u8() ) );
    let metrics = Arc::new( StreamMetrics::default() );
    let config_arc = Arc::new( Mutex::new( config.clone() ) );
    let timeout_notify = Arc::new( Notify::new() );

    // Spawn optimized stream management task
    let state_clone = state.clone();
    let metrics_clone = metrics.clone();
    let config_clone = config_arc.clone();
    let timeout_notify_clone = timeout_notify.clone();

    tokio ::spawn( async move {
      Self::manage_stream_optimized(
        stream,
        control_rx,
        data_tx,
        state_clone,
        metrics_clone,
        config_clone,
        timeout_notify_clone
      ).await;
    });

    Self {
      control_tx,
      data_rx,
      state,
      config : config_arc,
      metrics,
      timeout_notify,
    }
  }

  /// Optimized stream management task with better performance and event-driven timeouts
  async fn manage_stream_optimized(
    mut stream : std::pin::Pin< Box< dyn futures::Stream< Item = Result< T, crate::error::Error > > + Send > >,
    mut control_rx : mpsc::UnboundedReceiver< StreamCommand >,
    data_tx : mpsc::UnboundedSender< Result< T, crate::error::Error > >,
    state : Arc< AtomicU8 >,
    metrics : Arc< StreamMetrics >,
    config : Arc< Mutex< StreamControlConfig > >,
    timeout_notify : Arc< Notify >,
  )
  {
    let mut is_paused = false;
    let current_config = config.lock().unwrap().clone();
    let mut buffer = StreamBuffer::< T >::new( &current_config.buffer_strategy, None );
    let mut pause_start : Option< Instant > = None;

    // Event-driven timeout task
    let _timeout_task = if current_config.event_driven_timeouts
    {
      Some( Self::spawn_timeout_monitor( state.clone(), timeout_notify.clone(), current_config.pause_timeout ) )
    } else {
      None
    };

    loop
    {
      tokio ::select! {
        // Handle control commands with response tracking
        command = control_rx.recv() => {
          match command
          {
            Some( StreamCommand::Pause { response_tx, start_time } ) => {
              let current_state = StreamState::from_u8( state.load( Ordering::Acquire ) );
              let result = if !is_paused && current_state == StreamState::Running
              {
                is_paused = true;
                pause_start = Some( Instant::now() );
                state.store( StreamState::Paused.to_u8(), Ordering::Release );

                // Update metrics atomically
                if current_config.metrics_level != MetricsLevel::None
                {
                  metrics.pause_count.fetch_add( 1, Ordering::Relaxed );
                  metrics.state_changes.fetch_add( 1, Ordering::Relaxed );

                  if current_config.metrics_level == MetricsLevel::Detailed
                  {
                    let response_time = start_time.elapsed().as_micros() as u64;
                    Self::update_avg_response_time( &metrics, response_time );
                  }
                }
                Ok( () )
              } else {
                Err( crate::error::Error::ApiError(
                  format!( "Cannot pause stream in state : {:?}", current_state )
                ) )
              };
              let _ = response_tx.send( result );
            },

            Some( StreamCommand::Resume { response_tx, start_time } ) => {
              let current_state = StreamState::from_u8( state.load( Ordering::Acquire ) );
              let result = if is_paused && current_state == StreamState::Paused
              {
                is_paused = false;
                pause_start = None;
                state.store( StreamState::Running.to_u8(), Ordering::Release );

                // Flush buffer efficiently
                let buffered_items = buffer.drain_all();
                let mut should_exit = false;
                for item in buffered_items
                {
                  if data_tx.send( item ).is_err()
                  {
                    should_exit = true;
                    break; // Receiver dropped
                  }
                }
                if should_exit
                {
                  break;
                }

                // Update metrics atomically
                if current_config.metrics_level != MetricsLevel::None
                {
                  metrics.resume_count.fetch_add( 1, Ordering::Relaxed );
                  metrics.state_changes.fetch_add( 1, Ordering::Relaxed );
                  metrics.buffer_size.store( 0, Ordering::Relaxed );

                  if current_config.metrics_level == MetricsLevel::Detailed
                  {
                    let response_time = start_time.elapsed().as_micros() as u64;
                    Self::update_avg_response_time( &metrics, response_time );
                  }
                }
                Ok( () )
              } else {
                Err( crate::error::Error::ApiError(
                  format!( "Cannot resume stream in state : {:?}", current_state )
                ) )
              };
              let _ = response_tx.send( result );
            },

            Some( StreamCommand::Cancel { response_tx, start_time } ) => {
              state.store( StreamState::Cancelled.to_u8(), Ordering::Release );

              if current_config.metrics_level != MetricsLevel::None
              {
                metrics.state_changes.fetch_add( 1, Ordering::Relaxed );

                if current_config.metrics_level == MetricsLevel::Detailed
                {
                  let response_time = start_time.elapsed().as_micros() as u64;
                  Self::update_avg_response_time( &metrics, response_time );
                }
              }

              let _ = response_tx.send( Ok( () ) );
              break; // Exit task
            },

            Some( StreamCommand::GetState( tx ) ) => {
              let current_state = StreamState::from_u8( state.load( Ordering::Relaxed ) );
              let _ = tx.send( current_state );
            },

            Some( StreamCommand::GetMetrics( tx ) ) => {
              let _ = tx.send( metrics.snapshot() );
            },

            Some( StreamCommand::UpdateConfig { new_config, response_tx } ) => {
              if let Ok( mut config_guard ) = config.try_lock()
              {
                *config_guard = new_config.clone();
                let _ = response_tx.send( Ok( () ) );
              } else {
                let _ = response_tx.send( Err( crate::error::Error::ApiError(
                  "Unable to update config : config is locked".to_string()
                ) ) );
              }
            },

            None => break, // Control channel closed
          }
        },

        // Handle stream data
        item = stream.next() => {
          if !is_paused
          {
            match item
            {
              Some( Ok( data ) ) => {
                if current_config.metrics_level != MetricsLevel::None
                {
                  metrics.total_chunks.fetch_add( 1, Ordering::Relaxed );
                }

                if data_tx.send( Ok( data ) ).is_err()
                {
                  break; // Receiver dropped
                }
              },
              Some( Err( error ) ) => {
                state.store( StreamState::Error.to_u8(), Ordering::Release );
                if current_config.metrics_level != MetricsLevel::None
                {
                  metrics.state_changes.fetch_add( 1, Ordering::Relaxed );
                }
                let _ = data_tx.send( Err( error ) );
                break;
              },
              None => {
                state.store( StreamState::Completed.to_u8(), Ordering::Release );
                if current_config.metrics_level != MetricsLevel::None
                {
                  metrics.state_changes.fetch_add( 1, Ordering::Relaxed );
                }
                break; // Stream ended
              },
            }
          } else {
            // Handle stream data while paused (buffer it efficiently)
            match item
            {
              Some( Ok( data ) ) => {
                if buffer.len() < current_config.max_buffered_chunks
                {
                  buffer.push( Ok( data ) );
                  let new_size = buffer.len();

                  if current_config.metrics_level != MetricsLevel::None
                  {
                    metrics.buffer_size.store( new_size, Ordering::Relaxed );

                    // Update peak buffer size
                    let current_peak = metrics.peak_buffer_size.load( Ordering::Relaxed );
                    if new_size > current_peak
                    {
                      metrics.peak_buffer_size.store( new_size, Ordering::Relaxed );
                    }
                  }
                } else {
                  // Buffer overflow - cancel stream
                  state.store( StreamState::Cancelled.to_u8(), Ordering::Release );
                  if current_config.metrics_level != MetricsLevel::None
                  {
                    metrics.state_changes.fetch_add( 1, Ordering::Relaxed );
                    metrics.buffer_overflows.fetch_add( 1, Ordering::Relaxed );
                  }
                  break;
                }
              },
              Some( Err( error ) ) => {
                state.store( StreamState::Error.to_u8(), Ordering::Release );
                if current_config.metrics_level != MetricsLevel::None
                {
                  metrics.state_changes.fetch_add( 1, Ordering::Relaxed );
                }
                buffer.push( Err( error ) );
                break;
              },
              None => {
                state.store( StreamState::Completed.to_u8(), Ordering::Release );
                if current_config.metrics_level != MetricsLevel::None
                {
                  metrics.state_changes.fetch_add( 1, Ordering::Relaxed );
                }
                break;
              },
            }
          }
        },

        // Event-driven timeout handling (only if not using event-driven timeouts)
        _ = tokio::time::sleep( Duration::from_millis( 500 ) ), if is_paused && !current_config.event_driven_timeouts =>
        {
          if let Some( start ) = pause_start
          {
            if start.elapsed() > current_config.pause_timeout
            {
              state.store( StreamState::TimedOut.to_u8(), Ordering::Release );
              if current_config.metrics_level != MetricsLevel::None
              {
                metrics.state_changes.fetch_add( 1, Ordering::Relaxed );
              }
              break;
            }
          }
        },

        // Handle timeout notifications from event-driven timeout monitor
        _ = timeout_notify.notified(), if current_config.event_driven_timeouts =>
        {
          let current_state = StreamState::from_u8( state.load( Ordering::Relaxed ) );
          if current_state == StreamState::TimedOut
          {
            break; // Timeout was triggered
          }
        },
      }
    }
    // Timeout task is automatically cleaned up when dropped
  }

  /// Spawn a timeout monitoring task for event-driven timeout handling
  fn spawn_timeout_monitor(
    state : Arc< AtomicU8 >,
    timeout_notify : Arc< Notify >,
    timeout_duration : Duration
  ) -> tokio::task::JoinHandle< () >
  {
    tokio ::spawn( async move {
      let mut pause_start : Option< Instant > = None;

      loop
      {
        tokio ::time::sleep( Duration::from_millis( 100 ) ).await;

        let current_state = StreamState::from_u8( state.load( Ordering::Relaxed ) );

        match current_state
        {
          StreamState::Paused => {
            if pause_start.is_none()
            {
              pause_start = Some( Instant::now() );
            } else if let Some( start ) = pause_start
            {
              if start.elapsed() > timeout_duration
              {
                // Set timeout state and notify
                state.store( StreamState::TimedOut.to_u8(), Ordering::Release );
                timeout_notify.notify_one();
                break;
              }
            }
          },
          StreamState::Running => {
            pause_start = None; // Reset timeout tracking
          },
          StreamState::Cancelled | StreamState::Completed | StreamState::Error | StreamState::TimedOut => {
            break; // Stream is done, exit timeout monitor
          },
        }
      }
    })
  }

  /// Update average response time using a running average
  fn update_avg_response_time( metrics : &StreamMetrics, new_response_time : u64 )
  {
    metrics.control_operations.fetch_add( 1, Ordering::Relaxed );

    // Use a simple exponential moving average for response time
    let current_avg = metrics.avg_control_response_time_us.load( Ordering::Relaxed );
    let operations = metrics.control_operations.load( Ordering::Relaxed );

    let new_avg = if operations == 1
    {
      new_response_time
    } else {
      // Weighted average : 90% old avg + 10% new sample
      ( current_avg * 9 + new_response_time ) / 10
    };

    metrics.avg_control_response_time_us.store( new_avg, Ordering::Relaxed );
  }

  /// Pause the stream with optimized response handling
  pub async fn pause( &mut self ) -> Result< (), crate::error::Error >
  {
    let ( response_tx, response_rx ) = oneshot::channel();
    let start_time = Instant::now();

    self.control_tx.send( StreamCommand::Pause { response_tx, start_time } )
      .map_err( |_| crate::error::Error::ApiError( "Stream control channel closed".to_string() ) )?;

    // Wait for response with timeout
    let config_timeout = {
      let config_guard = self.config.lock().unwrap();
      config_guard.control_operation_timeout
    };

    tokio ::time::timeout( config_timeout, response_rx )
      .await
      .map_err( |_| crate::error::Error::ApiError( "Pause operation timed out".to_string() ) )?
      .map_err( |_| crate::error::Error::ApiError( "Pause operation channel closed".to_string() ) )?
  }

  /// Resume the stream with optimized response handling
  pub async fn resume( &mut self ) -> Result< (), crate::error::Error >
  {
    let ( response_tx, response_rx ) = oneshot::channel();
    let start_time = Instant::now();

    self.control_tx.send( StreamCommand::Resume { response_tx, start_time } )
      .map_err( |_| crate::error::Error::ApiError( "Stream control channel closed".to_string() ) )?;

    // Wait for response with timeout
    let config_timeout = {
      let config_guard = self.config.lock().unwrap();
      config_guard.control_operation_timeout
    };

    tokio ::time::timeout( config_timeout, response_rx )
      .await
      .map_err( |_| crate::error::Error::ApiError( "Resume operation timed out".to_string() ) )?
      .map_err( |_| crate::error::Error::ApiError( "Resume operation channel closed".to_string() ) )?
  }

  /// Cancel the stream with optimized response handling
  pub async fn cancel( &mut self ) -> Result< (), crate::error::Error >
  {
    let ( response_tx, response_rx ) = oneshot::channel();
    let start_time = Instant::now();

    self.control_tx.send( StreamCommand::Cancel { response_tx, start_time } )
      .map_err( |_| crate::error::Error::ApiError( "Stream control channel closed".to_string() ) )?;

    // Wait for response with timeout
    let config_timeout = {
      let config_guard = self.config.lock().unwrap();
      config_guard.control_operation_timeout
    };

    tokio ::time::timeout( config_timeout, response_rx )
      .await
      .map_err( |_| crate::error::Error::ApiError( "Cancel operation timed out".to_string() ) )?
      .map_err( |_| crate::error::Error::ApiError( "Cancel operation channel closed".to_string() ) )?
  }

  /// Get the current stream state (lock-free atomic operation)
  pub fn state( &self ) -> StreamState
  {
    StreamState::from_u8( self.state.load( Ordering::Relaxed ) )
  }

  /// Check if the stream is paused (lock-free operation)
  pub fn is_paused( &self ) -> bool
  {
    self.state() == StreamState::Paused
  }

  /// Check if the stream is cancelled (lock-free operation)
  pub fn is_cancelled( &self ) -> bool
  {
    matches!( self.state(), StreamState::Cancelled | StreamState::TimedOut )
  }

  /// Get stream metrics snapshot (lock-free atomic operations)
  pub fn get_metrics( &self ) -> StreamMetricsSnapshot
  {
    self.metrics.snapshot()
  }

  /// Update stream configuration at runtime
  pub async fn update_config( &mut self, new_config : StreamControlConfig ) -> Result< (), crate::error::Error >
  {
    let ( response_tx, response_rx ) = oneshot::channel();

    self.control_tx.send( StreamCommand::UpdateConfig { new_config, response_tx } )
      .map_err( |_| crate::error::Error::ApiError( "Stream control channel closed".to_string() ) )?;

    let config_timeout = {
      let config_guard = self.config.lock().unwrap();
      config_guard.control_operation_timeout
    };

    tokio ::time::timeout( config_timeout, response_rx )
      .await
      .map_err( |_| crate::error::Error::ApiError( "Config update operation timed out".to_string() ) )?
      .map_err( |_| crate::error::Error::ApiError( "Config update operation channel closed".to_string() ) )?
  }

  /// Get the next item from the stream
  pub async fn next( &mut self ) -> Option< Result< T, crate::error::Error > >
  {
    self.data_rx.recv().await
  }
}

/// Builder for creating controllable streams from model API
#[ derive( Debug ) ]
pub struct ControllableStreamBuilder< 'a >
{
  #[ allow( dead_code ) ] // Used in build method but compiler doesnt detect it
  model : &'a crate::models::api::ModelApi< 'a >,
  request : crate::models::GenerateContentRequest,
  config : StreamControlConfig,
}

impl< 'a > ControllableStreamBuilder< 'a >
{
  /// Create a new controllable stream builder
  pub fn new( model : &'a crate::models::api::ModelApi< 'a > ) -> Self
  {
    Self {
      model,
      request : crate::models::GenerateContentRequest::default(),
      config : StreamControlConfig::default(),
    }
  }

  /// Add text content to the request
  pub fn text( mut self, text : &str ) -> Self
  {
    self.request.contents.push( crate::models::Content {
      parts : vec![ crate::models::Part {
        text : Some( text.to_string() ),
        ..Default::default()
      } ],
      role : "user".to_string(),
    } );
    self
  }

  /// Set buffer size for the controllable stream
  pub fn buffer_size( mut self, size : usize ) -> Self
  {
    self.config.buffer_size = size;
    self
  }

  /// Set pause timeout for the controllable stream
  pub fn pause_timeout( mut self, timeout : Duration ) -> Self
  {
    self.config.pause_timeout = timeout;
    self
  }

  /// Create the controllable stream
  #[ cfg( feature = "streaming" ) ]
  pub async fn create( self ) -> Result< ControllableStream< crate::models::StreamingResponse >, crate::error::Error >
  {
    // Get the underlying stream from the model API
    let stream = self.model.generate_content_stream( &self.request ).await?;

    // Box and pin the stream to make it Unpin
    let boxed_stream = Box::pin( stream );

    // Wrap it in a controllable stream
    Ok( ControllableStream::new( boxed_stream, self.config ) )
  }
}

/// Streaming control API for unified stream lifecycle management
#[ derive( Debug ) ]
pub struct StreamingControlApi< 'a >
{
  /// Reference to the Gemini client
  client : &'a crate::client::Client,
  /// Default configuration for new controllable streams
  config : StreamControlConfig,
}

impl< 'a > StreamingControlApi< 'a >
{
  /// Create a new streaming control API instance
  pub fn new( client : &'a crate::client::Client ) -> Self
  {
    Self {
      client,
      config : StreamControlConfig::default(),
    }
  }

  /// Create a new streaming control API with custom configuration
  pub fn with_config( client : &'a crate::client::Client, config : StreamControlConfig ) -> Self
  {
    Self {
      client,
      config,
    }
  }

  /// Create a configuration builder for streaming control
  pub fn config_builder() -> super::StreamControlConfigBuilder
  {
    super ::StreamControlConfigBuilder::new()
  }

  /// Create a controllable stream from a content generation request (SSE)
  #[ cfg( feature = "streaming" ) ]
  pub async fn create_stream_from_request(
    &self,
    _request : &crate::models::GenerateContentRequest
  ) -> Result< ControllableStream< crate::models::StreamingResponse >, crate::error::Error >
  {
    // qqq : Implement streaming functionality once API structure is clarified (task/verified/004)
    Err( crate::error::Error::ApiError( "Streaming functionality not yet implemented".to_string() ) )
  }

  /// Create a controllable stream builder for fluent API
  pub fn stream_builder( &self ) -> StreamControlStreamBuilder< '_ >
  {
    StreamControlStreamBuilder::new( self.client, "gemini-pro", self.config.clone() )
  }

  /// Create a controllable stream builder for specific model
  pub fn stream_builder_for_model( &self, model_name : &str ) -> StreamControlStreamBuilder< '_ >
  {
    StreamControlStreamBuilder::new( self.client, model_name, self.config.clone() )
  }

  /// Convert an existing stream to a controllable stream
  pub fn make_controllable< T >(
    &self,
    stream : std::pin::Pin< Box< dyn futures::Stream< Item = Result< T, crate::error::Error > > + Send > >
  ) -> ControllableStream< T >
  where
    T: Clone + Send + 'static,
  {
    ControllableStream::new( stream, self.config.clone() )
  }

  /// Create a controllable WebSocket stream
  #[ cfg( all( feature = "websocket_streaming", feature = "streaming_control" ) ) ]
  pub async fn create_websocket_stream(
    &self,
    endpoint : &str
  ) -> Result< ControllableWebSocketStream, crate::error::Error >
  {
    // Get WebSocket streaming API
    let ws_api = self.client.websocket_streaming();

    // Create WebSocket session
    let session_id = ws_api.create_stream( endpoint ).await?;

    // Get the session
    let session = ws_api.get_session( &session_id )
      .ok_or_else( || crate::error::Error::ApiError( "Failed to get WebSocket session".to_string() ) )?;

    // Create controllable WebSocket stream
    Ok( ControllableWebSocketStream::new( session, self.config.clone() ) )
  }

  /// Get the default configuration
  pub fn get_config( &self ) -> &StreamControlConfig
  {
    &self.config
  }

  /// Update the default configuration
  pub fn set_config( &mut self, config : StreamControlConfig )
  {
    self.config = config;
  }
}

/// Controllable WebSocket stream wrapper
#[ cfg( all( feature = "websocket_streaming", feature = "streaming_control" ) ) ]
#[ derive( Debug ) ]
pub struct ControllableWebSocketStream
{
  /// WebSocket session
  session : std::sync::Arc< crate::websocket::WebSocketStreamSession >,
  /// Stream configuration
  config : StreamControlConfig,
  /// Current state
  state : Arc< AtomicU8 >,
  /// Stream metrics
  metrics : Arc< StreamMetrics >,
}

#[ cfg( all( feature = "websocket_streaming", feature = "streaming_control" ) ) ]
impl ControllableWebSocketStream
{
  /// Create a new controllable WebSocket stream
  pub fn new(
    session : std::sync::Arc< crate::websocket::WebSocketStreamSession >,
    config : StreamControlConfig
  ) -> Self
  {
    Self {
      session,
      config,
      state : Arc::new( AtomicU8::new( StreamState::Running.to_u8() ) ),
      metrics : Arc::new( StreamMetrics::new() ),
    }
  }

  /// Pause the WebSocket stream
  pub async fn pause( &self ) -> Result< (), crate::error::Error >
  {
    self.state.store( StreamState::Paused.to_u8(), Ordering::Relaxed );
    self.session.set_state( crate::websocket::StreamSessionState::Paused );
    Ok( () )
  }

  /// Resume the WebSocket stream
  pub async fn resume( &self ) -> Result< (), crate::error::Error >
  {
    self.state.store( StreamState::Running.to_u8(), Ordering::Relaxed );
    self.session.set_state( crate::websocket::StreamSessionState::Active );
    Ok( () )
  }

  /// Cancel the WebSocket stream
  pub async fn cancel( &self ) -> Result< (), crate::error::Error >
  {
    self.state.store( StreamState::Cancelled.to_u8(), Ordering::Relaxed );
    self.session.close().await
  }

  /// Get the current state
  pub fn get_state( &self ) -> StreamState
  {
    StreamState::from_u8( self.state.load( Ordering::Relaxed ) )
  }

  /// Get stream metrics
  pub fn get_metrics( &self ) -> StreamMetricsSnapshot
  {
    self.metrics.snapshot()
  }

  /// Get stream configuration
  pub fn get_config( &self ) -> &StreamControlConfig
  {
    &self.config
  }

  /// Send a message through the WebSocket
  pub async fn send_message( &self, message : crate::websocket::WebSocketStreamMessage ) -> Result< (), crate::error::Error >
  {
    if self.get_state() != StreamState::Running
    {
      return Err( crate::error::Error::ApiError( "Stream is not in running state".to_string() ) );
    }

    self.session.send_message( message ).await?;
    self.metrics.items_sent.fetch_add( 1, Ordering::Relaxed );
    Ok( () )
  }

  /// Subscribe to incoming messages
  pub fn subscribe( &self ) -> tokio::sync::broadcast::Receiver< crate::websocket::WebSocketStreamMessage >
  {
    self.session.subscribe()
  }
}

/// Builder for creating controllable streams through the streaming control API
#[ derive( Debug ) ]
pub struct StreamControlStreamBuilder< 'a >
{
  /// Reference to the Gemini client
  #[ allow( dead_code ) ] // Used in full implementation, currently stubbed
  client : &'a crate::client::Client,
  /// Model name to use
  #[ allow( dead_code ) ] // Used in full implementation, currently stubbed
  model_name : String,
  /// Stream control configuration
  config : StreamControlConfig,
  /// Request to be built
  request : crate::models::GenerateContentRequest,
}

impl< 'a > StreamControlStreamBuilder< 'a >
{
  /// Create a new stream control stream builder
  pub fn new( client : &'a crate::client::Client, model_name : &str, config : StreamControlConfig ) -> Self
  {
    Self {
      client,
      model_name : model_name.to_string(),
      config,
      request : crate::models::GenerateContentRequest::default(),
    }
  }

  /// Add text content to the request
  pub fn text( mut self, text : &str ) -> Self
  {
    self.request.contents.push( crate::models::Content {
      parts : vec![ crate::models::Part {
        text : Some( text.to_string() ),
        ..Default::default()
      } ],
      role : "user".to_string(),
    } );
    self
  }

  /// Set buffer size for the controllable stream
  pub fn buffer_size( mut self, size : usize ) -> Self
  {
    self.config.buffer_size = size;
    self
  }

  /// Set pause timeout for the controllable stream
  pub fn pause_timeout( mut self, timeout : Duration ) -> Self
  {
    self.config.pause_timeout = timeout;
    self
  }

  /// Create the controllable stream
  #[ cfg( feature = "streaming" ) ]
  pub async fn create( self ) -> Result< ControllableStream< crate::models::StreamingResponse >, crate::error::Error >
  {
    // qqq : Implement streaming functionality once API structure is clarified (task/verified/004)
    Err( crate::error::Error::ApiError( "Streaming functionality not yet implemented".to_string() ) )
  }
}
