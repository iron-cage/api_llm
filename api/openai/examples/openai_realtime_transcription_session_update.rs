//! Example of updating a transcription session configuration using the OpenAI API.
//!
//! Run with:
//! `cargo run --example realtime_transcription_session_update`
//!
//! Make sure you have set the `OPENAI_API_KEY` environment variable
//! or have a `secret/-secret.sh` file with the key.
//!
//! **IMPORTANT NOTE:** This example *assumes* the initial session was created
//! as a **transcription session** via the REST API endpoint
//! `/realtime/transcription_sessions`. The standard session creation used in
//! other examples (`/realtime/sessions`) will likely result in an error if you
//! send this event. The setup code here still creates a regular session for
//! simplicity of demonstration, but the server will likely reject the update event.

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  error ::OpenAIError,
  components ::realtime_shared::
  {
    RealtimeTranscriptionSessionCreateRequest,
    RealtimeClientEventTranscriptionSessionUpdate,
    RealtimeClientEvent,
    RealtimeServerEvent,
    RealtimeSessionInputAudioTranscription,
  },

};



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

  // 2. **WARNING:** Creating a standard session here. A real transcription update
  //    scenario requires creating a transcription session via the REST API first.
  tracing ::info!( "Building standard session request (for demo purposes)..." );
  let initial_request = RealtimeTranscriptionSessionCreateRequest::former()
  .input_audio_format( "pcm16" ) // Need input format configured
  .input_audio_transcription( RealtimeSessionInputAudioTranscription::former().model( "whisper-1".to_string() ).form() )
  .form();

  tracing ::info!( "Sending request to OpenAI API to create standard session..." );
  // 3. Call the API endpoint to get session details.
  let session = client.realtime().create_transcription_session( initial_request ).await?;

  tracing ::info!( "Creating Realtime WebSocket Session Client..." );
  let _token = session.client_secret.expect("Client secret").value;
  // 4. Establish the WebSocket connection using the session token.
  let session_client = client.realtime().connect_ws( &session.id ).await?;

  // --- Prepare the transcription session update ---
  let new_model = "gpt-4o-transcribe";
  let new_language = "es"; // Change language to Spanish

  // 5. Prepare the update payload using RealtimeTranscriptionSessionCreateRequest struct.
  let transcription_update_payload = RealtimeTranscriptionSessionCreateRequest::former()
  .input_audio_transcription( // Update transcription settings
    RealtimeSessionInputAudioTranscription::former()
    .model( new_model )
    .language( new_language )
    // .prompt("Focus on technical terms.") // Example : Add a prompt
    .form()
  )
  // Example : update turn detection settings for the transcription session
  // .turn_detection(RealtimeSessionTurnDetection::former().silence_duration_ms(700).form())
  .form();

  // 6. Prepare the client event to update the transcription session.
  let tsu_update = RealtimeClientEventTranscriptionSessionUpdate::former()
  .session( transcription_update_payload ) // Embed the update payload
  .form();

  tracing ::info!( language = new_language, "Sending transcription_session.update event..." );
  // 7. Send the transcription session update event over the WebSocket.
  session_client.send_event( RealtimeClientEvent::TranscriptionSessionUpdate( tsu_update ) ).await?;

  // 8. Loop to read responses. We expect either TranscriptionSessionUpdated (if context was correct)
  //    or an Error (more likely given the standard session setup).
  tracing ::info!( "Waiting for transcription_session.updated or error confirmation..." );
  let mut confirmation_received = false; // Tracks if TranscriptionSessionUpdated was received
  let mut error_received = false; // Tracks if a relevant error was received

  loop
  {
    let response = session_client.recv_event().await.map( Some );
    #[ allow( unreachable_patterns ) ]
    match response
    {
      Ok( Some( event ) ) =>
      {
        match event
        {
          RealtimeServerEvent::TranscriptionSessionUpdated( updated_event ) =>
          {
            println!( "\n--- Transcription Session Updated Confirmation Received ---" );
            println!( "{updated_event:?}" );
            // Verify the updated fields if possible
            let updated_session = updated_event.session;
            let lang_matches = updated_session.input_audio_transcription.as_ref()
            .map( | t | t.language.as_deref() == Some( new_language ) )
            .unwrap_or( false );
            let include_matches =updated_session.input_audio_transcription.as_ref()
            .and_then( | iat | iat.model.as_deref() ) == Some( &new_model );

            if lang_matches && include_matches
            {
              println!( "Successfully received transcription_session.updated confirmation with expected changes." );
              confirmation_received = true;
              break; // Break after receiving confirmation
            }
            else
            {
              eprintln!( "Received transcription_session.updated confirmation, but changes did not match request fully (Lang match : {}, Include match : {}).", lang_matches, include_matches);
              break; // Break, but don't confirm success
            }
          }
          RealtimeServerEvent::Error( error_event ) =>
          {
            eprintln!( "\n--- Received Server Error Event ---" );
            println!( "{error_event:?}" );
            // Check if the error is likely related to sending a transcription update to a non-transcription session
            if error_event.error.message.contains("transcription session") || error_event.error.r#type.contains("invalid_request")
            {
              eprintln!( "Received expected error due to sending transcription update to a standard session." );
              error_received = true;
              break; // Treat expected error as completion for this demo
            }
          }
          // Handle SessionCreated (initial event after connection)
          RealtimeServerEvent::SessionCreated( session_info ) =>
          {
            println!( "\n--- Received Session Info (Initial) ---" );
            println!( "{session_info:?}" );
          }
          _ => { println!( "\n--- Received Other Event (while waiting for transcription update confirmation) --- \n{event:?}" ); }
        }
      }
      Ok( None ) =>
      {
        println!( "\nWebSocket connection closed by server." );
        break; // Exit loop if connection closed
      }
      Err( e ) =>
      {
        eprintln!( "\nError reading from WebSocket : {:?}", e );
        return Err( e.into() ); // Propagate the error
      }
    }
  }

  // For this specific example, receiving the *expected error* is also considered "working as intended".
  if !confirmation_received && !error_received
  {
    eprintln!( "Loop finished without receiving transcription_session.updated or an expected error." );
    return Err( OpenAIError::WsInvalidMessage( "Did not receive expected transcription update confirmation or relevant error".to_string() ).into() );
  }

if confirmation_received
{
    println!( "Successfully updated transcription session (Note : This implies the initial session WAS a transcription session)." );
  } else if error_received
  {
    println!( "Received expected error when trying to update a standard session as if it were a transcription session. This demonstrates the event sending mechanism." );
  }


  Ok( () )
}
