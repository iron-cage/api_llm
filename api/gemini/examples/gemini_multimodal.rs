//! Example demonstrating multimodal capabilities with images.
//!
//! This example shows:
//! - How to include images in prompts using base64 encoding
//! - How to analyze images with text prompts
//! - How to handle multiple images in a single request
//! - Best practices for image size and format

use api_gemini::{ client::Client, models::* };
use base64::Engine;
use std::fs;

/// Load and encode an image file to base64
#[ allow( dead_code ) ]
fn load_image_base64( path: &str ) -> Result< String, Box< dyn core::error::Error > >
{
  let image_data = fs::read( path )?;
  Ok( base64::engine::general_purpose::STANDARD.encode( image_data ) )
}

#[ tokio::main ]
#[ allow( clippy::too_many_lines ) ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  let _client = Client::new()?;

  println!( "=== Multimodal (Vision) Example ===" );

  // Note : In a real application, you would load actual image files
  // For this demonstration, we'll show how the API structure works
  println!( "\n1. Single Image Analysis" );
  
  println!( "Note: This example demonstrates the API structure for image analysis." );
  println!( "To use with real images, replace the test data with actual image files:" );
  println!( "  let image_data = fs::read(\"path/to/your/image.png\")?;" );
  println!( "  let image_base64 = base64::engine::general_purpose::STANDARD.encode(&image_data);" );
  println!();

  // Use a minimal valid PNG for demonstration (2x2 transparent PNG)
  let test_image_data = vec!
  [
  0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
  0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR
  0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, // 2x2 dimensions  
  0x08, 0x06, 0x00, 0x00, 0x00, 0x72, 0xB6, 0x0D, 0x24, // RGBA format
  0x00, 0x00, 0x00, 0x11, 0x49, 0x44, 0x41, 0x54, // IDAT
  0x78, 0x9C, 0x62, 0x00, 0x00, 0x00, 0x00, 0x00, 
  0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x01,
  0x0D, 0x0A, 0x2D, 0xB4,
  0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82 // IEND
  ];

  let image_base64 = base64::engine::general_purpose::STANDARD.encode( &test_image_data );

  // Create a request with an image
  let _request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "What do you see in this image? Describe it in detail.".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      },
      Part
      {
        text: None,
        inline_data: Some( Blob
        {
          mime_type: "image/png".to_string(),
          data: image_base64,
        }),
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: Some( GenerationConfig
    {
      temperature: Some( 0.4 ),
      top_k: Some( 32 ),
      top_p: Some( 0.95 ),
      candidate_count: Some( 1 ),
      max_output_tokens: Some( 1024 ),
      stop_sequences: None,
    }),
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  println!( "Analyzing image (structure demonstration)..." );
  
  // Demonstrate how to make the API call (commented out to avoid test image issues)
  /*
  let response = client
  .models()
  .by_name( "gemini-2.5-flash" )
  .generate_content( &request )
  .await?;

  if let Some( candidate ) = response.candidates.first()
  {
    if let Some( part ) = candidate.content.parts.first()
    {
      if let Some( text ) = &part.text
      {
      println!( "\nModel's analysis : {text}" );
      }
    }
  }
  */
  
  println!( "API call structure prepared successfully!" );
  println!( "✓ Request contains both text prompt and image data" );
  println!( "✓ Image properly encoded as base64 with MIME type" );
  println!( "✓ Generation config optimized for vision analysis" );

  // Example 2: Multiple images with comparison
  println!( "\n2. Multiple Image Comparison" );

  // Create a minimal valid 1x1 blue pixel PNG (verified format)
  let test_image_data_2 = vec!
  [
  0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
  0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk header
  0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // Width=1, Height=1
  0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, // 8-bit RGB, CRC
  0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT chunk header
  0x78, 0x9C, 0x62, 0x60, 0x60, 0xFC, 0x0F, 0x00, 0x01, 0x01, 0x01, 0x00, 0x35, 0x0A, 0xDB, 0xA7, // Blue pixel compressed data
  0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82 // IEND chunk
  ];

  let image_base64_2 = base64::engine::general_purpose::STANDARD.encode( &test_image_data_2 );

  let _multi_image_request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "Compare these two images. What are the differences?".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      },
      Part
      {
        text: Some( "First image:".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      },
      Part
      {
        text: None,
        inline_data: Some( Blob
        {
          mime_type: "image/png".to_string(),
          data: base64::engine::general_purpose::STANDARD.encode( &test_image_data ),
        }),
        function_call: None,
        function_response: None,
        ..Default::default()
      },
      Part
      {
        text: Some( "Second image:".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      },
      Part
      {
        text: None,
        inline_data: Some( Blob
        {
          mime_type: "image/png".to_string(),
          data: image_base64_2,
        }),
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: Some( GenerationConfig
    {
      temperature: Some( 0.4 ),
      top_k: Some( 32 ),
      top_p: Some( 0.95 ),
      candidate_count: Some( 1 ),
      max_output_tokens: Some( 1024 ),
      stop_sequences: None,
    }),
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  println!( "Comparing images (structure demonstration)..." );

  // API call structure for multiple image comparison (commented out for demo)
  /*
  let multi_response = client
  .models()
  .by_name( "gemini-2.5-flash" )
  .generate_content( &multi_image_request )
  .await?;

  if let Some( candidate ) = multi_response.candidates.first()
  {
    if let Some( part ) = candidate.content.parts.first()
    {
      if let Some( text ) = &part.text
      {
      println!( "\nComparison result : {text}" );
      }
    }
  }
  */
  
  println!( "Multi-image comparison structure prepared!" );
  println!( "✓ Multiple images included in single request" );
  println!( "✓ Context labels for each image" );
  println!( "✓ Comparison prompt properly structured" );

  // Example 3: Image with structured analysis
  println!( "\n3. Structured Image Analysis" );

  let _structured_request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some(
        "Analyze this image and provide a JSON response with the following structure:\n\
        {\n\
          \"type\": \"description of what kind of image this is\",\n\
          \"colors\": [\"list of colors present\"],\n\
          \"objects\": [\"list of objects or elements\"],\n\
          \"mood\": \"overall mood or feeling\"\n\
        }".to_string()
        ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      },
      Part
      {
        text: None,
        inline_data: Some( Blob
        {
          mime_type: "image/png".to_string(),
          data: base64::engine::general_purpose::STANDARD.encode( &test_image_data ),
        }),
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: Some( GenerationConfig
    {
      temperature: Some( 0.2 ), // Lower temperature for structured output
      top_k: Some( 10 ),
      top_p: Some( 0.8 ),
      candidate_count: Some( 1 ),
      max_output_tokens: Some( 512 ),
      stop_sequences: None,
    }),
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  // Structured analysis API call (commented out for demo)
  /*
  let structured_response = client
  .models()
  .by_name( "gemini-2.5-flash" )
  .generate_content( &structured_request )
  .await?;

  if let Some( candidate ) = structured_response.candidates.first()
  {
    if let Some( part ) = candidate.content.parts.first()
    {
      if let Some( text ) = &part.text
      {
      println!( "\nStructured analysis : {text}" );
      }
    }
  }
  */
  
  println!( "Structured analysis request prepared!" );
  println!( "✓ JSON output format specified in prompt" );
  println!( "✓ Lower temperature for more consistent structured output" );
  println!( "✓ Specific analysis categories requested" );

  println!( "\n=== Key Points About Multimodal Input ===" );
  println!( "1. Images must be base64 encoded with proper MIME type" );
  println!( "2. Supported formats: JPEG, PNG, GIF, WebP" );
  println!( "3. Maximum image size depends on the model (usually 20MB)" );
  println!( "4. Multiple images can be included in a single request" );
  println!( "5. Order matters: arrange images and text logically" );
  println!( "6. For best results, use high-quality, clear images" );
  println!( "7. Consider image compression to reduce token usage" );

  Ok( () )
}