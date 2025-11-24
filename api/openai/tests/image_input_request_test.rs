//! Test for image input request structure
//!
//! This test reproduces the issue where the image input request fails
//! due to incorrect request structure format.

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput },
    input ::{ InputItem, InputMessage, InputContentPart, InputText, InputImage },
  },
};

#[ tokio::test ]
async fn test_image_input_request_structure_failure()
{
  // This test should initially fail, demonstrating the issue
  // Load secret from workspace (should work after previous fix)
  let secret = api_openai::secret::Secret::load_with_fallbacks("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY should be available in workspace secrets");

  let env = api_openai::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
    .expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // Use base64-encoded image data to avoid external URL dependencies
  // This is a 1x1 red pixel PNG - minimal valid image for testing
  let image_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==".to_string();

  // Build request that should work correctly
  let request = CreateResponseRequest::former()
    .model( "gpt-5.1-chat-latest".to_string() )
    .input
    (
      ResponseInput::Items
      (
        vec!
        [
          InputItem::Message
          (
            InputMessage::former()
            .role( "user" )
            .content
            (
              vec!
              [
                InputContentPart::Text
                (
                  InputText::former()
                  .text( "What is in this image?".to_string() )
                  .form()
                ),
                InputContentPart::Image
                (
                  InputImage::former()
                  .image_url( image_url )
                  .detail( "high" )
                  .form()
                ),
              ]
            )
            .form()
          ),
        ]
      )
    )
    .form();

  // This should succeed without 400 Bad Request errors about invalid input[0] value
  let result = client.responses().create( request ).await;

  match result
  {
    Ok(response) =>
    {
      // Verify we got a valid response with content
      assert!( !response.output.is_empty(), "Response should contain output" );
      println!( "✅ Image input request successful!" );

      // Verify the response contains reasonable content
      // Using minimal 1x1 pixel image, so just verify we got a non-empty response
      let output_text = format!( "{:?}", response.output );
      assert!( !output_text.is_empty(), "Response should contain output content" );

      println!( "✅ Image input request structure validated successfully" );
    },
    Err(e) =>
    {
      panic!("Image input request failed : {e:?}");
    }
  }
}