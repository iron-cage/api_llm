//! Image Classification
//!
//! Classify images into predefined categories.

use crate::
{
  error::Result,
  vision::{ Vision, ImageInput, ClassificationResult },
  environment::HuggingFaceEnvironment,
};
use serde::{ Serialize, Deserialize };

/// Image classification request
#[ derive( Debug, Serialize ) ]
struct ClassificationRequest
{
  /// Image data (base64 or URL)
  inputs : String,
}

/// Image classification response
#[ derive( Debug, Deserialize ) ]
#[ serde( untagged ) ]
enum ClassificationResponse
{
  /// Single result
  Single( Vec< ClassificationResult > ),

  /// Batch results
  Batch( Vec< Vec< ClassificationResult > > ),
}

impl< E > Vision< E >
where
  E : HuggingFaceEnvironment + crate::environment::EnvironmentInterface + Send + Sync + 'static + Clone,
{
  /// Classify an image using a vision model
  ///
  /// # Arguments
  ///
  /// * `image` - Image input (bytes, base64, or URL)
  /// * `model` - Model identifier (e.g., "google/vit-base-patch16-224")
  ///
  /// # Returns
  ///
  /// Vector of classification results with labels and confidence scores
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
  /// let image_data = fs::read( "cat.jpg" )?;
  /// let input = ImageInput::from_bytes( image_data );
  ///
  /// let results = vision.classify_image( input, "google/vit-base-patch16-224" ).await?;
  ///
  /// for result in results
  /// {
  ///   println!( "{}: {:.2}%", result.label, result.score * 100.0 );
  /// }
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Errors
  ///
  /// Returns error if API request fails or response cannot be parsed
  #[ inline ]
  pub async fn classify_image(
  &self,
  image : ImageInput,
  model : impl AsRef< str >
  ) -> Result< Vec< ClassificationResult > >
  {
  let request = ClassificationRequest
  {
      inputs : image.to_base64(),
  };

  let endpoint = format!( "/models/{}", model.as_ref() );
  let url = self.client.environment.endpoint_url( &endpoint )?;

  let response : ClassificationResponse = self.client
      .post( url.as_str(), &request )
      .await?;

  match response
  {
      ClassificationResponse::Single( results ) => Ok( results ),
      ClassificationResponse::Batch( batch ) =>
      {
  // Return first batch result
  Ok( batch.into_iter().next().unwrap_or_default() )
      }
  }
  }
}
