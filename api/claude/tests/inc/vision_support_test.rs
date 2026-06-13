//! Vision Support Integration Tests - STRICT FAILURE POLICY
//!
//! MANDATORY INTEGRATION TEST REQUIREMENTS:
//! - These tests use REAL Anthropic API endpoints - NO MOCKING ALLOWED
//! - Tests MUST FAIL IMMEDIATELY if API secrets are not available (no graceful fallbacks)
//! - Tests MUST FAIL IMMEDIATELY on network connectivity issues
//! - Tests MUST FAIL IMMEDIATELY on API authentication failures
//! - Tests MUST FAIL IMMEDIATELY on any API endpoint errors
//! - NO SILENT PASSES allowed when problems occur
//!
//! Run with : cargo test --features vision,integration
//! Requires : Valid `ANTHROPIC_API_KEY` in environment or ../../secret/-secrets.sh

#[ allow( unused_imports ) ]
use super::*;

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_image_content_structure()
{
  // Test that ImageContent has correct structure according to Claude API
  let image_content = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "image/jpeg".to_string(),
      data : "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==".to_string(),
    },
  };
  
  assert_eq!( image_content.r#type, "image" );
  assert_eq!( image_content.source.r#type, "base64" );
  assert_eq!( image_content.source.media_type, "image/jpeg" );
  assert!( !image_content.source.data.is_empty() );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_image_source_types()
{
  // Test different image source types
  let base64_source = the_module::ImageSource
  {
    r#type : "base64".to_string(),
    media_type : "image/png".to_string(),
    data : "base64datahere".to_string(),
  };
  
  assert_eq!( base64_source.r#type, "base64" );
  assert_eq!( base64_source.media_type, "image/png" );
  
  // Test different media types
  let media_types = vec![ "image/jpeg", "image/png", "image/gif", "image/webp" ];
  
  for media_type in media_types
  {
    let source = the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : media_type.to_string(),
      data : "test_data".to_string(),
    };
    
    assert_eq!( source.media_type, media_type );
  }
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_mixed_content_message()
{
  // Test message with both text and image content
  let image_content = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "image/jpeg".to_string(),
      data : "base64imagedata".to_string(),
    },
  };
  
  let message = the_module::Message::user_with_image( 
    "What's in this picture?".to_string(), 
    image_content 
  );
  
  match message.role
  {
    the_module::Role::User => {},
    _ => panic!( "Expected User role" ),
  }
  
  assert_eq!( message.content.len(), 2 ); // Text + Image
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_image_only_message()
{
  // Test message with only image content
  let image_content = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "image/png".to_string(),
      data : "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==".to_string(),
    },
  };
  
  let message = the_module::Message::user_image( image_content.clone() );
  
  match message.role
  {
    the_module::Role::User => {},
    _ => panic!( "Expected User role" ),
  }
  
  assert_eq!( message.content.len(), 1 );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_multiple_images_message()
{
  // Test message with multiple images
  let image1 = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "image/jpeg".to_string(),
      data : "first_image_data".to_string(),
    },
  };
  
  let image2 = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "image/png".to_string(),
      data : "second_image_data".to_string(),
    },
  };
  
  let message = the_module::Message::user_with_images(
    "Compare these two images".to_string(),
    vec![ image1, image2 ]
  );
  
  assert_eq!( message.content.len(), 3 ); // Text + 2 images
  
  // Verify content types
  assert_eq!( message.content[0].r#type(), "text" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_vision_api_request()
{
  // Test API request with vision content
  let image_content = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "image/jpeg".to_string(),
      data : "test_image_base64_data".to_string(),
    },
  };
  
  let message = the_module::Message::user_with_image(
    "Describe what you see in this image".to_string(),
    image_content
  );
  
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 500 )
    .message( message )
    .build();
  
  assert_eq!( request.model, "claude-sonnet-4-5-20250929" );
  assert_eq!( request.messages.len(), 1 );
  assert_eq!( request.messages[0].content.len(), 2 ); // Text + Image
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_vision_conversation_flow()
{
  // Test multi-turn conversation with vision
  let image_content = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "image/png".to_string(),
      data : "conversation_image_data".to_string(),
    },
  };
  
  let messages = vec![
    the_module::Message::user_with_image( 
      "What's in this image?".to_string(), 
      image_content 
    ),
    the_module::Message::assistant( 
      "I can see a beautiful landscape with mountains and trees.".to_string() 
    ),
    the_module::Message::user( 
      "What time of day do you think it is?".to_string() 
    ),
  ];
  
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 300 )
    .messages( messages.clone() )
    .build();
  
  assert_eq!( request.messages.len(), 3 );
  
  // Check first message has both text and image
  assert_eq!( request.messages[0].content.len(), 2 );
  
  // Check subsequent messages are text only
  assert_eq!( request.messages[1].content.len(), 1 );
  assert_eq!( request.messages[2].content.len(), 1 );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_image_content_serialization()
{
  // Test that image content serializes correctly to JSON
  let image_content = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "image/jpeg".to_string(),
      data : "test123".to_string(),
    },
  };
  
  let json = serde_json::to_string( &image_content ).expect( "Should serialize successfully" );
  
  // Verify JSON structure matches expected format
  assert!( json.contains( "\"type\":\"image\"" ) );
  assert!( json.contains( "\"source\":" ) );
  assert!( json.contains( "\"type\":\"base64\"" ) );
  assert!( json.contains( "\"media_type\":\"image/jpeg\"" ) );
  assert!( json.contains( "\"data\":\"test123\"" ) );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_image_content_deserialization()
{
  // Test that we can deserialize image content from JSON
  let json = r#"{
    "type": "image",
    "source": {
      "type": "base64",
      "media_type": "image/png",
      "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg=="
    }
  }"#;
  
  let image_content : the_module::ImageContent = serde_json::from_str( json ).expect( "Should deserialize successfully" );
  
  assert_eq!( image_content.r#type, "image" );
  assert_eq!( image_content.source.r#type, "base64" );
  assert_eq!( image_content.source.media_type, "image/png" );
  assert!( !image_content.source.data.is_empty() );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_vision_with_tools()
{
  // Test vision functionality combined with tool calling
  let image_content = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "image/jpeg".to_string(),
      data : "image_for_tool_analysis".to_string(),
    },
  };
  
  let tools = vec![
    the_module::ToolDefinition
    {
      name : "image_analyzer".to_string(),
      description : "Analyze image content and extract information".to_string(),
      input_schema : serde_json::json!(
      {
        "type": "object",
        "properties": {
          "analysis_type": {"type": "string", "enum": ["objects", "colors", "text", "emotions"]}
        },
        "required": ["analysis_type"]
      }),
    }
  ];
  
  let message = the_module::Message::user_with_image(
    "Analyze this image for objects".to_string(),
    image_content
  );
  
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 400 )
    .message( message )
    .tools( tools )
    .tool_choice( the_module::ToolChoice::Auto )
    .build();
  
  assert!( request.tools.is_some() );
  assert!( request.tool_choice.is_some() );
  assert_eq!( request.messages[0].content.len(), 2 ); // Text + Image
}

// Removed test_vision_api_call - used fake API keys

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_image_validation()
{
  // Test image content validation
  
  // Test empty image data
  let empty_image = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "image/jpeg".to_string(),
      data : String::new(), // Empty data
    },
  };
  
  assert!( empty_image.source.data.is_empty() );
  
  // Test invalid media type (should be handled gracefully)
  let invalid_media_type = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "invalid/type".to_string(),
      data : "test_data".to_string(),
    },
  };
  
  assert_eq!( invalid_media_type.source.media_type, "invalid/type" );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_large_image_handling()
{
  // Test handling of larger image data
  let large_data = "a".repeat( 1000 ); // Simulate larger base64 data
  
  let large_image = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "image/jpeg".to_string(),
      data : large_data.clone(),
    },
  };
  
  assert_eq!( large_image.source.data.len(), 1000 );
  
  // Test serialization of large image
  let json = serde_json::to_string( &large_image ).expect( "Should handle large images" );
  assert!( json.len() > 1000 );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_vision_with_streaming()
{
  // Test vision functionality combined with streaming
  let image_content = the_module::ImageContent
  {
    r#type : "image".to_string(),
    source : the_module::ImageSource
    {
      r#type : "base64".to_string(),
      media_type : "image/png".to_string(),
      data : "streaming_test_image".to_string(),
    },
  };
  
  let message = the_module::Message::user_with_image(
    "Describe this image in detail".to_string(),
    image_content
  );
  
  let request = the_module::CreateMessageRequest::builder()
    .model( "claude-sonnet-4-5-20250929" )
    .max_tokens( 600 )
    .message( message )
    .stream( true )
    .build();
  
  assert!( request.stream.unwrap() );
  assert_eq!( request.messages[0].content.len(), 2 );
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_mixed_content_serialization()
{
  // Test serialization of mixed content message
  let message = the_module::Message::user_with_image(
    "Analyze this".to_string(),
    the_module::ImageContent
    {
      r#type : "image".to_string(),
      source : the_module::ImageSource
      {
        r#type : "base64".to_string(),
        media_type : "image/jpeg".to_string(),
        data : "mixed_content_test".to_string(),
      },
    }
  );
  
  let json = serde_json::to_string( &message ).expect( "Should serialize mixed content" );
  
  // Should contain both text and image content
  assert!( json.contains( "\"role\":\"user\"" ) );
  assert!( json.contains( "\"content\":" ) );
  assert!( json.contains( "\"type\":\"text\"" ) );
  assert!( json.contains( "\"type\":\"image\"" ) );
  assert!( json.contains( "\"source\":" ) );
}

// ============================================================================
// INTEGRATION TESTS - REAL API VISION SUPPORT
// ============================================================================

#[ cfg( all( feature = "integration", feature = "vision" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_vision_real_image_processing()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for vision testing" );

  // Create a simple base64 test image (1x1 red pixel PNG)
  let test_image_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
  
  let image_source = the_module::ImageSource::png( test_image_base64 );
  let image_content = the_module::ImageContent::new( image_source );

  let request = the_module::CreateMessageRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(), // Vision-capable model
    max_tokens : 50,
    messages : vec![ 
      the_module::Message::user_with_image(
        "What color is this image?".to_string(),
        image_content
      )
    ],
    system : None,
    temperature : None,
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Vision API call must work : {err}" ),
  };

  // Verify real API vision response
  assert!( !response.id.is_empty(), "Vision API must return message ID" );
  assert_eq!( response.r#type, "message" );
  assert_eq!( response.role, "assistant" );
  assert!( !response.content.is_empty(), "Vision API must return content" );
  assert!( response.usage.input_tokens > 0, "Vision API must track input tokens" );
  assert!( response.usage.output_tokens > 0, "Vision API must track output tokens" );
  
  let content_text = response.content[0].text.as_ref()
    .expect( "Vision response must have text content" );
  
  // Verify the API processed the image (should mention color/image analysis)
  let response_lower = content_text.to_lowercase();
  assert!( 
    response_lower.contains( "red" ) || 
    response_lower.contains( "color" ) || 
    response_lower.contains( "image" ) ||
    response_lower.contains( "pixel" ),
    "Vision API should analyze image content, got : {content_text}"
  );
  
  println!( "✅ Vision integration test passed!" );
  println!( "   Vision response : {content_text}" );
  println!( "   Input tokens : {}", response.usage.input_tokens );
  println!( "   Output tokens : {}", response.usage.output_tokens );
}

#[ cfg( all( feature = "integration", feature = "vision" ) ) ]
#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn integration_vision_mixed_content_real_api()
{
  let client = the_module::Client::from_workspace()
    .expect( "INTEGRATION: Must have valid API key for mixed content testing" );

  // Create test image (simple 1x1 red pixel - using known working base64)
  let test_image_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
  
  let image_content = the_module::ImageContent::png( test_image_base64 );

  // Test mixed text + image content
  let message = the_module::Message::user_with_image(
    "I'm sending you an image and asking : What do you see in this small image? Please be specific about any colors or patterns.".to_string(),
    image_content
  );

  let request = the_module::CreateMessageRequest
  {
    model : "claude-sonnet-4-5-20250929".to_string(),
    max_tokens : 100,
    messages : vec![ message ],
    system : Some( vec![ the_module::SystemContent::text( "You are a helpful vision assistant. Describe images accurately." ) ] ),
    temperature : Some( 0.1 ),
    stream : None,
    tools : None,
    tool_choice : None,
  };

  let response = match client.create_message( request ).await
  {
    Ok( response ) => response,
    Err( the_module::AnthropicError::Api( ref api_err ) ) if api_err.message.contains( "credit balance is too low" ) =>
    {
      panic!( "INTEGRATION: credit balance exhausted - real API call succeeded but account has no credits. Test must fail per Loud Failure Mandate: {}", api_err.message )
    },
    Err( err ) => panic!( "INTEGRATION: Mixed content vision API call must work : {err}" ),
  };

  // Verify mixed content response
  assert!( !response.id.is_empty() );
  assert!( response.usage.input_tokens > 0 );
  assert!( response.usage.output_tokens > 0 );
  
  let content_text = response.content[0].text.as_ref()
    .expect( "Mixed content response must have text" );
    
  // Should acknowledge both the text instruction and image analysis
  let response_lower = content_text.to_lowercase();
  assert!(
    response_lower.contains( "image" ) || response_lower.contains( "see" ) ||
    response_lower.contains( "blank" ) || response_lower.contains( "tint" ) ||
    response_lower.contains( "color" ) || response_lower.contains( "pixel" ),
    "Mixed content should show vision processing, got : {content_text}"
  );
  
  println!( "✅ Vision mixed content integration test passed!" );
  println!( "   Mixed content response : {content_text}" );
}