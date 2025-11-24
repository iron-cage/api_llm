//! Document Analyzer with Vision - Process images/documents and extract insights
//!
//! This example demonstrates how to use the `OpenAI` API's vision capabilities
//! to analyze documents, images, charts, and diagrams. It can:
//! - Extract text content from images (OCR functionality)
//! - Analyze charts, graphs, and data visualizations
//! - Process screenshots, invoices, receipts, and forms
//! - Understand complex visual layouts and relationships
//! - Generate structured summaries and actionable insights
//!
//! Run with:
//! ```bash
//! cargo run --example document_analyzer_vision
//! ```

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput },
    input ::{ InputItem, InputMessage, InputContentPart, InputText, InputImage },
    output ::{ OutputItem, OutputContentPart },
  },
};
// use std::path::Path; // Not needed for this example

#[ allow( clippy::too_many_lines ) ]
#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  // Load API key from workspace secrets
  let secret = api_openai::secret::Secret::load_with_fallbacks("OPENAI_API_KEY")
    .expect("Failed to load OPENAI_API_KEY. Please set environment variable or add to workspace secrets file.");

  let env = api_openai::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
    .expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  println!("👁️  Document Analyzer with Vision");
  println!("{}", "=".repeat(50));
  println!("Analyzing documents, charts, and images with AI vision!\n");

  // Create sample image URLs for demonstration (using publicly available images)
  // In a real application, you would use actual local files or user-provided images
  // Note : These URLs are tested to work with OpenAI's vision API (JPEG format works best)
  let image_urls = [
    "https://upload.wikimedia.org/wikipedia/commons/thumb/1/15/Cat_August_2010-4.jpg/256px-Cat_August_2010-4.jpg",
    "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/320px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg",
  ];

  // Analysis scenarios to demonstrate different use cases
  let analysis_scenarios = [
    (
      "General Image Analysis",
      "Please analyze this image in detail. Describe what you see, identify key elements, \
      and provide any relevant insights or observations. If there's text, extract it. \
      If there are charts or data, interpret them."
    ),
    (
      "Business Document Analysis",
      "Analyze this document/image from a business perspective. Extract key information, \
      identify important data points, and provide a structured summary that could be useful \
      for business decision-making."
    ),
  ];

  for (scenario_idx, (scenario_name, base_prompt)) in analysis_scenarios.iter().enumerate()
  {
    let scenario_num = scenario_idx + 1;
    println!("📊 Scenario #{scenario_num}: {scenario_name}");
    println!("{}", "-".repeat(60));

    // For demo purposes, we'll use the first image URL for both scenarios
    // In a real app, you'd have different images for different scenarios
    let image_url = image_urls[scenario_idx % image_urls.len()];

    println!("🖼️  Analyzing image : {image_url}");

    // Create input with image
    let request = CreateResponseRequest::former()
      .model("gpt-5.1-chat-latest".to_string())
      .input(
        ResponseInput::Items(
          vec![
            InputItem::Message(
              InputMessage::former()
                .role("user")
                .content(
                  vec![
                    InputContentPart::Text(
                      InputText::former()
                        .text((*base_prompt).to_string())
                        .form()
                    ),
                    InputContentPart::Image(
                      InputImage::former()
                        .image_url(image_url.to_string())
                        .detail("high")
                        .form()
                    ),
                  ]
                )
                .form()
            ),
          ]
        )
      )
      .form();

    println!("🤖 Processing with AI vision...\n");

    let response = client.responses().create(request).await?;

    // Process and display analysis results
    println!("📋 Analysis Results:");
    println!("{}", "-".repeat(40));

    for item in response.output
    {
      if let OutputItem::Message(message) = item
      {
        for content_part in message.content
        {
          if let OutputContentPart::Text { text, .. } = content_part
          {
            println!("{text}");
          }
        }
      }
    }

    // Display analysis statistics
    println!("\n📊 Analysis Statistics:");
    if let Some(usage) = &response.usage
    {
      println!("  Prompt tokens : {}", usage.prompt_tokens);
      println!("  Completion tokens : {:?}", usage.completion_tokens);
      println!("  Total tokens : {}", usage.total_tokens);
    }

    println!("\n{}\n", "=".repeat(80));

    // Pause between analyses
    if scenario_idx < analysis_scenarios.len() - 1
    {
      println!("⏳ Preparing next analysis...\n");
      tokio ::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
  }

  // Demonstrate structured data extraction
  println!("📋 Advanced Use Case : Structured Data Extraction");
  println!("{}", "-".repeat(60));

  let structured_prompt = r#"
Please analyze this image and extract information in the following JSON format:
{
  "content_type": "description of what type of content this is",
  "key_elements": ["list", "of", "main", "elements", "found"],
  "text_content": "any text found in the image",
  "data_insights": "interpretation of any data/charts if present",
  "actionable_items": ["list", "of", "actionable", "insights"],
  "confidence_score": "how confident you are in this analysis (1-10)"
}
"#;

  let structured_request = CreateResponseRequest::former()
    .model("gpt-5.1-chat-latest".to_string())
    .input(
      ResponseInput::Items(
        vec![
          InputItem::Message(
            InputMessage::former()
              .role("user")
              .content(
                vec![
                  InputContentPart::Text(
                    InputText::former()
                      .text(structured_prompt.to_string())
                      .form()
                  ),
                  InputContentPart::Image(
                    InputImage::former()
                      .image_url(image_urls[0].to_string())
                      .detail("high")
                      .form()
                  ),
                ]
              )
              .form()
          ),
        ]
      )
    )
    .form();

  println!("🤖 Extracting structured data...\n");

  let structured_response = client.responses().create(structured_request).await?;

  println!("📋 Structured Analysis Results:");
  println!("{}", "-".repeat(40));

  for item in structured_response.output
  {
    if let OutputItem::Message(message) = item
    {
      for content_part in message.content
      {
        if let OutputContentPart::Text { text, .. } = content_part
        {
          println!("{text}");
        }
      }
    }
  }

  println!("\n✅ Document analysis completed!");
  println!("\n💡 This example demonstrates how to:");
  println!("   • Process images with AI vision capabilities");
  println!("   • Extract text content from images (OCR)");
  println!("   • Analyze charts, graphs, and visual data");
  println!("   • Generate structured summaries from visual content");
  println!("   • Handle different types of document analysis scenarios");

  Ok(())
}