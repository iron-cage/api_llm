//! Example of deleting a conversation item using the OpenAI API.
#![ allow( clippy::doc_markdown ) ]
//!
//! Run with:
//! `cargo run --example realtime_conversation_item_delete`
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
    RealtimeClientEventConversationItemDelete,
    RealtimeClientEvent,
    RealtimeServerEvent,
  },
  components ::common::ModelIds,
};


use std::sync::{ Arc, Mutex }; // To share the item ID

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
  let _ = session.client_secret.value;
  // 4. Establish the WebSocket connection using the session token.
  let session_client = client.realtime().connect_ws( &session.id ).await?;

  // --- Create an item first to get its ID ---
  let item_id_to_delete = Arc::new( Mutex::new( None::< String > ) );

  // 5. Prepare the content for the conversation item to be created (and then deleted).
  let content = RealtimeConversationItemContent::former()
  .r#type( "input_text" )
  .text( "This message will be deleted." )
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

  tracing ::info!( "Sending conversation.item.create event to get an item ID..." );
  // 8. Send the create event over the WebSocket.
  session_client.send_event( RealtimeClientEvent::ConversationItemCreate( cic_create ) ).await?;

  // 9. Loop to read responses, specifically looking for the creation confirmation.
  tracing ::info!( "Waiting for conversation.item.created confirmation to get ID..." );
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
          RealtimeServerEvent::ConversationItemCreated( created_event ) =>
          {
            println!( "\n--- Item Created Confirmation Received ---" );
            println!( "{created_event:?}" );
if let Some(id) = created_event.item.id
{
              println!( "Captured item ID for deletion : {id}" );
              *item_id_to_delete.lock().unwrap() = Some( id );
              break; // Got the ID, break to proceed with deletion
            }
            eprintln!( "Created item did not have an ID!" );
            return Err( OpenAIError::WsInvalidMessage( "Created item missing ID".to_string() ).into() );
          }
          // Handle other events if necessary while waiting
          _ => { println!( "\n--- Received Other Event (while waiting for create confirmation) --- \n{event:?}" ); }
        }
      }
      Err( e ) =>
      {
        eprintln!( "\nError reading from WebSocket : {e:?}" );
        return Err( e.into() );
      }
      _ => {}
    }
  } // End create confirmation loop

  // --- Now Delete the Item ---
  let item_id = item_id_to_delete.lock().unwrap().clone();
  assert!( item_id.is_some(), "Failed to obtain item ID for deletion" );
  let item_id = item_id.unwrap();

  // 10. Prepare the client event to delete the conversation item.
  let cid_delete = RealtimeClientEventConversationItemDelete::former()
  .item_id( &item_id )
  .form();

  tracing ::info!( item_id = %item_id, "Sending conversation.item.delete event..." );
  // 11. Send the delete event over the WebSocket.
  session_client.send_event( RealtimeClientEvent::ConversationItemDelete( cid_delete ) ).await?;

  // 12. Loop to read responses, specifically looking for the deletion confirmation.
  tracing ::info!( "Waiting for conversation.item.deleted confirmation..." );
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
          RealtimeServerEvent::ConversationItemDeleted( deleted_event ) =>
          {
            println!( "\n--- Deletion Confirmation Received ---" );
            println!( "{deleted_event:?}" );
            if deleted_event.item_id == item_id
            {
              println!( "Successfully received conversation.item.deleted confirmation for item {item_id}." );
              confirmation_received = true;
              break; // Break after receiving confirmation
            }
            println!( "Received deletion confirmation for a different item ID: {}", deleted_event.item_id );
          }
          // Handle other events
          _ => { println!( "\n--- Received Other Event (while waiting for delete confirmation) --- \n{event:?}" ); }
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
    eprintln!("Loop finished without receiving conversation.item.deleted confirmation.");
    return Err( OpenAIError::WsInvalidMessage( "Did not receive expected deletion confirmation".to_string() ).into() );
  }

  Ok( () )
}