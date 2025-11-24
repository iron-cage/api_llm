//! System Instructions and Structured Behavior Control Example
//!
//! This example demonstrates system instruction capabilities including:
//! - Defining consistent AI personality and behavior
//! - Role-based instruction sets (assistant, teacher, analyst, etc.)
//! - Context-aware behavior modification
//! - Multi-turn conversation consistency
//! - Domain-specific instruction templates
//! - Instruction inheritance and composition
//! - Behavioral constraint enforcement
//!
//! Usage:
//! ```bash
//! # Basic system instruction demo
//! cargo run --example gemini_system_instructions
//!
//! # Specific role-based instruction
//! cargo run --example gemini_system_instructions -- --role teacher --subject "mathematics"
//!
//! # Professional assistant mode
//! cargo run --example gemini_system_instructions -- --role assistant --domain "software engineering"
//!
//! # Creative writing coach
//! cargo run --example gemini_system_instructions -- --role coach --specialty "creative writing"
//!
//! # Interactive conversation with system instructions
//! cargo run --example gemini_system_instructions -- --mode interactive --role analyst
//! ```

use api_gemini::{ client::Client, models::* };
use std::env;
use tokio::time::{ timeout, Duration };
use std::io::{ self, Write };

/// Configuration for system instruction examples
#[ derive( Debug, Clone ) ]
pub struct SystemInstructionConfig
{
  /// Execution mode
  pub mode: InstructionMode,
  /// AI role (teacher, assistant, coach, analyst, etc.)
  pub role: Option< String >,
  /// Subject or domain specialization
  pub subject: Option< String >,
  /// Professional domain
  pub domain: Option< String >,
  /// Specialty area
  pub specialty: Option< String >,
  /// Conversation style (formal, casual, academic, etc.)
  pub style: Option< String >,
  /// Enable multi-turn conversation tracking
  pub track_conversation: bool,
  /// Enable detailed instruction analysis
  pub analyze_instructions: bool,
}

/// System instruction execution modes
#[ derive( Debug, Clone ) ]
pub enum InstructionMode
{
  /// Basic system instruction demonstration
  Basic,
  /// Role-based instruction examples (teacher, assistant, etc.)
  RoleBased,
  /// Interactive multi-turn conversation
  Interactive,
  /// Comparison between instructed and non-instructed responses
  Comparison,
  /// Template-based instruction generation
  Template,
}

impl Default for SystemInstructionConfig
{
  fn default() -> Self
  {
    Self
    {
      mode: InstructionMode::Basic,
      role: None,
      subject: None,
      domain: None,
      specialty: None,
      style: None,
      track_conversation: true,
      analyze_instructions: true,
    }
  }
}

/// Predefined system instruction templates
struct InstructionTemplates;

impl InstructionTemplates
{
  /// Get system instruction for a teacher role
  fn teacher( subject: &str ) -> String
  {
    format!(
  "You are an expert {} teacher with years of experience in education. Your role is to:

    • Explain concepts clearly and progressively, building from basic to advanced topics
    • Use appropriate analogies and examples to make complex ideas understandable
    • Ask probing questions to check student understanding
    • Provide constructive feedback and encouragement
    • Adapt your teaching style to the student's level and learning pace
    • Break down complex problems into manageable steps
    • Always maintain a patient, supportive, and encouraging tone

    Teaching Philosophy:
    - Every student can learn with the right approach
    - Mistakes are learning opportunities, not failures
    - Understanding is more important than memorization
    - Real-world connections make learning meaningful

  When explaining {}, always:
    - Start with fundamentals and build up
    - Use concrete examples before abstract concepts
    - Check for understanding before moving forward
    - Encourage questions and curiosity",
    subject, subject
    )
  }

  /// Get system instruction for a professional assistant role
  fn assistant( domain: &str ) -> String
  {
    format!(
  "You are a highly skilled professional assistant specializing in {}. Your role is to:

    • Provide accurate, actionable, and well-researched information
    • Maintain a professional yet approachable communication style
    • Prioritize efficiency and clarity in all interactions
    • Anticipate needs and offer proactive suggestions
    • Organize information logically and present it clearly
    • Follow up on previous conversations and maintain context
    • Respect confidentiality and professional boundaries

    Professional Standards:
    - Accuracy and reliability in all information provided
    - Timely and responsive communication
    - Continuous learning and staying current with industry trends
    - Ethical and responsible advice
    - Clear documentation and follow-through

  For {} topics, I will:
    - Provide industry-standard best practices
    - Offer multiple solution approaches when appropriate
    - Include relevant tools, resources, and references
    - Consider practical implementation challenges
    - Suggest next steps and action items",
    domain, domain
    )
  }

  /// Get system instruction for a coach role
  fn coach( specialty: &str ) -> String
  {
    format!(
  "You are an experienced {} coach dedicated to helping people achieve their goals. Your approach includes:

    • Active listening and empathetic understanding
    • Asking powerful questions that promote self-discovery
    • Providing structured guidance and accountability
    • Celebrating progress and learning from setbacks
    • Creating safe spaces for vulnerability and growth
    • Adapting coaching style to individual needs and preferences
    • Focusing on actionable steps and measurable outcomes

    Coaching Philosophy:
    - People have the answers within them; coaching helps unlock them
    - Growth happens outside the comfort zone
    - Progress, not perfection, is the goal
    - Every experience is a learning opportunity

  As a {} coach, I will:
    - Help you clarify your goals and vision
    - Identify obstacles and develop strategies to overcome them
  - Provide tools and techniques specific to {}
    - Offer honest, constructive feedback
    - Support you in building confidence and skills
    - Track progress and adjust approaches as needed",
    specialty, specialty, specialty
    )
  }

  /// Get system instruction for an analyst role
  fn analyst( domain: &str ) -> String
  {
    format!(
  "You are a skilled {} analyst with expertise in data interpretation, critical thinking, and strategic insights. Your analytical approach includes:

    • Systematic examination of information and data
    • Identification of patterns, trends, and anomalies
    • Objective evaluation of evidence and sources
    • Clear communication of findings and implications
    • Risk assessment and scenario planning
    • Data-driven recommendations and conclusions
    • Maintaining objectivity while acknowledging limitations

    Analytical Framework:
    - Gather comprehensive and reliable data
    - Apply appropriate analytical methods and tools
    - Consider multiple perspectives and potential biases
    - Validate findings through cross-referencing
    - Present conclusions with supporting evidence
    - Recommend actionable next steps

  For {} analysis, I will:
    - Break down complex problems into manageable components
    - Use relevant metrics and benchmarks
    - Provide context and historical perspective
    - Identify key drivers and influential factors
    - Assess risks, opportunities, and trade-offs
    - Deliver insights that support informed decision-making",
    domain, domain
    )
  }

  /// Get a basic helpful assistant instruction
  fn basic_assistant() -> String
  {
    "You are a helpful, accurate, and thoughtful AI assistant. Your goal is to:

    • Provide clear, accurate, and useful information
    • Be honest about what you know and don't know
    • Ask clarifying questions when needed
    • Offer multiple perspectives when appropriate
    • Maintain a friendly but professional tone
    • Respect user privacy and ethical boundaries
    • Encourage learning and critical thinking

    Core Principles:
    - Accuracy and truthfulness in all responses
    - Respect for human agency and decision-making
    - Promoting understanding over mere answers
    - Being helpful while being honest about limitations
    - Encouraging good information literacy practices

    I will strive to be genuinely helpful while being transparent about my capabilities and limitations.".to_string()
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
fn parse_args() -> SystemInstructionConfig
{
  let args: Vec< String > = env::args().collect();
  let mut config = SystemInstructionConfig::default();

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
            "role" => InstructionMode::RoleBased,
            "interactive" => InstructionMode::Interactive,
            "comparison" => InstructionMode::Comparison,
            "template" => InstructionMode::Template,
            _ => InstructionMode::Basic,
          };
          i += 1;
        }
      },
      "--role" => {
        if i + 1 < args.len()
        {
          config.role = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      },
      "--subject" => {
        if i + 1 < args.len()
        {
          config.subject = Some( args[ i + 1 ].clone() );
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
      "--specialty" => {
        if i + 1 < args.len()
        {
          config.specialty = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      },
      "--style" => {
        if i + 1 < args.len()
        {
          config.style = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      },
    _ => {}
    }
    i += 1;
  }

  config
}

/// Execute a request with system instructions
async fn execute_with_system_instruction(
client: &Client,
system_instruction_text: &str,
user_message: &str,
conversation_history: Option< &Vec< Content > >,
) -> Result< GenerateContentResponse, Box< dyn std::error::Error > >
{
  // Create system instruction
  let system_instruction = SystemInstruction {
    role: "system".to_string(),
    parts : vec![ Part {
      text: Some( system_instruction_text.to_string() ),
      inline_data: None,
      function_call: None,
      function_response: None,
      file_data: None,
      video_metadata: None,
    } ],
  };

  // Build conversation contents
  let mut contents = conversation_history.cloned().unwrap_or_default();
  contents.push( Content {
    parts : vec![ Part {
      text: Some( user_message.to_string() ),
      inline_data: None,
      function_call: None,
      function_response: None,
      file_data: None,
      video_metadata: None,
    } ],
    role: "user".to_string(),
  } );

  let request = GenerateContentRequest {
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
  };

  println!( "🤖 Executing with system instructions..." );
  let start_time = std::time::Instant::now();

  let response = timeout(
  Duration::from_secs( 30 ),
  client.models().by_name( "gemini-2.5-flash" ).generate_content( &request )
  ).await??;

  let duration = start_time.elapsed();
println!( "⚡ Response received in {:.2}s", duration.as_secs_f64() );

  Ok( response )
}

/// Display response with system instruction analysis
fn display_response_with_analysis(
response: &GenerateContentResponse,
system_instruction: &str,
analyze: bool,
)
{
  println!( "\n📝 AI Response:" );
println!( "{}", "=".repeat( 80 ) );

  if let Some( candidate ) = response.candidates.first()
  {
    if let Some( part ) = candidate.content.parts.first()
    {
      if let Some( text ) = &part.text
      {
      println!( "{}", text );
      }
    }
  }

  if analyze
  {
    println!( "\n🔍 System Instruction Analysis:" );
  println!( "{}", "-".repeat( 40 ) );
  println!( "📋 Instruction Length : {} characters", system_instruction.len() );
  println!( "📄 Instruction Word Count : {}", system_instruction.split_whitespace().count() );

    // Analyze key instruction components
    let instruction_lower = system_instruction.to_lowercase();
    let has_role_definition = instruction_lower.contains( "you are" ) || instruction_lower.contains( "your role" );
    let has_guidelines = instruction_lower.contains( "•" ) || instruction_lower.contains( "-" );
    let has_examples = instruction_lower.contains( "example" ) || instruction_lower.contains( "for instance" );
    let has_constraints = instruction_lower.contains( "don't" ) || instruction_lower.contains( "avoid" ) || instruction_lower.contains( "never" );

    println!( "✅ Components Present:" );
println!( "   📋 Role Definition : {}", if has_role_definition { "Yes" } else { "No" } );
println!( "   📝 Guidelines/Bullets : {}", if has_guidelines { "Yes" } else { "No" } );
println!( "   📚 Examples : {}", if has_examples { "Yes" } else { "No" } );
println!( "   ⚠️  Constraints : {}", if has_constraints { "Yes" } else { "No" } );
  }

  // Display token usage
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

/// Basic system instruction demonstration
async fn basic_system_instruction_demo(
client: &Client,
config: &SystemInstructionConfig,
) -> Result< (), Box< dyn std::error::Error > >
{
  println!( "🎭 Basic System Instruction Demo" );
println!( "{}", "=".repeat( 80 ) );

  let system_instruction = InstructionTemplates::basic_assistant();
  let user_message = "What is the difference between machine learning and artificial intelligence?";

  println!( "📋 System Instruction:" );
println!( "{}", "-".repeat( 40 ) );
println!( "{}", system_instruction );
  println!();

println!( "💬 User Message : {}", user_message );
  println!();

  let response = execute_with_system_instruction(
  client,
  &system_instruction,
  user_message,
  None,
  ).await?;

  display_response_with_analysis( &response, &system_instruction, config.analyze_instructions );

  Ok( () )
}

/// Role-based system instruction demonstration
async fn role_based_instruction_demo(
client: &Client,
config: &SystemInstructionConfig,
) -> Result< (), Box< dyn std::error::Error > >
{
  let role = config.role.as_deref().unwrap_or( "teacher" );

println!( "🎭 Role-Based System Instruction Demo : {}", role );
println!( "{}", "=".repeat( 80 ) );

  let ( system_instruction, test_message ) = match role
  {
    "teacher" => {
      let subject = config.subject.as_deref().unwrap_or( "computer science" );
    ( InstructionTemplates::teacher( subject ), format!( "Can you explain recursion in {} to a beginner?", subject ) )
    },
    "assistant" => {
      let domain = config.domain.as_deref().unwrap_or( "software engineering" );
    ( InstructionTemplates::assistant( domain ), format!( "What are the best practices for {} project management?", domain ) )
    },
    "coach" => {
      let specialty = config.specialty.as_deref().unwrap_or( "productivity" );
    ( InstructionTemplates::coach( specialty ), format!( "I'm struggling with procrastination. How can {} coaching help me?", specialty ) )
    },
    "analyst" => {
      let domain = config.domain.as_deref().unwrap_or( "business" );
    ( InstructionTemplates::analyst( domain ), format!( "What factors should I consider when analyzing {} market trends?", domain ) )
    },
    _ => {
      ( InstructionTemplates::basic_assistant(), "Tell me about yourself and how you can help.".to_string() )
    }
  };

println!( "📋 System Instruction for {} role:", role );
println!( "{}", "-".repeat( 40 ) );
println!( "{}", system_instruction );
  println!();

println!( "💬 Test Message : {}", test_message );
  println!();

  let response = execute_with_system_instruction(
  client,
  &system_instruction,
  &test_message,
  None,
  ).await?;

  display_response_with_analysis( &response, &system_instruction, config.analyze_instructions );

  Ok( () )
}

/// Interactive conversation with system instructions
async fn interactive_instruction_demo(
client: &Client,
config: &SystemInstructionConfig,
) -> Result< (), Box< dyn std::error::Error > >
{
  let role = config.role.as_deref().unwrap_or( "assistant" );

println!( "💬 Interactive System Instruction Demo : {}", role );
println!( "{}", "=".repeat( 80 ) );

  let system_instruction = match role
  {
    "teacher" => InstructionTemplates::teacher( config.subject.as_deref().unwrap_or( "general knowledge" ) ),
    "assistant" => InstructionTemplates::assistant( config.domain.as_deref().unwrap_or( "general" ) ),
    "coach" => InstructionTemplates::coach( config.specialty.as_deref().unwrap_or( "life" ) ),
    "analyst" => InstructionTemplates::analyst( config.domain.as_deref().unwrap_or( "data" ) ),
    _ => InstructionTemplates::basic_assistant(),
  };

println!( "🎭 Active Role : {}", role );
  println!( "📋 System instructions loaded. Type 'quit' to exit." );
  println!();

  let mut conversation_history = Vec::new();

  loop
  {
    print!( "💬 You: " );
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

    println!( "\n🤖 Processing..." );

    match execute_with_system_instruction(
    client,
    &system_instruction,
    input,
    Some( &conversation_history ),
    ).await {
      Ok( response ) => {
        if let Some( candidate ) = response.candidates.first()
        {
          if let Some( part ) = candidate.content.parts.first()
          {
            if let Some( text ) = &part.text
            {
            println!( "🤖 AI: {}", text );

              // Update conversation history if tracking is enabled
              if config.track_conversation
              {
                conversation_history.push( Content {
                  parts : vec![ Part {
                    text: Some( input.to_string() ),
                    ..Default::default()
                  } ],
                  role: "user".to_string(),
                } );

                conversation_history.push( Content {
                  parts : vec![ Part {
                    text: Some( text.clone() ),
                    ..Default::default()
                  } ],
                  role: "model".to_string(),
                } );
              }
            }
          }
        }
      },
      Err( e ) => {
      println!( "❌ Error : {}", e );
      }
    }

    println!();
  }

println!( "👋 Conversation ended. History length : {} turns", conversation_history.len() / 2 );
  Ok( () )
}

/// Compare responses with and without system instructions
async fn instruction_comparison_demo(
client: &Client,
_config: &SystemInstructionConfig,
) -> Result< (), Box< dyn std::error::Error > >
{
  println!( "🔍 System Instruction Comparison Demo" );
println!( "{}", "=".repeat( 80 ) );

  let test_message = "How should I approach learning a new programming language?";
  let system_instruction = InstructionTemplates::teacher( "computer science" );

println!( "💬 Test Question : {}", test_message );
  println!();

  // Response without system instructions
  println!( "📝 Response WITHOUT System Instructions:" );
println!( "{}", "-".repeat( 60 ) );

  let basic_request = GenerateContentRequest {
    contents : vec![ Content {
      parts : vec![ Part {
        text: Some( test_message.to_string() ),
        ..Default::default()
      } ],
      role: "user".to_string(),
    } ],
    generation_config : Some( GenerationConfig {
      temperature: Some( 0.7 ),
      max_output_tokens: Some( 512 ),
      ..Default::default()
    } ),
    ..Default::default()
  };

  let basic_response = client
  .models()
  .by_name( "gemini-2.5-flash" )
  .generate_content( &basic_request )
  .await?;

  if let Some( candidate ) = basic_response.candidates.first()
  {
    if let Some( part ) = candidate.content.parts.first()
    {
      if let Some( text ) = &part.text
      {
      println!( "{}", text );
      }
    }
  }

  println!( "\n📝 Response WITH System Instructions (Teacher Role):" );
println!( "{}", "-".repeat( 60 ) );

  let instructed_response = execute_with_system_instruction(
  client,
  &system_instruction,
  test_message,
  None,
  ).await?;

  if let Some( candidate ) = instructed_response.candidates.first()
  {
    if let Some( part ) = candidate.content.parts.first()
    {
      if let Some( text ) = &part.text
      {
      println!( "{}", text );
      }
    }
  }

  // Compare token usage
  println!( "\n📊 Comparison Analysis:" );
println!( "{}", "-".repeat( 40 ) );

  if let ( Some( basic_usage ), Some( instructed_usage ) ) = ( &basic_response.usage_metadata, &instructed_response.usage_metadata )
  {
    println!( "Token Usage Comparison:" );
    if let ( Some( basic_total ), Some( instructed_total ) ) = ( basic_usage.total_token_count, instructed_usage.total_token_count )
    {
    println!( "   Without instructions : {} tokens", basic_total );
    println!( "   With instructions : {} tokens", instructed_total );
  println!( "   Difference : {} tokens ({:.1}% increase)",
      instructed_total - basic_total,
      ( ( instructed_total - basic_total ) as f64 / basic_total as f64 ) * 100.0
      );
    }
  }

  Ok( () )
}

/// Demonstrate system instruction templates
async fn template_demo(
_client: &Client,
_config: &SystemInstructionConfig,
) -> Result< (), Box< dyn std::error::Error > >
{
  println!( "📋 System Instruction Templates Demo" );
println!( "{}", "=".repeat( 80 ) );

  let templates = vec![
  ( "Teacher", InstructionTemplates::teacher( "mathematics" ) ),
  ( "Assistant", InstructionTemplates::assistant( "project management" ) ),
  ( "Coach", InstructionTemplates::coach( "fitness" ) ),
  ( "Analyst", InstructionTemplates::analyst( "market research" ) ),
  ];

  for ( name, template ) in templates
  {
  println!( "\n🎭 {} Template:", name );
  println!( "{}", "-".repeat( 60 ) );
  println!( "{}", template );
    println!();

    // Show template statistics
    let word_count = template.split_whitespace().count();
    let char_count = template.len();
    let line_count = template.lines().count();

    println!( "📊 Template Statistics:" );
  println!( "   Characters : {}", char_count );
  println!( "   Words : {}", word_count );
  println!( "   Lines : {}", line_count );
  }

  Ok( () )
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  println!( "🎭 System Instructions Example" );
  println!( "==============================" );

  let client = create_client()?;
  let config = parse_args();

  match config.mode
  {
    InstructionMode::Basic => {
      basic_system_instruction_demo( &client, &config ).await?;
    },
    InstructionMode::RoleBased => {
      role_based_instruction_demo( &client, &config ).await?;
    },
    InstructionMode::Interactive => {
      interactive_instruction_demo( &client, &config ).await?;
    },
    InstructionMode::Comparison => {
      instruction_comparison_demo( &client, &config ).await?;
    },
    InstructionMode::Template => {
      template_demo( &client, &config ).await?;
    },
  }

  println!( "\n✅ System instruction examples completed successfully!" );
  println!( "\n💡 Tips:" );
  println!( "   • Clear role definitions improve response consistency" );
  println!( "   • Include specific guidelines and examples in instructions" );
  println!( "   • System instructions are processed with each request" );
  println!( "   • Longer instructions increase token usage" );
  println!( "   • Test instruction effectiveness with diverse queries" );
  println!( "   • Use role-specific templates for consistent behavior" );

  Ok( () )
}