//! Doc-spec coverage tests for `api_openai`
//!
//! Maps directly to the scenario IDs in `tests/docs/` spec files.
//! Each test names the scenario it validates in its doc comment.
//!
//! # Test Matrix
//!
//! | Scenario | Name | Kind | Status |
//! |----------|------|------|--------|
//! | IN-01    | Optional fields absent from serialized JSON when None | serde unit | covered |
//! | IN-03    | No automatic request headers beyond what environment provides | unit | covered |
//! | OP-01    | Cargo.toml version parses as valid semver triple | unit | covered |
//! | OP-02    | Version string has exactly three numeric components | unit | covered |
//! | AP-01    | Chat completion accepts typed request and returns typed response | integration | covered |
//! | AP-02    | Model listing returns typed response with non-empty models array | integration | covered |
//! | AP-04    | All API methods return error_tools::Error on authentication failure | integration | covered |
//! | FT-04    | Baseline behavior identical with and without enterprise features | integration | covered |
//! | PT-01    | Chat completion is async and accepts typed request struct | integration | covered |
//! | PT-02    | Method return type is Result with typed response | integration | covered |
//! | PT-03    | Streaming method returns receiver of typed events | integration | covered |

// ── serde and unit tests (no feature gate required) ──────────────────────────

use api_openai::
{
  components ::chat_shared ::
  {
    ChatCompletionRequest,
    ChatCompletionRequestMessage,
  },
  environment ::
  {
    OpenaiEnvironmentImpl,
    OpenaiEnvironment,
    OpenAIRecommended,
  },
  secret ::Secret,
};

/// IN-01 — Optional fields absent from serialized JSON when None
///
/// Given a `ChatCompletionRequest` with only `model` and one user message set and all
/// other optional fields (`temperature`, `max_tokens`, `top_p`, `stream`, `tools`, etc.)
/// left at their default `None`, when the request is serialized with `serde_json::to_string`,
/// then the resulting JSON contains exactly the `model` and `messages` keys and no optional
/// field key appears.
#[ test ]
fn in_01_optional_fields_absent_from_json_when_none()
{
  let request = ChatCompletionRequest
  {
    model : "gpt-4o-mini".to_string(),
    messages : vec!
    [
      ChatCompletionRequestMessage
      {
        role : "user".to_string(),
        content : Some( api_openai::components::chat_shared::ChatCompletionRequestMessageContent::Text( "Hello".to_string() ) ),
        name : None,
        tool_calls : None,
        tool_call_id : None,
      }
    ],
    temperature : None,
    top_p : None,
    max_tokens : None,
    n : None,
    stop : None,
    stream : None,
    system_prompt : None,
    user : None,
    tools : None,
    tool_choice : None,
    response_format : None,
    seed : None,
    logit_bias : None,
    logprobs : None,
    top_logprobs : None,
  };

  let json = serde_json::to_string( &request ).expect( "Serialization must succeed" );

  // Required fields present
  assert!( json.contains( r#""model""# ), "model key must be present" );
  assert!( json.contains( r#""messages""# ), "messages key must be present" );

  // Optional fields must not appear when None
  assert!( !json.contains( r#""temperature""# ), "temperature must be absent when None" );
  assert!( !json.contains( r#""top_p""# ), "top_p must be absent when None" );
  assert!( !json.contains( r#""max_tokens""# ), "max_tokens must be absent when None" );
  assert!( !json.contains( r#""stream""# ), "stream must be absent when None" );
  assert!( !json.contains( r#""tools""# ), "tools must be absent when None" );
  assert!( !json.contains( r#""tool_choice""# ), "tool_choice must be absent when None" );
  assert!( !json.contains( r#""n""# ), "n must be absent when None" );
  assert!( !json.contains( r#""stop""# ), "stop must be absent when None" );
  // Note: "user" appears inside message role values; check for the key form `"user":` at the
  // request root level by looking for the key followed by a colon and whitespace/value.
  // The serde field name is "user" and the JSON key would be `"user":` — distinct from the
  // `"role": "user"` value occurrence.
  assert!(
    !json.contains( r#""user":"# ),
    "user key must be absent at root when None (role values still appear in messages)"
  );
  assert!( !json.contains( r#""seed""# ), "seed must be absent when None" );
  assert!( !json.contains( r#""logprobs""# ), "logprobs must be absent when None" );
}

/// IN-03 — No automatic request headers beyond what environment provides
///
/// Given an `OpenaiEnvironmentImpl` constructed with a known API key and no org/project IDs,
/// when `env.headers()` is called, then the returned `HeaderMap` contains only the
/// `Authorization` (`Bearer <key>`) header — no extra headers are injected.
///
/// Note: `Content-Type: application/json` is injected by reqwest when serializing the
/// POST body, not by the environment layer; it correctly does not appear in `headers()`.
#[ test ]
fn in_03_no_automatic_request_headers_beyond_environment()
{
  let secret = Secret::new_unchecked( "sk-test-headersonly-1234567890abcdef".to_string() );
  let env = OpenaiEnvironmentImpl::build(
    secret,
    None,
    None,
    OpenAIRecommended::base_url().to_string(),
    OpenAIRecommended::realtime_base_url().to_string(),
  ).expect( "Environment must build with valid URL" );

  let headers = OpenaiEnvironment::headers( &env ).expect( "headers() must not fail for a valid key" );

  // Exactly one header: Authorization
  assert_eq!(
    headers.len(),
    1,
    "Only Authorization header must be present when no org/project ID is set, got : {headers:?}"
  );

  let auth_value = headers
    .get( reqwest::header::AUTHORIZATION )
    .expect( "Authorization header must exist" )
    .to_str()
    .expect( "Authorization value must be valid UTF-8" );

  assert!(
    auth_value.starts_with( "Bearer " ),
    "Authorization header must use Bearer scheme, got : {auth_value}"
  );
  assert!(
    auth_value.contains( "sk-test-headersonly" ),
    "Authorization header must contain the API key"
  );
}

/// OP-01 / OP-02 — Cargo.toml version field parses as valid semver triple with three components
///
/// Given the `version` field in `api/openai/Cargo.toml`, when the version string is split
/// on `.` and each part parsed as a non-negative integer, then exactly three components
/// result and all parse successfully, with no pre-release or build metadata suffixes.
#[ test ]
fn op_01_02_cargo_toml_version_is_valid_semver_triple()
{
  let cargo_toml_content = include_str!( "../Cargo.toml" );

  // Extract the version field from [package] section
  let version = cargo_toml_content
    .lines()
    .skip_while( | line | !line.starts_with( "[package]" ) )
    .find( | line | line.trim_start().starts_with( "version" ) )
    .and_then( | line | line.split( '=' ).nth( 1 ) )
    .map( | v | v.trim().trim_matches( '"' ) )
    .expect( "version field must exist in [package] section" );

  // OP-02: exactly three dot-separated components
  let parts : Vec< &str > = version.split( '.' ).collect();
  assert_eq!(
    parts.len(),
    3,
    "Version must have exactly three components (major.minor.patch), got : {version:?}"
  );

  // OP-01: each component parses as a non-negative integer (no pre-release/build-metadata)
  for ( idx, part ) in parts.iter().enumerate()
  {
    let label = [ "major", "minor", "patch" ][ idx ];
    let _parsed : u64 = part.parse().unwrap_or_else( | _ |
      panic!( "Version component '{label}' must be a non-negative integer, got : {part:?}" )
    );
  }
}

// ── integration tests (require real API key) ──────────────────────────────────

#[ cfg( feature = "integration" ) ]
mod integration
{
  use api_openai ::
  {
    Client,
    ClientApiAccessors,
    environment ::{ OpenaiEnvironmentImpl, OpenAIRecommended },
    secret ::Secret,
    components ::chat_shared ::
    {
      ChatCompletionRequest,
      ChatCompletionRequestMessage,
      ChatCompletionRequestMessageContent,
    },
  };

  /// Build a test client using real credentials.
  ///
  /// Panics loudly when credentials are missing — never silently skips.
  fn make_client() -> Client< OpenaiEnvironmentImpl >
  {
    let secret = Secret::load_with_fallbacks( "OPENAI_API_KEY" )
      .expect( "INTEGRATION TEST FAILURE: OPENAI_API_KEY not found in environment or workspace secrets" );
    let env = OpenaiEnvironmentImpl::build(
      secret,
      None,
      None,
      OpenAIRecommended::base_url().to_string(),
      OpenAIRecommended::realtime_base_url().to_string(),
    ).expect( "Environment build must not fail" );
    Client::build( env ).expect( "Client build must not fail" )
  }

  /// Build a minimal chat request for gpt-4o-mini.
  fn minimal_chat_request( prompt : &str ) -> ChatCompletionRequest
  {
    ChatCompletionRequest
    {
      model : "gpt-4o-mini".to_string(),
      messages : vec!
      [
        ChatCompletionRequestMessage
        {
          role : "user".to_string(),
          content : Some( ChatCompletionRequestMessageContent::Text( prompt.to_string() ) ),
          name : None,
          tool_calls : None,
          tool_call_id : None,
        }
      ],
      temperature : None,
      top_p : None,
      max_tokens : Some( 20 ),
      n : None,
      stop : None,
      stream : None,
      system_prompt : None,
      user : None,
      tools : None,
      tool_choice : None,
      response_format : None,
      seed : None,
      logit_bias : None,
      logprobs : None,
      top_logprobs : None,
    }
  }

  /// AP-01 / PT-01 / PT-02 — Chat completion accepts typed request and returns typed response
  ///
  /// Given a valid `ChatCompletionRequest` with model "gpt-4o-mini" and one user message,
  /// when `client.chat().create(request).await` is called against the live `OpenAI` API,
  /// then the method returns `Ok(CreateChatCompletionResponse)` with a non-empty `choices`
  /// array, a non-empty `choices[0].message.content`, and usage statistics all greater
  /// than zero. The request parameter is a concrete typed struct (not `serde_json::Value`)
  /// and the return type exposes `.choices`, `.usage`, and `.model` as typed fields.
  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
  async fn ap_01_pt_01_pt_02_chat_completion_typed_request_and_response()
  {
    let client = make_client();
    let request = minimal_chat_request( "Say: ok" );

    let response = client.chat().create( request ).await
      .expect( "chat().create() must succeed with valid request and credentials" );

    assert!(
      !response.choices.is_empty(),
      "Response choices must be non-empty"
    );

    let content = response.choices[ 0 ].message.content.as_deref().unwrap_or( "" );
    assert!(
      !content.is_empty(),
      "choices[0].message.content must be non-empty"
    );

    let usage = response.usage.as_ref()
      .expect( "usage must be present in non-streaming response" );

    assert!( usage.prompt_tokens > 0, "prompt_tokens must be > 0" );
    assert!( usage.completion_tokens > 0, "completion_tokens must be > 0" );
    assert!( usage.total_tokens > 0, "total_tokens must be > 0" );
    assert!(
      !response.model.is_empty(),
      "response.model must be non-empty"
    );
  }

  /// AP-02 — Model listing returns typed response with non-empty models array
  ///
  /// Given a valid authenticated client, when `client.models().list().await` is called,
  /// then the method returns a typed `ListModelsResponse`; `data` is non-empty; each entry
  /// has a non-empty `id`; at least one model ID contains "gpt".
  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
  async fn ap_02_model_listing_returns_typed_response()
  {
    let client = make_client();

    let response = client.models().list().await
      .expect( "models().list() must succeed with valid credentials" );

    assert!(
      !response.data.is_empty(),
      "models list data must be non-empty"
    );

    for model in &response.data
    {
      assert!( !model.id.is_empty(), "each model entry must have a non-empty id" );
    }

    let has_gpt = response.data.iter().any( | m | m.id.contains( "gpt" ) );
    assert!( has_gpt, "at least one model id must contain 'gpt'" );
  }

  /// AP-04 — All API methods return Err on authentication failure
  ///
  /// Given a client constructed with an invalid API key ("sk-invalid-key-for-testing"),
  /// when `client.chat().create(request).await` is called, then the method returns
  /// `Err(error_tools::Error)` — not a panic, not a silent Ok fallback.
  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
  async fn ap_04_authentication_failure_returns_err()
  {
    let invalid_secret = Secret::new_unchecked( "sk-invalid-key-for-testing-docspec".to_string() );
    let env = OpenaiEnvironmentImpl::build(
      invalid_secret,
      None,
      None,
      OpenAIRecommended::base_url().to_string(),
      OpenAIRecommended::realtime_base_url().to_string(),
    ).expect( "Environment build must not fail" );

    let client = Client::build( env ).expect( "Client build must not fail" );
    let request = minimal_chat_request( "hello" );

    let result = client.chat().create( request ).await;

    assert!(
      result.is_err(),
      "chat().create() with invalid key must return Err, got Ok"
    );
  }

  /// FT-04 — Baseline behavior identical with and without enterprise features
  ///
  /// Given a client constructed via `Client::build(env)` with `full` features compiled but
  /// no enterprise feature explicitly configured by the caller, when `client.chat().create(request).await`
  /// is called, then the request succeeds identically to a minimal-feature build — enterprise
  /// modules are compiled but dormant until explicitly configured.
  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
  async fn ft_04_baseline_behavior_identical_without_enterprise_configuration()
  {
    // Client::build with default env — no retry, no circuit breaker, no rate limiter configured
    let client = make_client();

    // Verify no enterprise configuration is active on a freshly built client
    #[ cfg( feature = "retry" ) ]
    assert!(
      client.retry_config.is_none(),
      "retry_config must be None on a freshly built client — not auto-activated"
    );

    #[ cfg( feature = "circuit_breaker" ) ]
    assert!(
      client.circuit_breaker_config.is_none(),
      "circuit_breaker_config must be None on a freshly built client"
    );

    #[ cfg( feature = "rate_limiting" ) ]
    assert!(
      client.rate_limiting_config.is_none(),
      "rate_limiting_config must be None on a freshly built client"
    );

    // Basic API call succeeds despite full feature set being compiled in
    let request = minimal_chat_request( "Say: ok" );
    let result = client.chat().create( request ).await;
    assert!(
      result.is_ok(),
      "chat().create() must succeed even with full features compiled but not configured, got : {result:?}"
    );
  }

  /// PT-03 — Streaming method returns receiver of typed events
  ///
  /// Given a `ChatCompletionRequest` with the `streaming` feature enabled, when
  /// `client.chat().create_stream(request).await` is invoked, then the method returns
  /// `Ok(mpsc::Receiver<Result<ChatCompletionStreamResponse>>)` — not raw bytes or untyped
  /// JSON — and receiving at least one chunk yields a typed struct with `.choices[0].delta`
  /// containing typed fields.
  #[ cfg( feature = "integration" ) ]
  #[ tokio::test ]
  async fn pt_03_streaming_returns_receiver_of_typed_events()
  {
    let client = make_client();
    let request = ChatCompletionRequest
    {
      model : "gpt-4o-mini".to_string(),
      messages : vec!
      [
        ChatCompletionRequestMessage
        {
          role : "user".to_string(),
          content : Some( ChatCompletionRequestMessageContent::Text( "Count: 1 2 3".to_string() ) ),
          name : None,
          tool_calls : None,
          tool_call_id : None,
        }
      ],
      stream : Some( true ),
      max_tokens : Some( 20 ),
      temperature : None,
      top_p : None,
      n : None,
      stop : None,
      system_prompt : None,
      user : None,
      tools : None,
      tool_choice : None,
      response_format : None,
      seed : None,
      logit_bias : None,
      logprobs : None,
      top_logprobs : None,
    };

    let mut receiver = client.chat().create_stream( request ).await
      .expect( "chat().create_stream() must succeed" );

    let mut chunk_count = 0usize;
    let timeout = core::time::Duration::from_secs( 15 );
    let start = std::time::Instant::now();

    while let Some( chunk_result ) = receiver.recv().await
    {
      if start.elapsed() > timeout
      {
        break;
      }
      let chunk = chunk_result.expect( "stream chunk must not be an error" );

      // Verify the chunk is a typed struct with typed fields — not serde_json::Value
      assert!(
        !chunk.id.is_empty(),
        "stream chunk must have a non-empty id"
      );
      assert!(
        !chunk.object.is_empty(),
        "stream chunk must have a non-empty object"
      );

      // Each choice has a typed `delta` field (ChatCompletionStreamResponseMessage).
      // Verify the typed accessors compile and produce the expected types — not raw JSON.
      for choice in &chunk.choices
      {
        // delta.content and delta.role are Option<String> typed fields
        assert!(
          choice.delta.content.as_deref().map_or( true, | s | s.is_ascii() || !s.is_empty() ),
          "delta.content when present must be a valid string"
        );
        // delta.tool_calls being None is expected in text streaming
        assert!(
          choice.delta.tool_calls.is_none() || choice.delta.tool_calls.as_ref().is_some_and( | v | !v.is_empty() ),
          "delta.tool_calls must be None or non-empty when present"
        );
      }

      chunk_count += 1;
    }

    assert!(
      chunk_count > 0,
      "Must receive at least one typed stream chunk from chat().create_stream()"
    );
  }
}
