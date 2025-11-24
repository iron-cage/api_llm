//! Test for `document_analyzer_vision` example URL issue
//!
//! This test reproduces the issue where the document analyzer fails
//! when trying to access certain image URLs that return HTTP 400 errors.

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
async fn test_document_analyzer_vision_new_url_works()
{
  // Load secret using the comprehensive fallback system
  let secret = api_openai::secret::Secret::load_with_fallbacks("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY should be available in workspace secrets");

  let env = api_openai::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
    .expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // Use base64-encoded image data to avoid external URL dependencies
  // This is a 1x1 red pixel PNG - minimal valid image for testing
  let base64_image = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";

  println!("🔍 Testing base64-encoded image input");

  let request = CreateResponseRequest::former()
    .model("gpt-5.1-chat-latest".to_string())
    .input(
      ResponseInput::Items(
        vec![
          InputItem::Message(
            InputMessage::former()
              .role("user")
              .content(
                vec![
                  InputContentPart::Text(
                    InputText::former()
                      .text("Please analyze this image.".to_string())
                      .form()
                  ),
                  InputContentPart::Image(
                    InputImage::former()
                      .image_url(base64_image.to_string())
                      .detail("high")
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

  let result = client.responses().create(request).await;

  match result
  {
    Ok(_) =>
    {
      println!("✅ Base64 image input works correctly");
    },
    Err(e) =>
    {
      let error_msg = format!("{e:?}");
      panic!("Base64 image input failed unexpectedly : {error_msg}");
    }
  }
}

#[ tokio::test ]
async fn test_document_analyzer_vision_working_url()
{
  // Load secret using the comprehensive fallback system
  let secret = api_openai::secret::Secret::load_with_fallbacks("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY should be available in workspace secrets");

  let env = api_openai::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
    .expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // Use base64-encoded image data to avoid external URL dependencies
  // This is a 1x1 red pixel PNG - minimal valid image for testing
  let base64_image = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";

  println!("🔍 Testing base64-encoded image input");

  let request = CreateResponseRequest::former()
    .model("gpt-5.1-chat-latest".to_string())
    .input(
      ResponseInput::Items(
        vec![
          InputItem::Message(
            InputMessage::former()
              .role("user")
              .content(
                vec![
                  InputContentPart::Text(
                    InputText::former()
                      .text("Please analyze this image.".to_string())
                      .form()
                  ),
                  InputContentPart::Image(
                    InputImage::former()
                      .image_url(base64_image.to_string())
                      .detail("high")
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

  let result = client.responses().create(request).await;

  match result
  {
    Ok(response) =>
    {
      println!("✅ Base64 image processed successfully");

      // Verify we got some analysis content
      assert!(!response.output.is_empty(), "Should have received analysis output");
      println!("✅ Analysis content received");
    },
    Err(e) =>
    {
      panic!("❌ Base64 image input should not fail : {e:?}");
    }
  }
}