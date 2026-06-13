//! Integration tests for the `HuggingFace` providers API.
//!
//! Verifies that the client can connect to real `HuggingFace` endpoints and receive
//! valid responses for inference and embeddings providers.

#![ allow( clippy::missing_inline_in_public_items ) ]

mod inc;

use api_huggingface::
{
  Client,
  environment::HuggingFaceEnvironmentImpl,
  components::
  {
    input::InferenceParameters,
    models::Models,
  },
  secret::Secret,
};


#[ cfg( feature = "integration" ) ]
fn create_integration_client() -> Client< HuggingFaceEnvironmentImpl >
{
  let api_key = crate::inc::get_api_key_for_integration();
  let secret = Secret::new( api_key );
  let env = HuggingFaceEnvironmentImpl::build( secret, None )
    .expect( "Failed to build environment" );
  Client::build( env ).expect( "Failed to create client" )
}

#[ cfg( feature = "integration" ) ]
#[ tokio::test ]
async fn test_inference_provider_reachable()
{
  let client = create_integration_client();
  let model = Models::llama_3_3_70b_instruct();

  let response = client
    .inference()
    .create_with_parameters(
      "Reply with exactly one word: hello",
      &model,
      InferenceParameters::new()
        .with_max_new_tokens( 10 )
        .with_temperature( 0.1 ),
    )
    .await;

  match response
  {
    Ok( r ) =>
    {
      let text = r.extract_text_or_default( "" );
      assert!( !text.is_empty(), "Inference provider returned empty response" );
    },
    Err( e ) => panic!( "Inference provider request failed: {e}" ),
  }
}

