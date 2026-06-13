//! Buffered Streaming Tests
//!
//! Tests for BufferConfig construction and BufferedStream behaviour:
//! chunk accumulation, newline flushing, and size-threshold flushing.

#[ cfg( feature = "buffered_streaming" ) ]
mod buffered_streaming_tests
{
  use std::time::Duration;

  use api_gemini::buffered_streaming::{ BufferConfig, BufferedStreamExt };
  use tokio_stream::StreamExt;

  #[ tokio::test ]
  async fn test_buffer_config_creation()
  {
    let config = BufferConfig::new();
    assert_eq!( config.min_buffer_size, 50 );
    assert_eq!( config.max_buffer_time, Duration::from_millis( 100 ) );
    assert!( config.flush_on_newline );
  }

  #[ tokio::test ]
  async fn test_buffer_config_builder()
  {
    let config = BufferConfig::new()
      .with_min_buffer_size( 100 )
      .with_max_buffer_time( Duration::from_millis( 200 ) )
      .with_flush_on_newline( false );

    assert_eq!( config.min_buffer_size, 100 );
    assert_eq!( config.max_buffer_time, Duration::from_millis( 200 ) );
    assert!( !config.flush_on_newline );
  }

  #[ tokio::test ]
  async fn test_buffered_stream_basic()
  {
    let items = vec![ "a".to_string(), "b".to_string(), "c".to_string() ];
    let stream = tokio_stream::iter( items );

    let config = BufferConfig::new().with_min_buffer_size( 2 );
    let mut buffered = stream.buffered( config );

    let mut results = vec![];
    while let Some( chunk ) = buffered.next().await
    {
      results.push( chunk );
    }

    assert!( !results.is_empty(), "Buffered stream must yield at least one chunk" );
  }

  #[ tokio::test ]
  async fn test_buffered_stream_flush_on_newline()
  {
    let items = vec![ "hello".to_string(), "\n".to_string(), "world".to_string() ];
    let stream = tokio_stream::iter( items );

    let config = BufferConfig::new()
      .with_min_buffer_size( 100 )  // Large buffer — only newline triggers flush
      .with_flush_on_newline( true );

    let mut buffered = stream.buffered( config );

    let mut results = vec![];
    while let Some( chunk ) = buffered.next().await
    {
      results.push( chunk );
    }

    assert!( results.len() >= 2, "Newline must flush the buffer even with a large min_buffer_size" );
  }

  #[ tokio::test ]
  async fn test_buffered_stream_size_threshold()
  {
    // Single item already exceeds min_buffer_size — must flush immediately
    let items = vec![ "x".repeat( 60 ) ];
    let stream = tokio_stream::iter( items );

    let config = BufferConfig::new().with_min_buffer_size( 50 );
    let mut buffered = stream.buffered( config );

    let result = buffered.next().await;
    assert!( result.is_some() );
    assert_eq!( result.unwrap().len(), 60 );
  }
}
