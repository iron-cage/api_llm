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
    input : ResponseInput::String( "Hello for get example!".to_string() ),
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

  // Now, retrieve the response using its ID
  let response = client.responses().get( &created_response.id ).await?;
  println!( "Retrieved response ID: {}", response.id );

  // Clean up : delete the created response
  client.responses().delete( &created_response.id ).await?;

  Ok( () )
}
*/