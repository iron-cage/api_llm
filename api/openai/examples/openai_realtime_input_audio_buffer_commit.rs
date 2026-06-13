//! Example of committing the input audio buffer using the OpenAI API.
//!
//! Run with:
//! `cargo run --example realtime_input_audio_buffer_commit`
//!
//! Make sure you have set the `OPENAI_API_KEY` environment variable
//! or have a `secret/-secret.sh` file with the key.
//!
//! **NOTE:** This event is typically used when *not* using server-side VAD
//! (`turn_detection`) or when `turn_detection.create_response` is false.
//! Committing signals the end of user audio input and creates a user message item.

use api_openai::ClientApiAccessors;
#[ allow( unused_imports ) ]
use api_openai::
{
  Client,
  error ::OpenAIError,
  components ::realtime_shared::
  {
    RealtimeSessionCreateRequest,
    RealtimeClientEventInputAudioBufferAppend,
    RealtimeClientEventInputAudioBufferCommit,
    RealtimeClientEvent,
    RealtimeServerEvent,
    RealtimeSessionTurnDetection,
    RealtimeSessionInputAudioTranscription,
  },
  components ::common::ModelIds,
};


use base64::{ engine::general_purpose::STANDARD as base64_engine, Engine as _ }; // For base64 encoding
use std::sync::{ Arc, Mutex };
use tokio::time::{ sleep, Duration }; // For adding delays
use std::io::{ Write, stdout }; // For flushing print output

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

  // 2. Create the request payload to initiate the session.
  tracing ::info!( "Building realtime session request..." );
  let request = RealtimeSessionCreateRequest::former()
  .model( "gpt-4o-realtime-preview".to_string() )
  .input_audio_format( "pcm16" )
  .turn_detection
  (
    RealtimeSessionTurnDetection::former()
    .r#type( "semantic_vad" )
    .create_response( false )
    .interrupt_response( true )
    .form()
  )
  .input_audio_transcription( RealtimeSessionInputAudioTranscription::former().model( "whisper-1" ).form() )
  .temperature( 0.7 )
  .form();

  tracing ::info!( "Sending request to OpenAI API to create session..." );
  // 3. Call the API endpoint to get session details.
  let session = client.realtime().create_session( request ).await?;
  tracing ::info!( session_id = %session.id, "Session created." );


  tracing ::info!( "Creating Realtime WebSocket Session Client..." );
  let _token = session.client_secret.value;
  // 4. Establish the WebSocket connection using the session token.
  let session_client = client.realtime().connect_ws( &session.id ).await?;
  tracing ::info!( "WebSocket client connected." );


  // --- Append some audio data first ---
  let dummy_audio_bytes = include_bytes!("data/example.wav");
  let audio_base64 = base64_engine.encode( &dummy_audio_bytes );

  let append_event = RealtimeClientEventInputAudioBufferAppend::former()
  .audio( audio_base64 )
  .form();
  tracing ::info!( "Sending input_audio_buffer.append event..." );
  session_client.send_event( RealtimeClientEvent::InputAudioBufferAppend( append_event ) ).await?;

  // Allow a moment for the server to process the append.
  tracing ::info!( "Waiting after append..." );
  sleep( Duration::from_millis( 3000 ) ).await;
  tracing ::info!( "Audio append sent and waited." );


  // 5. Prepare the client event to commit the audio buffer.
  let client_event_id = "commit-example-id";
  let commit_event = RealtimeClientEventInputAudioBufferCommit::former()
  .event_id( client_event_id ) // Use the dynamic client event ID
  .form();

  tracing ::info!( event_id = %client_event_id, "Sending input_audio_buffer.commit event..." );
  // 6. Send the commit event over the WebSocket.
  session_client.send_event( RealtimeClientEvent::InputAudioBufferCommit( commit_event ) ).await?;


  // 7. Loop to read responses, looking for commit confirmation AND the resulting user message creation.
  tracing ::info!( "Waiting for input_audio_buffer.committed and conversation.item.created confirmation..." );
  let mut commit_confirmed = false;
  let expected_item_id_from_commit = Arc::new( Mutex::new( None::< String > ) ); // Store ID received in commit event
  let expected_item_id_clone = expected_item_id_from_commit.clone();
  let mut item_created_confirmed = false; // Track item creation confirmation

  // Set a timeout for this loop to prevent infinite waiting
  let loop_timeout = Duration::from_secs( 15 ); // Increased timeout slightly
  let loop_start = tokio::time::Instant::now();

  loop
  {
    // Check overall loop timeout
    if loop_start.elapsed() > loop_timeout
    {
      eprintln!("Timeout waiting for commit/create confirmations.");
      // Include state in error message
      return Err( OpenAIError::WsInvalidMessage(
        format!( "Timeout waiting for commit/create confirmations (commit_confirmed : {}, item_created_confirmed : {})",
        commit_confirmed, item_created_confirmed
      ) ).into() );
    }

    // Use timeout for reading to avoid blocking forever if connection stalls
    let read_timeout = Duration::from_millis(500);
    match tokio::time::timeout( read_timeout, session_client.recv_event() ).await
    {
      Ok( Ok( event ) ) => // Successfully received and deserialized an event
      {
            match event
            {
              RealtimeServerEvent::InputAudioBufferCommitted( committed_event ) =>
              {
                println!( "\n--- Commit Confirmation Received ---" );
                println!( "{:?}", committed_event ); // Keep full print for debugging this specific event
                let item_id = committed_event.item_id;
                println!( "Successfully received input_audio_buffer.committed. User item ID expected : {}", item_id );
                *expected_item_id_clone.lock().unwrap() = Some( item_id ); // Store the expected ID
                commit_confirmed = true;
                if item_created_confirmed { break; } // Break if item already confirmed
              }
              RealtimeServerEvent::ConversationItemCreated( created_event ) =>
              {
                println!( "\n--- Conversation Item Created Received ---" );
                // Check if this created item matches the one we expected from the commit event
                let maybe_expected_id = expected_item_id_from_commit.lock().unwrap().clone();

                if let Some( expected_id ) = maybe_expected_id
                {
if created_event.item.id.as_deref() == Some( expected_id.as_str() )
{
                       println!( "Successfully received conversation.item.created matching committed item (ID: {}).", expected_id);
                       item_created_confirmed = true;
                       if commit_confirmed { break; } // Break if commit already confirmed
                    }
                    else
                    {
                       // Log mismatch but don't necessarily fail yet, could be another item
                       println!( "Received conversation.item.created for a different item ID: {:?}, expected : {}", created_event.item.id, expected_id );
                    }
                }
                else
                {
                   // Commit confirmation hasn't arrived yet. Check if this looks like our user item.
if created_event.item.role.as_deref() == Some("user")
{
                      println!( "Received user conversation.item.created before commit provided expected ID.");
                      // If commit *is* already confirmed (somehow missed setting ID?), assume this is it. Unlikely path.
if commit_confirmed
{
                          item_created_confirmed = true;
                          break;
                      }
                      // Otherwise, just log and wait for the commit confirmation + its expected ID.
                   }
                   else
                   {
                      println!("Received unexpected non-user item created : {:?}", created_event.item.role);
                   }
                }
              }
              // Handle transcription events which might occur after commit
              RealtimeServerEvent::ConversationItemInputAudioTranscriptionDelta( delta ) =>
              {
                print!("{}", delta.delta); // Print delta directly for live feedback
                let _ = stdout().flush(); // Ensure it prints immediately
              }
              RealtimeServerEvent::ConversationItemInputAudioTranscriptionCompleted( completed ) =>
              {
                println!("\n--- Transcription Completed ---");
                println!("{completed:?}");
              }
              // Handle potential errors specifically related to the commit event
              RealtimeServerEvent::Error( error_event ) =>
              {
                eprintln!( "\n--- Received Server Error Event ---" );
                println!( "{error_event:?}" );
                // Check if the error is related to the commit event ID we sent
if error_event.error.event_id.as_deref() == Some(&client_event_id)
{
                  eprintln!("Server error explicitly linked to our commit request (event_id : {}).", client_event_id);
                  // Return the specific API error
                  return Err( OpenAIError::WsInvalidMessage(
                    format!( "Commit failed : type={}, code={:?}, message={}",
                      error_event.error.r#type, error_event.error.code, error_event.error.message
                  ) ).into() );
                } else if error_event.error.message.to_lowercase().contains("commit")
                {
                  eprintln!("Server error message mentions 'commit'.");
                    return Err( OpenAIError::WsInvalidMessage(
                      format!( "Commit likely failed : type={}, code={:?}, message={}",
                        error_event.error.r#type, error_event.error.code, error_event.error.message
                  ) ).into() );
                }
                // Otherwise, log as a general error but continue waiting for confirmations if needed
               }
              // Handle other events not explicitly checked above
              _ => { println!( "\n--- Received Other Event --- \n{event:?}" ); }
            }
      }
      Ok( Err( e ) ) => // Error during WebSocket read/deserialization
      {
        eprintln!( "\nError reading from WebSocket : {:?}", e );
        return Err( e.into() ); // Propagate the error
      }
      Err( _elapsed ) => // Read timed out
      {
         // Timeout on read is expected if server is idle, just continue loop and check overall timeout
         continue;
      }
    }

  } // End confirmation loop

  // Final check after loop exits (due to break, close, or error)
  if !commit_confirmed || !item_created_confirmed
  {
    eprintln!("Loop finished without receiving full confirmation (commit : {}, item_created : {}).", commit_confirmed, item_created_confirmed);
    // Determine if connection closed prematurely or confirmations just weren't received
    if session_client.recv_event().await.is_ok() { // Check if connection is still technically open
         return Err( OpenAIError::WsInvalidMessage( "Did not receive expected commit/create confirmations".to_string() ).into() );
    }
    else
    {
         return Err( OpenAIError::Ws( "WebSocket connection closed".to_string() ).into() ); // Assume closed if read fails now
    }
  }

  println!( "\nCommit and item creation successfully confirmed." );
  Ok( () )
}