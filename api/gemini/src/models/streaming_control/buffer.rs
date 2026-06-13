//! Buffer management for streaming control with multiple strategies.

use std::collections::VecDeque;

/// Strategy for managing buffered data during pause operations
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum BufferStrategy
{
  /// Use a simple vector buffer (default, good for small streams)
  Vector,
  /// Use a circular buffer for better memory efficiency
  Circular,
  /// Use a chunked buffer for large streams
  Chunked {
    /// Size of each chunk in the buffer
    chunk_size : usize
  },
}

/// Efficient buffer implementation based on strategy
pub( crate ) enum StreamBuffer< T >
{
  Vector( Vec< Result< T, crate::error::Error > > ),
  Circular( VecDeque< Result< T, crate::error::Error > > ),
  Chunked {
    chunks : Vec< Vec< Result< T, crate::error::Error > > >,
    current_chunk : Vec< Result< T, crate::error::Error > >,
    chunk_size : usize
  },
}

impl< T > StreamBuffer< T >
{
  pub fn new( strategy : &BufferStrategy, _chunk_size : Option< usize > ) -> Self
  {
    match strategy
    {
      BufferStrategy::Vector => Self::Vector( Vec::new() ),
      BufferStrategy::Circular => Self::Circular( VecDeque::new() ),
      BufferStrategy::Chunked { chunk_size } => Self::Chunked {
        chunks : Vec::new(),
        current_chunk : Vec::new(),
        chunk_size : *chunk_size,
      },
    }
  }

  pub fn push( &mut self, item : Result< T, crate::error::Error > )
  {
    match self
    {
      Self::Vector( vec ) => vec.push( item ),
      Self::Circular( deque ) => deque.push_back( item ),
      Self::Chunked { chunks, current_chunk, chunk_size } => {
        if current_chunk.len() >= *chunk_size
        {
          let mut new_chunk = Vec::new();
          std ::mem::swap( current_chunk, &mut new_chunk );
          chunks.push( new_chunk );
          current_chunk.clear();
        }
        current_chunk.push( item );
      },
    }
  }

  pub fn drain_all( &mut self ) -> Vec< Result< T, crate::error::Error > >
  {
    match self
    {
      Self::Vector( vec ) => std::mem::take( vec ),
      Self::Circular( deque ) => deque.drain( .. ).collect(),
      Self::Chunked { chunks, current_chunk, .. } => {
        let mut all_items = Vec::new();
        for chunk in chunks.drain( .. )
        {
          all_items.extend( chunk );
        }
        all_items.append( current_chunk );
        all_items
      },
    }
  }

  pub fn len( &self ) -> usize
  {
    match self
    {
      Self::Vector( vec ) => vec.len(),
      Self::Circular( deque ) => deque.len(),
      Self::Chunked { chunks, current_chunk, .. } => {
        chunks.iter().map( |c| c.len() ).sum::< usize >() + current_chunk.len()
      },
    }
  }

  #[ allow( dead_code ) ]
  pub fn is_empty( &self ) -> bool
  {
    self.len() == 0
  }
}
