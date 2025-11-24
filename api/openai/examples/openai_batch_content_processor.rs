//! Batch Content Processor - Process multiple items efficiently
//!
//! This example demonstrates how to efficiently process multiple content items
//! using the `OpenAI` API with proper concurrency, rate limiting, and error handling:
//! - Process multiple texts, documents, or data items in batches
//! - Implement concurrent processing with controlled parallelism
//! - Handle rate limits and API errors gracefully
//! - Aggregate and summarize results across all processed items
//! - Track progress and provide detailed reporting
//!
//! Run with:
//! ```bash
//! cargo run --example batch_content_processor
//! ```

use api_openai::ClientApiAccessors;
use api_openai::
{
  Client,
  components ::
  {
    responses ::{ CreateResponseRequest, ResponseInput },
    output ::{ OutputItem, OutputContentPart },
  },
};
use tokio::time::{ sleep, Duration };
use std::sync::Arc;
use futures::future;

#[ derive( Debug, Clone ) ]
struct ContentItem
{
  id : String,
  title : String,
  content : String,
}

#[ derive( Debug ) ]
struct ProcessingResult
{
  id : String,
  title : String,
  success : bool,
  summary : String,
  sentiment : String,
  key_topics : Vec< String >,
  word_count : usize,
  processing_time_ms : u128,
  error : Option< String >,
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  // Load API key from workspace secrets
  let secret = api_openai::secret::Secret::load_with_fallbacks("OPENAI_API_KEY")
    .expect("Failed to load OPENAI_API_KEY. Please set environment variable or add to workspace secrets file.");

  let env = api_openai::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
    .expect("Failed to create environment");
  let client = Arc::new(Client::build(env).expect("Failed to create client"));

  println!("⚡ Batch Content Processor");
  println!("{}", "=".repeat(50));
  println!("Processing multiple content items efficiently with AI analysis!\n");

  let content_items = create_sample_content_items();

  println!("📋 Content Items to Process : {}", content_items.len());
  for item in &content_items
  {
    println!("  • {} (ID: {})", item.title, item.id);
  }
  println!();

  let max_concurrent = 3;
  let delay_between_batches = Duration::from_millis(500);

  println!("⚙️  Processing Configuration:");
  println!("  Max concurrent requests : {max_concurrent}");
  println!("  Delay between batches : {delay_between_batches:?}");
  println!();

  let results = process_content_batches(client, content_items, max_concurrent, delay_between_batches).await?;
  display_batch_results(&results);
  display_cross_content_analysis(&results);

  println!("\n✅ Batch processing completed successfully!");
  println!("\n💡 This example demonstrates how to:");
  println!("   • Process multiple content items efficiently with controlled concurrency");
  println!("   • Implement proper rate limiting and error handling");
  println!("   • Aggregate and analyze results across multiple processed items");
  println!("   • Track detailed performance metrics and statistics");
  println!("   • Scale processing workloads while respecting API constraints");

  Ok(())
}

async fn process_content_item(
  client : Arc< Client< api_openai::environment::OpenaiEnvironmentImpl > >,
  item : ContentItem
) -> Result< ProcessingResult, Box< dyn std::error::Error + Send + Sync > >
{
  let start_time = std::time::Instant::now();

  let analysis_prompt = format!(
    "Please analyze the following content and provide:\n\
    1. A concise summary (1-2 sentences)\n\
    2. The overall sentiment (positive, negative, or neutral)\n\
    3. 3-5 key topics or themes\n\
    4. Word count estimation\n\n\
    Please format your response as:\n\
    SUMMARY: [your summary]\n\
    SENTIMENT: [sentiment]\n\
    TOPICS: [topic1, topic2, topic3, ...]\n\
    WORD_COUNT: [estimated count]\n\n\
    Content to analyze:\n\
    Title : {}\n\
    Content : {}",
    item.title, item.content
  );

  let request = CreateResponseRequest::former()
    .model("gpt-5-mini".to_string())
    .input(ResponseInput::String(analysis_prompt))
    .form();

  match client.responses().create(request).await
  {
    Ok(response) =>
    {
      let processing_time = start_time.elapsed().as_millis();

      // Extract response text
      let mut response_text = String::new();
      for output_item in response.output
      {
        if let OutputItem::Message(message) = output_item
        {
          for content_part in message.content
          {
            if let OutputContentPart::Text { text, .. } = content_part
            {
              response_text = text;
              break;
            }
          }
        }
      }

      // Parse the structured response
      let (summary, sentiment, key_topics, word_count) = parse_analysis_response(&response_text);

      Ok(ProcessingResult {
        id : item.id,
        title : item.title,
        success : true,
        summary,
        sentiment,
        key_topics,
        word_count,
        processing_time_ms : processing_time,
        error : None,
      })
    },
    Err(e) =>
    {
      let processing_time = start_time.elapsed().as_millis();
      Ok(ProcessingResult {
        id : item.id,
        title : item.title,
        success : false,
        summary : "Analysis failed".to_string(),
        sentiment : "unknown".to_string(),
        key_topics : vec![],
        word_count : 0,
        processing_time_ms : processing_time,
        error : Some(e.to_string()),
      })
    }
  }
}

fn parse_analysis_response(response : &str) -> (String, String, Vec< String >, usize)
{
  let mut summary = "No summary available".to_string();
  let mut sentiment = "neutral".to_string();
  let mut topics = vec![];
  let mut word_count = 0;

  for line in response.lines()
  {
    if line.starts_with("SUMMARY:")
    {
      summary = line.replace("SUMMARY:", "").trim().to_string();
    } else if line.starts_with("SENTIMENT:")
    {
      sentiment = line.replace("SENTIMENT:", "").trim().to_string();
    } else if line.starts_with("TOPICS:")
    {
      let topics_line = line.replace("TOPICS:", "");
      let topics_str = topics_line.trim();
      topics = topics_str.split(',').map(|t| t.trim().to_string()).collect();
    } else if line.starts_with("WORD_COUNT:")
    {
      if let Ok(count) = line.replace("WORD_COUNT:", "").trim().parse::< usize >()
      {
        word_count = count;
      }
    }
  }

  (summary, sentiment, topics, word_count)
}

fn create_sample_content_items() -> Vec< ContentItem >
{
  vec![
    ContentItem {
      id : "article_001".to_string(),
      title : "The Future of Renewable Energy".to_string(),
      content : "Renewable energy technologies have seen unprecedented growth in recent years. \
                Solar and wind power are becoming increasingly cost-effective, leading to \
                widespread adoption across both developed and developing nations. The integration \
                of smart grids and energy storage solutions is revolutionizing how we generate, \
                distribute, and consume energy. Governments worldwide are implementing policies \
                to accelerate the transition to clean energy, while private sector investments \
                continue to drive innovation in this sector.".to_string(),
    },
    ContentItem {
      id : "article_002".to_string(),
      title : "Artificial Intelligence in Healthcare".to_string(),
      content : "AI applications in healthcare are transforming patient care and medical research. \
                Machine learning algorithms can now diagnose diseases with remarkable accuracy, \
                often surpassing human specialists in specific areas. Drug discovery processes \
                are being accelerated through AI-powered molecular analysis. However, challenges \
                remain in terms of data privacy, algorithmic bias, and the need for regulatory \
                frameworks that can keep pace with technological advancement.".to_string(),
    },
    ContentItem {
      id : "article_003".to_string(),
      title : "Remote Work Revolution".to_string(),
      content : "The global shift to remote work has fundamentally changed how businesses operate. \
                Companies are discovering that many roles can be performed effectively from anywhere, \
                leading to reduced office costs and access to global talent pools. However, this \
                transition also presents challenges in maintaining team cohesion, company culture, \
                and work-life balance. New tools and methodologies for remote collaboration are \
                continuously evolving to address these challenges.".to_string(),
    },
    ContentItem {
      id : "article_004".to_string(),
      title : "Sustainable Urban Development".to_string(),
      content : "Cities around the world are reimagining urban planning with sustainability at the forefront. \
                Green building standards, public transportation improvements, and urban farming initiatives \
                are becoming standard practices. Smart city technologies are being deployed to optimize \
                resource usage and reduce environmental impact. The goal is to create livable, efficient \
                urban environments that can accommodate growing populations while minimizing ecological footprint.".to_string(),
    },
    ContentItem {
      id : "article_005".to_string(),
      title : "Blockchain and Digital Finance".to_string(),
      content : "Blockchain technology is reshaping the financial industry beyond cryptocurrencies. \
                Decentralized finance (DeFi) platforms are offering traditional banking services \
                without intermediaries. Central banks are exploring digital currencies, while \
                enterprises are implementing blockchain for supply chain transparency and smart contracts. \
                The technology promises increased security, reduced costs, and greater financial inclusion, \
                though regulatory clarity remains a key challenge.".to_string(),
    },
  ]
}

async fn process_content_batches(
  client : Arc< Client< api_openai::environment::OpenaiEnvironmentImpl > >,
  content_items : Vec< ContentItem >,
  max_concurrent : usize,
  delay_between_batches : Duration
) -> Result< Vec< ProcessingResult >, Box< dyn std::error::Error > >
{
  let start_time = std::time::Instant::now();
  let mut results = Vec::new();

  println!("🚀 Starting batch processing...\n");

  for (batch_num, chunk) in content_items.chunks(max_concurrent).enumerate()
  {
    println!("📦 Processing Batch #{} ({} items):", batch_num + 1, chunk.len());

    let batch_futures : Vec< _ > = chunk.iter().map(|item| {
      let client = Arc::clone(&client);
      let item = item.clone();

      async move {
        process_content_item(client, item).await
      }
    }).collect();

    let batch_results = future::join_all(batch_futures).await;

    for result in &batch_results
    {
      match result
      {
        Ok(processing_result) =>
        {
          if processing_result.success
          {
            println!("  ✅ {} ({}ms)", processing_result.title, processing_result.processing_time_ms);
          }
          else
          {
            println!("  ❌ {} - Error : {:?}", processing_result.title, processing_result.error);
          }
        },
        Err(e) =>
        {
          println!("  ❌ Processing failed : {e}");
        }
      }
    }

    for processing_result in batch_results.into_iter().flatten()
    {
      results.push(processing_result);
    }

    if batch_num < (content_items.len() + max_concurrent - 1) / max_concurrent - 1
    {
      println!("  ⏳ Waiting before next batch...\n");
      sleep(delay_between_batches).await;
    }
  }

  let total_time = start_time.elapsed();
  let successful_items = results.iter().filter(|r| r.success).count();
  let failed_items = results.len() - successful_items;
  let avg_processing_time = if successful_items > 0
  {
    results.iter().filter(|r| r.success).map(|r| r.processing_time_ms).sum::< u128 >() / successful_items as u128
  }
  else
  {
    0
  };
  let total_words_processed = results.iter().map(|r| r.word_count).sum::< usize >();

  println!("\n{}", "=".repeat(60));
  println!("📊 Batch Processing Results");
  println!("{}", "=".repeat(60));

  println!("🎯 Overall Statistics:");
  println!("  Total items processed : {}", content_items.len());
  println!("  Successful : {successful_items}");
  println!("  Failed : {failed_items}");
  println!("  Success rate : {:.1}%", (successful_items as f64 / content_items.len() as f64) * 100.0);
  println!("  Total processing time : {:.2}s", total_time.as_secs_f64());
  println!("  Average processing time per item : {avg_processing_time}ms");
  println!("  Total words processed : {total_words_processed}");
  println!("  Processing speed : {:.1} words/second", total_words_processed as f64 / total_time.as_secs_f64());

  Ok(results)
}

fn display_batch_results(results : &[ProcessingResult])
{
  println!("\n📋 Detailed Results:");
  for result in results
  {
    if result.success
    {
      println!("\n  📄 {} (ID: {})", result.title, result.id);
      println!("    📝 Summary : {}", result.summary.chars().take(100).collect::< String >() + "...");
      println!("    😊 Sentiment : {}", result.sentiment);
      println!("    🏷️  Key Topics : {}", result.key_topics.join(", "));
      println!("    📊 Word Count : {}", result.word_count);
      println!("    ⏱️  Processing Time : {}ms", result.processing_time_ms);
    }
  }
}

fn display_cross_content_analysis(results : &[ProcessingResult])
{
  println!("\n🔍 Cross-Content Analysis:");
  let all_topics : Vec< String > = results.iter()
    .flat_map(|r| r.key_topics.clone())
    .collect();
  let mut topic_counts = std::collections::HashMap::new();
  for topic in all_topics
  {
    *topic_counts.entry(topic).or_insert(0) += 1;
  }

  let mut sorted_topics : Vec< _ > = topic_counts.into_iter().collect();
  sorted_topics.sort_by(|a, b| b.1.cmp(&a.1));

  println!("  🏷️  Most Common Topics:");
  for (topic, count) in sorted_topics.iter().take(5)
  {
    println!("    • {topic} (appeared in {count} articles)");
  }

  let positive_sentiment = results.iter().filter(|r| r.sentiment.to_lowercase().contains("positive")).count();
  let neutral_sentiment = results.iter().filter(|r| r.sentiment.to_lowercase().contains("neutral")).count();
  let negative_sentiment = results.iter().filter(|r| r.sentiment.to_lowercase().contains("negative")).count();

  println!("  😊 Overall Sentiment Distribution:");
  println!("    • Positive : {positive_sentiment} articles");
  println!("    • Neutral : {neutral_sentiment} articles");
  println!("    • Negative : {negative_sentiment} articles");
}