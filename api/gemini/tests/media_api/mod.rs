//! Media API Integration Tests for Gemini API Client
//!
//! These tests verify comprehensive media processing and management capabilities including:
//! - Media upload and storage operations for various file types (images, audio, video, documents)
//! - Media processing and transformation capabilities
//! - Media metadata extraction and analysis
//! - Media search and retrieval operations
//! - Media versioning and management
//! - Media access control and permissions
//! - Integration with existing multimodal content generation
//! - Large file handling and streaming uploads
//! - Error handling and edge cases
//!
//! All tests use real API tokens and make actual API calls where possible.

// Import shared test utilities from common module
mod common;
#[ cfg( feature = "integration" ) ]
use common::create_integration_client;

use api_gemini::models::{
  FileMetadata, UploadFileRequest, ListFilesRequest,
  DeleteFileRequest, Content, Part, Blob,
  GenerateContentRequest
};
use std::collections::HashMap;

// ======================
// Unit Tests
// ======================

#[ cfg( test ) ]
mod unit_tests
{
  use super::*;

  /// Test file metadata structure and validation
  #[ test ]
  fn test_file_metadata_structure()
  {
    // Test basic file metadata creation
    let metadata = FileMetadata {
      name: "files/test-image-123".to_string(),
      display_name: Some( "Test Image".to_string() ),
      mime_type: "image/png".to_string(),
      size_bytes: Some( 1024000 ),
      create_time: Some( "2024-01-01T00:00:00Z".to_string() ),
      update_time: Some( "2024-01-01T00:00:00Z".to_string() ),
      expiration_time: None,
      sha256_hash: Some( "abc123def456".to_string() ),
      uri: Some( "https://generativelanguage.googleapis.com/v1beta/files/test-image-123".to_string() ),
      state: Some( "ACTIVE".to_string() ),
      error: None,
      video_metadata: None,
    };

    assert_eq!( metadata.name, "files/test-image-123" );
    assert_eq!( metadata.display_name, Some( "Test Image".to_string() ) );
    assert_eq!( metadata.mime_type, "image/png" );
    assert_eq!( metadata.size_bytes, Some( 1024000 ) );
    assert!( metadata.uri.is_some() );
    assert!( metadata.state.is_some() );

    println!( "✓ File metadata structure validation passed" );
  }

  /// Test upload request validation
  #[ test ]
  fn test_upload_request_validation()
  {
    // Valid upload request
    let valid_request = UploadFileRequest {
      file_data: vec![ 1, 2, 3, 4, 5 ],
      mime_type: "image/jpeg".to_string(),
      display_name: Some( "Test Upload".to_string() ),
    };

    assert!( !valid_request.file_data.is_empty() );
    assert!( !valid_request.mime_type.is_empty() );
    assert!( valid_request.display_name.is_some() );

    // Empty data should be invalid (validation in real API)
    let empty_request = UploadFileRequest {
      file_data: vec![],
      mime_type: "image/jpeg".to_string(),
      display_name: Some( "Empty File".to_string() ),
    };

    assert!( empty_request.file_data.is_empty() );

    println!( "✓ Upload request validation passed" );
  }

  /// Test supported media types enumeration
  #[ test ]
  fn test_supported_media_types()
  {
    let supported_types = vec![
    // Images
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/bmp",
    "image/tiff",
    // Audio
    "audio/mp3",
    "audio/wav",
    "audio/flac",
    "audio/aac",
    "audio/ogg",
    // Video
    "video/mp4",
    "video/avi",
    "video/mov",
    "video/webm",
    "video/mkv",
    // Documents
    "application/pdf",
    "text/plain",
    "text/csv",
    "application/json",
    "text/xml",
    ];

    for mime_type in supported_types
    {
      assert!( !mime_type.is_empty() );
      assert!( mime_type.contains( "/" ) );
    }

    println!( "✓ Supported media types validation passed" );
  }

  /// Test list files request configuration
  #[ test ]
  fn test_list_files_request_config()
  {
    // Default request
    let default_request = ListFilesRequest::default();
    assert!( default_request.page_size.is_none() );
    assert!( default_request.page_token.is_none() );

    // Configured request
    let configured_request = ListFilesRequest {
      page_size: Some( 50 ),
      page_token: Some( "next_page_token_123".to_string() ),
    };

    assert_eq!( configured_request.page_size, Some( 50 ) );
    assert!( configured_request.page_token.is_some() );

    println!( "✓ List files request configuration passed" );
  }

  /// Test delete request structure
  #[ test ]
  fn test_delete_request_structure()
  {
    let delete_request = DeleteFileRequest {
      name: "files/test-file-123".to_string(),
    };

    assert!( !delete_request.name.is_empty() );
    assert!( delete_request.name.starts_with( "files/" ) );

    println!( "✓ Delete request structure validation passed" );
  }

  /// Test video metadata structure
  #[ test ]
  fn test_video_metadata_structure()
  {
    use api_gemini::models::VideoMetadata;

    let video_metadata = VideoMetadata {
      video_duration: Some( "PT2M30S".to_string() ), // ISO 8601 duration format
    };

    assert!( video_metadata.video_duration.is_some() );
    assert!( video_metadata.video_duration.as_ref().unwrap().starts_with( "PT" ) );

    println!( "✓ Video metadata structure validation passed" );
  }

  /// Test blob structure for inline media
  #[ test ]
  fn test_blob_structure()
  {
    let test_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";

    let blob = Blob {
      mime_type: "image/png".to_string(),
      data: test_data.to_string(),
    };

    assert!( !blob.mime_type.is_empty() );
    assert!( !blob.data.is_empty() );
    assert!( blob.mime_type.starts_with( "image/" ) );

    println!( "✓ Blob structure validation passed" );
  }
}

// Sub-modules with integration tests
#[ cfg( feature = "integration" ) ]
mod basic_operations;
#[ cfg( feature = "integration" ) ]
mod advanced_features;
#[ cfg( feature = "integration" ) ]
mod reliability;
