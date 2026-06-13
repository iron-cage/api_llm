//! Request/response compression support for bandwidth optimization.
//!
//! This module provides HTTP compression using gzip, deflate, or brotli algorithms.
//! Compression can significantly reduce bandwidth usage for large payloads like
//! multimodal content, batch embeddings, and long conversation histories.

use std::io::{ Read, Write };
use flate2::{ Compression as FlateLevel, read::{ GzDecoder, DeflateDecoder }, write::{ GzEncoder, DeflateEncoder } };
use crate::error::Error;

/// Compression algorithm to use for requests/responses
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum CompressionAlgorithm
{
  /// GZIP compression (RFC 1952)
  Gzip,
  /// DEFLATE compression (RFC 1951)
  Deflate,
  /// Brotli compression (RFC 7932)
  Brotli,
  /// No compression
  None,
}

impl CompressionAlgorithm
{
  /// Get the Content-Encoding header value for this algorithm
  #[ must_use ]
  pub fn content_encoding( &self ) -> Option< &'static str >
  {
    match self
    {
      Self::Gzip => Some( "gzip" ),
      Self::Deflate => Some( "deflate" ),
      Self::Brotli => Some( "br" ),
      Self::None => None,
    }
  }
}

impl Default for CompressionAlgorithm
{
  #[ inline ]
  fn default() -> Self
  {
    Self::Gzip
  }
}

/// Configuration for compression behavior
#[ derive( Debug, Clone ) ]
pub struct CompressionConfig
{
  /// Compression algorithm to use
  pub algorithm : CompressionAlgorithm,
  /// Compression level (0-9 for gzip/deflate, 0-11 for brotli)
  pub level : u32,
  /// Minimum payload size to compress (bytes)
  /// Payloads smaller than this won't be compressed
  pub min_size : usize,
}

impl CompressionConfig
{
  /// Create default compression configuration
  ///
  /// Uses gzip compression at level 6 with 1KB minimum size
  #[ must_use ]
  pub fn new() -> Self
  {
    Self {
      algorithm : CompressionAlgorithm::Gzip,
      level : 6,
      min_size : 1024,
    }
  }

  /// Set compression algorithm
  #[ must_use ]
  pub fn algorithm( mut self, algorithm : CompressionAlgorithm ) -> Self
  {
    self.algorithm = algorithm;
    self
  }

  /// Set compression level
  ///
  /// - For gzip/deflate : 0-9 (0=none, 1=fastest, 9=best compression)
  /// - For brotli : 0-11 (0=none, 1=fastest, 11=best compression)
  #[ must_use ]
  pub fn level( mut self, level : u32 ) -> Self
  {
    self.level = level;
    self
  }

  /// Set minimum payload size to compress
  #[ must_use ]
  pub fn min_size( mut self, min_size : usize ) -> Self
  {
    self.min_size = min_size;
    self
  }
}

impl Default for CompressionConfig
{
  #[ inline ]
  fn default() -> Self
  {
    Self::new()
  }
}

/// Compress data using the specified configuration
///
/// # Errors
///
/// Returns error if compression fails
pub fn compress( data : &[ u8 ], config : &CompressionConfig ) -> Result< Vec< u8 >, Error >
{
  // Skip compression for small payloads
  if data.len() < config.min_size
  {
    return Ok( data.to_vec() );
  }

  match config.algorithm
  {
    CompressionAlgorithm::Gzip => {
      let mut encoder = GzEncoder::new( Vec::new(), FlateLevel::new( config.level ) );
      encoder.write_all( data )
        .map_err( | e | Error::ConfigurationError( format!( "Gzip compression failed : {}", e ) ) )?;
      encoder.finish()
        .map_err( | e | Error::ConfigurationError( format!( "Gzip compression finish failed : {}", e ) ) )
    },
    CompressionAlgorithm::Deflate => {
      let mut encoder = DeflateEncoder::new( Vec::new(), FlateLevel::new( config.level ) );
      encoder.write_all( data )
        .map_err( | e | Error::ConfigurationError( format!( "Deflate compression failed : {}", e ) ) )?;
      encoder.finish()
        .map_err( | e | Error::ConfigurationError( format!( "Deflate compression finish failed : {}", e ) ) )
    },
    CompressionAlgorithm::Brotli => {
      use brotli::enc::BrotliEncoderParams;
      let params = BrotliEncoderParams {
        quality : config.level as i32,
        ..Default::default()
      };
      let mut output = Vec::new();
      {
        let mut compressor = brotli::CompressorWriter::with_params(
          &mut output,
          4096,
          &params
        );
        compressor.write_all( data )
          .map_err( | e | Error::ConfigurationError( format!( "Brotli compression failed : {}", e ) ) )?;
      }
      Ok( output )
    },
    CompressionAlgorithm::None => Ok( data.to_vec() ),
  }
}

/// Decompress data using the specified algorithm
///
/// # Errors
///
/// Returns error if decompression fails
pub fn decompress( data : &[ u8 ], algorithm : CompressionAlgorithm ) -> Result< Vec< u8 >, Error >
{
  match algorithm
  {
    CompressionAlgorithm::Gzip => {
      let mut decoder = GzDecoder::new( data );
      let mut output = Vec::new();
      decoder.read_to_end( &mut output )
        .map_err( | e | Error::ConfigurationError( format!( "Gzip decompression failed : {}", e ) ) )?;
      Ok( output )
    },
    CompressionAlgorithm::Deflate => {
      let mut decoder = DeflateDecoder::new( data );
      let mut output = Vec::new();
      decoder.read_to_end( &mut output )
        .map_err( | e | Error::ConfigurationError( format!( "Deflate decompression failed : {}", e ) ) )?;
      Ok( output )
    },
    CompressionAlgorithm::Brotli => {
      let mut output = Vec::new();
      {
        let mut decompressor = brotli::Decompressor::new( data, 4096 );
        decompressor.read_to_end( &mut output )
          .map_err( | e | Error::ConfigurationError( format!( "Brotli decompression failed : {}", e ) ) )?;
      }
      Ok( output )
    },
    CompressionAlgorithm::None => Ok( data.to_vec() ),
  }
}
