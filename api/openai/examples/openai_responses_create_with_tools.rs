//! Example for creating a response with tool use (web search).
//!
//! This example demonstrates how to use the `responses().create()` method
//! with the `web_search_preview` tool enabled.
//!
//! Run with:
//! ```bash
//! cargo run --example responses_create_with_tools
//! ```

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput },
    tools ::{ Tool, ToolChoice, WebSearchTool },
    output ::{ OutputItem, OutputContentPart, Annotation },
  },
};



#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{


  let secret = api_openai::secret::Secret::load_with_fallbacks("OPENAI_API_KEY")
    .expect("Failed to load OPENAI_API_KEY. Please set environment variable or add to workspace secrets file.");
  let env = api_openai::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string()).expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  let request = CreateResponseRequest::former()
  .model( "gpt-5.1-chat-latest".to_string() )
  .input( ResponseInput::String( "What was a positive news story from today?".to_string() ) )
  .tools
  (
    vec!
    [
      Tool::WebSearch( WebSearchTool::default() ),
    ]
  )
  .tool_choice( ToolChoice::String( "auto".to_string() ) )
  .form();

  println!( "Sending request with web search tool..." );

  let response = client.responses().create( request ).await?;

  println!( "Response Status : {:?}", response.status );
  for item in response.output
  {
    match item
    {
      OutputItem::WebSearchCall( call ) =>
      {
        println!( "Web Search Call : {call:?}" );
      },
      OutputItem::Message( message ) =>
      {
        println!( "Message Content:" );
        for content_part in message.content
        {
          if let OutputContentPart::Text { text, annotations } = content_part
          {
            println!( "  Text : {text}" );
            for annotation in annotations
            {
              if let Annotation::UrlCitation { url, title, start_index, end_index } = annotation
              {
                println!( "    URL Citation : URL='{url}', Title='{title}', Start={start_index}, End={end_index}" );
              }
            }
          }
        }
      },
      _ => println!( "Other Output Item : {item:?}" ),
    }
  }

  Ok( () )
}