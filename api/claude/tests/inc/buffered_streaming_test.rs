//! Buffered Streaming Tests
//!
//! Tests for buffered streaming configuration and stream behavior.
//! (Migrated from `src/buffered_streaming.rs` `#[cfg(test)]` block)

#[ allow( unused_imports ) ]
use super::*;
use the_module::StreamBufferExt;
use futures_util::{ stream, StreamExt };
use core::time::Duration;

// ============================================================================
// UNIT TESTS - BUFFER CONFIG
// ============================================================================

#[ tokio::test ]
async fn test_buffer_config_creation()
{
  let config = the_module::BufferConfig::new();
  assert_eq!( config.max_buffer_size, 64 );
  assert_eq!( config.max_buffer_time, Duration::from_millis( 100 ) );
  assert!( config.flush_on_newline );
}

#[ tokio::test ]
async fn test_buffer_config_builder()
{
  let config = the_module::BufferConfig::new()
    .with_max_buffer_size( 128 )
    .with_max_buffer_time( Duration::from_millis( 200 ) )
    .with_flush_on_newline( false );

  assert_eq!( config.max_buffer_size, 128 );
  assert_eq!( config.max_buffer_time, Duration::from_millis( 200 ) );
  assert!( !config.flush_on_newline );
}

#[ tokio::test ]
async fn test_buffered_stream_basic()
{
  let items = vec![ "a".to_string(), "b".to_string(), "c".to_string() ];
  let stream = stream::iter( items );

  let config = the_module::BufferConfig::new()
    .with_max_buffer_size( 10 )
    .with_flush_on_newline( false );

  let mut buffered = stream.with_buffer( config );

  let result = buffered.next().await;
  assert!( result.is_some() );
}

#[ tokio::test ]
async fn test_buffered_stream_size_threshold()
{
  let items = vec![ "x".to_string(); 100 ]; // 100 single characters
  let stream = stream::iter( items );

  let config = the_module::BufferConfig::new()
    .with_max_buffer_size( 10 )
    .with_flush_on_newline( false );

  let mut buffered = stream.with_buffer( config );

  let mut chunks = Vec::new();
  while let Some( chunk ) = buffered.next().await
  {
    chunks.push( chunk );
  }

  // Should have created multiple chunks due to size limit
  assert!( chunks.len() > 1 );
}

#[ tokio::test ]
async fn test_buffered_stream_flush_on_newline()
{
  let items = vec![ "hello\n".to_string(), "world".to_string() ];
  let stream = stream::iter( items );

  let config = the_module::BufferConfig::new()
    .with_max_buffer_size( 100 )
    .with_flush_on_newline( true );

  let mut buffered = stream.with_buffer( config );

  // First chunk should flush on newline
  let first = buffered.next().await;
  assert!( first.is_some() );
  assert!( first.unwrap().contains( '\n' ) );
}
