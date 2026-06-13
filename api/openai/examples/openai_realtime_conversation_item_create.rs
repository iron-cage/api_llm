//! Example of creating a response using the OpenAI API.
//!
//! Run with:
//! `cargo run --example realtime_conversation_item_create`
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
    RealtimeClientEvent,
    RealtimeServerEvent,
  },
  components ::common::ModelIds,
};



#[ tokio::main( flavor = "current_thread" ) ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  // Setup tracing for logging, especially useful for WebSocket events
  tracing_subscriber::fmt::init();

  // Load environment variables (e.g., from .env file)
  // dotenv().ok();

  // 1. Create a new OpenAI client.
  //    By default, it reads the API key from the OPENAI_API_KEY environment variable.
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

  // 5. Prepare the content for the conversation item.
  let content = RealtimeConversationItemContent::former()
  .r#type( "input_text" )
  .text( "Give some movie recommendations, please." )
  .form();

  // 6. Prepare the conversation item itself.
  let ci = RealtimeConversationItem::former()
  .r#type( "message" )
  .role( "user" )
  .content( vec![ content ] )
  .form();

  // 7. Prepare the client event to create the conversation item.
  let cic = RealtimeClientEventConversationItemCreate::former()
  .item( ci )
  .form();

  tracing ::info!( "Sending conversation.item.create event..." );
  // 8. Send the event over the WebSocket.
  session_client.send_event( RealtimeClientEvent::ConversationItemCreate( cic ) ).await?;

  // 9. Loop to read responses, specifically looking for the confirmation.
  tracing ::info!( "Waiting for conversation.item.created confirmation..." );
  let mut confirmation_received = false;
  loop
  {
    // Read the next event from the server.
    let response = session_client.recv_event().await.map( Some );

    #[ allow( unreachable_patterns ) ]
    match response
    {
      // Successfully received an event
      Ok( Some( event ) ) =>
      {
        match event
        {
          RealtimeServerEvent::ConversationItemCreated( created_event ) =>
          {
            // Optionally, you could inspect created_event details here, e.g., created_event.item.id
            println!( "\n--- Confirmation Received ---" );
            println!( "{created_event:?}" );
            println!( "Successfully received conversation.item.created confirmation." );
            confirmation_received = true;
            break; // Break after receiving confirmation
          }
          // Handle other expected events (like message deltas from the assistant)
          RealtimeServerEvent::ResponseTextDelta( delta_event ) =>
          {
            println!( "\n--- Received Delta ---" );
            println!( "{delta_event:?}" );
          }
          RealtimeServerEvent::SessionCreated( session_info ) =>
          {
            println!( "\n--- Received Session Info ---" );
            println!( "{session_info:?}" );
          }
          // Handle potential errors sent by the server
          RealtimeServerEvent::Error( error_event ) =>
          {
            eprintln!( "\n--- Received Server Error Event ---" );
            println!( "{error_event:?}" );
          }
          // Handle unexpected events if necessary
          _ =>
          {
            println!( "\n--- Received Other Event ---" );
            println!( "{event:?}" );
          }
        }
      }
      // The WebSocket stream closed gracefully
      Ok( None ) =>
      {
        println!( "\nWebSocket connection closed by server." );
        break; // Exit loop if connection closed
      }
      // An error occurred while reading from the WebSocket or deserializing
      Err( e ) =>
      {
        eprintln!( "\nError reading from WebSocket : {:?}", e );
        return Err( e.into() ); // Propagate the error
      }
    }
    // Example condition to stop listening eventually if needed (e.g., after confirmation)
    // if confirmation_received {
    //   println!("Stopping listener after receiving confirmation.");
    //   break;
    // }
  }

  if !confirmation_received
  {
    eprintln!("Loop finished without receiving conversation.item.created confirmation.");
    // Indicate failure if confirmation wasn't received before connection close
    return Err( OpenAIError::WsInvalidMessage( "Did not receive expected confirmation".to_string() ).into() );
  }

  Ok( () )
}