//! `HuggingFace` Inference Providers API Demo — Pro Plan Models
//!
//! Demonstrates the chat completions endpoint that provides access to Pro models.
//! Requires `HUGGINGFACE_API_KEY` environment variable with Pro plan access.
//!
//! ```bash
//! cargo run --example providers_api_demo --all-features
//! ```

use api_huggingface::
{
  client::Client,
  environment::HuggingFaceEnvironmentImpl,
  providers::{ ChatMessage, Providers },
  Secret,
};

/// Find a working Pro model from the list
async fn find_working_model( providers : &Providers< HuggingFaceEnvironmentImpl >, math_question : &str ) -> Option< &'static str >
{
  let pro_models = [
  "meta-llama/Meta-Llama-3-8B-Instruct",
  "meta-llama/Llama-2-7b-chat-hf",
  "mistralai/Mistral-7B-Instruct-v0.2",
  "codellama/CodeLlama-7b-Instruct-hf",
  ];

  for ( i, model ) in pro_models.iter().enumerate()
  {
  println!( "🧪 Test {}: {}", i + 1, model );
  println!( "📤 Input : {math_question:?}" );

  match providers.math_completion( model, math_question ).await
  {
      Ok( response ) =>
      {
  if let Some( choice ) = response.choices.first()
  {
          println!( "✅ SUCCESS! Model {model} is available" );
          println!( "📝 Response : {:?}", choice.message.content );
          println!( "🎉 WORKING PRO MODEL FOUND: {model}" );
          println!( "================================================================================\n" );
          return Some( *model );
  }
  println!( "❌ FAILED: {model} - No choices in response" );
      },
      Err( e ) =>
      {
  println!( "❌ FAILED: {model} - {e}" );
      }
  }
  println!( "================================================================================\n" );
  }
  None
}

/// Test the working model with various scenarios
async fn test_working_model( providers : &Providers< HuggingFaceEnvironmentImpl >, model : &str )
{
  println!( "🎊 SUCCESS : Found working Pro model : {model}" );
  println!( "\n🧪 Testing simple chat with the working model..." );

  match providers.simple_chat( model, "Hello, how are you?" ).await
  {
  Ok( response ) =>
  {
      if let Some( choice ) = response.choices.first()
      {
  println!( "📝 Simple chat response : {:?}", choice.message.content );
      }
  },
  Err( e ) =>
  {
      println!( "❌ Simple chat failed : {e}" );
  }
  }

  println!( "\n🧪 Testing conversation with context..." );

  let messages = vec![
  ChatMessage
  {
      role : "system".to_string(),
      content : "You are a helpful math assistant.".to_string(),
      tool_calls : None,
      tool_call_id : None,
  },
  ChatMessage
  {
      role : "user".to_string(),
      content : "I have x = 13".to_string(),
      tool_calls : None,
      tool_call_id : None,
  },
  ChatMessage
  {
      role : "user".to_string(),
      content : "What is x * 3?".to_string(),
      tool_calls : None,
      tool_call_id : None,
  }
  ];

  match providers.chat_completion( model, messages, Some( 100 ), Some( 0.7 ), Some( 0.9 ) ).await
  {
  Ok( response ) =>
  {
      if let Some( choice ) = response.choices.first()
      {
  println!( "📝 Math conversation response : {:?}", choice.message.content );
      }
  },
  Err( e ) =>
  {
      println!( "❌ Math conversation failed : {e}" );
  }
  }
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  println!( "🧪 HuggingFace Providers API Demo - Pro Plan Models" );
  println!( "===============================================" );

  let secret = match Secret::load_from_env( "HUGGINGFACE_API_KEY" )
  {
  Ok( s ) => s,
  Err( e ) =>
  {
      eprintln!( "❌ Failed to load API key : {e}" );
      eprintln!( "💡 Please set HUGGINGFACE_API_KEY environment variable" );
      return Err( e.into() );
  }
  };

  let env = HuggingFaceEnvironmentImpl::build( secret, None )?;
  let client = Client::build( env )?;
  let providers = client.providers();

  println!( "✅ Client initialized successfully\n" );

  let math_question = "If x = 13, what is x * 3?";
  println!( "🧪 Testing Pro models with math question : \"{math_question}\"" );
  println!( "🔍 Using Inference Providers API (/v1/chat/completions)\n" );

  if let Some( model ) = find_working_model( &providers, math_question ).await
  {
  test_working_model( &providers, model ).await;
  }
  else
  {
  println!( "❌ No Pro models are working. This might indicate:" );
  println!( "   1. Your HuggingFace account doesn't have Pro plan access" );
  println!( "   2. The API key doesn't have the right permissions" );
  println!( "   3. The Inference Providers API endpoint is not accessible" );
  }

  println!( "\n🏁 Demo completed" );
  Ok( () )
}
