//! Example of cancelling an in-progress response using the OpenAI API,
//! after explicitly triggering the response with `response.create`.
//!
//! Run with:
//! `cargo run --example realtime_response_cancel`
//!
//! Make sure you have set the `OPENAI_API_KEY` environment variable
//! or have a `secret/-secret.sh` file with the key.

use api_openai::ClientApiAccessors;
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
    RealtimeClientEventResponseCancel,
    RealtimeClientEvent,
    RealtimeServerEvent,
  },

};

use std::sync::{ Arc, Mutex };

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
  .temperature( 0.7 )
  .form();

  tracing ::info!( "Sending request to OpenAI API to create session..." );
  // 3. Call the API endpoint to get session details.
  let session = client.realtime().create_session( request ).await?;

  tracing ::info!( "Creating Realtime WebSocket Session Client..." );
  let _token = session.client_secret.value;
  // 4. Establish the WebSocket connection using the session token.
  let session_client = client.realtime().connect_ws( &session.id ).await?;

  // --- Send User Message (but don't expect automatic response) ---
  let user_message_id_arc = Arc::new( Mutex::new( None::< String > ) );
  let user_message_id_clone = user_message_id_arc.clone();

  // 5. Prepare the content for the user message.
  let content = RealtimeConversationItemContent::former()
  .r#type( "input_text" )
  .text( "Tell me a long story about a brave knight." ) // Ask for something potentially lengthy
  .form();

  // 6. Prepare the conversation item itself.
  let ci_to_create = RealtimeConversationItem::former()
  .r#type( "message" )
  .role( "user" )
  .content( vec![ content ] )
  .form();

  // 7. Prepare the client event to create the conversation item.
  let cic_create = RealtimeClientEventConversationItemCreate::former()
  .item( ci_to_create )
  .form();

  tracing ::info!( "Sending conversation.item.create (user message)..." );
  // 8. Send the create event over the WebSocket.
  session_client.send_event( RealtimeClientEvent::ConversationItemCreate( cic_create ) ).await?;

  // --- Wait for User Message Confirmation ---
  tracing ::info!( "Waiting for user message conversation.item.created confirmation..." );
  loop
  {
      let response = session_client.recv_event().await.map( Some );
      #[ allow( unreachable_patterns ) ]
    match response
      {
          Ok(Some(RealtimeServerEvent::ConversationItemCreated(created_event))) =>
          {
              // Check if it's our user message (optional, but good practice if sending multiple items)
if created_event.item.role.as_deref() == Some("user")
{
                  println!("\n--- User Message Created Confirmation Received ---");
                  println!("{created_event:?}");
if let Some(id) = created_event.item.id
{
                       *user_message_id_clone.lock().unwrap() = Some(id);
                  }
                  break; // User message confirmed, proceed to trigger response
              }
              else
              {
                  println!("\n--- Received Other Item Created --- \n{created_event:?}");
              }
          }
          Ok(Some(event)) =>
          {
              // Handle initial SessionCreated etc. while waiting
              println!("\n--- Received Other Event (while waiting for user message confirmation) --- \n{event:?}");
          }
          Ok(None) =>
          {
              println!("\nWebSocket connection closed unexpectedly before user message confirmed.");
              return Err( OpenAIError::Ws( "WebSocket connection closed".to_string() ).into() );
          }
          Err(e) =>
          {
              eprintln!("\nError reading from WebSocket : {:?}", e);
              return Err( e.into() );
          }
      }
  }
if user_message_id_arc.lock().unwrap().is_none()
{
       tracing ::warn!("Did not capture user message ID, proceeding anyway...");
  }


  // --- Explicitly Trigger Response ---
  let response_id_to_cancel = Arc::new( Mutex::new( None::< String > ) );
  let response_id_clone = response_id_to_cancel.clone();

  // 9. Prepare the client event to explicitly create a response.
  let rc_create = RealtimeClientEventResponseCreate::former()
      // No specific overrides needed for this example, use session defaults
      .form();

  tracing ::info!("Sending explicit response.create event...");
  // 10. Send the response create event.
  session_client.send_event( RealtimeClientEvent::ResponseCreate( rc_create ) ).await?;


  // --- Wait for ResponseCreated to get the Response ID ---
  tracing ::info!( "Waiting for EXPLICIT response.created event to capture response ID..." );
  loop
  {
    let response = session_client.recv_event().await.map( Some );
    #[ allow( unreachable_patterns ) ]
    match response
    {
        Ok(Some(RealtimeServerEvent::ResponseCreated(created_event))) =>
        {
            println!("\n--- Explicit Response Created Event Received ---");
            println!("{created_event:?}");
            let response_id = created_event.response.id;
            println!("Captured response ID for cancellation : {}", response_id);
            *response_id_clone.lock().unwrap() = Some(response_id);
            break; // Got the ID, proceed to cancel
        }
        Ok(Some(event)) =>
        {
            // Might receive deltas etc. from the response we just triggered
            println!("\n--- Received Other Event (while waiting for explicit ResponseCreated) --- \n{event:?}");
        }
        Ok(None) =>
        {
            println!("\nWebSocket connection closed unexpectedly before response created.");
            return Err( OpenAIError::Ws( "WebSocket connection closed".to_string() ).into() );
        }
        Err(e) =>
        {
            eprintln!("\nError reading from WebSocket : {:?}", e);
            return Err( e.into() );
        }
    }
  }

  let response_id = response_id_to_cancel.lock().unwrap().clone();
if response_id.is_none()
{
    return Err( OpenAIError::WsInvalidMessage( "Failed to obtain response ID for cancellation".to_string() ).into() );
  }
  let response_id = response_id.unwrap();

  // Introduce a small delay to ensure the response is likely generating output before cancelling
  tokio ::time::sleep( tokio::time::Duration::from_millis( 500 ) ).await;

  // 11. Prepare the client event to cancel the response.
  let rc_cancel = RealtimeClientEventResponseCancel::former()
  .response_id( &response_id ) // Specify the ID of the response to cancel
  .form();

  tracing ::info!( response_id = %response_id, "Sending response.cancel event..." );
  // 12. Send the cancel event over the WebSocket.
  session_client.send_event( RealtimeClientEvent::ResponseCancel( rc_cancel ) ).await?;

  // 13. Loop to read responses, specifically looking for the ResponseDone event confirming cancellation.
  tracing ::info!( "Waiting for response.done confirmation (status : cancelled)..." );
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
          RealtimeServerEvent::ResponseDone( done_event ) =>
          {
            println!( "\n--- Response Done Event Received ---" );
            println!( "{done_event:?}" );
            // Check if it's the response we cancelled and its status is 'cancelled'
            if done_event.response.id == response_id
            {
              if done_event.response.status == "cancelled"
              {
                println!( "Successfully received response.done confirmation with status 'cancelled' for response {}.", response_id );
                confirmation_received = true;
                break; // Break after receiving confirmation
              }
              else
              {
                println!( "Received response.done for response {}, but status was '{}', not 'cancelled'.", response_id, done_event.response.status );
                // Treat this as unexpected and break without confirming success
                break;
              }
            }
            else
            {
              println!( "Received response.done for a different response ID: {}", done_event.response.id );
            }
          }
          // Handle other events that might still arrive after cancellation request
          RealtimeServerEvent::ResponseTextDelta( _ ) |
          RealtimeServerEvent::ResponseAudioDelta( _ ) =>
          {
            println!( "\n--- Received Delta (after cancel request) --- \n{event:?}" );
          }
          _ => { println!( "\n--- Received Other Event (while waiting for cancel confirmation) --- \n{event:?}" ); }
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

  if !confirmation_received
  {
    eprintln!("Loop finished without receiving response.done (status : cancelled) confirmation for response {}.", response_id);
    return Err( OpenAIError::WsInvalidMessage( "Did not receive expected cancellation confirmation".to_string() ).into() );
  }

  Ok( () )
}