//! Audio processing extension for OllamaClient.

#[ cfg( feature = "audio_processing" ) ]
use crate::client::OllamaClient;
#[ cfg( feature = "audio_processing" ) ]
use error_tools::untyped::{ format_err, Result as OllamaResult };

/// Extension to OllamaClient for audio processing
#[ cfg( feature = "audio_processing" ) ]
impl OllamaClient
{
  /// Configure audio processing settings
  #[ inline ]
  pub fn with_audio_config( mut self, config : crate::audio::AudioProcessingConfig ) -> Self
  {
    self.audio_config = Some( config );
    self
  }

  /// Get current audio processing configuration
  #[ inline ]
  pub fn audio_config( &self ) -> Option< &crate::audio::AudioProcessingConfig >
  {
    self.audio_config.as_ref()
  }

  /// Convert speech to text using Ollama models
  ///
  /// This method processes audio input and returns transcribed text.
  ///
  /// # Arguments
  /// * `request` - Speech-to-text request with audio data
  ///
  /// # Returns
  /// * `Ok(SpeechToTextResponse)` - Transcribed text with metadata
  /// * `Err(OllamaError)` - Audio processing or API error
  ///
  /// # Errors
  /// Returns error if:
  /// - Audio format is unsupported
  /// - Audio data is invalid or corrupted
  /// - Model doesn't support audio processing
  /// - Network request fails
  /// - API returns an error
  #[ inline ]
  pub async fn speech_to_text( &mut self, request : crate::audio::SpeechToTextRequest ) -> OllamaResult< crate::audio::SpeechToTextResponse >
  {
    // Validate audio format
    if !self.is_audio_format_supported( &request.format )
    {
      return Err( format_err!( "Audio format {:?} is not supported", request.format ) );
    }

    // Check circuit breaker
    #[ cfg( feature = "circuit_breaker" ) ]
    {
      if let Some( ref circuit_breaker ) = &self.circuit_breaker
      {
        if !circuit_breaker.can_execute()
        {
          return Err( format_err!( "Circuit breaker is open" ) );
        }
      }
    }

    // Check rate limiting
    #[ cfg( feature = "rate_limiting" ) ]
    {
      if let Some( ref rate_limiter ) = &self.rate_limiter
      {
        if !rate_limiter.should_allow_request()
        {
          return Err( format_err!( "Rate limit exceeded. Please try again later." ) );
        }
      }
    }

    let start_time = std::time::Instant::now();
    let request_id = format!( "req-{}", std::time::SystemTime::now().duration_since( std::time::UNIX_EPOCH ).unwrap().as_millis() );

    // Store audio data length before moving
    let audio_data_len = request.audio_data.len();

    // Build request URL
    let url = format!( "{}/api/audio/transcribe", self.base_url );

    // Prepare multipart form data
    let form = reqwest::multipart::Form::new()
      .text( "model", request.model.clone() )
      .part( "audio", reqwest::multipart::Part::bytes( request.audio_data )
        .file_name( format!( "audio.{}", request.format.file_extension() ) )
        .mime_str( request.format.mime_type() )
        .map_err( | e | format_err!( "Failed to set MIME type : {}", e ) )?
      );

    // Make request
    let response = self.client
      .post( &url )
      .multipart( form )
      .timeout( self.timeout )
      .send()
      .await;

    let processing_time_ms = start_time.elapsed().as_millis() as u64;

    match response
    {
      Ok( resp ) =>
      {
        let status = resp.status();
        if status.is_success()
        {
          let response_text = resp.text().await
            .map_err( | e | format_err!( "Failed to read response : {}", e ) )?;

          let transcription : serde_json::Value = serde_json::from_str( &response_text )
            .map_err( | e | format_err!( "Failed to parse response : {}", e ) )?;

          // Record success
          #[ cfg( feature = "circuit_breaker" ) ]
          {
            if let Some( ref circuit_breaker ) = &self.circuit_breaker
            {
              circuit_breaker.record_success();
            }
          }

          #[ cfg( feature = "general_diagnostics" ) ]
          {
            if let Some( ref diagnostics ) = &self.diagnostics_collector
            {
              diagnostics.track_request_success( &request_id, audio_data_len );
            }
          }

          Ok( crate::audio::SpeechToTextResponse
          {
            text : transcription[ "text" ].as_str().unwrap_or( "" ).to_string(),
            confidence : transcription[ "confidence" ].as_f64(),
            language : transcription[ "language" ].as_str().map( | s | s.to_string() ),
            duration : Some( ( processing_time_ms as f64 ) / 1000.0 ),
            metadata : None,
          })
        }
        else
        {
          // Record failure
          #[ cfg( feature = "circuit_breaker" ) ]
          {
            if let Some( ref circuit_breaker ) = &self.circuit_breaker
            {
              circuit_breaker.record_failure();
            }
          }

          let error_text = resp.text().await.unwrap_or_else( | _ | "Unknown error".to_string() );

          #[ cfg( feature = "general_diagnostics" ) ]
          {
            if let Some( ref diagnostics ) = &self.diagnostics_collector
            {
              diagnostics.track_request_failure( &request_id, status.as_u16(), &error_text );
            }
          }

          Err( format_err!( "Speech-to-text failed : {}", error_text ) )
        }
      }
      Err( e ) =>
      {
        // Record failure
        #[ cfg( feature = "circuit_breaker" ) ]
        {
          if let Some( ref circuit_breaker ) = &self.circuit_breaker
          {
            circuit_breaker.record_failure();
          }
        }

        #[ cfg( feature = "general_diagnostics" ) ]
        {
          if let Some( ref diagnostics ) = &self.diagnostics_collector
          {
            diagnostics.track_request_failure( &request_id, 500, &e.to_string() );
          }
        }

        Err( format_err!( "Speech-to-text request failed : {}", e ) )
      }
    }
  }

  /// Convert text to speech using Ollama models
  ///
  /// This method generates audio from text input.
  ///
  /// # Arguments
  /// * `request` - Text-to-speech request with text and voice parameters
  ///
  /// # Returns
  /// * `Ok(TextToSpeechResponse)` - Generated audio data
  /// * `Err(OllamaError)` - Audio generation or API error
  ///
  /// # Errors
  /// Returns error if:
  /// - Text input is invalid or too long
  /// - Voice parameter is invalid
  /// - Model doesn't support TTS
  /// - Network request fails
  /// - API returns an error
  #[ inline ]
  pub async fn text_to_speech( &mut self, request : crate::audio::TextToSpeechRequest ) -> OllamaResult< crate::audio::TextToSpeechResponse >
  {
    // Check circuit breaker
    #[ cfg( feature = "circuit_breaker" ) ]
    {
      if let Some( ref circuit_breaker ) = &self.circuit_breaker
      {
        if !circuit_breaker.can_execute()
        {
          return Err( format_err!( "Circuit breaker is open" ) );
        }
      }
    }

    // Check rate limiting
    #[ cfg( feature = "rate_limiting" ) ]
    {
      if let Some( ref rate_limiter ) = &self.rate_limiter
      {
        if !rate_limiter.should_allow_request()
        {
          return Err( format_err!( "Rate limit exceeded. Please try again later." ) );
        }
      }
    }

    let start_time = std::time::Instant::now();
    let request_id = format!( "req-{}", std::time::SystemTime::now().duration_since( std::time::UNIX_EPOCH ).unwrap().as_millis() );

    // Build request URL
    let url = format!( "{}/api/audio/synthesize", self.base_url );

    // Build request body
    let request_body = serde_json::json!({
      "model": request.model,
      "text": request.text,
      "voice": request.voice,
      "format": request.format.file_extension(),
    });

    // Make request
    let response = self.client
      .post( &url )
      .json( &request_body )
      .timeout( self.timeout )
      .send()
      .await;

    let processing_time_ms = start_time.elapsed().as_millis() as u64;

    match response
    {
      Ok( resp ) =>
      {
        let status = resp.status();
        if status.is_success()
        {
          let audio_data = resp.bytes().await
            .map_err( | e | format_err!( "Failed to read audio data : {}", e ) )?;

          // Record success
          #[ cfg( feature = "circuit_breaker" ) ]
          {
            if let Some( ref circuit_breaker ) = &self.circuit_breaker
            {
              circuit_breaker.record_success();
            }
          }

          #[ cfg( feature = "general_diagnostics" ) ]
          {
            if let Some( ref diagnostics ) = &self.diagnostics_collector
            {
              diagnostics.track_request_success( &request_id, audio_data.len() );
            }
          }

          Ok( crate::audio::TextToSpeechResponse
          {
            audio_data : audio_data.to_vec(),
            format : request.format,
            duration : Some( ( processing_time_ms as f64 ) / 1000.0 ),
            sample_rate : Some( 24000 ), // Default, would come from API in real implementation
            metadata : None,
          })
        }
        else
        {
          // Record failure
          #[ cfg( feature = "circuit_breaker" ) ]
          {
            if let Some( ref circuit_breaker ) = &self.circuit_breaker
            {
              circuit_breaker.record_failure();
            }
          }

          let error_text = resp.text().await.unwrap_or_else( | _ | "Unknown error".to_string() );

          #[ cfg( feature = "general_diagnostics" ) ]
          {
            if let Some( ref diagnostics ) = &self.diagnostics_collector
            {
              diagnostics.track_request_failure( &request_id, status.as_u16(), &error_text );
            }
          }

          Err( format_err!( "Text-to-speech failed : {}", error_text ) )
        }
      }
      Err( e ) =>
      {
        // Record failure
        #[ cfg( feature = "circuit_breaker" ) ]
        {
          if let Some( ref circuit_breaker ) = &self.circuit_breaker
          {
            circuit_breaker.record_failure();
          }
        }

        #[ cfg( feature = "general_diagnostics" ) ]
        {
          if let Some( ref diagnostics ) = &self.diagnostics_collector
          {
            diagnostics.track_request_failure( &request_id, 500, &e.to_string() );
          }
        }

        Err( format_err!( "Text-to-speech request failed : {}", e ) )
      }
    }
  }

  /// Check if audio format is supported
  #[ inline ]
  fn is_audio_format_supported( &self, format : &crate::audio::AudioFormat ) -> bool
  {
    matches!( format,
      crate ::audio::AudioFormat::Mp3 |
      crate ::audio::AudioFormat::Wav |
      crate ::audio::AudioFormat::Ogg |
      crate ::audio::AudioFormat::Flac
    )
  }

  /// Process voice chat with integrated STT and TTS
  ///
  /// This is a convenience method that combines speech-to-text, chat, and text-to-speech
  /// into a single voice interaction.
  #[ inline ]
  pub async fn voice_chat( &mut self, audio_input : Vec< u8 >, format : crate::audio::AudioFormat, model : String ) -> OllamaResult< crate::audio::TextToSpeechResponse >
  {
    // Convert speech to text
    let stt_request = crate::audio::SpeechToTextRequest
    {
      model : model.clone(),
      audio_data : audio_input,
      format,
      language : None,
      options : None,
    };

    let transcription = self.speech_to_text( stt_request ).await?;

    // Send to chat
    #[ cfg( feature = "vision_support" ) ]
    let chat_request = crate::ChatRequest
    {
      model : model.clone(),
      messages : vec![
        crate ::ChatMessage
        {
          role : crate::MessageRole::User,
          content : transcription.text,
          images : None,
          #[ cfg( feature = "tool_calling" ) ]
          tool_calls : None,
        }
      ],
      stream : None,
      think : None,
      options : None,
      #[ cfg( feature = "tool_calling" ) ]
      tools : None,
      #[ cfg( feature = "tool_calling" ) ]
      tool_messages : None,
    };

    #[ cfg( not( feature = "vision_support" ) ) ]
    let chat_request = crate::ChatRequest
    {
      model : model.clone(),
      messages : vec![
        crate ::Message
        {
          role : "user".to_string(),
          content : transcription.text,
        }
      ],
      stream : None,
      options : None,
      #[ cfg( feature = "tool_calling" ) ]
      tools : None,
      #[ cfg( feature = "tool_calling" ) ]
      tool_messages : None,
    };

    let chat_response = self.chat( chat_request ).await?;

    // Convert response to speech
    let tts_request = crate::audio::TextToSpeechRequest
    {
      model,
      text : chat_response.message.content,
      voice : Some( "default".to_string() ),
      format : crate::audio::AudioFormat::Mp3,
      speed : None,
      options : None,
    };

    self.text_to_speech( tts_request ).await
  }
}
