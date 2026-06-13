//! WebSocket Streaming Tests
//!
//! Comprehensive test suite for WebSocket streaming functionality including:
//! - WebSocket message types and operations
//! - Connection state management and transitions
//! - Configuration validation and defaults
//! - Connection pool management
//! - Event notification systems
//! - Message queuing and processing
//! - Statistics and monitoring

#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::useless_vec ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::must_use_candidate ) ]
#![ allow( clippy::missing_panics_doc ) ]
#![ allow( clippy::missing_errors_doc ) ]
#![ allow( clippy::similar_names ) ] // Acceptable in tests for related test variables
#![ allow( clippy::doc_markdown ) ]

#[ cfg( test ) ]
mod websocket_streaming_tests
{
  use api_openai::websocket_streaming::*;
  use std::time::Duration;
  use tokio::time;

  // ===== WEBSOCKET MESSAGE TESTS =====

  #[ tokio::test ]
  async fn test_websocket_message_creation()
  {
    let text_msg = WebSocketMessage::Text( "Hello, WebSocket!".to_string() );
    let binary_msg = WebSocketMessage::Binary( vec![ 1, 2, 3, 4 ] );
    let ping_msg = WebSocketMessage::Ping( vec![ 5, 6 ] );
    let pong_msg = WebSocketMessage::Pong( vec![ 7, 8 ] );
    let close_msg = WebSocketMessage::Close( Some( "Normal closure".to_string() ) );

    assert_eq!( text_msg.as_text(), Some( "Hello, WebSocket!" ) );
    assert_eq!( binary_msg.as_binary(), Some( [ 1, 2, 3, 4 ].as_slice() ) );
    assert!( !text_msg.is_control() );
    assert!( ping_msg.is_control() );
    assert!( pong_msg.is_control() );
    assert!( close_msg.is_control() );
  }

  #[ tokio::test ]
  async fn test_websocket_message_size()
  {
    let text_msg = WebSocketMessage::Text( "Hello".to_string() );
    let binary_msg = WebSocketMessage::Binary( vec![ 1, 2, 3 ] );
    let close_msg = WebSocketMessage::Close( Some( "Bye".to_string() ) );
    let close_empty = WebSocketMessage::Close( None );

    assert_eq!( text_msg.size(), 5 );
    assert_eq!( binary_msg.size(), 3 );
    assert_eq!( close_msg.size(), 3 );
    assert_eq!( close_empty.size(), 0 );
  }

  #[ tokio::test ]
  async fn test_websocket_message_serialization()
  {
    let messages = vec![
      WebSocketMessage::Text( "test".to_string() ),
      WebSocketMessage::Binary( vec![ 1, 2, 3 ] ),
      WebSocketMessage::Ping( vec![ 4, 5 ] ),
      WebSocketMessage::Pong( vec![ 6, 7 ] ),
      WebSocketMessage::Close( Some( "reason".to_string() ) ),
      WebSocketMessage::Close( None ),
    ];

    for message in messages
    {
      // Test serialization
      let serialized = serde_json::to_string( &message ).expect( "Failed to serialize message" );
      assert!( !serialized.is_empty() );

      // Test deserialization
      let deserialized : WebSocketMessage = serde_json::from_str( &serialized )
        .expect( "Failed to deserialize message" );

      assert_eq!( message, deserialized );
    }
  }

  // ===== WEBSOCKET STATE TESTS =====

  #[ tokio::test ]
  async fn test_websocket_state_transitions()
  {
    let states = vec![
      WebSocketState::Connecting,
      WebSocketState::Connected,
      WebSocketState::Closing,
      WebSocketState::Closed,
      WebSocketState::Failed( "Connection error".to_string() ),
    ];

    for state in &states
    {
      // Test serialization
      let serialized = serde_json::to_string( state ).expect( "Failed to serialize state" );
      assert!( !serialized.is_empty() );

      // Test deserialization
      let deserialized : WebSocketState = serde_json::from_str( &serialized )
        .expect( "Failed to deserialize state" );

      assert_eq!( *state, deserialized );
    }
  }

  #[ tokio::test ]
  async fn test_websocket_state_equality()
  {
    assert_eq!( WebSocketState::Connecting, WebSocketState::Connecting );
    assert_eq!( WebSocketState::Connected, WebSocketState::Connected );
    assert_ne!( WebSocketState::Connecting, WebSocketState::Connected );

    let failed1 = WebSocketState::Failed( "Error 1".to_string() );
    let failed2 = WebSocketState::Failed( "Error 1".to_string() );
    let failed3 = WebSocketState::Failed( "Error 2".to_string() );

    assert_eq!( failed1, failed2 );
    assert_ne!( failed1, failed3 );
  }

  // ===== WEBSOCKET CONFIG TESTS =====

  #[ tokio::test ]
  async fn test_websocket_config_defaults()
  {
    let config = WebSocketConfig::default();

    assert_eq!( config.connect_timeout_ms, 30000 );
    assert_eq!( config.max_message_size, 16 * 1024 * 1024 );
    assert_eq!( config.heartbeat_interval_ms, 30000 );
    assert_eq!( config.max_queue_size, 1000 );
    assert!( config.enable_compression );
    assert_eq!( config.max_reconnect_attempts, 3 );
    assert_eq!( config.reconnect_delay_ms, 1000 );
  }

  #[ tokio::test ]
  async fn test_websocket_config_validation()
  {
    // Valid config
    let valid_config = WebSocketConfig::default();
    assert!( WebSocketStreamer::validate_config( &valid_config ).is_ok() );

    // Invalid : zero timeout
    let invalid_config1 = WebSocketConfig
    {
      connect_timeout_ms : 0,
      ..Default::default()
    };
    assert!( WebSocketStreamer::validate_config( &invalid_config1 ).is_err() );

    // Invalid : zero message size
    let invalid_config2 = WebSocketConfig
    {
      max_message_size : 0,
      ..Default::default()
    };
    assert!( WebSocketStreamer::validate_config( &invalid_config2 ).is_err() );

    // Invalid : zero heartbeat interval
    let invalid_config3 = WebSocketConfig
    {
      heartbeat_interval_ms : 0,
      ..Default::default()
    };
    assert!( WebSocketStreamer::validate_config( &invalid_config3 ).is_err() );

    // Invalid : zero queue size
    let invalid_config4 = WebSocketConfig
    {
      max_queue_size : 0,
      ..Default::default()
    };
    assert!( WebSocketStreamer::validate_config( &invalid_config4 ).is_err() );
  }

  #[ tokio::test ]
  async fn test_websocket_config_serialization()
  {
    let config = WebSocketConfig
    {
      connect_timeout_ms : 10000,
      max_message_size : 1024,
      heartbeat_interval_ms : 15000,
      max_queue_size : 500,
      enable_compression : false,
      max_reconnect_attempts : 5,
      reconnect_delay_ms : 2000,
    };

    // Test serialization
    let serialized = serde_json::to_string( &config ).expect( "Failed to serialize config" );
    assert!( !serialized.is_empty() );

    // Test deserialization
    let deserialized : WebSocketConfig = serde_json::from_str( &serialized )
      .expect( "Failed to deserialize config" );

    assert_eq!( config.connect_timeout_ms, deserialized.connect_timeout_ms );
    assert_eq!( config.max_message_size, deserialized.max_message_size );
    assert_eq!( config.enable_compression, deserialized.enable_compression );
  }

  // ===== WEBSOCKET CONNECTION TESTS =====

  #[ tokio::test ]
  async fn test_websocket_connection_creation()
  {
    let config = WebSocketConfig::default();
    let connection = WebSocketConnection::new(
      "test_conn_1".to_string(),
      "wss://api.example.com/ws".to_string(),
      config,
    );

    assert_eq!( connection.id, "test_conn_1" );
    assert_eq!( connection.url, "wss://api.example.com/ws" );
    assert_eq!( connection.state, WebSocketState::Connecting );
    assert!( connection.connected_at.is_none() );
    assert!( !connection.is_active() );
    assert_eq!( connection.queue_size(), 0 );
  }

  #[ tokio::test ]
  async fn test_websocket_connection_state_updates()
  {
    let config = WebSocketConfig::default();
    let mut connection = WebSocketConnection::new(
      "test_conn_1".to_string(),
      "wss://api.example.com/ws".to_string(),
      config,
    );

    // Initial state
    assert_eq!( connection.state, WebSocketState::Connecting );
    assert!( !connection.is_active() );

    // Update to connected
    connection.update_state( WebSocketState::Connected );
    assert_eq!( connection.state, WebSocketState::Connected );
    assert!( connection.is_active() );
    assert!( connection.connected_at.is_some() );

    time ::sleep( Duration::from_millis( 10 ) ).await;

    // Check connection duration
    let duration = connection.connection_duration();
    assert!( duration.is_some() );
    assert!( duration.unwrap() >= Duration::from_millis( 10 ) );

    // Update to closing
    connection.update_state( WebSocketState::Closing );
    assert_eq!( connection.state, WebSocketState::Closing );
    assert!( !connection.is_active() );
  }

  #[ tokio::test ]
  async fn test_websocket_connection_idle_duration()
  {
    let config = WebSocketConfig::default();
    let connection = WebSocketConnection::new(
      "test_conn_1".to_string(),
      "wss://api.example.com/ws".to_string(),
      config,
    );

    time ::sleep( Duration::from_millis( 10 ) ).await;

    let idle_duration = connection.idle_duration();
    assert!( idle_duration >= Duration::from_millis( 10 ) );
    assert!( idle_duration < Duration::from_millis( 100 ) );
  }

  #[ tokio::test ]
  async fn test_websocket_connection_message_queue()
  {
    let config = WebSocketConfig
    {
      max_queue_size : 3,
      max_message_size : 100,
      ..Default::default()
    };

    let connection = WebSocketConnection::new(
      "test_conn_1".to_string(),
      "wss://api.example.com/ws".to_string(),
      config,
    );

    // Queue messages
    assert!( connection.queue_message( WebSocketMessage::Text( "Message 1".to_string() ) ).is_ok() );
    assert!( connection.queue_message( WebSocketMessage::Text( "Message 2".to_string() ) ).is_ok() );
    assert!( connection.queue_message( WebSocketMessage::Text( "Message 3".to_string() ) ).is_ok() );

    assert_eq!( connection.queue_size(), 3 );

    // Queue should be full
    assert!( connection.queue_message( WebSocketMessage::Text( "Message 4".to_string() ) ).is_err() );

    // Dequeue messages
    let msg1 = connection.dequeue_message();
    assert!( msg1.is_some() );
    assert_eq!( msg1.unwrap().as_text(), Some( "Message 1" ) );

    let msg2 = connection.dequeue_message();
    assert!( msg2.is_some() );
    assert_eq!( msg2.unwrap().as_text(), Some( "Message 2" ) );

    assert_eq!( connection.queue_size(), 1 );

    // Clear queue
    connection.clear_queue();
    assert_eq!( connection.queue_size(), 0 );
  }

  #[ tokio::test ]
  async fn test_websocket_connection_message_size_limit()
  {
    let config = WebSocketConfig
    {
      max_message_size : 10,
      ..Default::default()
    };

    let connection = WebSocketConnection::new(
      "test_conn_1".to_string(),
      "wss://api.example.com/ws".to_string(),
      config,
    );

    // Message within limit
    assert!( connection.queue_message( WebSocketMessage::Text( "Short".to_string() ) ).is_ok() );

    // Message exceeds limit
    assert!( connection.queue_message( WebSocketMessage::Text( "This message is too long".to_string() ) ).is_err() );
  }

  // ===== WEBSOCKET POOL TESTS =====

  #[ tokio::test ]
  async fn test_websocket_pool_creation()
  {
    let pool_config = WebSocketPoolConfig::default();
    let pool = WebSocketPool::new( pool_config );

    assert_eq!( pool.active_connection_count(), 0 );
    assert!( pool.connection_ids().is_empty() );
  }

  #[ tokio::test ]
  async fn test_websocket_pool_config_defaults()
  {
    let config = WebSocketPoolConfig::default();

    assert_eq!( config.max_connections, 100 );
    assert_eq!( config.idle_timeout_ms, 300000 );
    assert_eq!( config.cleanup_interval_ms, 60000 );
  }

  #[ tokio::test ]
  async fn test_websocket_pool_add_remove_connections()
  {
    let pool_config = WebSocketPoolConfig::default();
    let mut pool = WebSocketPool::new( pool_config );

    let ws_config = WebSocketConfig::default();
    let connection1 = WebSocketConnection::new(
      "conn_1".to_string(),
      "wss://api.example.com/ws1".to_string(),
      ws_config.clone(),
    );

    let connection2 = WebSocketConnection::new(
      "conn_2".to_string(),
      "wss://api.example.com/ws2".to_string(),
      ws_config,
    );

    // Add connections
    assert!( pool.add_connection( connection1 ).is_ok() );
    assert!( pool.add_connection( connection2 ).is_ok() );

    assert_eq!( pool.connection_ids().len(), 2 );
    assert!( pool.connection_ids().contains( &"conn_1".to_string() ) );
    assert!( pool.connection_ids().contains( &"conn_2".to_string() ) );

    // Get connections
    assert!( pool.get_connection( "conn_1" ).is_some() );
    assert!( pool.get_connection( "conn_2" ).is_some() );
    assert!( pool.get_connection( "conn_3" ).is_none() );

    // Remove connection
    let removed = pool.remove_connection( "conn_1" );
    assert!( removed.is_some() );
    assert_eq!( removed.unwrap().id, "conn_1" );

    assert_eq!( pool.connection_ids().len(), 1 );
    assert!( pool.get_connection( "conn_1" ).is_none() );
  }

  #[ tokio::test ]
  async fn test_websocket_pool_max_connections()
  {
    let pool_config = WebSocketPoolConfig
    {
      max_connections : 2,
      ..Default::default()
    };

    let mut pool = WebSocketPool::new( pool_config );
    let ws_config = WebSocketConfig::default();

    let connection1 = WebSocketConnection::new(
      "conn_1".to_string(),
      "wss://api.example.com/ws1".to_string(),
      ws_config.clone(),
    );

    let connection2 = WebSocketConnection::new(
      "conn_2".to_string(),
      "wss://api.example.com/ws2".to_string(),
      ws_config.clone(),
    );

    let connection3 = WebSocketConnection::new(
      "conn_3".to_string(),
      "wss://api.example.com/ws3".to_string(),
      ws_config,
    );

    // Add up to max connections
    assert!( pool.add_connection( connection1 ).is_ok() );
    assert!( pool.add_connection( connection2 ).is_ok() );

    // Should fail to add beyond limit
    assert!( pool.add_connection( connection3 ).is_err() );
  }

  #[ tokio::test ]
  async fn test_websocket_pool_active_count()
  {
    let pool_config = WebSocketPoolConfig::default();
    let mut pool = WebSocketPool::new( pool_config );
    let ws_config = WebSocketConfig::default();

    let mut connection1 = WebSocketConnection::new(
      "conn_1".to_string(),
      "wss://api.example.com/ws1".to_string(),
      ws_config.clone(),
    );

    let connection2 = WebSocketConnection::new(
      "conn_2".to_string(),
      "wss://api.example.com/ws2".to_string(),
      ws_config,
    );

    // Set one connection as active
    connection1.update_state( WebSocketState::Connected );

    pool.add_connection( connection1 ).unwrap();
    pool.add_connection( connection2 ).unwrap();

    assert_eq!( pool.active_connection_count(), 1 );
  }

  #[ tokio::test ]
  async fn test_websocket_pool_cleanup_idle()
  {
    let pool_config = WebSocketPoolConfig
    {
      idle_timeout_ms : 50, // Very short timeout for testing
      ..Default::default()
    };

    let mut pool = WebSocketPool::new( pool_config );
    let ws_config = WebSocketConfig::default();

    let connection1 = WebSocketConnection::new(
      "conn_1".to_string(),
      "wss://api.example.com/ws1".to_string(),
      ws_config.clone(),
    );

    let connection2 = WebSocketConnection::new(
      "conn_2".to_string(),
      "wss://api.example.com/ws2".to_string(),
      ws_config,
    );

    pool.add_connection( connection1 ).unwrap();
    pool.add_connection( connection2 ).unwrap();

    assert_eq!( pool.connection_ids().len(), 2 );

    // Wait for connections to become idle
    time ::sleep( Duration::from_millis( 60 ) ).await;

    // Cleanup should remove idle connections
    let removed = pool.cleanup_idle_connections();
    assert_eq!( removed.len(), 2 );
    assert!( removed.contains( &"conn_1".to_string() ) );
    assert!( removed.contains( &"conn_2".to_string() ) );

    assert_eq!( pool.connection_ids().len(), 0 );
  }

  // ===== WEBSOCKET STREAMER UTILITY TESTS =====

  #[ tokio::test ]
  async fn test_websocket_streamer_event_notifications()
  {
    let ( sender, mut receiver ) = WebSocketStreamer::create_event_notifier();

    // Send connected event
    sender.send_connected( "conn_1".to_string() ).expect( "Failed to send event" );

    let event = receiver.try_recv();
    assert!( event.is_some() );

    match event.unwrap()
    {
      WebSocketEvent::Connected { connection_id } =>
      {
        assert_eq!( connection_id, "conn_1" );
      }
      _ => panic!( "Wrong event type received" ),
    }

    // Send error event
    sender.send_error( "conn_1".to_string(), "Connection failed".to_string() ).expect( "Failed to send error" );

    let error_event = receiver.try_recv();
    assert!( error_event.is_some() );

    match error_event.unwrap()
    {
      WebSocketEvent::Error { connection_id, error } =>
      {
        assert_eq!( connection_id, "conn_1" );
        assert_eq!( error, "Connection failed" );
      }
      _ => panic!( "Wrong event type received" ),
    }
  }

  #[ tokio::test ]
  async fn test_websocket_streamer_message_channel()
  {
    let ( sender, mut receiver ) = WebSocketStreamer::create_message_channel();

    // Send text message
    sender.send_text( "Hello, WebSocket!".to_string() ).expect( "Failed to send text" );

    let message = receiver.try_recv();
    assert!( message.is_some() );
    assert_eq!( message.unwrap().as_text(), Some( "Hello, WebSocket!" ) );

    // Send binary message
    sender.send_binary( vec![ 1, 2, 3, 4 ] ).expect( "Failed to send binary" );

    let binary_message = receiver.try_recv();
    assert!( binary_message.is_some() );
    assert_eq!( binary_message.unwrap().as_binary(), Some( [ 1, 2, 3, 4 ].as_slice() ) );

    // Send ping message
    sender.send_ping( vec![ 5, 6 ] ).expect( "Failed to send ping" );

    let ping_message = receiver.try_recv();
    assert!( ping_message.is_some() );
    assert!( ping_message.unwrap().is_control() );
  }

  #[ tokio::test ]
  async fn test_websocket_streamer_state_watcher()
  {
    let ( sender, mut receiver ) = WebSocketStreamer::create_state_watcher( WebSocketState::Connecting );

    assert_eq!( *receiver.borrow(), WebSocketState::Connecting );

    sender.send( WebSocketState::Connected ).expect( "Failed to send state update" );

    // Wait for change
    receiver.changed().await.expect( "Failed to receive state change" );
    assert_eq!( *receiver.borrow(), WebSocketState::Connected );
  }

  #[ tokio::test ]
  async fn test_websocket_streamer_reconnect_delay()
  {
    // Test exponential backoff
    assert_eq!(
      WebSocketStreamer::calculate_reconnect_delay( 0, 1000, 10000 ),
      Duration::from_secs( 1 )
    );

    assert_eq!(
      WebSocketStreamer::calculate_reconnect_delay( 1, 1000, 10000 ),
      Duration::from_secs( 2 )
    );

    assert_eq!(
      WebSocketStreamer::calculate_reconnect_delay( 2, 1000, 10000 ),
      Duration::from_secs( 4 )
    );

    assert_eq!(
      WebSocketStreamer::calculate_reconnect_delay( 3, 1000, 10000 ),
      Duration::from_secs( 8 )
    );

    // Should cap at max delay
    assert_eq!(
      WebSocketStreamer::calculate_reconnect_delay( 10, 1000, 5000 ),
      Duration::from_secs( 5 )
    );
  }

  #[ tokio::test ]
  async fn test_websocket_streamer_process_message_queue()
  {
    let config = WebSocketConfig::default();
    let connection = WebSocketConnection::new(
      "test_conn".to_string(),
      "wss://api.example.com/ws".to_string(),
      config,
    );

    // Queue several messages
    connection.queue_message( WebSocketMessage::Text( "Message 1".to_string() ) ).unwrap();
    connection.queue_message( WebSocketMessage::Text( "Message 2".to_string() ) ).unwrap();
    connection.queue_message( WebSocketMessage::Text( "Message 3".to_string() ) ).unwrap();

    // Process limited number of messages
    let messages = WebSocketStreamer::process_message_queue( &connection, 2 );
    assert_eq!( messages.len(), 2 );
    assert_eq!( messages[ 0 ].as_text(), Some( "Message 1" ) );
    assert_eq!( messages[ 1 ].as_text(), Some( "Message 2" ) );

    // One message should remain in queue
    assert_eq!( connection.queue_size(), 1 );

    // Process remaining messages
    let remaining = WebSocketStreamer::process_message_queue( &connection, 5 );
    assert_eq!( remaining.len(), 1 );
    assert_eq!( remaining[ 0 ].as_text(), Some( "Message 3" ) );

    // Queue should be empty
    assert_eq!( connection.queue_size(), 0 );
  }

  #[ tokio::test ]
  async fn test_websocket_streamer_connection_statistics()
  {
    let config = WebSocketConfig::default();
    let mut connection = WebSocketConnection::new(
      "test_conn".to_string(),
      "wss://api.example.com/ws".to_string(),
      config,
    );

    connection.update_state( WebSocketState::Connected );
    connection.queue_message( WebSocketMessage::Text( "Test".to_string() ) ).unwrap();

    let stats = WebSocketStreamer::connection_statistics( &connection );

    assert_eq!( stats.connection_id, "test_conn" );
    assert_eq!( stats.state, WebSocketState::Connected );
    assert!( stats.connected_duration.is_some() );
    assert_eq!( stats.queue_size, 1 );
  }

  // ===== INTEGRATION TESTS =====

  #[ tokio::test ]
  async fn test_websocket_event_async_recv()
  {
    let ( sender, mut receiver ) = WebSocketStreamer::create_event_notifier();

    // Send event after a delay
    tokio ::spawn( async move
    {
      time ::sleep( Duration::from_millis( 25 ) ).await;
      let _ = sender.send_connected( "async_conn".to_string() );
    });

    // Receive asynchronously
    let event = receiver.recv().await;
    assert!( event.is_some() );

    match event.unwrap()
    {
      WebSocketEvent::Connected { connection_id } =>
      {
        assert_eq!( connection_id, "async_conn" );
      }
      _ => panic!( "Wrong event type received" ),
    }
  }

  #[ tokio::test ]
  async fn test_websocket_message_async_recv()
  {
    let ( sender, mut receiver ) = WebSocketStreamer::create_message_channel();

    // Send message after a delay
    tokio ::spawn( async move
    {
      time ::sleep( Duration::from_millis( 25 ) ).await;
      let _ = sender.send_text( "Async message".to_string() );
    });

    // Receive asynchronously
    let message = receiver.recv().await;
    assert!( message.is_some() );
    assert_eq!( message.unwrap().as_text(), Some( "Async message" ) );
  }

  #[ tokio::test ]
  async fn test_complete_websocket_workflow()
  {
    let pool_config = WebSocketPoolConfig::default();
    let mut pool = WebSocketPool::new( pool_config );

    let ws_config = WebSocketConfig::default();
    let mut connection = WebSocketConnection::new(
      "workflow_conn".to_string(),
      "wss://api.example.com/ws".to_string(),
      ws_config,
    );

    let ( event_sender, mut event_receiver ) = WebSocketStreamer::create_event_notifier();
    let ( msg_sender, mut msg_receiver ) = WebSocketStreamer::create_message_channel();

    // Simulate connection workflow
    event_sender.send_connected( connection.id.clone() ).unwrap();
    connection.update_state( WebSocketState::Connected );

    // Queue and process messages
    connection.queue_message( WebSocketMessage::Text( "Hello".to_string() ) ).unwrap();
    msg_sender.send_text( "Response".to_string() ).unwrap();

    // Add to pool
    pool.add_connection( connection ).unwrap();

    // Verify events
    let event = event_receiver.try_recv();
    assert!( event.is_some() );

    let message = msg_receiver.try_recv();
    assert!( message.is_some() );
    assert_eq!( message.unwrap().as_text(), Some( "Response" ) );

    // Verify pool state
    assert_eq!( pool.active_connection_count(), 1 );
    assert!( pool.get_connection( "workflow_conn" ).is_some() );

    // Get statistics
    let conn = pool.get_connection( "workflow_conn" ).unwrap();
    let stats = WebSocketStreamer::connection_statistics( conn );
    assert_eq!( stats.connection_id, "workflow_conn" );
    assert_eq!( stats.state, WebSocketState::Connected );
  }

  #[ tokio::test ]
  async fn test_heartbeat_timer()
  {
    let mut heartbeat_receiver = WebSocketStreamer::create_heartbeat_timer( Duration::from_millis( 50 ) );

    // Should receive heartbeat signals
    let start = std::time::Instant::now();
    let mut count = 0;

    while count < 3 && start.elapsed() < Duration::from_millis( 200 )
    {
      if let Ok( _heartbeat ) = heartbeat_receiver.try_recv()
      {
        count += 1;
      }
      time ::sleep( Duration::from_millis( 10 ) ).await;
    }

    assert!( count >= 2, "Should have received at least 2 heartbeats" );
  }

  #[ tokio::test ]
  async fn test_performance_message_operations()
  {
    let config = WebSocketConfig
    {
      max_queue_size : 10000,
      ..Default::default()
    };

    let connection = WebSocketConnection::new(
      "perf_conn".to_string(),
      "wss://api.example.com/ws".to_string(),
      config,
    );

    let start = std::time::Instant::now();

    // Queue many messages quickly
    for i in 0..1000
    {
      let result = connection.queue_message( WebSocketMessage::Text( format!( "Message {}", i ) ) );
      assert!( result.is_ok() );
    }

    let queue_duration = start.elapsed();
    assert!( queue_duration < Duration::from_millis( 100 ) ); // Should be fast

    // Process messages quickly
    let process_start = std::time::Instant::now();
    let messages = WebSocketStreamer::process_message_queue( &connection, 1000 );
    let process_duration = process_start.elapsed();

    assert_eq!( messages.len(), 1000 );
    assert!( process_duration < Duration::from_millis( 50 ) ); // Should be very fast
  }

  #[ tokio::test ]
  async fn test_concurrent_pool_operations()
  {
    let pool_config = WebSocketPoolConfig::default();
    let pool = std::sync::Arc::new( std::sync::Mutex::new( WebSocketPool::new( pool_config ) ) );

    let mut handles = Vec::new();

    // Create multiple concurrent connection addition tasks
    for i in 0..10
    {
      let pool_clone = pool.clone();
      let handle = tokio::spawn( async move
      {
        let ws_config = WebSocketConfig::default();
        let connection = WebSocketConnection::new(
          format!( "concurrent_conn_{}", i ),
          format!( "wss://api.example.com/ws{}", i ),
          ws_config,
        );

        let mut pool = pool_clone.lock().unwrap();
        pool.add_connection( connection )
      });
      handles.push( handle );
    }

    // Wait for all tasks to complete
    for handle in handles
    {
      let result = handle.await.expect( "Task should complete successfully" );
      assert!( result.is_ok() );
    }

    // Verify all connections were added
    let pool = pool.lock().unwrap();
    assert_eq!( pool.connection_ids().len(), 10 );
  }
}