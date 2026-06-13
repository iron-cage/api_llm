//! Automatic Speech Recognition ( ASR )
//!
//! Convert speech audio to text using ASR models.

use crate::
{
  error::Result,
  audio::{ Audio, AudioInput, TranscriptionResult },
  environment::HuggingFaceEnvironment,
};
use serde::{ Serialize, Deserialize };

/// ASR request
#[ derive( Debug, Serialize ) ]
struct AsrRequest
{
  /// Audio data ( base64 or URL )
  inputs : String,
}

/// ASR response
#[ derive( Debug, Deserialize ) ]
#[ serde( untagged ) ]
enum AsrResponse
{
  /// Single transcription
  Single( TranscriptionResult ),

  /// Wrapped transcription
  Wrapped { text : String },
}

impl< E > Audio< E >
where
  E : HuggingFaceEnvironment + crate::environment::EnvironmentInterface + Send + Sync + 'static + Clone,
{
  /// Transcribe audio to text using ASR model
  ///
  /// # Arguments
  ///
  /// * `audio` - Audio input ( bytes, base64, or URL )
  /// * `model` - Model identifier ( e.g., "openai/whisper-base" )
  ///
  /// # Returns
  ///
  /// Transcribed text
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
  /// let audio_data = fs::read( "speech.wav" )?;
  /// let input = AudioInput::from_bytes( audio_data );
  ///
  /// let result = audio.transcribe( input, "openai/whisper-base" ).await?;
  /// println!( "Transcription: {}", result );
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Errors
  ///
  /// Returns error if API request fails or response cannot be parsed
  #[ inline ]
  pub async fn transcribe(
  &self,
  audio : AudioInput,
  model : impl AsRef< str >
  ) -> Result< String >
  {
  let request = AsrRequest
  {
      inputs : audio.to_base64( ),
  };

  let endpoint = format!( "/models/{}", model.as_ref( ) );
  let url = self.client.environment.endpoint_url( &endpoint )?;

  let response : AsrResponse = self.client
      .post( url.as_str( ), &request )
      .await?;

  let text = match response
  {
      AsrResponse::Single( result ) => result.text,
      AsrResponse::Wrapped { text } => text,
  };

  Ok( text )
  }
}
