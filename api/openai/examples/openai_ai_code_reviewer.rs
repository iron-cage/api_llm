//! AI Code Reviewer - Analyze Rust code for bugs, improvements, and security issues
//!
//! This example demonstrates how to use the `OpenAI` API to create an AI-powered
//! code reviewer that can analyze Rust code and provide detailed feedback on:
//! - Potential bugs and logic errors
//! - Performance improvements
//! - Security vulnerabilities
//! - Code style and best practices
//! - Suggestions for refactoring
//!
//! Run with:
//! ```bash
//! cargo run --example ai_code_reviewer
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

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  // Load API key from workspace secrets
  let secret = api_openai::secret::Secret::load_with_fallbacks("OPENAI_API_KEY")
    .expect("Failed to load OPENAI_API_KEY. Please set environment variable or add to workspace secrets file.");

  let env = api_openai::environment::OpenaiEnvironmentImpl::build(secret, None, None, api_openai::environment::OpenAIRecommended::base_url().to_string(), api_openai::environment::OpenAIRecommended::realtime_base_url().to_string())
    .expect("Failed to create environment");
  let client = Client::build(env).expect("Failed to create client");

  // Sample Rust code to review (intentionally has some issues)
  let code_to_review = r"
use std::fs::File;
use std::io::Read;

pub fn read_config_file(filename : &str) -> String
{
    let mut file = File::open(filename).unwrap(); // Issue : unwrap() can panic
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap(); // Issue : unwrap() can panic
    contents
}

pub fn calculate_discount(price : f64, discount_percent : f64) -> f64
{
    if discount_percent > 100.0 {  // Issue : should also check < 0
        return price;
    }
    price - (price * discount_percent / 100.0)
}

pub struct UserData
{
    pub password : String,  // Issue : storing plain text password
    pub email : String,
}

impl UserData
{
    pub fn new(email : String, password : String) -> UserData
    {
        UserData { email, password }  // Issue : no validation
    }
}
";

  // Create detailed prompt for code review
  let review_prompt = format!(
    "Please perform a comprehensive code review of the following Rust code. \
    Analyze it for:\n\
    1. **Bugs and Logic Errors**: Identify potential runtime issues, edge cases, or logical mistakes\n\
    2. **Security Issues**: Look for security vulnerabilities like unsafe practices, data exposure, etc.\n\
    3. **Performance Issues**: Suggest optimizations and more efficient approaches\n\
    4. **Code Quality**: Comment on style, readability, and Rust best practices\n\
    5. **Error Handling**: Evaluate error handling patterns and suggest improvements\n\n\
    For each issue found, please provide:\n\
    - The specific line or section with the problem\n\
    - Why it's an issue\n\
    - A concrete fix or improvement suggestion\n\n\
    Code to review:\n```rust\n{code_to_review}\n```"
  );

  println!("🔍 Starting AI Code Review...\n");
  println!("Code being analyzed:");
  println!("```rust");
  let code_trimmed = code_to_review.trim();
  println!("{code_trimmed}");
  println!("```\n");

  let request = CreateResponseRequest::former()
    .model("gpt-5.1-chat-latest".to_string())
    .input(ResponseInput::String(review_prompt))
    .form();

  println!("🤖 AI Code Review Results:");
  println!("{}", "=".repeat(50));

  let response = client.responses().create(request).await?;

  // Process and display the review results
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

  println!("\n{}", "=".repeat(50));
  println!("✅ Code review completed!");

  // Display usage statistics if available
  if let Some(usage) = response.usage
  {
    println!("\n📊 API Usage:");
    println!("  Prompt tokens : {}", usage.prompt_tokens);
    println!("  Completion tokens : {:?}", usage.completion_tokens);
    println!("  Total tokens : {}", usage.total_tokens);
  }

  Ok(())
}