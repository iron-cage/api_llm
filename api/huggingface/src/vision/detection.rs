//! Object Detection
//!
//! Detect and locate objects within images.

use crate::
{
  error::Result,
  vision::{ Vision, ImageInput, DetectionResult },
  environment::HuggingFaceEnvironment,
};
use serde::Serialize;

/// Object detection request
#[ derive( Debug, Serialize ) ]
struct DetectionRequest
{
  /// Image data (base64 or URL)
  inputs : String,
}

impl< E > Vision< E >
where
  E : HuggingFaceEnvironment + crate::environment::EnvironmentInterface + Send + Sync + 'static + Clone,
{
  /// Detect objects in an image
  ///
  /// # Arguments
  ///
  /// * `image` - Image input (bytes, base64, or URL)
  /// * `model` - Model identifier (e.g., "facebook/detr-resnet-50")
  ///
  /// # Returns
  ///
  /// Vector of detected objects with labels, scores, and bounding boxes
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
  /// let image_data = fs::read( "street.jpg" )?;
  /// let input = ImageInput::from_bytes( image_data );
  ///
  /// let results = vision.detect_objects( input, "facebook/detr-resnet-50" ).await?;
  ///
  /// for detection in results
  /// {
  ///   println!(
  ///     "{}: {:.2}% at ({}, {}) - ({}, {})",
  ///     detection.label,
  ///     detection.score * 100.0,
  ///     detection.box_coords.xmin,
  ///     detection.box_coords.ymin,
  ///     detection.box_coords.xmax,
  ///     detection.box_coords.ymax
  ///   );
  /// }
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Errors
  ///
  /// Returns error if API request fails or response cannot be parsed
  #[ inline ]
  pub async fn detect_objects(
  &self,
  image : ImageInput,
  model : impl AsRef< str >
  ) -> Result< Vec< DetectionResult > >
  {
  let request = DetectionRequest
  {
      inputs : image.to_base64(),
  };

  let endpoint = format!( "/models/{}", model.as_ref() );
  let url = self.client.environment.endpoint_url( &endpoint )?;

  let results : Vec< DetectionResult > = self.client
      .post( url.as_str(), &request )
      .await?;

  Ok( results )
  }
}
