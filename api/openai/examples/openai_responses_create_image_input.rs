//! Example for creating a response with image input.
//!
//! This example demonstrates how to use the `responses().create()` method
//! with `ResponseInput::Items` containing both `input_text` and `input_image` content parts.
//!
//! Run with:
//! ```bash
//! cargo run --example responses_create_image_input
//! ```

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



#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{


  let secret = api_openai::secret::Secret::load_with_fallbacks("OPENAI_API_KEY")
    .expect("Failed to load OPENAI_API_KEY. Please set environment variable or add to workspace secrets file.");
  let env = api_openai::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string()).expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // Example image URL (replace with a real image URL or base64 encoded image)
  let image_url = "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/2560px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg".to_string();

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

  println!( "Sending request with image input..." );

  let response = client.responses().create( request ).await?;

  println!( "Response : {response:#?}" );

  Ok( () )
}