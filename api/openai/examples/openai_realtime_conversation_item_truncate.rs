//! Example of truncating an assistant audio message using the OpenAI API.
//! This example is self-contained and generates the necessary assistant message first.
#![ allow( clippy::doc_markdown, clippy::too_many_lines ) ]
//!
//! Run with:
//! `cargo run --example realtime_conversation_item_truncate`
//!
//! Make sure you have set the `OPENAI_API_KEY` environment variable
//! or have a `secret/-secret.sh` file with the key.

use api_openai::ClientApiAccessors;
#[ allow( unused_imports ) ]
use api_openai::
{
  Client,
  error ::OpenAIError,
  components ::realtime_shared::
  {
    RealtimeSessionCreateRequest,
    RealtimeConversationItemContent,
    RealtimeConversationItem,
    RealtimeClientEventConversationItemCreate,
    RealtimeClientEventResponseCreate,
    RealtimeClientEventConversationItemTruncate,
    RealtimeClientEvent,
    RealtimeServerEvent,
  },
  components ::common::ModelIds,
};


use std::sync::{ Arc, Mutex }; // To share the item ID
use tokio::time::{ sleep, Duration }; // For adding delays

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
  //    - Request audio output using standard format string.
  tracing ::info!( "Building realtime session request..." );
  let request = RealtimeSessionCreateRequest::former()
  .model( "gpt-4o-realtime-preview".to_string() )
  .output_audio_format( "pcm16" )
  .temperature( 0.7 )
  .form();

  tracing ::info!( "Sending request to OpenAI API to create session..." );
  // 3. Call the API endpoint to get session details.
  let session = client.realtime().create_session( request ).await?;

  tracing ::info!( "Creating Realtime WebSocket Session Client..." );
  let _ = session.client_secret.value;
  // 4. Establish the WebSocket connection using the session token.
  let session_client = client.realtime().connect_ws( &session.id ).await?;

  // --- State for capturing the Assistant's Audio Item ID ---
  let assistant_item_id = Arc::new( Mutex::new( None::< String > ) );
  let assistant_item_id_clone = assistant_item_id.clone();
  let response_id_arc = Arc::new( Mutex::new( None::< String > ) ); // To track current response
  let response_id_clone_for_listener = response_id_arc.clone();


  // --- Send User Message ---
  let user_message_id_arc = Arc::new( Mutex::new( None::< String > ) );
  let user_message_id_clone = user_message_id_arc.clone();

  let content = RealtimeConversationItemContent::former()
  .r#type( "input_text" )
  .text( "Please say 'Hello world, this is a test'." ) // Simple prompt for audio
  .form();
  let ci_to_create = RealtimeConversationItem::former()
  .r#type( "message" )
  .role( "user" )
  .content( vec![ content ] )
  .form();
  let cic_create = RealtimeClientEventConversationItemCreate::former()
  .item( ci_to_create )
  .form();

  tracing ::info!( "Sending conversation.item.create (user message)..." );
  session_client.send_event( RealtimeClientEvent::ConversationItemCreate( cic_create ) ).await?;

  // --- Wait for User Message Confirmation ---
  tracing ::info!( "Waiting for user message conversation.item.created confirmation..." );
  loop
  {
    let response = session_client.recv_event().await.map( Some );
    #[ allow( unreachable_patterns ) ]
    match response
    {
      Ok( Some( RealtimeServerEvent::ConversationItemCreated( created_event ) ) ) =>
      {
        if created_event.item.role.as_deref() == Some( "user" )
        {
          println!( "\n--- User Message Created Confirmation Received ---" );
          // println!( "{created_event:?}" ); // Reduce verbosity
          if let Some(id) = created_event.item.id
          {
            println!( "User message ID: {id}" );
            *user_message_id_clone.lock().unwrap() = Some(id);
          }
          break; // User message confirmed, proceed to trigger response
        }
      }
      Ok( Some( event ) ) =>
      {
        // Handle initial SessionCreated etc. while waiting
        println!( "\n--- Received Other Event (while waiting for user msg confirmation) --- \n{event:?}" );
      }
      Ok( None ) =>
      {
        eprintln!("\nWebSocket connection closed unexpectedly before user message confirmed.");
        return Err( OpenAIError::Ws( "WebSocket connection closed".to_string() ).into() );
      }
      Err( e ) => return Err( e.into() ),
    }
  }
  if user_message_id_arc.lock().unwrap().is_none()
  {
    tracing ::warn!( "Did not capture user message ID, proceeding anyway..." );
  }


  // --- Explicitly Trigger Response ---
  let rc_create = RealtimeClientEventResponseCreate::former().form();
  tracing ::info!( "Sending explicit response.create event..." );
  session_client.send_event( RealtimeClientEvent::ResponseCreate( rc_create ) ).await?;


  // --- Wait for Assistant Item ID and Response Done ---
  // We need to capture the ID of the assistant's message item.
  tracing ::info!( "Waiting for response events (output_item.added and done)..." );
  loop
  {
    // Check if we already got the ID and the response is done
    if assistant_item_id.lock().unwrap().is_some()
    {
      tracing ::info!( "Captured assistant item ID and response is done." );
      break;
    }

    let response = session_client.recv_event().await.map( Some );
    #[ allow( unreachable_patterns ) ]
    match response
    {
      Ok( Some( event ) ) => match event
      {
        RealtimeServerEvent::ResponseCreated( created_event ) =>
        {
          println!( "\n--- Response Created ---" );
          // println!( "{created_event:?}" );
          let resp_id = created_event.response.id;
          println!( "Response ID: {resp_id}" );
          *response_id_clone_for_listener.lock().unwrap() = Some( resp_id );
        }
        RealtimeServerEvent::ResponseOutputItemAdded( added_event ) =>
        {
          println!( "\n--- Response Output Item Added ---" );
          // println!( "{added_event:?}" );
          // *** Simplified ID Capture Logic ***
          // If it's an assistant message, assume it's the one we want (potentially containing audio).
          // If the assistant sends multiple items (e.g., text then audio), this might grab the first one.
          // The truncate call should fail later if this wasn't an audio item.
          if added_event.item.role.as_deref() == Some( "assistant" )
          {
            // Only capture if we haven't already grabbed one for this response
            if assistant_item_id.lock().unwrap().is_none()
            {
              if let Some( id ) = added_event.item.id.clone()
              {
                println!( "Captured potential assistant item ID: {id}" );
                *assistant_item_id_clone.lock().unwrap() = Some( id );
                // Don't break yet, wait for ResponseDone
              }
            }
            else
            {
              println!( "Already captured an assistant item ID, ignoring additional item : {:?}", added_event.item.id );
            }
          }
        }
        RealtimeServerEvent::ResponseDone( done_event ) =>
        {
          println!( "\n--- Response Done ---" );
          // println!( "{done_event:?}" );
          let current_response_id = response_id_arc.lock().unwrap();
          // Ensure this 'done' event matches the response we tracked
          if current_response_id.as_ref() == Some( &done_event.response.id )
          {
            println!( "Response {} is done. Status : {}", done_event.response.id, done_event.response.status );
            // Now check if we can break (both ID captured and response done)
            if assistant_item_id.lock().unwrap().is_some()
            {
              break;
            }
            // Response finished, but we never saw an assistant item added?
            tracing ::warn!( "Response finished, but no assistant item ID was captured." );
            // Break here anyway, the next step will fail gracefully.
            break;
          }
          println!( "Received ResponseDone for an unexpected response ID: {}", done_event.response.id );
        }
        // Handle deltas etc. - Reduce verbosity
        RealtimeServerEvent::ResponseTextDelta( _ ) |
        RealtimeServerEvent::ResponseAudioDelta( _ ) |
        RealtimeServerEvent::ResponseAudioTranscriptDelta( _ ) => { /* Optionally log */ }
        _ => { println!( "\n--- Received Other Event (while waiting for assistant item/done) --- \n{event:?}" ); }
      }
      Ok( None ) =>
      {
        eprintln!( "\nWebSocket connection closed unexpectedly while waiting for response." );
        return Err( OpenAIError::Ws( "WebSocket connection closed".to_string() ).into() );
      }
      Err( e ) =>
      {
        eprintln!( "\nError reading from WebSocket : {e:?}" );
        return Err( e.into() );
      }
    }
  } // End response listener loop

  // --- Proceed with Truncation ---
  let item_id_to_truncate = assistant_item_id.lock().unwrap().clone();

  if item_id_to_truncate.is_none()
  {
    eprintln!( "Failed to capture an assistant item ID from the response. Cannot proceed with truncation." );
    panic!( "Could not find assistant item ID to truncate" );
  }
  let item_id_to_truncate = item_id_to_truncate.unwrap();
  let audio_end_ms_target = 500; // Truncate audio up to 0.5 seconds

  // *** Add Delay Before Truncating ***
  tracing ::info!("Waiting briefly before sending truncate...");
  sleep( Duration::from_millis( 200 ) ).await;

  // 5. Prepare the client event to truncate the conversation item.
  let cit_truncate = RealtimeClientEventConversationItemTruncate::former()
  .item_id( &item_id_to_truncate ) // Use the captured ID
  .content_index( 0 ) // Assuming audio is the first/only content part
  .audio_end_ms( audio_end_ms_target )
  .form();

  tracing ::info!( item_id = %item_id_to_truncate, audio_end_ms = audio_end_ms_target, "Sending conversation.item.truncate event..." );
  // 6. Send the truncate event over the WebSocket.
  session_client.send_event( RealtimeClientEvent::ConversationItemTruncate( cit_truncate ) ).await?;

  // 7. Loop to read responses, specifically looking for the truncation confirmation.
  tracing ::info!( "Waiting for conversation.item.truncated confirmation..." );
  let mut confirmation_received = false;
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
          RealtimeServerEvent::ConversationItemTruncated( truncated_event ) =>
          {
            println!( "\n--- Truncation Confirmation Received ---" );
            println!( "{truncated_event:?}" );
            // Check if the confirmation matches the item we intended to truncate
            if truncated_event.item_id == item_id_to_truncate
            && truncated_event.audio_end_ms == audio_end_ms_target
            // && truncated_event.content_index == 0 // Content index check removed for robustness
            {
              println!( "Successfully received conversation.item.truncated confirmation for item {item_id_to_truncate}." );
              confirmation_received = true;
              break; // Break after receiving confirmation
            }
            println!( "Received truncation confirmation for a different item/parameters : {truncated_event:?}" );
          }
          // Handle server errors (e.g., item not found, invalid time)
          RealtimeServerEvent::Error( error_event ) =>
          {
            eprintln!( "\n--- Received Server Error Event ---" );
            println!( "{error_event:?}" );
            // If the error relates to our request, treat it as a form of "completion" (though unsuccessful)
            if error_event.error.message.contains( &item_id_to_truncate ) || error_event.error.param.as_deref() == Some( "item_id" )
            {
              eprintln!( "Server error likely related to the truncation request for item {item_id_to_truncate}." );
              // Break, but don't confirm success
              break;
            }
            else if error_event.error.param.as_deref() == Some( "audio_end_ms" )
            {
              eprintln!( "Server error likely related to invalid audio_end_ms for item {item_id_to_truncate}." );
              break;
            }
            else if error_event.error.message.contains( "truncate" )
            {
              eprintln!( "Server error related to truncation, possibly wrong content_index or item type." );
              break;
            }
          }
          // Handle other events
          _ => { println!( "\n--- Received Other Event (while waiting for truncate confirmation) --- \n{event:?}" ); }
        }
      }
      Ok( None ) =>
      {
        println!( "\nWebSocket connection closed by server." );
        break; // Exit loop if connection closed
      }
      Err( e ) =>
      {
        eprintln!( "\nError reading from WebSocket : {e:?}" );
        return Err( e.into() ); // Propagate the error
      }
    }
  }

  if !confirmation_received
  {
    eprintln!( "Loop finished without receiving specific conversation.item.truncated confirmation for item {item_id_to_truncate}." );
    // Consider the case where an expected error was received
    // If the goal is just to test sending, maybe Ok(()) is fine even if an error occurred.
    // But for demonstrating successful truncation, we need the confirmation.
    return Err( OpenAIError::WsInvalidMessage( "Did not receive expected truncation confirmation".to_string() ).into() );
  }

  Ok( () )
}