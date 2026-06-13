//! Audio Classification
//!
//! Classify audio into predefined categories.

use crate::
{
  error::Result,
  audio::{ Audio, AudioInput, AudioClassificationResult },
  environment::HuggingFaceEnvironment,
  components::input::BinaryClassificationInput,
};

impl< E > Audio< E >
where
  E : HuggingFaceEnvironment + crate::environment::EnvironmentInterface + Send + Sync + 'static + Clone,
{
  /// Classify audio using an audio classification model
  ///
  /// # Arguments
  ///
  /// * `audio` - Audio input (bytes, base64, or URL)
  /// * `model` - Model identifier (e.g., "superb/hubert-large-superb-er")
  ///
  /// # Returns
  ///
  /// Vector of classification results with labels and confidence scores
  ///
  /// # Example
  ///
  /// ```no_run
  /// # use api_huggingface::{ Client, environment::HuggingFaceEnvironmentImpl, secret::Secret };
  /// # use api_huggingface::audio::AudioInput;
  /// # use std::fs;
  /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// # let api_key = Secret::new( "test".to_string() );
  /// # let env = HuggingFaceEnvironmentImpl::build( api_key, None )?;
  /// # let client = Client::build( env )?;
  /// # let audio = client.audio();
  /// let audio_data = fs::read( "sound.wav" )?;
  /// let input = AudioInput::from_bytes( audio_data );
  ///
  /// let results = audio.classify_audio( input, "superb/hubert-large-superb-er" ).await?;
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
  pub async fn classify_audio(
  &self,
  audio : AudioInput,
  model : impl AsRef< str >
  ) -> Result< Vec< AudioClassificationResult > >
  {
  let request = BinaryClassificationInput
  {
      inputs : audio.to_base64(),
  };

  let endpoint = format!( "/models/{}", model.as_ref() );
  let url = self.client.environment.endpoint_url( &endpoint )?;

  let results : Vec< AudioClassificationResult > = self.client
      .post( url.as_str(), &request )
      .await?;

  Ok( results )
  }
}
