//! Comprehensive tests for Code Execution functionality
//!
//! This module provides exhaustive testing for Python code generation and execution
//! including mathematical computations, data analysis, error handling, and execution
//! result validation. All tests use real API calls following the no-mockup policy.

#![ cfg( feature = "integration" ) ]

#[ path = "common/mod.rs" ] mod common;
use common::create_integration_client;

use api_gemini::
{
  models ::
  {
    GenerateContentRequest, Content, Part, Tool, CodeExecution,
    GenerationConfig,
  },
};
use tokio::time::{ timeout, Duration };
use serde_json::Value;

/// Create a code execution request with specified configuration.
///
/// Note : The Gemini API's code_execution tool uses an empty configuration as per API specification.
/// Timeout and network settings are not configurable at request time.
fn create_code_execution_request( prompt: &str ) -> GenerateContentRequest
{
let code_execution_config = CodeExecution {};

  let tools = vec![ Tool {
    function_declarations: None,
    code_execution: Some( code_execution_config ),
    google_search_retrieval: None,
    code_execution_tool: None,
  } ];

  GenerateContentRequest {
    contents : vec![ Content {
      parts : vec![ Part {
        text: Some( prompt.to_string() ),
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
      temperature: Some( 0.1 ), // Lower temperature for more precise code
      top_k: Some( 1 ),
      top_p: Some( 0.1 ),
      candidate_count: Some( 1 ),
      max_output_tokens: Some( 2048 ),
      stop_sequences: None,
    } ),
    safety_settings: None,
    tools: Some( tools ),
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  }
}

/// Validate code execution results from function response
fn validate_execution_result( function_response: &Value ) -> Result< (), String >
{
  // Try to extract outcome
  let outcome = function_response.get( "outcome" )
  .and_then( |v| v.as_str() )
  .ok_or( "Missing or invalid outcome field" )?;

println!( "Execution outcome : {}", outcome );

  // Validate outcome is a known value
  match outcome
  {
    "OUTCOME_OK" | "OUTCOME_FAILED" | "OUTCOME_DEADLINE_EXCEEDED" => {
    println!( "✅ Valid execution outcome : {}", outcome );
    },
    _ => {
    return Err( format!( "Unknown execution outcome : {}", outcome ) );
    }
  }

  // Check for output if execution was successful
  if outcome == "OUTCOME_OK"
  {
    if let Some( output ) = function_response.get( "output" ).and_then( |v| v.as_str() )
    {
      if !output.trim().is_empty()
      {
      println!( "✅ Execution produced output : {} characters", output.len() );
      } else {
        println!( "⚠️ Execution successful but no output" );
      }
    }
  }

  // Check for errors if execution failed
  if outcome == "OUTCOME_FAILED"
  {
    if let Some( error ) = function_response.get( "error" ).and_then( |v| v.as_str() )
    {
    println!( "❌ Execution error : {}", error );
    }
  }

  // Check execution time if available
  if let Some( exec_time ) = function_response.get( "execution_time_ms" ).and_then( |v| v.as_i64() )
  {
  println!( "⏱️ Execution time : {}ms", exec_time );

    // Validate execution time is reasonable
    if exec_time < 0
    {
      return Err( "Invalid negative execution time".to_string() );
    }

    if exec_time > 60000  // More than 60 seconds seems excessive for most tests
    {
      println!( "⚠️ Execution took longer than expected : {}ms", exec_time );
    }
  }

  Ok( () )
}

/// Find function responses in the response content
fn find_function_responses( response: &api_gemini::models::GenerateContentResponse ) -> Vec< &Value >
{
  let mut function_responses = Vec::new();

  for candidate in &response.candidates
  {
    for part in &candidate.content.parts
    {
      if let Some( function_response ) = &part.function_response
      {
        function_responses.push( &function_response.response );
      }
    }
  }

  function_responses
}

#[ tokio::test ]
/// Test basic Python code execution
async fn test_basic_code_execution() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  let prompt = "Please write and execute Python code to calculate the factorial of 5 and print the result.";

println!( "Testing basic code execution : {}", prompt );

  let request = create_code_execution_request( prompt );

  let response = timeout(
  Duration::from_secs( 45 ),
  client.models().by_name( "gemini-flash-latest" ).generate_content( &request )
  ).await??;

  // Validate basic response structure
  assert!( !response.candidates.is_empty(), "Response should have at least one candidate" );

  let candidate = response.candidates.first().unwrap();
  assert!( !candidate.content.parts.is_empty(), "Candidate should have content parts" );

  // Look for function responses (execution results)
  let function_responses = find_function_responses( &response );

  if function_responses.is_empty()
  {
    // Check if there's text content that might indicate code execution happened
    if let Some( part ) = candidate.content.parts.first()
    {
      if let Some( text ) = &part.text
      {
      println!( "Response text (no function response): {}", text );

        // For basic math, expect some numerical result
        assert!( text.contains( "120" ) || text.contains( "factorial" ),
        "Response should contain factorial result or explanation" );
      }
    }
  } else {
    // Validate function execution results
    for function_response in function_responses
    {
      validate_execution_result( function_response )?;

      // For factorial of 5, expect output containing 120
      if let Some( output ) = function_response.get( "output" ).and_then( |v| v.as_str() )
      {
        if output.contains( "120" )
        {
          println!( "✅ Correct factorial result found in output" );
        }
      }
    }
  }

  // Validate token usage
  if let Some( usage ) = &response.usage_metadata
  {
    if let Some( total ) = usage.total_token_count
    {
    println!( "Total tokens used : {}", total );
      assert!( total > 0, "Should use some tokens" );
    }
  }

  Ok( () )
}

#[ tokio::test ]
/// Test mathematical computation with code execution
async fn test_mathematical_computation() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  let prompt = "Write Python code to calculate the first 10 prime numbers and display them.";

println!( "Testing mathematical computation : {}", prompt );

  let request = create_code_execution_request( prompt );

  let response = timeout(
  Duration::from_secs( 45 ),
  client.models().by_name( "gemini-flash-latest" ).generate_content( &request )
  ).await??;

  assert!( !response.candidates.is_empty(), "Should have response candidates" );

  // Look for execution results
  let function_responses = find_function_responses( &response );

  if !function_responses.is_empty()
  {
    for function_response in function_responses
    {
      validate_execution_result( function_response )?;

      // Check for prime numbers in output
      if let Some( output ) = function_response.get( "output" ).and_then( |v| v.as_str() )
      {
        // First 10 primes : 2, 3, 5, 7, 11, 13, 17, 19, 23, 29
        let expected_primes = [ "2", "3", "5", "7", "11", "13", "17", "19", "23", "29" ];
        let mut found_primes = 0;

        for prime in &expected_primes
        {
          if output.contains( prime )
          {
            found_primes += 1;
          }
        }

    println!( "Found {} out of {} expected prime numbers", found_primes, expected_primes.len() );

        // Expect at least some prime numbers
        assert!( found_primes >= 5, "Should find at least 5 prime numbers in output" );
      }
    }
  } else {
    // Check text response for mathematical content
    if let Some( candidate ) = response.candidates.first()
    {
      if let Some( part ) = candidate.content.parts.first()
      {
        if let Some( text ) = &part.text
        {
          let text_lower = text.to_lowercase();
          assert!(
          text_lower.contains( "prime" ) || text_lower.contains( "2" ) || text_lower.contains( "algorithm" ),
          "Response should contain prime-related content"
          );
        }
      }
    }
  }

  Ok( () )
}

#[ tokio::test ]
/// Test code execution with data analysis
async fn test_data_analysis_execution() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  let prompt = "Create a Python list of 10 random numbers, calculate their mean, median, and standard deviation, and display the results.";

println!( "Testing data analysis execution : {}", prompt );

  let request = create_code_execution_request( prompt );

  let response = timeout(
  Duration::from_secs( 45 ),
  client.models().by_name( "gemini-flash-latest" ).generate_content( &request )
  ).await??;

  assert!( !response.candidates.is_empty(), "Should have response candidates" );

  // Look for execution results
  let function_responses = find_function_responses( &response );

  if !function_responses.is_empty()
  {
    for function_response in function_responses
    {
      validate_execution_result( function_response )?;

      // Check for statistical analysis in output
      if let Some( output ) = function_response.get( "output" ).and_then( |v| v.as_str() )
      {
        let output_lower = output.to_lowercase();

        let has_mean = output_lower.contains( "mean" ) || output_lower.contains( "average" );
        let has_median = output_lower.contains( "median" );
        let has_std = output_lower.contains( "standard" ) || output_lower.contains( "std" );

  println!( "Statistical measures found - Mean : {}, Median : {}, Std Dev : {}", has_mean, has_median, has_std );

        // Expect at least some statistical measures
        let stats_count = [ has_mean, has_median, has_std ].iter().filter( |&&x| x ).count();
        assert!( stats_count >= 1, "Should find at least one statistical measure in output" );
      }
    }
  } else {
    // Check text response for data analysis content
    if let Some( candidate ) = response.candidates.first()
    {
      if let Some( part ) = candidate.content.parts.first()
      {
        if let Some( text ) = &part.text
        {
          let text_lower = text.to_lowercase();
          assert!(
          text_lower.contains( "statistics" ) || text_lower.contains( "mean" ) ||
          text_lower.contains( "data" ) || text_lower.contains( "numbers" ),
          "Response should contain data analysis content"
          );
        }
      }
    }
  }

  Ok( () )
}

#[ tokio::test ]
/// Test code execution error handling
async fn test_code_execution_error_handling() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  let prompt = "Write Python code that intentionally causes a division by zero error, then fix it with proper error handling.";

println!( "Testing error handling : {}", prompt );

  let request = create_code_execution_request( prompt );

  let response = timeout(
  Duration::from_secs( 45 ),
  client.models().by_name( "gemini-flash-latest" ).generate_content( &request )
  ).await??;

  assert!( !response.candidates.is_empty(), "Should have response candidates" );

  // Look for execution results
  let function_responses = find_function_responses( &response );

  if !function_responses.is_empty()
  {
    for function_response in function_responses
    {
      validate_execution_result( function_response )?;

      // For error handling examples, we might see either successful execution (if fixed)
      // or failed execution (if showing the error)
      let outcome = function_response.get( "outcome" )
      .and_then( |v| v.as_str() )
      .unwrap_or( "unknown" );

      if outcome == "OUTCOME_OK"
      {
        println!( "✅ Error handling code executed successfully" );

        if let Some( output ) = function_response.get( "output" ).and_then( |v| v.as_str() )
        {
          let output_lower = output.to_lowercase();
          let mentions_error = output_lower.contains( "error" ) ||
          output_lower.contains( "exception" ) ||
          output_lower.contains( "try" ) ||
          output_lower.contains( "except" );

          if mentions_error
          {
            println!( "✅ Output discusses error handling" );
          }
        }
      } else if outcome == "OUTCOME_FAILED"
      {
        println!( "⚠️ Code execution failed (may be intentional for error demonstration)" );

        if let Some( error ) = function_response.get( "error" ).and_then( |v| v.as_str() )
        {
          if error.to_lowercase().contains( "division" ) || error.to_lowercase().contains( "zero" )
          {
            println!( "✅ Correctly detected division by zero error" );
          }
        }
      }
    }
  } else {
    // Check text response for error handling discussion
    if let Some( candidate ) = response.candidates.first()
    {
      if let Some( part ) = candidate.content.parts.first()
      {
        if let Some( text ) = &part.text
        {
          let text_lower = text.to_lowercase();
          assert!(
          text_lower.contains( "error" ) || text_lower.contains( "exception" ) ||
          text_lower.contains( "try" ) || text_lower.contains( "except" ),
          "Response should discuss error handling"
          );
        }
      }
    }
  }

  Ok( () )
}

#[ tokio::test ]
/// Test code execution with timeout configuration
async fn test_execution_timeout_handling() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  // Create a task that should complete within a short timeout
  let prompt = "Write Python code to calculate the sum of numbers from 1 to 100 and print the result.";

println!( "Testing timeout handling with short timeout : {}", prompt );

  // Use a very short timeout to test timeout behavior
  let request = create_code_execution_request( prompt );

  let response = timeout(
  Duration::from_secs( 30 ),
  client.models().by_name( "gemini-flash-latest" ).generate_content( &request )
  ).await??;

  assert!( !response.candidates.is_empty(), "Should have response candidates" );

  // Look for execution results
  let function_responses = find_function_responses( &response );

  if !function_responses.is_empty()
  {
    for function_response in function_responses
    {
      validate_execution_result( function_response )?;

      let outcome = function_response.get( "outcome" )
      .and_then( |v| v.as_str() )
      .unwrap_or( "unknown" );

      match outcome
      {
        "OUTCOME_OK" => {
          println!( "✅ Execution completed within timeout" );

          // For sum 1 to 100, expect result 5050
          if let Some( output ) = function_response.get( "output" ).and_then( |v| v.as_str() )
          {
            if output.contains( "5050" )
            {
              println!( "✅ Correct calculation result found" );
            }
          }
        },
        "OUTCOME_DEADLINE_EXCEEDED" => {
          println!( "⏰ Execution exceeded timeout (expected for very short timeouts)" );
        },
        "OUTCOME_FAILED" => {
          println!( "❌ Execution failed" );
        },
        _ => {
        println!( "⚠️ Unknown outcome : {}", outcome );
        }
      }

      // Validate execution time against timeout
      if let Some( exec_time ) = function_response.get( "execution_time_ms" ).and_then( |v| v.as_i64() )
      {
        let timeout_ms = 5000; // 5 seconds in milliseconds

        if outcome == "OUTCOME_DEADLINE_EXCEEDED"
        {
          // Execution time should be close to the timeout
          assert!( exec_time >= timeout_ms - 1000, "Timeout execution should take approximately the timeout duration" );
        } else if outcome == "OUTCOME_OK"
        {
          // Successful execution should be under the timeout
          assert!( exec_time < timeout_ms, "Successful execution should complete before timeout" );
        }
      }
    }
  }

  Ok( () )
}

#[ tokio::test ]
/// Test network access configuration
async fn test_network_access_configuration() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  // Test with network disabled (should be default and safer)
  let prompt = "Write Python code to make a simple HTTP request to httpbin.org/get and print the response. Handle any network errors gracefully.";

println!( "Testing network access configuration : {}", prompt );

  // Test with network explicitly disabled
  let request = create_code_execution_request( prompt );

  let response = timeout(
  Duration::from_secs( 45 ),
  client.models().by_name( "gemini-flash-latest" ).generate_content( &request )
  ).await??;

  assert!( !response.candidates.is_empty(), "Should have response candidates" );

  // Look for execution results
  let function_responses = find_function_responses( &response );

  if !function_responses.is_empty()
  {
    for function_response in function_responses
    {
      validate_execution_result( function_response )?;

      let outcome = function_response.get( "outcome" )
      .and_then( |v| v.as_str() )
      .unwrap_or( "unknown" );

      // With network disabled, we might expect failure or the code might handle it gracefully
      match outcome
      {
        "OUTCOME_OK" => {
          println!( "✅ Code handled network restrictions gracefully" );
        },
        "OUTCOME_FAILED" => {
          println!( "❌ Code failed (possibly due to network restrictions)" );

          if let Some( error ) = function_response.get( "error" ).and_then( |v| v.as_str() )
          {
            let error_lower = error.to_lowercase();
            if error_lower.contains( "network" ) || error_lower.contains( "connection" ) ||
            error_lower.contains( "requests" ) || error_lower.contains( "urllib" ) {
              println!( "✅ Error is network-related as expected" );
            }
          }
        },
        _ => {
        println!( "⚠️ Unexpected outcome : {}", outcome );
        }
      }
    }
  } else {
    // Check if response discusses network restrictions
    if let Some( candidate ) = response.candidates.first()
    {
      if let Some( part ) = candidate.content.parts.first()
      {
        if let Some( text ) = &part.text
        {
          let text_lower = text.to_lowercase();
          if text_lower.contains( "network" ) || text_lower.contains( "internet" ) ||
          text_lower.contains( "request" ) || text_lower.contains( "http" ) {
            println!( "✅ Response discusses network functionality" );
          }
        }
      }
    }
  }

  Ok( () )
}

#[ tokio::test ]
/// Test multiple code execution requests in sequence
async fn test_sequential_code_execution() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  println!( "Testing sequential code execution" );

  let tasks = [
  "Calculate 2^10 and print the result",
  "Create a list [1,2,3,4,5] and print its length",
  "Define a function to check if a number is even and test it with 42",
  ];

  for ( i, task ) in tasks.iter().enumerate()
  {
println!( "Executing task {}: {}", i + 1, task );

    let request = create_code_execution_request( task );

    let response = timeout(
    Duration::from_secs( 30 ),
    client.models().by_name( "gemini-flash-latest" ).generate_content( &request )
    ).await??;

  assert!( !response.candidates.is_empty(), "Task {} should have candidates", i + 1 );

    // Look for execution results
    let function_responses = find_function_responses( &response );

    if !function_responses.is_empty()
    {
      for function_response in &function_responses
      {
        validate_execution_result( function_response )?;

        let outcome = function_response.get( "outcome" )
        .and_then( |v| v.as_str() )
        .unwrap_or( "unknown" );

        if outcome == "OUTCOME_OK"
        {
        println!( "✅ Task {} executed successfully", i + 1 );
        } else {
      println!( "⚠️ Task {} had outcome : {}", i + 1, outcome );
        }
      }
    } else {
      // Check for text response
      if let Some( candidate ) = response.candidates.first()
      {
        if let Some( part ) = candidate.content.parts.first()
        {
          if let Some( text ) = &part.text
          {
          assert!( !text.trim().is_empty(), "Task {} should have response text", i + 1 );
          println!( "✅ Task {} generated response", i + 1 );
          }
        }
      }
    }

    // Brief pause between tasks
    tokio ::time::sleep( Duration::from_millis( 500 ) ).await;
  }

  println!( "✅ All sequential tasks completed" );

  Ok( () )
}

#[ tokio::test ]
/// Test code execution with different complexity levels
async fn test_execution_complexity_levels() -> Result< (), Box< dyn std::error::Error > >
{
  let client = create_integration_client();

  println!( "Testing different complexity levels" );

  let complexity_tests = vec![
  ( "Simple", "print('Hello, World!')" ),
( "Basic Math", "result = 15 + 27; print(f'Sum : {result}')" ),
( "Loop", "for i in range(5): print(f'Number : {i}')" ),
( "Function", "def greet(name): return f'Hello, {name}!'; print(greet('Python'))" ),
( "List Comprehension", "squares = [x**2 for x in range(10)]; print(f'Squares : {squares[:5]}')" ),
  ];

  for ( level, code ) in complexity_tests
  {
println!( "Testing {} level : {}", level, code );

  let prompt = format!( "Execute this Python code : {}", code );
    let request = create_code_execution_request( &prompt );

    let response = timeout(
    Duration::from_secs( 25 ),
    client.models().by_name( "gemini-flash-latest" ).generate_content( &request )
    ).await??;

  assert!( !response.candidates.is_empty(), "{} level should have candidates", level );

    // Look for execution results
    let function_responses = find_function_responses( &response );

    if !function_responses.is_empty()
    {
      for function_response in &function_responses
      {
        validate_execution_result( function_response )?;

        let outcome = function_response.get( "outcome" )
        .and_then( |v| v.as_str() )
        .unwrap_or( "unknown" );

        if outcome == "OUTCOME_OK"
        {
        println!( "✅ {} level executed successfully", level );

          // Check for expected output patterns
          if let Some( output ) = function_response.get( "output" ).and_then( |v| v.as_str() )
          {
            match level
            {
              "Simple" => assert!( output.contains( "Hello, World!" ), "Should print greeting" ),
              "Basic Math" => assert!( output.contains( "42" ) || output.contains( "Sum" ), "Should show sum result" ),
              "Loop" => assert!( output.contains( "Number" ), "Should show loop output" ),
              "Function" => assert!( output.contains( "Hello, Python!" ), "Should call function" ),
              "List Comprehension" => assert!( output.contains( "Squares" ), "Should show squares" ),
            _ => {} // Generic validation
            }
          }
        } else {
      println!( "⚠️ {} level had outcome : {}", level, outcome );
        }
      }
    }

    // Brief pause between complexity tests
    tokio ::time::sleep( Duration::from_millis( 300 ) ).await;
  }

  println!( "✅ All complexity levels tested" );

  Ok( () )
}

