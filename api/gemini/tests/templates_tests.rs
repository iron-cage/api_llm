//! Request Template Tests
//!
//! Tests for RequestTemplate factory methods and builder setters.
//! Verifies that each template preset applies the expected generation config,
//! and that builder methods correctly override individual fields.

#[ cfg( feature = "request_templates" ) ]
mod templates_tests
{
  use api_gemini::templates::RequestTemplate;

  #[ test ]
  fn test_chat_template()
  {
    let template = RequestTemplate::chat().with_prompt( "Hello" ).build();
    assert_eq!( template.contents.len(), 1 );
    assert!( template.generation_config.is_some() );
  }

  #[ test ]
  fn test_code_generation_template()
  {
    let template = RequestTemplate::code_generation().build();
    let config = template.generation_config.unwrap();
    assert_eq!( config.temperature, Some( 0.2 ) );
  }

  #[ test ]
  fn test_creative_writing_template()
  {
    let template = RequestTemplate::creative_writing().build();
    let config = template.generation_config.unwrap();
    assert_eq!( config.temperature, Some( 1.2 ) );
  }

  #[ test ]
  fn test_factual_qa_template()
  {
    let template = RequestTemplate::factual_qa().build();
    let config = template.generation_config.unwrap();
    assert_eq!( config.temperature, Some( 0.1 ) );
  }

  #[ test ]
  fn test_summarization_template()
  {
    let template = RequestTemplate::summarization().build();
    let config = template.generation_config.unwrap();
    assert_eq!( config.temperature, Some( 0.3 ) );
  }

  #[ test ]
  fn test_with_prompt()
  {
    let template = RequestTemplate::chat().with_prompt( "Test prompt" ).build();
    assert_eq!( template.contents[ 0 ].parts[ 0 ].text, Some( "Test prompt".to_string() ) );
  }

  #[ test ]
  fn test_with_temperature()
  {
    let template = RequestTemplate::chat().with_temperature( 0.5 ).build();
    let config = template.generation_config.unwrap();
    assert_eq!( config.temperature, Some( 0.5 ) );
  }

  #[ test ]
  fn test_with_max_tokens()
  {
    let template = RequestTemplate::chat().with_max_tokens( 1000 ).build();
    let config = template.generation_config.unwrap();
    assert_eq!( config.max_output_tokens, Some( 1000 ) );
  }
}
