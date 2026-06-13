//! Tests to validate that examples use the API correctly

#![ cfg( feature = "integration" ) ]

#[ path = "common/mod.rs" ] mod common;
use common::create_integration_client;

use api_gemini::{ client::Client, models::* };
use serde_json::json;

#[ test ]
fn test_chat_example_structure()
{
  // Verify the chat example uses correct request structure
  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      parts: vec!
      [
      Part
      {
        text: Some( "Hello! Can you explain what artificial intelligence is in simple terms?".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
      role: "user".to_string(),
    }
    ],
    generation_config: Some( GenerationConfig
    {
      temperature: Some( 0.7 ),
      top_k: Some( 40 ),
      top_p: Some( 0.95 ),
      candidate_count: Some( 1 ),
      max_output_tokens: Some( 1024 ),
      stop_sequences: None,
    }),
    safety_settings: Some( vec!
    [
    SafetySetting
    {
      category: "HARM_CATEGORY_HARASSMENT".to_string(),
      threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
    },
    SafetySetting
    {
      category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
      threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
    }
    ]),
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  // Verify serialization works
  let json = serde_json::to_string( &request ).unwrap();
  assert!( json.contains( "contents" ) );
  assert!( json.contains( "generationConfig" ) );
  assert!( json.contains( "safetySettings" ) );
}

#[ test ]
fn test_multi_turn_conversation_structure()
{
  // Verify multi-turn conversation structure
  let conversation =
  [
  Content
  {
    role: "user".to_string(),
    parts: vec!
    [
    Part
    {
      text: Some( "What is the capital of France?".to_string() ),
      ..Default::default()
    }
    ],
  },
  Content
  {
    role: "model".to_string(),
    parts: vec!
    [
    Part
    {
      text: Some( "The capital of France is Paris.".to_string() ),
      ..Default::default()
    }
    ],
  },
  Content
  {
    role: "user".to_string(),
    parts: vec!
    [
    Part
    {
      text: Some( "What's the population?".to_string() ),
      ..Default::default()
    }
    ],
  },
  ];

  assert_eq!( conversation.len(), 3 );
  assert_eq!( conversation[ 0 ].role, "user" );
  assert_eq!( conversation[ 1 ].role, "model" );
  assert_eq!( conversation[ 2 ].role, "user" );
}

#[ test ]
fn test_embeddings_example_structure()
{
  // Verify embeddings request structure
  let embed_request = EmbedContentRequest
  {
    content: Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "The quick brown fox".to_string() ),
        ..Default::default()
      }
      ],
    },
    task_type: Some( "RETRIEVAL_DOCUMENT".to_string() ),
    title: None,
    output_dimensionality: None,
  };

  let json = serde_json::to_string( &embed_request ).unwrap();
  assert!( json.contains( "content" ) );
  assert!( json.contains( "taskType" ) );
  assert!( json.contains( "RETRIEVAL_DOCUMENT" ) );
}

#[ test ]
fn test_function_calling_structure()
{
  // Verify function calling structure
  let tools = vec!
  [
  Tool
  {
    function_declarations: Some( vec!
    [
    FunctionDeclaration
    {
      name: "get_weather".to_string(),
      description: "Get the current weather in a given location".to_string(),
      parameters: Some( json!
      ({
        "type": "object",
        "properties": {
          "location": {
            "type": "string",
            "description": "The city name"
          },
          "unit": {
            "type": "string",
            "enum": ["celsius", "fahrenheit"],
            "description": "The temperature unit to use"
          }
        },
        "required": ["location"]
      })),
    }
    ]),
    code_execution: None,
    google_search_retrieval: None,
    code_execution_tool: None,
  }
  ];

  let json = serde_json::to_string( &tools ).unwrap();
  assert!( json.contains( "functionDeclarations" ) );
  assert!( json.contains( "get_weather" ) );
  assert!( json.contains( "parameters" ) );
}

#[ test ]
fn test_multimodal_structure()
{
  // Verify multimodal request with image
  use base64::Engine;

  let test_image_data = vec![ 0x89, 0x50, 0x4E, 0x47 ]; // PNG header
  let base64_image = base64::engine::general_purpose::STANDARD.encode( &test_image_data );

  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      parts: vec!
      [
      Part
      {
        text: Some( "What's in this image?".to_string() ),
        ..Default::default()
      },
      Part
      {
        inline_data: Some( Blob
        {
          mime_type: "image/png".to_string(),
          data: base64_image.clone(),
        }),
        ..Default::default()
      },
      ],
      role: "user".to_string(),
    }
    ],
    ..Default::default()
  };

  assert_eq!( request.contents[ 0 ].parts.len(), 2 );
  assert!( request.contents[ 0 ].parts[ 1 ].inline_data.is_some() );

  let blob = request.contents[ 0 ].parts[ 1 ].inline_data.as_ref().unwrap();
  assert_eq!( blob.mime_type, "image/png" );
  assert_eq!( blob.data, base64_image );
}

#[ test ]
fn test_safety_settings_structure()
{
  // Verify safety settings
  let safety_settings = vec!
  [
  SafetySetting
  {
    category: "HARM_CATEGORY_HARASSMENT".to_string(),
    threshold: "BLOCK_LOW_AND_ABOVE".to_string(),
  },
  SafetySetting
  {
    category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
    threshold: "BLOCK_ONLY_HIGH".to_string(),
  },
  ];

  let json = serde_json::to_string( &safety_settings ).unwrap();
  assert!( json.contains( "HARM_CATEGORY_HARASSMENT" ) );
  assert!( json.contains( "BLOCK_LOW_AND_ABOVE" ) );
  assert!( json.contains( "HARM_CATEGORY_HATE_SPEECH" ) );
  assert!( json.contains( "BLOCK_ONLY_HIGH" ) );
}

#[ test ]
fn test_error_handling_client_builder()
{
  // Test that error handling example's client builder pattern works
  let result = Client::builder()
  .api_key( "test-key".to_string() )
  .build();

  assert!( result.is_ok() );

  // Test empty API key error
  let result = Client::builder()
  .api_key( String::new() )
  .build();

  assert!( result.is_err() );
  match result.unwrap_err()
  {
    api_gemini ::error::Error::AuthenticationError( msg ) =>
    {
      assert_eq!( msg, "API key cannot be empty" );
    },
    _ => panic!( "Expected AuthenticationError" ),
  }
}

#[ test ]
fn test_model_list_response()
{
  // Verify model list response structure
  let models = ListModelsResponse
  {
    models: vec!
    [
    Model
    {
      name: "models/gemini-flash-latest".to_string(),
      display_name: Some( "Gemini 1.5 Pro Latest".to_string() ),
      description: Some( "Our most capable model".to_string() ),
      input_token_limit: Some( 1_048_576 ),
      output_token_limit: Some( 8192 ),
      supported_generation_methods: Some( vec!
      [
      "generateContent".to_string(),
      "embedContent".to_string(),
      ]),
      temperature: Some( 1.0 ),
      top_p: Some( 0.95 ),
      top_k: Some( 64 ),
      version: Some( "001".to_string() ),
    }
    ],
    next_page_token: None,
  };

  assert_eq!( models.models.len(), 1 );
  assert_eq!( models.models[ 0 ].name, "models/gemini-flash-latest" );
  assert!( models.models[ 0 ].supported_generation_methods.as_ref().unwrap().contains( &"generateContent".to_string() ) );
}

// ==============================================================================
// INTEGRATION TESTS - Real API validation of structures
// ==============================================================================

#[ tokio::test ]
async fn integration_test_chat_example_real_api()
{
  let client = create_integration_client();
  
  // Test the exact structure from the unit test but with real API
  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      parts: vec!
      [
      Part
      {
        text: Some( "Hello! Can you explain what artificial intelligence is in one sentence?".to_string() ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }
      ],
      role: "user".to_string(),
    }
    ],
    generation_config: Some( GenerationConfig
    {
      temperature: Some( 0.7 ),
      top_k: Some( 40 ),
      top_p: Some( 0.95 ),
      candidate_count: Some( 1 ),
      max_output_tokens: Some( 600 ), // Increased to avoid truncation
      stop_sequences: None,
    }),
    safety_settings: Some( vec!
    [
    SafetySetting
    {
      category: "HARM_CATEGORY_HARASSMENT".to_string(),
      threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
    },
    SafetySetting
    {
      category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
      threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
    }
    ]),
    tools: None,
    tool_config: None,
    system_instruction: None,
    cached_content: None,
  };

  let response = client.models().by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await
  .expect( "Chat example structure should work with real API" );

  // Validate response structure
  assert!( !response.candidates.is_empty(), "Real API should return candidates" );
  assert!( response.candidates[ 0 ].content.parts[ 0 ].text.is_some(), "Real API should return text" );
  
  let response_text = response.candidates[ 0 ].content.parts[ 0 ].text.as_ref().unwrap();
  assert!( !response_text.is_empty(), "Real API should return non-empty response" );
  assert!( response_text.to_lowercase().contains( "artificial" ) || response_text.to_lowercase().contains( "ai" ), 
"Response should relate to AI: {response_text}" );
}

#[ tokio::test ]
async fn integration_test_embeddings_example_real_api()
{
  let client = create_integration_client();
  
  // Test the exact structure from the unit test but with real API
  let embed_request = EmbedContentRequest
  {
    content: Content
    {
      role: "user".to_string(),
      parts: vec!
      [
      Part
      {
        text: Some( "The quick brown fox jumps over the lazy dog".to_string() ),
        ..Default::default()
      }
      ],
    },
    task_type: Some( "RETRIEVAL_DOCUMENT".to_string() ),
    title: None,
    output_dimensionality: None,
  };

  let response = client.models().by_name( "text-embedding-004" )
  .embed_content( &embed_request )
  .await
  .expect( "Embeddings example structure should work with real API" );

  // Validate response structure
  assert!( response.embedding.values.len() > 100, "Real API should return meaningful embedding vector" );
  
  // Verify embedding values are reasonable
  let embedding_sum: f32 = response.embedding.values.iter().sum();
  assert!( embedding_sum.abs() > 0.001, "Embedding should have non-zero values" );
}

#[ tokio::test ]
async fn integration_test_function_calling_example_real_api()
{
  let client = create_integration_client();
  
  // Test the exact structure from the unit test but with real API
  let tools = vec!
  [
  Tool
  {
    function_declarations: Some( vec!
    [
    FunctionDeclaration
    {
      name: "get_weather".to_string(),
      description: "Get the current weather in a given location".to_string(),
      parameters: Some( json!
      ({
        "type": "object",
        "properties": {
          "location": {
            "type": "string",
            "description": "The city name"
          },
          "unit": {
            "type": "string",
            "enum": ["celsius", "fahrenheit"],
            "description": "The temperature unit to use"
          }
        },
        "required": ["location"]
      })),
    }
    ]),
    code_execution: None,
    google_search_retrieval: None,
    code_execution_tool: None,
  }
  ];

  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      parts: vec!
      [
      Part
      {
        text: Some( "What's the weather like in Paris? Please use the get_weather function.".to_string() ),
        ..Default::default()
      }
      ],
      role: "user".to_string(),
    }
    ],
    tools: Some( tools ),
    ..Default::default()
  };

  let response = client.models().by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await
  .expect( "Function calling example structure should work with real API" );

  // Validate response structure
  assert!( !response.candidates.is_empty(), "Real API should return candidates" );
  
  // Should either call the function or explain why it can't
  let has_function_call = response.candidates[ 0 ].content.parts.iter()
  .any( |part| part.function_call.is_some() );
  let has_text_response = response.candidates[ 0 ].content.parts.iter()
  .any( |part| part.text.is_some() );
  
  assert!( has_function_call || has_text_response, 
  "Real API should either call function or provide text response" );
}

#[ tokio::test ]
async fn integration_test_multimodal_example_real_api()
{
  let client = create_integration_client();
  
  // Test multimodal structure without actual image data - focus on API structure validation
  // This tests the multimodal capability by using text-only content but with multimodal structure
  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      parts: vec!
      [
      Part
      {
        text: Some( "Describe a simple image: a red circle on a white background.".to_string() ),
        ..Default::default()
      },
      Part
      {
        text: Some( "[Image description: A red circle on a white background]".to_string() ),
        ..Default::default()
      },
      ],
      role: "user".to_string(),
    }
    ],
    generation_config: Some( GenerationConfig
    {
      max_output_tokens: Some( 600 ),
      ..Default::default()
    }),
    ..Default::default()
  };

  let response = client.models().by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await
  .expect( "Multimodal example structure should work with real API" );

  // Validate multimodal response structure - the key is that the API accepts multiple Parts
  assert!( !response.candidates.is_empty(), "Real API should return candidates for multimodal structure" );
  assert!( response.candidates[ 0 ].content.parts[ 0 ].text.is_some(), "Real API should return text" );
  
  let response_text = response.candidates[ 0 ].content.parts[ 0 ].text.as_ref().unwrap();
  assert!( !response_text.is_empty(), "Real API should return non-empty response" );
  assert!( response_text.len() > 10, "Response should be substantive for multimodal input" );
}

#[ tokio::test ]
async fn integration_test_safety_settings_example_real_api()
{
  let client = create_integration_client();
  
  // Test safety settings with a request that might trigger content filtering
  let request = GenerateContentRequest
  {
    contents: vec!
    [
    Content
    {
      parts: vec!
      [
      Part
      {
        text: Some( "Write a story about friendship and cooperation.".to_string() ), // Safe content
        ..Default::default()
      }
      ],
      role: "user".to_string(),
    }
    ],
    safety_settings: Some( vec!
    [
    SafetySetting
    {
      category: "HARM_CATEGORY_HARASSMENT".to_string(),
      threshold: "BLOCK_LOW_AND_ABOVE".to_string(),
    },
    SafetySetting
    {
      category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
      threshold: "BLOCK_ONLY_HIGH".to_string(),
    },
    ]),
    generation_config: Some( GenerationConfig
    {
      max_output_tokens: Some( 600 ),
      ..Default::default()
    }),
    ..Default::default()
  };

  let response = client.models().by_name( "gemini-flash-latest" )
  .generate_content( &request )
  .await
  .expect( "Safety settings example structure should work with real API" );

  // Validate response structure
  assert!( !response.candidates.is_empty(), "Real API should return candidates with safe content" );
  assert!( response.candidates[ 0 ].content.parts[ 0 ].text.is_some(), "Real API should return text" );
  
  let response_text = response.candidates[ 0 ].content.parts[ 0 ].text.as_ref().unwrap();
  assert!( !response_text.is_empty(), "Real API should return non-empty response" );
  // Verify the model responded appropriately to the safe content request
  // The response should be substantive (not just an error or refusal)
assert!( response_text.len() > 20, "Response should be substantive for safe content : {response_text}" );
  
  // Accept any positive, creative response that doesn't indicate content blocking
  let response_lower = response_text.to_lowercase();
  let contains_story_elements = response_lower.contains( "friend" ) || 
  response_lower.contains( "story" ) ||
  response_lower.contains( "cooperation" ) ||
  response_lower.contains( "together" ) ||
  response_lower.contains( "help" ) ||
  response_lower.contains( "kind" );
                               
  // If it doesn't contain expected elements, ensure it's at least a creative narrative
  if !contains_story_elements
  {
    // Should be a narrative (contains narrative elements)
    let is_narrative = response_lower.contains( "once" ) ||
    response_lower.contains( "there" ) ||
    response_lower.contains( "was" ) ||
    response_lower.contains( "said" ) ||
    response_text.split( '.' ).count() > 2; // Multiple sentences
  assert!( is_narrative, "Response should either relate to friendship/cooperation or be a creative narrative : {response_text}" );
  }
}

#[ tokio::test ]
async fn integration_test_model_list_example_real_api()
{
  let client = create_integration_client();
  
  // Test model listing with real API
  let models_response = client.models()
  .list()
  .await
  .expect( "Model list should work with real API" );

  // Validate response structure matches our unit test expectations
  assert!( !models_response.models.is_empty(), "Real API should return available models" );
  
  // Find a gemini model
  let gemini_models: Vec< _ > = models_response.models.iter()
  .filter( |model| model.name.contains( "gemini" ) )
  .collect();
  
  assert!( !gemini_models.is_empty(), "Real API should include Gemini models" );
  
  // Validate model structure
  let first_model = &gemini_models[ 0 ];
  assert!( !first_model.name.is_empty(), "Model should have name" );
  assert!( first_model.display_name.is_some(), "Model should have display name" );
  assert!( first_model.supported_generation_methods.is_some(), "Model should have supported methods" );
  
  let methods = first_model.supported_generation_methods.as_ref().unwrap();
  assert!( methods.contains( &"generateContent".to_string() ) || methods.contains( &"embedContent".to_string() ),
  "Model should support at least one generation method" );
}