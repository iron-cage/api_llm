use api_openai::ClientApiAccessors;
/*
use api_openai::
{
  client ::Client,
  error ::OpenAIError,
  api ::responses::
  {
    CreateResponseRequest,
    ResponseInput,
  },
  components ::
  {
    common ::ModelIdsResponses,
  },
};

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  let client = Client::new();

  // First, create a response to get an ID
  let create_request = CreateResponseRequest
  {
    model : ModelIdsResponses::from( "gpt-5.1-chat-latest".to_string() ),
    input : ResponseInput::String( "Hello for delete example!".to_string() ),
    previous_response_id : None,
    reasoning : None,
    max_output_tokens : None,
    instructions : None,
    text : None,
    tools : None,
    tool_choice : None,
    truncation : None,
    metadata : None,
    temperature : None,
    top_p : None,
    user : None,
    include : None,
    parallel_tool_calls : None,
    store : None,
    stream : None,
  };
  let created_response = client.responses().create( create_request ).await?;
  let response_id = created_response.id;
  println!( "Created response ID: {}", response_id );

  // Now, delete the response
  let delete_response = client.responses().delete( &response_id ).await?;
  println!( "Deleted response ID: {}", delete_response.id );

  Ok( () )
}
*/