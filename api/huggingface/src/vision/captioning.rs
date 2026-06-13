//! Image Captioning (Image-to-Text)
//!
//! Generate text descriptions and captions for images.

use crate::
{
  error::Result,
  vision::{ Vision, ImageInput, CaptionResult },
  environment::HuggingFaceEnvironment,
};
use serde::{ Serialize, Deserialize };

/// Image captioning request
#[ derive( Debug, Serialize ) ]
struct CaptioningRequest
{
  /// Image data (base64 or URL)
  inputs : String,
}

/// Image captioning response
#[ derive( Debug, Deserialize ) ]
#[ serde( untagged ) ]
enum CaptioningResponse
{
  /// Single caption
  Single( CaptionResult ),

  /// Multiple captions
  Multiple( Vec< CaptionResult > ),
}

impl< E > Vision< E >
where
  E : HuggingFaceEnvironment + crate::environment::EnvironmentInterface + Send + Sync + 'static + Clone,
{
  /// Generate a caption for an image
  ///
  /// # Arguments
  ///
  /// * `image` - Image input (bytes, base64, or URL)
  /// * `model` - Model identifier (e.g., "Salesforce/blip-image-captioning-base")
  ///
  /// # Returns
  ///
  /// Generated caption text
  ///
  /// # Example
  ///
  /// ```no_run
  /// # use api_huggingface::{ Client, environment::HuggingFaceEnvironmentImpl, secret::Secret };
  /// # use api_huggingface::vision::ImageInput;
  /// # use std::fs;
  /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// # let api_key = Secret::new( "test".to_string() );
  /// # let env = HuggingFaceEnvironmentImpl::build( api_key, None )?;
  /// # let client = Client::build( env )?;
  /// # let vision = client.vision();
  /// let image_data = fs::read( "sunset.jpg" )?;
  /// let input = ImageInput::from_bytes( image_data );
  ///
  /// let caption = vision.caption_image( input, "Salesforce/blip-image-captioning-base" ).await?;
  /// println!( "Caption: {}", caption );
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Errors
  ///
  /// Returns error if API request fails or response cannot be parsed
  #[ inline ]
  pub async fn caption_image(
  &self,
  image : ImageInput,
  model : impl AsRef< str >
  ) -> Result< String >
  {
  let request = CaptioningRequest
  {
      inputs : image.to_base64(),
  };

  let endpoint = format!( "/models/{}", model.as_ref() );
  let url = self.client.environment.endpoint_url( &endpoint )?;

  let response : CaptioningResponse = self.client
      .post( url.as_str(), &request )
      .await?;

  let caption = match response
  {
      CaptioningResponse::Single( result ) => result.generated_text,
      CaptioningResponse::Multiple( results ) =>
      {
  results
          .into_iter()
          .next()
          .map( | r | r.generated_text )
          .unwrap_or_default()
      }
  };

  Ok( caption )
  }
}
