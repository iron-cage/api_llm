//! Audio-to-Audio Transformation
//!
//! Transform audio (noise reduction, enhancement, etc.)

use crate::
{
  error::Result,
  audio::{ Audio, AudioInput, AudioTransformResult },
  environment::HuggingFaceEnvironment,
};
use serde::Serialize;

/// Audio-to-audio transformation request
#[ derive( Debug, Serialize ) ]
struct AudioToAudioRequest
{
  /// Audio data (base64 or URL)
  inputs : String,
}

impl< E > Audio< E >
where
  E : HuggingFaceEnvironment + crate::environment::EnvironmentInterface + Send + Sync + 'static + Clone,
{
  /// Transform audio using audio-to-audio model
  ///
  /// # Arguments
  ///
  /// * `audio` - Audio input (bytes, base64, or URL)
  /// * `model` - Model identifier (e.g., "facebook/hdemucs-mmi")
  ///
  /// # Returns
  ///
  /// Transformed audio data
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
  /// let audio_data = fs::read( "noisy_audio.wav" )?;
  /// let input = AudioInput::from_bytes( audio_data );
  ///
  /// let result = audio.transform_audio( input, "facebook/hdemucs-mmi" ).await?;
  ///
  /// // Save cleaned audio
  /// fs::write( "clean_audio.wav", &result.audio_data )?;
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Errors
  ///
  /// Returns error if API request fails or response cannot be parsed
  #[ inline ]
  pub async fn transform_audio(
  &self,
  audio : AudioInput,
  model : impl AsRef< str >
  ) -> Result< AudioTransformResult >
  {
  let request = AudioToAudioRequest
  {
      inputs : audio.to_base64(),
  };

  let endpoint = format!( "/models/{}", model.as_ref() );
  let url = self.client.environment.endpoint_url( &endpoint )?;

  // Audio transformation models return raw audio bytes
  let audio_data : Vec< u8 > = self.client
      .post_bytes( url.as_str(), &request )
      .await?;

  Ok( AudioTransformResult
  {
      audio_data,
      sample_rate : None, // Model-specific, not returned by API
      format : None,      // Typically WAV but not specified
  } )
  }
}
