//! Web Research Assistant - Search and summarize information with citations
//!
//! This example demonstrates how to use the `OpenAI` API with web search capabilities
//! to create a research assistant that can:
//! - Search the web for current information on any topic
//! - Provide comprehensive summaries with proper citations
//! - Handle multiple research queries in sequence
//! - Extract key insights and present them in an organized format
//!
//! Run with:
//! ```bash
//! cargo run --example web_research_assistant
//! ```

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput },
    tools ::{ Tool, WebSearchTool },
    output ::{ OutputItem, OutputContentPart, Annotation },
  },
};
// use std::io::{self, Write}; // Not needed for this example

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  // Load API key from workspace secrets
  let secret = api_openai::secret::Secret::load_with_fallbacks("OPENAI_API_KEY")
    .expect("Failed to load OPENAI_API_KEY. Please set environment variable or add to workspace secrets file.");

  let env = api_openai::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
    .expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  println!("🔍 Web Research Assistant");
  println!("{}", "=".repeat(50));
  println!("Ask me anything and I'll research it using current web data!\n");

  // Demo research queries - in a real app, these could come from user input
  let research_queries = [
    "What are the latest developments in Rust programming language in 2024?",
    "What is the current status of renewable energy adoption worldwide?",
    "What are the most recent breakthroughs in AI and machine learning?",
  ];

  for (i, query) in research_queries.iter().enumerate()
  {
    let query_num = i + 1;
    println!("📝 Research Query #{query_num}: {query}");
    println!("{}", "-".repeat(60));

    // Create a comprehensive research prompt
    let research_prompt = format!(
      "Please research and provide a comprehensive summary on : \"{query}\"\n\n\
      Your response should include:\n\
      1. **Current Status**: What's the latest information available\n\
      2. **Key Developments**: Recent important changes or breakthroughs\n\
      3. **Key Players**: Important organizations, companies, or individuals involved\n\
      4. **Future Outlook**: What to expect going forward\n\
      5. **Sources**: Make sure to cite your sources properly\n\n\
      Please provide accurate, up-to-date information with proper citations."
    );

    let request = CreateResponseRequest::former()
      .model("gpt-5.1-chat-latest".to_string())
      .input(ResponseInput::String(research_prompt))
      .tools(vec![Tool::WebSearch(WebSearchTool::default())])
      .form();

    println!("🌐 Searching the web and analyzing information...\n");

    let response = client.responses().create(request).await?;

    // Process and display research results
    let mut web_searches_performed = 0;
    let mut citations_found = Vec::new();

    for item in response.output
    {
      match item
      {
        OutputItem::WebSearchCall(search_call) =>
        {
          web_searches_performed += 1;
          let search_id = &search_call.id;
          let search_status = &search_call.status;
          println!("🔎 Web Search #{web_searches_performed}: {search_id} (Status : {search_status})");
        },
        OutputItem::Message(message) =>
        {
          println!("📋 Research Summary:");
          println!("{}", "-".repeat(40));

          for content_part in message.content
          {
            if let OutputContentPart::Text { text, annotations } = content_part
            {
              println!("{text}");

              // Extract and collect citations
              for annotation in annotations
              {
                if let Annotation::UrlCitation { url, title, .. } = annotation
                {
                  citations_found.push((title, url));
                }
              }
            }
          }
        },
        _ => {} // Handle other output types if needed
      }
    }

    // Display citations in a organized format
    if !citations_found.is_empty()
    {
      println!("\n📚 Sources and Citations:");
      println!("{}", "-".repeat(40));
      for (i, (title, url)) in citations_found.iter().enumerate()
      {
        let citation_num = i + 1;
        println!("{citation_num}. {title} - {url}");
      }
    }

    // Display research statistics
    println!("\n📊 Research Statistics:");
    println!("  Web searches performed : {web_searches_performed}");
    let citations_count = citations_found.len();
    println!("  Citations found : {citations_count}");

    if let Some(usage) = &response.usage
    {
      let prompt_tokens = usage.prompt_tokens;
      println!("  Prompt tokens : {prompt_tokens}");
      let completion_tokens = usage.completion_tokens;
      println!("  Completion tokens : {completion_tokens:?}");
      let total_tokens = usage.total_tokens;
      println!("  Total tokens : {total_tokens}");
    }

    println!("\n{}\n", "=".repeat(80));

    // Pause between queries to be respectful to the API
    if i < research_queries.len() - 1
    {
      println!("⏳ Preparing next research query...\n");
      tokio ::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
  }

  println!("✅ Research session completed!");
  println!("\n💡 This example demonstrates how to:");
  println!("   • Use web search tools for current information");
  println!("   • Process and organize research results");
  println!("   • Extract and display citations properly");
  println!("   • Handle multiple research queries systematically");

  Ok(())
}