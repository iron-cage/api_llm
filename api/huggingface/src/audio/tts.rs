//! Text-to-Speech (TTS)
//!
//! Generate speech audio from text using TTS models.

use crate::
{
  error::Result,
  audio::{ Audio, SpeechGenerationResult },
  environment::HuggingFaceEnvironment,
};
use serde::Serialize;

/// TTS request
#[ derive( Debug, Serialize ) ]
struct TtsRequest
{
  /// Input text to synthesize
  inputs : String,
}

impl< E > Audio< E >
where
  E : HuggingFaceEnvironment + crate::environment::EnvironmentInterface + Send + Sync + 'static + Clone,
{
  /// Generate speech from text using TTS model
  ///
  /// # Arguments
  ///
  /// * `text` - Text to convert to speech
  /// * `model` - Model identifier (e.g., "espnet/kan-bayashi_ljspeech_vits")
  ///
  /// # Returns
  ///
  /// Generated audio data
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
  /// let result = audio.generate_speech(
  ///   "Hello, how are you?",
  ///   "espnet/kan-bayashi_ljspeech_vits"
  /// ).await?;
  ///
  /// // Save to file
  /// fs::write( "output.wav", &result.audio_data )?;
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Errors
  ///
  /// Returns error if API request fails or response cannot be parsed
  #[ inline ]
  pub async fn generate_speech(
  &self,
  text : impl AsRef< str >,
  model : impl AsRef< str >
  ) -> Result< SpeechGenerationResult >
  {
  let request = TtsRequest
  {
      inputs : text.as_ref().to_string(),
  };

  let endpoint = format!( "/models/{}", model.as_ref() );
  let url = self.client.environment.endpoint_url( &endpoint )?;

  // TTS models return raw audio bytes
  let audio_data : Vec< u8 > = self.client
      .post_bytes( url.as_str(), &request )
      .await?;

  Ok( SpeechGenerationResult
  {
      audio_data,
      sample_rate : None, // Model-specific, not returned by API
      format : None,      // Typically WAV but not specified
  } )
  }
}
