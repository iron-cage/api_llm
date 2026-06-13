//! Streaming Control Tests
//!
//! Unit tests for the streaming-control feature that provides pause/resume/cancel
//! capabilities for streaming responses.
//!
//! These are unit tests that test the control logic without requiring API calls.

#[ allow( unused_imports ) ]
use super::*;

#[ cfg( feature = "streaming-control" ) ]
mod streaming_control_tests
{
  use super::*;
  use the_module::{ StreamControl, StreamState };
  use core::time::Duration;
  use tokio::time::sleep;

  #[ test ]
  fn test_stream_state_creation()
  {
    // Test that StreamState enum has correct variants
    let running = StreamState::Running;
    let paused = StreamState::Paused;
    let cancelled = StreamState::Cancelled;

    assert_eq!( running, StreamState::Running );
    assert_eq!( paused, StreamState::Paused );
    assert_eq!( cancelled, StreamState::Cancelled );
  }

  #[ test ]
  fn test_stream_state_equality()
  {
    // Test that states can be compared
    assert_ne!( StreamState::Running, StreamState::Paused );
    assert_ne!( StreamState::Running, StreamState::Cancelled );
    assert_ne!( StreamState::Paused, StreamState::Cancelled );
  }

  #[ test ]
  fn test_stream_control_creation()
  {
    // Test creating a StreamControl handle
    let control = StreamControl::new( 100 );

    // Should start in running state
    assert!( control.is_running() );
    assert!( !control.is_paused() );
    assert!( !control.is_cancelled() );
    assert_eq!( control.get_state(), StreamState::Running );
    assert_eq!( control.buffer_size(), 0 );
  }

  #[ test ]
  fn test_stream_control_pause()
  {
    // Test pausing a stream
    let control = StreamControl::new( 100 );

    assert!( control.is_running() );

    let result = control.pause();
    assert!( result.is_ok(), "Pause should succeed" );

    assert!( control.is_paused() );
    assert!( !control.is_running() );
    assert!( !control.is_cancelled() );
    assert_eq!( control.get_state(), StreamState::Paused );
  }

  #[ test ]
  fn test_stream_control_resume()
  {
    // Test resuming a paused stream
    let control = StreamControl::new( 100 );

    control.pause().expect( "Pause should succeed" );
    assert!( control.is_paused() );

    let result = control.resume();
    assert!( result.is_ok(), "Resume should succeed" );

    assert!( control.is_running() );
    assert!( !control.is_paused() );
    assert!( !control.is_cancelled() );
    assert_eq!( control.get_state(), StreamState::Running );
  }

  #[ test ]
  fn test_stream_control_cancel()
  {
    // Test cancelling a stream
    let control = StreamControl::new( 100 );

    control.cancel();

    assert!( control.is_cancelled() );
    assert!( !control.is_running() );
    assert!( !control.is_paused() );
    assert_eq!( control.get_state(), StreamState::Cancelled );
    assert_eq!( control.buffer_size(), 0 );
  }

  #[ test ]
  fn test_stream_control_cancel_clears_buffer()
  {
    // Test that cancelling clears any buffered events
    let control = StreamControl::new( 100 );

    // Note : We can't directly test buffer_event since it's private,
    // but we can verify buffer_size is 0 after cancel
    control.cancel();

    assert_eq!( control.buffer_size(), 0, "Cancel should clear buffer" );
  }

  #[ test ]
  fn test_stream_control_pause_cancelled_stream()
  {
    // Test that pausing a cancelled stream returns error
    let control = StreamControl::new( 100 );

    control.cancel();
    assert!( control.is_cancelled() );

    let result = control.pause();
    assert!( result.is_err(), "Cannot pause cancelled stream" );
    assert_eq!( result.unwrap_err(), "Cannot pause cancelled stream" );
  }

  #[ test ]
  fn test_stream_control_resume_cancelled_stream()
  {
    // Test that resuming a cancelled stream returns error
    let control = StreamControl::new( 100 );

    control.cancel();
    assert!( control.is_cancelled() );

    let result = control.resume();
    assert!( result.is_err(), "Cannot resume cancelled stream" );
    assert_eq!( result.unwrap_err(), "Cannot resume cancelled stream" );
  }

  #[ test ]
  fn test_stream_control_pause_already_paused()
  {
    // Test that pausing an already paused stream succeeds (idempotent)
    let control = StreamControl::new( 100 );

    control.pause().expect( "First pause should succeed" );
    assert!( control.is_paused() );

    let result = control.pause();
    assert!( result.is_ok(), "Second pause should succeed (idempotent)" );
    assert!( control.is_paused() );
  }

  #[ test ]
  fn test_stream_control_resume_already_running()
  {
    // Test that resuming an already running stream succeeds (idempotent)
    let control = StreamControl::new( 100 );

    assert!( control.is_running() );

    let result = control.resume();
    assert!( result.is_ok(), "Resume on running stream should succeed (idempotent)" );
    assert!( control.is_running() );
  }

  #[ test ]
  fn test_stream_control_cancel_is_irreversible()
  {
    // Test that cancel is irreversible
    let control = StreamControl::new( 100 );

    control.cancel();
    assert!( control.is_cancelled() );

    // Try to resume - should fail
    let resume_result = control.resume();
    assert!( resume_result.is_err() );
    assert!( control.is_cancelled(), "Should still be cancelled" );

    // Try to pause - should fail
    let pause_result = control.pause();
    assert!( pause_result.is_err() );
    assert!( control.is_cancelled(), "Should still be cancelled" );
  }

  #[ test ]
  fn test_stream_control_clone()
  {
    // Test that StreamControl can be cloned and both handles work
    let control1 = StreamControl::new( 100 );
    let control2 = control1.clone();

    // Both should be in running state
    assert!( control1.is_running() );
    assert!( control2.is_running() );

    // Pause via control1
    control1.pause().expect( "Pause should succeed" );

    // Both should see paused state (shared Arc< Mutex<> >)
    assert!( control1.is_paused() );
    assert!( control2.is_paused() );

    // Resume via control2
    control2.resume().expect( "Resume should succeed" );

    // Both should see running state
    assert!( control1.is_running() );
    assert!( control2.is_running() );
  }

  #[ test ]
  fn test_stream_control_state_transitions()
  {
    // Test complete state transition flow
    let control = StreamControl::new( 100 );

    // Start : Running
    assert_eq!( control.get_state(), StreamState::Running );

    // Transition : Running -> Paused
    control.pause().expect( "Should pause" );
    assert_eq!( control.get_state(), StreamState::Paused );

    // Transition : Paused -> Running
    control.resume().expect( "Should resume" );
    assert_eq!( control.get_state(), StreamState::Running );

    // Transition : Running -> Cancelled
    control.cancel();
    assert_eq!( control.get_state(), StreamState::Cancelled );

    // Cancelled is terminal state
    assert!( control.pause().is_err() );
    assert!( control.resume().is_err() );
    assert_eq!( control.get_state(), StreamState::Cancelled );
  }

  #[ test ]
  fn test_stream_control_buffer_size_tracking()
  {
    // Test that buffer_size() returns 0 initially
    let control = StreamControl::new( 100 );
    assert_eq!( control.buffer_size(), 0 );

    // Note : We can't test actual buffering without ControlledStream integration,
    // but we verify the method exists and returns correct initial value
  }

  #[ test ]
  fn test_stream_control_different_buffer_limits()
  {
    // Test creating controls with different buffer limits
    let control_small = StreamControl::new( 10 );
    let control_medium = StreamControl::new( 100 );
    let control_large = StreamControl::new( 1000 );

    // All should start in running state regardless of buffer limit
    assert!( control_small.is_running() );
    assert!( control_medium.is_running() );
    assert!( control_large.is_running() );
  }

  #[ tokio::test ]
  async fn test_stream_control_across_async_boundary()
  {
    // Test that StreamControl can be used across async boundaries
    let control = StreamControl::new( 100 );
    let control_clone = control.clone();

    // Spawn task that pauses after delay
    let pause_task = tokio::spawn( async move
    {
      sleep( Duration::from_millis( 50 ) ).await;
      control_clone.pause()
    } );

    // Initially running
    assert!( control.is_running() );

    // Wait for pause task
    let pause_result = pause_task.await.expect( "Task should complete" );
    assert!( pause_result.is_ok(), "Pause should succeed" );

    // Should now be paused
    assert!( control.is_paused() );
  }

  #[ tokio::test ]
  async fn test_stream_control_concurrent_operations()
  {
    // Test concurrent access from multiple tasks
    let control = StreamControl::new( 100 );

    let control1 = control.clone();
    let control2 = control.clone();
    let control3 = control.clone();

    // Spawn multiple tasks that access state
    let task1 = tokio::spawn( async move
    {
      for _ in 0..10
      {
        let _ = control1.get_state();
        sleep( Duration::from_micros( 100 ) ).await;
      }
    } );

    let task2 = tokio::spawn( async move
    {
      for _ in 0..5
      {
        let _ = control2.pause();
        sleep( Duration::from_micros( 200 ) ).await;
        let _ = control2.resume();
        sleep( Duration::from_micros( 200 ) ).await;
      }
    } );

    let task3 = tokio::spawn( async move
    {
      for _ in 0..10
      {
        let _ = control3.buffer_size();
        sleep( Duration::from_micros( 150 ) ).await;
      }
    } );

    // All tasks should complete without panic
    task1.await.expect( "Task 1 should complete" );
    task2.await.expect( "Task 2 should complete" );
    task3.await.expect( "Task 3 should complete" );

    // Final state should be valid (either Running or Paused, not Cancelled)
    let final_state = control.get_state();
    assert!( final_state != StreamState::Cancelled );
  }
}

#[ cfg( not( feature = "streaming-control" ) ) ]
mod streaming_control_feature_disabled
{
  #[ test ]
  fn test_streaming_control_feature_disabled()
  {
    // When streaming-control feature is disabled, this test verifies
    // that compilation succeeds without the feature
  }
}
