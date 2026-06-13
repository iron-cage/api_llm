//! Buffered Streaming for Smoother UX
//!
//! Buffer streaming responses for smoother display and better user experience.

#[ cfg( feature = "streaming" ) ]
mod private
{
  use futures_core::Stream;
  use std::pin::Pin;
  use std::task::{ Context, Poll };
  use std::time::{ Duration, Instant };

  /// Configuration for buffered streaming
  #[ derive( Debug, Clone ) ]
  pub struct BufferConfig
  {
    /// Maximum buffer size before forcing a flush
    pub max_buffer_size : usize,
    /// Maximum time to wait before forcing a flush
    pub max_buffer_time : Duration,
    /// Whether to flush on newline characters
    pub flush_on_newline : bool,
  }

  impl BufferConfig
  {
    /// Create default buffer configuration
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Set maximum buffer size
    #[ must_use ]
    pub fn with_max_buffer_size( mut self, size : usize ) -> Self
    {
      self.max_buffer_size = size;
      self
    }

    /// Set maximum buffer time
    #[ must_use ]
    pub fn with_max_buffer_time( mut self, duration : Duration ) -> Self
    {
      self.max_buffer_time = duration;
      self
    }

    /// Set flush on newline
    #[ must_use ]
    pub fn with_flush_on_newline( mut self, enabled : bool ) -> Self
    {
      self.flush_on_newline = enabled;
      self
    }
  }

  impl Default for BufferConfig
  {
    fn default() -> Self
    {
      Self
      {
        max_buffer_size : 64,
        max_buffer_time : Duration::from_millis( 100 ),
        flush_on_newline : true,
      }
    }
  }

  /// Buffered stream wrapper
  #[ derive( Debug ) ]
  pub struct BufferedStream< S >
  {
    inner : S,
    buffer : String,
    config : BufferConfig,
    last_flush : Instant,
  }

  impl< S > BufferedStream< S >
  where
    S : Stream< Item = String > + Unpin,
  {
    /// Create new buffered stream with configuration
    #[ must_use ]
    pub fn new( stream : S, config : BufferConfig ) -> Self
    {
      Self
      {
        inner : stream,
        buffer : String::new(),
        config,
        last_flush : Instant::now(),
      }
    }

    /// Check if buffer should be flushed
    fn should_flush( &self ) -> bool
    {
      // Flush if buffer size exceeded
      if self.buffer.len() >= self.config.max_buffer_size
      {
        return true;
      }

      // Flush if time exceeded
      if self.last_flush.elapsed() >= self.config.max_buffer_time
      {
        return true;
      }

      // Flush on newline if enabled
      if self.config.flush_on_newline && self.buffer.contains( '\n' )
      {
        return true;
      }

      false
    }

    /// Flush the buffer
    fn flush( &mut self ) -> Option< String >
    {
      if self.buffer.is_empty()
      {
        None
      }
      else
      {
        let content = self.buffer.clone();
        self.buffer.clear();
        self.last_flush = Instant::now();
        Some( content )
      }
    }
  }

  impl< S > Stream for BufferedStream< S >
  where
    S : Stream< Item = String > + Unpin,
  {
    type Item = String;

    fn poll_next( mut self : Pin< &mut Self >, cx : &mut Context< '_ > ) -> Poll< Option< Self::Item > >
    {
      // Poll the inner stream
      match Pin::new( &mut self.inner ).poll_next( cx )
      {
        Poll::Ready( Some( item ) ) => {
          // Add to buffer
          self.buffer.push_str( &item );

          // Check if we should flush
          if self.should_flush()
          {
            Poll::Ready( self.flush() )
          }
          else
          {
            // Continue polling
            cx.waker().wake_by_ref();
            Poll::Pending
          }
        },
        Poll::Ready( None ) => {
          // Stream ended, flush remaining buffer
          Poll::Ready( self.flush() )
        },
        Poll::Pending => {
          // Check if time-based flush is needed
          if !self.buffer.is_empty() && self.should_flush()
          {
            Poll::Ready( self.flush() )
          }
          else
          {
            Poll::Pending
          }
        }
      }
    }
  }

  /// Extension trait for streams
  pub trait StreamBufferExt : Stream< Item = String > + Sized + Unpin
  {
    /// Buffer this stream with custom configuration
    fn with_buffer( self, config : BufferConfig ) -> BufferedStream< Self >
    {
      BufferedStream::new( self, config )
    }

    /// Buffer this stream with default configuration
    fn with_buffer_default( self ) -> BufferedStream< Self >
    {
      BufferedStream::new( self, BufferConfig::default() )
    }
  }

  impl< T > StreamBufferExt for T
  where
    T : Stream< Item = String > + Unpin,
  {
  }

}

#[ cfg( feature = "streaming" ) ]
crate::mod_interface!
{
  exposed use
  {
    BufferConfig,
    BufferedStream,
    StreamBufferExt,
  };
}
