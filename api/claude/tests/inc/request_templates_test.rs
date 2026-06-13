//! Request Templates Tests
//!
//! Tests for pre-configured request templates for common AI tasks.
//! (Migrated from `src/request_templates.rs` `#[cfg(test)]` block)

#[ allow( unused_imports ) ]
use super::*;

// ============================================================================
// UNIT TESTS - REQUEST TEMPLATES
// ============================================================================

#[ test ]
fn test_chat_template()
{
  let template = the_module::RequestTemplate::chat( "claude-sonnet-4-6" );
  let request = template.build( "Hello!" );

  assert_eq!( request.model, "claude-sonnet-4-6" );
  assert_eq!( request.max_tokens, 4096 );
  assert_eq!( request.temperature, Some( 1.0 ) );
  assert!( request.system.is_some() );
}

#[ test ]
fn test_code_generation_template()
{
  let template = the_module::RequestTemplate::code_generation( "claude-sonnet-4-6" );
  let request = template.build( "Write a function" );

  assert_eq!( request.temperature, Some( 0.2 ) );
}

#[ test ]
fn test_creative_writing_template()
{
  let template = the_module::RequestTemplate::creative_writing( "claude-sonnet-4-6" );
  let request = template.build( "Write a story" );

  assert_eq!( request.temperature, Some( 1.2 ) );
}

#[ test ]
fn test_factual_qa_template()
{
  let template = the_module::RequestTemplate::factual_qa( "claude-sonnet-4-6" );
  let request = template.build( "What is the capital of France?" );

  assert_eq!( request.temperature, Some( 0.3 ) );
  assert_eq!( request.max_tokens, 2048 );
}

#[ test ]
fn test_summarization_template()
{
  let template = the_module::RequestTemplate::summarization( "claude-sonnet-4-6" );
  let request = template.build( "Summarize this text" );

  assert_eq!( request.temperature, Some( 0.5 ) );
  assert_eq!( request.max_tokens, 1024 );
}

#[ test ]
fn test_with_prompt()
{
  let template = the_module::RequestTemplate::chat( "claude-sonnet-4-6" )
    .with_prompt( "Custom system prompt" );
  let request = template.build( "Hello!" );

  assert!( request.system.is_some() );
  let system_text = &request.system.unwrap()[ 0 ].text;
  assert_eq!( system_text, "Custom system prompt" );
}

#[ test ]
fn test_with_temperature()
{
  let template = the_module::RequestTemplate::chat( "claude-sonnet-4-6" )
    .with_temperature( 0.7 );
  let request = template.build( "Hello!" );

  assert_eq!( request.temperature, Some( 0.7 ) );
}

#[ test ]
fn test_with_max_tokens()
{
  let template = the_module::RequestTemplate::chat( "claude-sonnet-4-6" )
    .with_max_tokens( 2000 );
  let request = template.build( "Hello!" );

  assert_eq!( request.max_tokens, 2000 );
}
