//! Example of appending audio to the input buffer using the OpenAI API.
#![ allow( clippy::doc_markdown ) ]
//!
//! Run with:
//! `cargo run --example realtime_input_audio_buffer_append`
//!
//! Make sure you have set the `OPENAI_API_KEY` environment variable
//! or have a `secret/-secret.sh` file with the key.
//!
//! **NOTE:** This event does not have a direct confirmation server event like
//! `input_audio_buffer.appended`. Confirmation is implicit or through subsequent
//! events like VAD detection or transcription results. This example sends the
//! data and waits briefly for any follow-up events.

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  components ::realtime_shared::
  {
    RealtimeClientEvent,
    RealtimeSessionCreateRequest,
    RealtimeClientEventInputAudioBufferAppend,
  },

};


use base64::{ engine::general_purpose::STANDARD as base64_engine, Engine as _ }; // For base64 encoding

#[ tokio::main( flavor = "current_thread" ) ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  // Setup tracing for logging
  tracing_subscriber::fmt::init();

  // Load environment variables

  // 1. Create a new OpenAI client.
  tracing ::info!( "Initializing client..." );
  let secret = api_openai::secret::Secret::load_with_fallbacks( "OPENAI_API_KEY" )
    .expect( "Failed to load OPENAI_API_KEY. Please set environment variable or add to workspace secrets file." );
  let env = api_openai::environment::OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    api_openai::environment::OpenAIRecommended::base_url().to_string(),
    api_openai::environment::OpenAIRecommended::realtime_base_url().to_string(),
  ).expect( "Failed to create environment" );
  let client = Client::build( env ).expect( "Failed to create client" );

  // 2. Create the request payload to initiate the session, configuring audio input.
  tracing ::info!( "Building realtime session request..." );
  let request = RealtimeSessionCreateRequest::former()
  .model( "gpt-4o-realtime-preview".to_string() )
  .input_audio_format( "pcm16" ) // Specify the format of the audio we'll send
  // Optional : Configure transcription if you want to see results
  // .input_audio_transcription( RealtimeSessionInputAudioTranscription::former().model( "whisper-1" ).form() )
  // Optional : Configure VAD
  // .turn_detection( RealtimeSessionTurnDetection::former().r#type( "server_vad" ).form() )
  .temperature( 0.7 )
  .form();

  tracing ::info!( "Sending request to OpenAI API to create session..." );
  // 3. Call the API endpoint to get session details.
  let session = client.realtime().create_session( request ).await?;

  tracing ::info!( "Creating Realtime WebSocket Session Client..." );
  let _ = session.client_secret.value;
  // 4. Establish the WebSocket connection using the session token.
  let session_client = client.realtime().connect_ws( &session.id ).await?;

  // --- Prepare Dummy Audio Data ---
  let dummy_audio_bytes = include_bytes!("data/example.wav");
  let audio_base64 = base64_engine.encode( dummy_audio_bytes );

  // 5. Prepare the client event to append the audio data.
  let iaba_append = RealtimeClientEventInputAudioBufferAppend::former()
  .audio( audio_base64 ) // Provide the base64 encoded audio
  .form();

  tracing ::info!( "Sending input_audio_buffer.append event..." );
  // 6. Send the append event over the WebSocket.
  session_client.send_event( RealtimeClientEvent::InputAudioBufferAppend( iaba_append ) ).await?;
  tracing ::info!( "Audio append event sent." );

  // 7. Wait briefly and read any subsequent events (like VAD or transcription).
  //    There's no direct 'appended' confirmation.
  tracing ::info!( "Waiting briefly for any subsequent events (no direct confirmation expected)..." );
  let wait_duration = tokio::time::Duration::from_secs( 2 ); // Wait for 2 seconds
  let start_time = tokio::time::Instant::now();
  loop
  {
    // Check timeout first
if start_time.elapsed() > wait_duration
{
      println!( "\nWait duration elapsed. No specific confirmation event for append." );
      break;
    }

    // Try reading with a small timeout to avoid blocking forever if nothing comes
    let read_timeout = tokio::time::Duration::from_millis( 100 );
    match tokio::time::timeout( read_timeout, session_client.recv_event() ).await
    {
      Ok( Ok( event ) ) =>
      {
        println!( "\n--- Received Subsequent Event ---" );
        println!( "{event:?}" );
        // Depending on session config, you might see:
        // - InputAudioBufferSpeechStarted/Stopped
        // - ConversationItemInputAudioTranscriptionDelta/Completed
        // etc.
      }
      Ok( Err( e ) ) =>
      {
        eprintln!( "\nError reading from WebSocket : {e:?}" );
        return Err( e.into() );
      }
      Err( _ ) =>
      {
        // Timeout elapsed for this read attempt, continue checking overall wait duration
      }
    }
  }

  // Since there's no direct confirmation, we usually consider the send successful if no error occurred.
  println!( "Successfully sent input_audio_buffer.append event." );
  Ok( () )
}
