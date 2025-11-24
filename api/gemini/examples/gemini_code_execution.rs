//! Python Code Generation and Execution Example
//!
//! This example demonstrates Gemini's code execution capabilities including:
//! - Python code generation and automatic execution
//! - Mathematical computations and data analysis
//! - File operations and data visualization
//! - Scientific computing with libraries
//! - Error handling and debugging assistance
//! - Security controls and execution timeouts
//! - Interactive coding assistance and optimization
//!
//! Usage:
//! ```bash
//! # Basic code execution
//! cargo run --example gemini_code_execution
//!
//! # Mathematical computation
//! cargo run --example gemini_code_execution -- --mode math --problem "Calculate fibonacci sequence up to 100"
//!
//! # Data analysis task
//! cargo run --example gemini_code_execution -- --mode data --task "Create a dataset and perform statistical analysis"
//!
//! # Scientific computing
//! cargo run --example gemini_code_execution -- --mode science --domain "physics" --problem "Simulate projectile motion"
//!
//! # Interactive coding session
//! cargo run --example gemini_code_execution -- --mode interactive
//! ```

use api_gemini::{ client::Client, models::* };
use std::env;
use tokio::time::{ timeout, Duration };
use std::io::{ self, Write };

/// Configuration for code execution examples
#[ derive( Debug, Clone ) ]
pub struct ExecutionExampleConfig
{
  /// Execution mode
  pub mode: ExecutionMode,
  /// Problem description for math mode
  pub problem: Option< String >,
  /// Task description for data mode
  pub task: Option< String >,
  /// Scientific domain
  pub domain: Option< String >,
  /// Execution timeout in seconds
  pub timeout_seconds: i32,
  /// Enable network access during execution
  pub enable_network: bool,
  /// Enable detailed execution logging
  pub detailed_logging: bool,
}

/// Code execution demonstration modes
#[ derive( Debug, Clone ) ]
pub enum ExecutionMode
{
  /// Basic Python code execution
  Basic,
  /// Mathematical computation and analysis
  Math,
  /// Data processing and visualization
  Data,
  /// Scientific computing demonstrations
  Science,
  /// Interactive code generation and execution
  Interactive,
}

impl Default for ExecutionExampleConfig
{
  fn default() -> Self
  {
    Self
    {
      mode: ExecutionMode::Basic,
      problem: None,
      task: None,
      domain: None,
      timeout_seconds: 30,
      enable_network: false,
      detailed_logging: true,
    }
  }
}

/// Create a test client using the API key from environment or file.
fn create_client() -> Result< Client, Box< dyn std::error::Error > >
{
  match std::env::var( "GEMINI_API_KEY" )
  {
    Ok( key ) if !key.is_empty() =>
    {
      Ok( Client::builder().api_key( key ).build()? )
    },
    _ => {
      // Try to read from secret file
      let secret_paths = vec![
      "secret/-secret.sh",
      "secret/gemini_api_key",
      ".env",
      ];

      for path in secret_paths
      {
        if let Ok( content ) = std::fs::read_to_string( path )
        {
          // Parse different formats
          for line in content.lines()
          {
            if line.starts_with( "GEMINI_API_KEY" )
            {
              if let Some( key ) = line.split( '=' ).nth( 1 )
              {
                let key = key.trim().trim_matches( '"' ).trim_matches( '\'' );
                if !key.is_empty()
                {
                  return Ok( Client::builder().api_key( key.to_string() ).build()? );
                }
              }
            }
          }
        }
      }

      Err( "No API key found. Set GEMINI_API_KEY environment variable or create secret file".into() )
    }
  }
}

/// Parse command line arguments
fn parse_args() -> ExecutionExampleConfig
{
  let args: Vec< String > = env::args().collect();
  let mut config = ExecutionExampleConfig::default();

  let mut i = 1;
  while i < args.len()
  {
    match args[ i ].as_str()
    {
      "--mode" => {
        if i + 1 < args.len()
        {
          config.mode = match args[ i + 1 ].as_str()
          {
            "math" => ExecutionMode::Math,
            "data" => ExecutionMode::Data,
            "science" => ExecutionMode::Science,
            "interactive" => ExecutionMode::Interactive,
            _ => ExecutionMode::Basic,
          };
          i += 1;
        }
      },
      "--problem" => {
        if i + 1 < args.len()
        {
          config.problem = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      },
      "--task" => {
        if i + 1 < args.len()
        {
          config.task = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      },
      "--domain" => {
        if i + 1 < args.len()
        {
          config.domain = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      },
      "--timeout" => {
        if i + 1 < args.len()
        {
          if let Ok( timeout ) = args[ i + 1 ].parse::< i32 >()
          {
            config.timeout_seconds = timeout;
          }
          i += 1;
        }
      },
      "--enable-network" => {
        config.enable_network = true;
      },
    _ => {}
    }
    i += 1;
  }

  config
}

/// Execute code with the given configuration and prompt
async fn execute_code_with_config(
client: &Client,
prompt: &str,
config: &ExecutionExampleConfig,
) -> Result< GenerateContentResponse, Box< dyn std::error::Error > >
{
  // Configure code execution tool
  let code_execution_tool = CodeExecutionTool {
    config : Some( api_gemini::models::CodeExecutionConfig {
      timeout: Some( config.timeout_seconds ),
      enable_network: Some( config.enable_network ),
    } ),
  };

  let tools = vec![ Tool {
    function_declarations: None,
    code_execution: None,
    google_search_retrieval: None,
    code_execution_tool: Some( code_execution_tool ),
  } ];

  let request = GenerateContentRequest {
    contents : vec![ Content {
      parts : vec![ Part {
        text: Some( prompt.to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        file_data: None,
        video_metadata: None,
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
  };

  println!( "🐍 Executing code with configuration..." );
  if config.detailed_logging
  {
println!( "⚙️  Timeout: {}s, Network: {}", config.timeout_seconds, config.enable_network );
  }

  let start_time = std::time::Instant::now();

  let response = timeout(
  Duration::from_secs( ( config.timeout_seconds + 15 ) as u64 ), // Add buffer time
  client.models().by_name( "gemini-2.5-flash" ).generate_content( &request )
  ).await??;

  let duration = start_time.elapsed();
println!( "⚡ Response received in {:.2}s", duration.as_secs_f64() );

  Ok( response )
}

/// Display code execution results with detailed analysis
fn display_execution_results( response: &GenerateContentResponse, detailed: bool )
{
  println!( "\n🔍 Code Execution Results:" );
println!( "{}", "=".repeat( 80 ) );

  // Display the main response content
  if let Some( candidate ) = response.candidates.first()
  {
    for ( part_idx, part ) in candidate.content.parts.iter().enumerate()
    {
      if let Some( text ) = &part.text
      {
      println!( "\n📝 Generated Content (Part {}):", part_idx + 1 );
      println!( "{}", "-".repeat( 40 ) );
      println!( "{}", text );
      }

      // Check for function calls (code execution requests)
      if let Some( function_call ) = &part.function_call
      {
        println!( "\n🔧 Function Call Detected:" );
      println!( "Function : {}", function_call.name );
        if detailed
        {
        println!( "Arguments : {}", serde_json::to_string_pretty( &function_call.args ).unwrap_or_default() );
        }
      }

      // Check for function responses (code execution results)
      if let Some( function_response ) = &part.function_response
      {
        println!( "\n🎯 Execution Result:" );
      println!( "{}", "-".repeat( 40 ) );
      println!( "Function : {}", function_response.name );

        // Try to parse the response as a structured result
        if let Ok( result ) = serde_json::from_value::< serde_json::Value >( function_response.response.clone() )
        {
          if detailed
          {
          println!( "Full Result : {}", serde_json::to_string_pretty( &result ).unwrap_or_default() );
          }

          // Extract specific fields if they exist
          if let Some( outcome ) = result.get( "outcome" ).and_then( |v| v.as_str() )
          {
          println!( "📊 Outcome : {}", outcome );
          }

          if let Some( output ) = result.get( "output" ).and_then( |v| v.as_str() )
          {
            println!( "📤 Output:" );
          println!( "{}", output );
          }

          if let Some( error ) = result.get( "error" ).and_then( |v| v.as_str() )
          {
            println!( "❌ Error:" );
          println!( "{}", error );
          }

          if let Some( execution_time ) = result.get( "execution_time_ms" ).and_then( |v| v.as_i64() )
          {
          println!( "⏱️  Execution Time : {}ms", execution_time );
          }
        } else {
        println!( "Raw Response : {}", function_response.response );
        }
      }
    }
  }

  // Display token usage if available
  if let Some( usage ) = &response.usage_metadata
  {
    println!( "\n📊 Token Usage:" );
  println!( "{}", "-".repeat( 40 ) );
    if let Some( prompt_tokens ) = usage.prompt_token_count
    {
    println!( "📥 Prompt tokens : {}", prompt_tokens );
    }
    if let Some( candidates_tokens ) = usage.candidates_token_count
    {
    println!( "📤 Response tokens : {}", candidates_tokens );
    }
    if let Some( total_tokens ) = usage.total_token_count
    {
    println!( "🔢 Total tokens : {}", total_tokens );
    }
  }
}

/// Basic code execution demonstration
async fn basic_code_execution(
client: &Client,
config: &ExecutionExampleConfig,
) -> Result< (), Box< dyn std::error::Error > >
{
  println!( "🚀 Basic Code Execution Demo" );
println!( "{}", "=".repeat( 80 ) );

  let prompt = "Please write and execute Python code to calculate the factorial of 10 and explain the algorithm.";

  let response = execute_code_with_config( client, prompt, config ).await?;
  display_execution_results( &response, config.detailed_logging );

  Ok( () )
}

/// Mathematical computation demonstration
async fn mathematical_computation(
client: &Client,
config: &ExecutionExampleConfig,
) -> Result< (), Box< dyn std::error::Error > >
{
  let problem = config.problem.as_deref().unwrap_or( "Calculate the first 20 prime numbers and plot their distribution" );

  println!( "🧮 Mathematical Computation Demo" );
println!( "{}", "=".repeat( 80 ) );
println!( "Problem : {}", problem );
  println!();

  let prompt = format!(
"Please solve this mathematical problem using Python code : {}. \
  Write efficient code, execute it, and explain your approach and results.",
  problem
  );

  let response = execute_code_with_config( client, &prompt, config ).await?;
  display_execution_results( &response, config.detailed_logging );

  Ok( () )
}

/// Data analysis demonstration
async fn data_analysis_demo(
client: &Client,
config: &ExecutionExampleConfig,
) -> Result< (), Box< dyn std::error::Error > >
{
  let task = config.task.as_deref().unwrap_or( "Generate sample sales data and perform statistical analysis" );

  println!( "📊 Data Analysis Demo" );
println!( "{}", "=".repeat( 80 ) );
println!( "Task : {}", task );
  println!();

  let prompt = format!(
"Please perform this data analysis task using Python : {}. \
  Use appropriate libraries like pandas, numpy, and matplotlib if needed. \
  Show your code, execute it, and interpret the results.",
  task
  );

  let response = execute_code_with_config( client, &prompt, config ).await?;
  display_execution_results( &response, config.detailed_logging );

  Ok( () )
}

/// Scientific computing demonstration
async fn scientific_computing_demo(
client: &Client,
config: &ExecutionExampleConfig,
) -> Result< (), Box< dyn std::error::Error > >
{
  let domain = config.domain.as_deref().unwrap_or( "physics" );
  let problem = config.problem.as_deref().unwrap_or( "Simulate simple harmonic motion" );

  println!( "🔬 Scientific Computing Demo" );
println!( "{}", "=".repeat( 80 ) );
println!( "Domain : {}", domain );
println!( "Problem : {}", problem );
  println!();

  let prompt = format!(
"Please solve this {} problem using Python : {}. \
  Use scientific libraries like numpy, scipy, or matplotlib as appropriate. \
  Show your calculations, execute the code, and visualize results if possible.",
  domain, problem
  );

  let response = execute_code_with_config( client, &prompt, config ).await?;
  display_execution_results( &response, config.detailed_logging );

  Ok( () )
}

/// Interactive coding session
async fn interactive_coding_session(
client: &Client,
config: &ExecutionExampleConfig,
) -> Result< (), Box< dyn std::error::Error > >
{
  println!( "💬 Interactive Coding Session" );
println!( "{}", "=".repeat( 80 ) );
  println!( "Enter your coding requests. Type 'quit' to exit." );
  println!( "Example: 'Create a function to calculate compound interest'" );
  println!();

  loop
  {
    print!( "🐍 Code Request: " );
    io ::stdout().flush()?;

    let mut input = String::new();
    io ::stdin().read_line( &mut input )?;
    let input = input.trim();

    if input.is_empty()
    {
      continue;
    }

    if input.eq_ignore_ascii_case( "quit" ) || input.eq_ignore_ascii_case( "exit" )
    {
      break;
    }

    println!( "\n🔄 Processing request..." );

    let prompt = format!(
  "Please write and execute Python code for the following request : {}. \
    Provide clear explanations and show the execution results.",
    input
    );

    match execute_code_with_config( client, &prompt, config ).await
    {
      Ok( response ) => {
        display_execution_results( &response, false ); // Less verbose in interactive mode
      },
      Err( e ) => {
      println!( "❌ Error : {}", e );
      }
    }

    println!( "\n" );
  }

  println!( "👋 Interactive session ended." );
  Ok( () )
}

/// Demonstrate code execution error handling
async fn error_handling_demo(
client: &Client,
config: &ExecutionExampleConfig,
) -> Result< (), Box< dyn std::error::Error > >
{
  println!( "🛠️  Error Handling Demo" );
println!( "{}", "=".repeat( 80 ) );

  let error_scenarios = vec![
  ( "Syntax Error", "Please write Python code that has a syntax error, then fix it." ),
  ( "Runtime Error", "Write code that causes a division by zero error, handle it gracefully." ),
  ( "Logic Error", "Create a function with a logical bug, identify and fix it." ),
  ( "Import Error", "Try to import a non-existent module and handle the error." ),
  ];

  for ( scenario_name, prompt ) in error_scenarios
  {
  println!( "\n🧪 Scenario : {}", scenario_name );
  println!( "{}", "-".repeat( 60 ) );

    match execute_code_with_config( client, prompt, config ).await
    {
      Ok( response ) => {
        display_execution_results( &response, false );
      },
      Err( e ) => {
      println!( "❌ Execution failed : {}", e );
      }
    }

    // Brief pause between scenarios
    tokio ::time::sleep( Duration::from_millis( 1000 ) ).await;
  }

  Ok( () )
}

/// Performance and optimization demonstration
async fn performance_demo(
client: &Client,
config: &ExecutionExampleConfig,
) -> Result< (), Box< dyn std::error::Error > >
{
  println!( "⚡ Performance and Optimization Demo" );
println!( "{}", "=".repeat( 80 ) );

  let prompt = "Please write two versions of a function to calculate fibonacci numbers: \
  1. A naive recursive version \
  2. An optimized version using memoization or iteration \
  Execute both, compare their performance, and explain the differences.";

  let response = execute_code_with_config( client, prompt, config ).await?;
  display_execution_results( &response, config.detailed_logging );

  Ok( () )
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  println!( "🐍 Python Code Execution Example" );
  println!( "=================================" );

  let client = create_client()?;
  let config = parse_args();

  match config.mode
  {
    ExecutionMode::Basic => {
      basic_code_execution( &client, &config ).await?;
    },
    ExecutionMode::Math => {
      mathematical_computation( &client, &config ).await?;
    },
    ExecutionMode::Data => {
      data_analysis_demo( &client, &config ).await?;
    },
    ExecutionMode::Science => {
      scientific_computing_demo( &client, &config ).await?;
    },
    ExecutionMode::Interactive => {
      interactive_coding_session( &client, &config ).await?;
    },
  }

  // Run additional demos if in basic mode
  if matches!( config.mode, ExecutionMode::Basic )
  {
    println!( "\n" );
    error_handling_demo( &client, &config ).await?;

    println!( "\n" );
    performance_demo( &client, &config ).await?;
  }

  println!( "\n✅ Code execution examples completed successfully!" );
  println!( "\n💡 Tips:" );
  println!( "   • Use lower temperature for more deterministic code" );
  println!( "   • Set appropriate timeouts for complex computations" );
  println!( "   • Enable network access only when needed for security" );
  println!( "   • Code execution is sandboxed for safety" );
  println!( "   • Check execution results for errors or warnings" );

  Ok( () )
}