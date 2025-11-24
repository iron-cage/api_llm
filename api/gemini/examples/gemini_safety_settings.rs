//! Example demonstrating safety settings configuration and handling.
//!
//! This example shows:
//! - How to configure safety settings for different harm categories
//! - How to handle blocked content responses
//! - How to adjust thresholds based on use case
//! - How to interpret safety ratings in responses

use api_gemini::{ client::Client, models::* };

#[ tokio::main ]
#[ allow( clippy::too_many_lines ) ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  let client = Client::new()?;

  println!( "=== Safety Settings Example ===" );

  // Example 1: Default safety settings
  println!( "\n1. Request with Default Safety Settings" );

  let request_default = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "Tell me about the importance of online safety and privacy.".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: Some( GenerationConfig
    {
      temperature: Some( 0.7 ),
      top_k: Some( 40 ),
      top_p: Some( 0.95 ),
      candidate_count: Some( 1 ),
      max_output_tokens: Some( 512 ),
      stop_sequences: None,
    }),
    safety_settings: None, // Using default safety settings
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  let response_default = client
  .models()
  .by_name( "gemini-2.5-flash" )
  .generate_content( &request_default )
  .await?;

  if let Some( candidate ) = response_default.candidates.first()
  {
    if let Some( part ) = candidate.content.parts.first()
    {
      if let Some( text ) = &part.text
      {
      println!( "Response : {text}" );
      }
    }

    // Check safety ratings
    if let Some( safety_ratings ) = &candidate.safety_ratings
    {
      println!( "\nSafety Ratings:" );
      for rating in safety_ratings
      {
  println!( "  - {}: {} (blocked : {})",
        rating.category,
        rating.probability,
        rating.blocked.unwrap_or( false )
        );
      }
    }
  }

  // Example 2: Strict safety settings
  println!( "\n2. Request with Strict Safety Settings" );

  let strict_safety_settings = vec!
  [
  SafetySetting
  {
    category: "HARM_CATEGORY_HARASSMENT".to_string(),
    threshold: "BLOCK_LOW_AND_ABOVE".to_string(), // Most restrictive
  },
  SafetySetting
  {
    category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
    threshold: "BLOCK_LOW_AND_ABOVE".to_string(),
  },
  SafetySetting
  {
    category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
    threshold: "BLOCK_LOW_AND_ABOVE".to_string(),
  },
  SafetySetting
  {
    category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
    threshold: "BLOCK_LOW_AND_ABOVE".to_string(),
  },
  ];

  let request_strict = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "Write a story about conflict resolution.".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: request_default.generation_config.clone(),
    safety_settings: Some( strict_safety_settings ),
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  println!( "Making request with strict safety settings..." );

  let response_strict = client
  .models()
  .by_name( "gemini-2.5-flash" )
  .generate_content( &request_strict )
  .await?;

  // Handle potential blocking
  if let Some( prompt_feedback ) = &response_strict.prompt_feedback
  {
    if let Some( block_reason ) = &prompt_feedback.block_reason
    {
    println!( "Content was blocked! Reason : {block_reason}" );

      if let Some( safety_ratings ) = &prompt_feedback.safety_ratings
      {
        println!( "Safety ratings that triggered blocking:" );
        for rating in safety_ratings
        {
      println!( "  - {}: {}", rating.category, rating.probability );
        }
      }
    }
  }

  if let Some( candidate ) = response_strict.candidates.first()
  {
    if let Some( part ) = candidate.content.parts.first()
    {
      if let Some( text ) = &part.text
      {
      println!( "Response generated successfully : {}", &text[ ..text.len().min( 200 ) ] );
        println!( "..." );
      }
    }
  }

  // Example 3: Permissive safety settings for educational content
  println!( "\n3. Request with Permissive Settings (Educational Context)" );

  let educational_safety_settings = vec!
  [
  SafetySetting
  {
    category: "HARM_CATEGORY_HARASSMENT".to_string(),
    threshold: "BLOCK_ONLY_HIGH".to_string(), // Less restrictive
  },
  SafetySetting
  {
    category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
    threshold: "BLOCK_ONLY_HIGH".to_string(),
  },
  SafetySetting
  {
    category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
  },
  SafetySetting
  {
    category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
    threshold: "BLOCK_ONLY_HIGH".to_string(),
  },
  ];

  let request_educational = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some(
        "For educational purposes, explain the historical context of cybersecurity threats \
        and how they've evolved over time.".to_string()
        ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
    }
    ],
    generation_config: request_default.generation_config.clone(),
    safety_settings: Some( educational_safety_settings ),
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  let response_educational = client
  .models()
  .by_name( "gemini-2.5-flash" )
  .generate_content( &request_educational )
  .await?;

  if let Some( candidate ) = response_educational.candidates.first()
  {
    if let Some( part ) = candidate.content.parts.first()
    {
      if let Some( text ) = &part.text
      {
      println!( "Educational response : {}", &text[ ..text.len().min( 300 ) ] );
        println!( "..." );
      }
    }
  }

  // Example 4: Understanding safety categories and thresholds
  println!( "\n=== Safety Categories and Thresholds ===" );
  println!( "\nHarm Categories:" );
  println!( "- HARM_CATEGORY_HARASSMENT: Negative or harmful comments targeting identity" );
  println!( "- HARM_CATEGORY_HATE_SPEECH: Content that promotes hate based on identity" );
  println!( "- HARM_CATEGORY_SEXUALLY_EXPLICIT: Sexual content" );
  println!( "- HARM_CATEGORY_DANGEROUS_CONTENT: Content that promotes harmful acts" );

  println!( "\nThreshold Levels (from most to least restrictive):" );
  println!( "- BLOCK_LOW_AND_ABOVE: Blocks even low probability harmful content" );
  println!( "- BLOCK_MEDIUM_AND_ABOVE: Blocks medium and high probability (default)" );
  println!( "- BLOCK_ONLY_HIGH: Only blocks high probability harmful content" );
  println!( "- BLOCK_NONE: Disables blocking (use with extreme caution)" );

  println!( "\nProbability Levels in Responses:" );
  println!( "- NEGLIGIBLE: Very unlikely to be harmful" );
  println!( "- LOW: Unlikely to be harmful" );
  println!( "- MEDIUM: Possibly harmful" );
  println!( "- HIGH: Likely harmful" );

  println!( "\n=== Best Practices for Safety Settings ===" );
  println!( "1. Start with default settings and adjust based on your use case" );
  println!( "2. Use stricter settings for user-facing applications" );
  println!( "3. Consider context: educational/medical content may need relaxed settings" );
  println!( "4. Always handle blocked content gracefully in your application" );
  println!( "5. Log safety ratings for monitoring and improvement" );
  println!( "6. Implement additional filtering layers for critical applications" );

  Ok( () )
}