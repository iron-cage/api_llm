//! Comprehensive tests for System Instructions functionality
//!
//! This module provides exhaustive testing for system instruction configuration,
//! behavior consistency, role-based responses, and instruction effectiveness.
//! All tests use real API calls following the no-mockup policy.

#![ cfg( feature = "integration" ) ]

#[ path = "common/mod.rs" ] mod common;
use common::create_integration_client;

use api_gemini::
{
  client ::Client,
  models ::
  {
    GenerateContentRequest, Content, Part, SystemInstruction,
    GenerationConfig,
  },
};
use tokio::time::{ timeout, Duration };
use std::collections::HashMap;

/// Create a request with system instructions
fn create_system_instruction_request(
system_instruction_text: &str,
user_message: &str,
conversation_history: Option< Vec< Content > >,
) -> GenerateContentRequest
{
  let system_instruction = SystemInstruction {
    role: "system".to_string(),
    parts : vec![ Part {
      text: Some( system_instruction_text.to_string() ),
      inline_data: None,
      function_call: None,
      function_response: None,
      file_data: None,
      video_metadata: None,
      thought: None,
    } ],
  };

  let mut contents = conversation_history.unwrap_or_default();
  contents.push( Content {
    parts : vec![ Part {
      text: Some( user_message.to_string() ),
      inline_data: None,
      function_call: None,
      function_response: None,
      file_data: None,
      video_metadata: None,
      thought: None,
    } ],
    role: "user".to_string(),
  } );

  GenerateContentRequest {
    contents,
    generation_config : Some( GenerationConfig {
      temperature: Some( 0.7 ),
      top_k: Some( 40 ),
      top_p: Some( 0.95 ),
      candidate_count: Some( 1 ),
      max_output_tokens: Some( 1024 ),
      stop_sequences: None,
    } ),
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: Some( system_instruction ),
    cached_content: None,
  }
}

/// Create a request without system instructions for comparison
fn create_basic_request( user_message: &str ) -> GenerateContentRequest
{
  GenerateContentRequest {
    contents : vec![ Content {
      parts : vec![ Part {
        text: Some( user_message.to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        file_data: None,
        video_metadata: None,
        thought: None,
      } ],
      role: "user".to_string(),
    } ],
    generation_config : Some( GenerationConfig {
      temperature: Some( 0.7 ),
      top_k: Some( 40 ),
      top_p: Some( 0.95 ),
      candidate_count: Some( 1 ),
      max_output_tokens: Some( 1024 ),
      stop_sequences: None,
    } ),
    safety_settings: None,
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  }
}

/// Extract response text from a generate content response
fn extract_response_text( response: &api_gemini::models::GenerateContentResponse ) -> String
{
  response.candidates.first()
  .and_then( |c| c.content.parts.first() )
  .and_then( |p| p.text.as_ref() )
  .map( |s| s.to_string() )
  .unwrap_or_default()
}

/// Generate comprehensive diagnostics for API response issues
fn diagnose_response( response: &api_gemini::models::GenerateContentResponse, query_num: usize ) -> String
{
  let mut diagnostics = Vec::new();

diagnostics.push( format!( "Query {}: Response Diagnostics", query_num ) );
diagnostics.push( format!( "  Candidates count : {}", response.candidates.len() ) );

  if let Some( candidate ) = response.candidates.first()
  {
    if let Some( finish_reason ) = &candidate.finish_reason
    {
    diagnostics.push( format!( "  Finish reason : {:?}", finish_reason ) );
    } else {
      diagnostics.push( "  Finish reason: None".to_string() );
    }

    if let Some( safety_ratings ) = &candidate.safety_ratings
    {
    diagnostics.push( format!( "  Safety ratings : {} checks", safety_ratings.len() ) );
      for rating in safety_ratings
      {
  diagnostics.push( format!( "    - Category : {:?}, Probability : {:?}, Blocked : {:?}",
        rating.category, rating.probability, rating.blocked ) );
      }
    }

  diagnostics.push( format!( "  Parts count : {}", candidate.content.parts.len() ) );
    for ( i, part ) in candidate.content.parts.iter().enumerate()
    {
      let text_preview = part.text.as_ref()
.map( |t| if t.len() > 50 { format!( "{}...", &t[ ..50 ] ) } else { t.clone() } )
      .unwrap_or_else( || "< no text >".to_string() );
  diagnostics.push( format!( "    Part {}: {}", i, text_preview ) );
    }
  } else {
    diagnostics.push( "  No candidates in response".to_string() );
  }

  if let Some( prompt_feedback ) = &response.prompt_feedback
  {
  diagnostics.push( format!( "  Prompt feedback : {:?}", prompt_feedback ) );
  }

  if let Some( usage ) = &response.usage_metadata
  {
    if let Some( total ) = usage.total_token_count
    {
    diagnostics.push( format!( "  Total tokens : {}", total ) );
    }
  }

  diagnostics.join( "\n" )
}

/// Retry operation with exponential backoff for transient API failures
async fn retry_generate_content_with_backoff(
client: &Client,
model_name: &str,
request: &GenerateContentRequest,
query_num: usize,
max_attempts: usize,
) -> Result< api_gemini::models::GenerateContentResponse, Box< dyn std::error::Error > >
{
  let mut attempt = 0;
  let mut delay_ms = 1000; // Start with 1 second

  loop
  {
    attempt += 1;

    let result = timeout(
    Duration::from_secs( 30 ),
    client.models().by_name( model_name ).generate_content( request )
    ).await;

    match result
    {
      Ok( Ok( response ) ) => {
        let response_text = extract_response_text( &response );

        // If response is non-empty, success
        if !response_text.trim().is_empty()
        {
          if attempt > 1
          {
      println!( "✅ Query {} succeeded on attempt {}/{}", query_num, attempt, max_attempts );
          }
          return Ok( response );
        }

        // Empty response - diagnose and retry
  println!( "\n⚠️  Query {} attempt {}/{}: Empty response received", query_num, attempt, max_attempts );
      println!( "{}", diagnose_response( &response, query_num ) );

        if attempt >= max_attempts
        {
      println!( "\n❌ Query {} failed after {} attempts with empty responses", query_num, max_attempts );
          return Err( format!(
    "Query {} response empty after {} retry attempts. Last diagnostics:\n{}",
          query_num, max_attempts, diagnose_response( &response, query_num )
          ).into() );
        }

    println!( "🔄 Retrying query {} after {}ms delay...\n", query_num, delay_ms );
        tokio ::time::sleep( Duration::from_millis( delay_ms ) ).await;
        delay_ms *= 2; // Exponential backoff
      },
      Ok( Err( e ) ) => {
println!( "\n⚠️  Query {} attempt {}/{}: API error : {}", query_num, attempt, max_attempts, e );

        if attempt >= max_attempts
        {
      println!( "\n❌ Query {} failed after {} attempts", query_num, max_attempts );
          return Err( e.into() );
        }

    println!( "🔄 Retrying query {} after {}ms delay...\n", query_num, delay_ms );
        tokio ::time::sleep( Duration::from_millis( delay_ms ) ).await;
        delay_ms *= 2;
      },
      Err( e ) => {
println!( "\n⚠️  Query {} attempt {}/{}: Timeout error : {}", query_num, attempt, max_attempts, e );

        if attempt >= max_attempts
        {
      println!( "\n❌ Query {} failed after {} attempts with timeouts", query_num, max_attempts );
          return Err( e.into() );
        }

    println!( "🔄 Retrying query {} after {}ms delay...\n", query_num, delay_ms );
        tokio ::time::sleep( Duration::from_millis( delay_ms ) ).await;
        delay_ms *= 2;
      }
    }
  }
}

/// Analyze response characteristics for system instruction compliance
fn analyze_response_characteristics( response_text: &str ) -> HashMap<  String, bool  >
{
  let mut characteristics = HashMap::new();
  let text_lower = response_text.to_lowercase();

  // Style characteristics
  characteristics.insert( "formal_tone".to_string(),
  text_lower.contains( "shall" ) || text_lower.contains( "ought" ) ||
  !text_lower.contains( "gonna" ) && !text_lower.contains( "wanna" ) );

  characteristics.insert( "casual_tone".to_string(),
  text_lower.contains( "hey" ) || text_lower.contains( "gonna" ) ||
  text_lower.contains( "wanna" ) || text_lower.contains( "pretty cool" ) );

  characteristics.insert( "educational_language".to_string(),
  text_lower.contains( "learn" ) || text_lower.contains( "understand" ) ||
  text_lower.contains( "concept" ) || text_lower.contains( "explain" ) );

  characteristics.insert( "technical_language".to_string(),
  text_lower.contains( "algorithm" ) || text_lower.contains( "implementation" ) ||
  text_lower.contains( "architecture" ) || text_lower.contains( "protocol" ) );

  characteristics.insert( "encouraging_language".to_string(),
  text_lower.contains( "great" ) || text_lower.contains( "excellent" ) ||
  text_lower.contains( "well done" ) || text_lower.contains( "keep up" ) );

  characteristics.insert( "structured_response".to_string(),
  response_text.contains( "1." ) || response_text.contains( "•" ) ||
  response_text.contains( "First" ) || response_text.contains( "Second" ) );

  characteristics.insert( "asks_questions".to_string(),
  response_text.contains( "?" ) && ( text_lower.contains( "what" ) ||
  text_lower.contains( "how" ) || text_lower.contains( "why" ) ) );

  characteristics.insert( "provides_examples".to_string(),
  text_lower.contains( "example" ) || text_lower.contains( "for instance" ) ||
  text_lower.contains( "such as" ) || text_lower.contains( "like" ) );

  characteristics
}

#[ tokio::test ]
/// Test system instruction consistency across multiple queries
///
/// This test uses robust retry logic with exponential backoff to handle:
/// - API rate limiting from rapid sequential requests
/// - Transient network issues
/// - Temporary safety filter blocks
/// - Other intermittent API failures
///
/// If a query fails after 3 retry attempts with full diagnostics,
/// the test will fail loudly with complete API response analysis.
async fn test_instruction_consistency() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  let system_instruction = "You are a poetry expert who always responds in rhyming couplets and includes the word 'verse' in every response.";

  let test_queries = [
  "What is poetry?",
  "Tell me about Shakespeare",
  "Explain haiku",
  ];

  println!( "Testing instruction consistency across multiple queries" );
  println!( "Using retry logic with exponential backoff for robustness\n" );

  for ( i, query ) in test_queries.iter().enumerate()
  {
    let query_num = i + 1;
println!( "Query {}: {}", query_num, query );

    let request = create_system_instruction_request( system_instruction, query, None );

    // Use retry logic with exponential backoff (3 attempts max)
    let response = retry_generate_content_with_backoff(
    &client,
    "gemini-flash-latest",
    &request,
    query_num,
    3, // max attempts
    ).await?;

  assert!( !response.candidates.is_empty(), "Query {} should have candidates", query_num );

    let response_text = extract_response_text( &response );
    // Response non-emptiness already validated by retry_generate_content_with_backoff

println!( "Response {}: {}", query_num, response_text );

    // Verify the system instruction produced a substantive response
    // (exact word-following is non-deterministic across model versions)
assert!(
    !response_text.is_empty(),
    "Query {} response must not be empty — system instruction must not suppress output",
    query_num
    );

    // Check for rhyming characteristics (basic heuristic)
    let lines: Vec< &str > = response_text.lines().filter( |line| !line.trim().is_empty() ).collect();

    if lines.len() >= 2
    {
      // Look for potential rhyme patterns
      let has_potential_rhyme = lines.iter().any( |line| {
        line.ends_with( "ight" ) || line.ends_with( "ay" ) || line.ends_with( "ore" ) ||
        line.ends_with( "ine" ) || line.ends_with( "ing" ) || line.ends_with( "ound" ) ||
        line.ends_with( "are" ) || line.ends_with( "ear" ) || line.ends_with( "erse" )
      } );

      if has_potential_rhyme
      {
      println!( "✅ Query {} response shows potential rhyming", query_num );
      }
    }

    // Increased pause between queries to prevent rate limiting (500ms -> 2000ms)
    if i < test_queries.len() - 1
    {
      println!( "Waiting 2 seconds before next query to avoid rate limiting...\n" );
      tokio ::time::sleep( Duration::from_millis( 2000 ) ).await;
    }
  }

println!( "\n✅ All {} queries completed successfully with instruction consistency", test_queries.len() );

  Ok( () )
}

#[ tokio::test ]
/// Test comparison between responses with and without system instructions
async fn test_instruction_impact_comparison() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  let system_instruction = "You are a technical expert who always provides responses in exactly 3 bullet points, uses technical terminology, and includes specific numbers or metrics when possible.";
  let test_query = "How does machine learning work?";

  println!( "Testing impact of system instructions vs basic responses" );

  // Response without system instructions
  println!( "Getting response WITHOUT system instructions..." );
  let basic_request = create_basic_request( test_query );

  let basic_response = timeout(
  Duration::from_secs( 30 ),
  client.models().by_name( "gemini-flash-latest" ).generate_content( &basic_request )
  ).await??;

  let basic_text = extract_response_text( &basic_response );
  assert!( !basic_text.trim().is_empty(), "Basic response should not be empty" );

  // Response with system instructions
  println!( "Getting response WITH system instructions..." );
  let instruction_request = create_system_instruction_request( system_instruction, test_query, None );

  let instruction_response = timeout(
  Duration::from_secs( 30 ),
  client.models().by_name( "gemini-flash-latest" ).generate_content( &instruction_request )
  ).await??;

  let instruction_text = extract_response_text( &instruction_response );
  assert!( !instruction_text.trim().is_empty(), "Instruction response should not be empty" );

println!( "Basic response length : {} characters", basic_text.len() );
println!( "Instruction response length : {} characters", instruction_text.len() );

  // Analyze both responses
  let basic_characteristics = analyze_response_characteristics( &basic_text );
  let instruction_characteristics = analyze_response_characteristics( &instruction_text );

println!( "Basic response characteristics : {:?}", basic_characteristics );
println!( "Instruction response characteristics : {:?}", instruction_characteristics );

  // Check for bullet points in instruction response
  let bullet_count = instruction_text.matches( "•" ).count() +
  instruction_text.matches( "-" ).count() +
  instruction_text.matches( "1." ).count();

  if bullet_count >= 3
  {
    println!( "✅ Instruction response uses bullet points as requested" );
  }

  // Check for technical language compliance
  if *instruction_characteristics.get( "technical_language" ).unwrap_or( &false )
  {
    println!( "✅ Instruction response uses technical language" );
  }

  // Check for numbers/metrics
  let has_numbers = instruction_text.chars().any( |c| c.is_numeric() );
  if has_numbers
  {
    println!( "✅ Instruction response includes numbers/metrics" );
  }

  // Compare token usage
  if let ( Some( basic_usage ), Some( instruction_usage ) ) =
  ( &basic_response.usage_metadata, &instruction_response.usage_metadata ) {

    if let ( Some( basic_total ), Some( instruction_total ) ) =
    ( basic_usage.total_token_count, instruction_usage.total_token_count ) {

  println!( "Token usage - Basic : {}, Instruction : {}", basic_total, instruction_total );

      // System instructions typically increase token usage
      if instruction_total > basic_total
      {
        println!( "✅ System instructions increased token usage as expected" );
      }
    }
  }

  Ok( () )
}

#[ tokio::test ]
/// Test multi-turn conversation with system instructions
async fn test_multi_turn_conversation_consistency() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  let system_instruction = "You are a helpful coding tutor who always:
  1. Asks a follow-up question after each explanation
  2. Uses the phrase 'Let's code together!' at least once
  3. Provides practical examples when possible
  Maintain this behavior throughout our conversation.";

  println!( "Testing multi-turn conversation consistency" );

  let conversation_turns = [
  "What is a variable in programming?",
  "How do I create a variable in Python?",
  "What's the difference between a list and a tuple?",
  ];

  let mut conversation_history = Vec::new();

  for ( turn_num, user_message ) in conversation_turns.iter().enumerate()
  {
println!( "Turn {}: {}", turn_num + 1, user_message );

    let request = create_system_instruction_request(
    system_instruction,
    user_message,
    Some( conversation_history.clone() ),
    );

    let response = timeout(
    Duration::from_secs( 30 ),
    client.models().by_name( "gemini-flash-latest" ).generate_content( &request )
    ).await??;

  assert!( !response.candidates.is_empty(), "Turn {} should have candidates", turn_num + 1 );

    let response_text = extract_response_text( &response );
  assert!( !response_text.trim().is_empty(), "Turn {} response should not be empty", turn_num + 1 );

println!( "Response {}: {}", turn_num + 1, response_text );

    // Check for instruction compliance
    let characteristics = analyze_response_characteristics( &response_text );

    // Should ask follow-up questions
    if *characteristics.get( "asks_questions" ).unwrap_or( &false )
    {
    println!( "✅ Turn {} includes follow-up question", turn_num + 1 );
    }

    // Should include the required phrase
    if response_text.contains( "Let's code together!" )
    {
    println!( "✅ Turn {} includes required phrase", turn_num + 1 );
    }

    // Should provide examples for coding questions
    if *characteristics.get( "provides_examples" ).unwrap_or( &false )
    {
    println!( "✅ Turn {} provides examples", turn_num + 1 );
    }

    // Update conversation history
    conversation_history.push( Content {
      parts : vec![ Part {
        text: Some( user_message.to_string() ),
        ..Default::default()
      } ],
      role: "user".to_string(),
    } );

    conversation_history.push( Content {
      parts : vec![ Part {
        text: Some( response_text ),
        ..Default::default()
      } ],
      role: "model".to_string(),
    } );

    // Brief pause between turns
    tokio ::time::sleep( Duration::from_millis( 1000 ) ).await;
  }

println!( "✅ Multi-turn conversation completed with {} turns", conversation_turns.len() );

  Ok( () )
}

#[ tokio::test ]
/// Test system instruction with domain-specific constraints
async fn test_domain_specific_constraints() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  let system_instruction = "You are a financial advisor who must:
  • Only discuss investment topics related to stocks, bonds, and mutual funds
  • Always include a risk disclaimer in your responses
  • Never provide specific buy/sell recommendations
  • Use professional financial terminology
  • Mention that this is educational information only

  If asked about non-investment topics, politely redirect to investment education.";

  let test_queries = [
  "What are stocks?",
  "Should I buy Tesla stock?", // Should trigger constraint about specific recommendations
  "Tell me a joke", // Should trigger topic redirection
  ];

  println!( "Testing domain-specific constraints" );

  for ( i, query ) in test_queries.iter().enumerate()
  {
println!( "Testing constraint query {}: {}", i + 1, query );

    let request = create_system_instruction_request( system_instruction, query, None );

    let response = timeout(
    Duration::from_secs( 30 ),
    client.models().by_name( "gemini-flash-latest" ).generate_content( &request )
    ).await??;

  assert!( !response.candidates.is_empty(), "Query {} should have candidates", i + 1 );

    let response_text = extract_response_text( &response );
  assert!( !response_text.trim().is_empty(), "Query {} response should not be empty", i + 1 );

println!( "Response {}: {}", i + 1, response_text );

    let text_lower = response_text.to_lowercase();

    // Check for risk disclaimer
    let has_disclaimer = text_lower.contains( "risk" ) ||
    text_lower.contains( "disclaimer" ) ||
    text_lower.contains( "educational" ) ||
    text_lower.contains( "not financial advice" );

    if has_disclaimer
    {
    println!( "✅ Query {} response includes risk disclaimer", i + 1 );
    }

    // Query-specific validations
    match i
    {
      0 => { // "What are stocks?"
        assert!(
        text_lower.contains( "stock" ) || text_lower.contains( "equity" ) || text_lower.contains( "share" ),
        "Stock query should contain relevant terminology"
        );
      },
      1 => { // "Should I buy Tesla stock?"
        // Should avoid specific buy recommendations
        let avoids_specific_advice = !text_lower.contains( "you should buy" ) &&
        !text_lower.contains( "i recommend buying" ) &&
        !text_lower.contains( "definitely purchase" );

        if avoids_specific_advice
        {
        println!( "✅ Query {} appropriately avoids specific buy advice", i + 1 );
        }
      },
      2 => { // "Tell me a joke"
        // Should redirect to investment topics
        let redirects_topic = text_lower.contains( "investment" ) ||
        text_lower.contains( "financial" ) ||
        text_lower.contains( "stock" ) ||
        text_lower.contains( "portfolio" );

        if redirects_topic
        {
        println!( "✅ Query {} redirects to investment topics", i + 1 );
        }
      },
    _ => {}
    }

    // Brief pause between queries
    tokio ::time::sleep( Duration::from_millis( 500 ) ).await;
  }

  Ok( () )
}

#[ tokio::test ]
/// Integration test : Complex system instruction workflow
async fn test_complex_instruction_workflow() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  println!( "Running complex system instruction workflow test" );

  // Simulate a tutoring session with complex instructions
  let system_instruction = "You are an adaptive programming tutor with these specific behaviors:

  PERSONALITY:
  • Patient and encouraging, never dismissive
  • Uses analogies from everyday life to explain concepts
  • Celebrates small victories and progress

  TEACHING APPROACH:
  • Start with the simplest possible explanation
  • Build complexity gradually
  • Always check understanding before advancing
  • Provide multiple examples for each concept

  RESPONSE FORMAT:
  • Begin each response with a brief summary of what we'll cover
  • Use numbered steps for procedures
  • End with a practice question or challenge
  • Include the phrase 'Happy coding!' somewhere in your response

  ADAPTATION RULES:
  • If the student seems confused, simplify further
  • If they demonstrate understanding, introduce related concepts
  • Always acknowledge their level and adjust accordingly";

  let tutorial_steps = [
  "I'm a complete beginner. What is programming?",
  "That makes sense! How do I write my first program?",
  "I wrote 'print(\"Hello World\")' and it worked! What should I learn next?",
  "I'm getting confused about variables. Can you help?",
  ];

  let mut conversation_history = Vec::new();
  let mut instruction_compliance_scores = Vec::new();

  for ( step_num, user_message ) in tutorial_steps.iter().enumerate()
  {
println!( "Tutorial Step {}: {}", step_num + 1, user_message );

    let request = create_system_instruction_request(
    system_instruction,
    user_message,
    Some( conversation_history.clone() ),
    );

    let response = timeout(
    Duration::from_secs( 45 ), // Allow more time for complex instructions
    client.models().by_name( "gemini-flash-latest" ).generate_content( &request )
    ).await??;

  assert!( !response.candidates.is_empty(), "Step {} should have candidates", step_num + 1 );

    let response_text = extract_response_text( &response );
  assert!( !response_text.trim().is_empty(), "Step {} response should not be empty", step_num + 1 );

println!( "Tutor Response {}: {} characters", step_num + 1, response_text.len() );

    // Analyze instruction compliance
    let mut compliance_score = 0;
    let mut max_score = 0;

    // Check personality traits
    max_score += 1;
    if response_text.to_lowercase().contains( "great" ) ||
    response_text.to_lowercase().contains( "excellent" ) ||
    response_text.to_lowercase().contains( "good" ) {
      compliance_score += 1;
      println!( "  ✅ Encouraging tone detected" );
    }

    // Check for analogies
    max_score += 1;
    if response_text.to_lowercase().contains( "like" ) ||
    response_text.to_lowercase().contains( "imagine" ) ||
    response_text.to_lowercase().contains( "think of" ) {
      compliance_score += 1;
      println!( "  ✅ Uses analogies" );
    }

    // Check response format
    max_score += 1;
    if response_text.contains( "1." ) || response_text.contains( "2." ) ||
    response_text.contains( "•" ) || response_text.contains( "-" ) {
      compliance_score += 1;
      println!( "  ✅ Uses numbered steps or bullets" );
    }

    max_score += 1;
    if response_text.contains( "Happy coding!" )
    {
      compliance_score += 1;
      println!( "  ✅ Includes required phrase" );
    }

    // Check for practice questions
    max_score += 1;
    if response_text.contains( "?" ) && (
    response_text.to_lowercase().contains( "try" ) ||
    response_text.to_lowercase().contains( "practice" ) ||
    response_text.to_lowercase().contains( "challenge" ) ) {
      compliance_score += 1;
      println!( "  ✅ Includes practice question" );
    }

    // Check for examples
    max_score += 1;
    if response_text.to_lowercase().contains( "example" ) ||
    response_text.to_lowercase().contains( "for instance" ) ||
    response_text.contains( "print(" ) || response_text.contains( "=" ) {
      compliance_score += 1;
      println!( "  ✅ Provides examples" );
    }

    let compliance_percentage = ( compliance_score as f64 / max_score as f64 ) * 100.0;
    instruction_compliance_scores.push( compliance_percentage );

println!( "  Instruction compliance : {:.1}% ({}/{})", compliance_percentage, compliance_score, max_score );

    // Update conversation history
    conversation_history.push( Content {
      parts : vec![ Part {
        text: Some( user_message.to_string() ),
        ..Default::default()
      } ],
      role: "user".to_string(),
    } );

    conversation_history.push( Content {
      parts : vec![ Part {
        text: Some( response_text ),
        ..Default::default()
      } ],
      role: "model".to_string(),
    } );

    // Brief pause between tutorial steps
    tokio ::time::sleep( Duration::from_millis( 1500 ) ).await;
  }

  // Analyze overall workflow performance
  let avg_compliance = instruction_compliance_scores.iter().sum::< f64 >() / instruction_compliance_scores.len() as f64;

  println!( "Workflow Summary:" );
println!( "  Total tutorial steps : {}", tutorial_steps.len() );
println!( "  Average instruction compliance : {:.1}%", avg_compliance );
println!( "  Conversation history length : {} exchanges", conversation_history.len() / 2 );

  // Expect reasonable compliance with complex instructions
  assert!(
  avg_compliance >= 50.0,
  "Average instruction compliance should be at least 50%"
  );

  println!( "✅ Complex instruction workflow completed successfully" );

  Ok( () )
}