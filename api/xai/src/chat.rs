mod private
{
  use crate::error::Result;
  use crate::environment::XaiEnvironment;
  use crate::client::Client;
  use crate::components::chat::{ ChatCompletionRequest, ChatCompletionResponse };

  #[ cfg( feature = "streaming" ) ]
  use crate::components::chat::ChatCompletionChunk;
  #[ cfg( feature = "streaming" ) ]
  use futures_core::Stream;
  #[ cfg( feature = "streaming" ) ]
  use std::pin::Pin;

  /// Chat completions API accessor.
  ///
  /// Provides methods for creating chat completions using the XAI API.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use api_xai::{ Client, XaiEnvironmentImpl, Secret, ChatCompletionRequest, Message, ClientApiAccessors };
  ///
  /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
  /// let env = XaiEnvironmentImpl::new( secret )?;
  /// let client = Client::build( env )?;
  ///
  /// let request = ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .form();
  ///
  /// let response = client.chat().create( request ).await?;
  /// println!( "Response : {:?}", response.choices[ 0 ].message.content );
  /// # Ok( () )
  /// # }
  /// ```
  #[ derive( Debug ) ]
  pub struct Chat< 'a, E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    client : &'a Client< E >,
  }

  impl< 'a, E > Chat< 'a, E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    /// Creates a new Chat API accessor.
    ///
    /// Typically not called directly - use `client.chat()` instead.
    ///
    /// # Arguments
    ///
    /// * `client` - Reference to the client
    pub fn new( client : &'a Client< E > ) -> Self
    {
      Self { client }
    }

    /// Creates a chat completion.
    ///
    /// Sends a conversation to the model and receives a completion.
    ///
    /// # Arguments
    ///
    /// * `request` - Chat completion request with model and messages
    ///
    /// # Errors
    ///
    /// Returns errors for network failures, API errors, or invalid requests.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use api_xai::{ Client, XaiEnvironmentImpl, Secret, ChatCompletionRequest, Message, ClientApiAccessors };
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// # let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
    /// # let env = XaiEnvironmentImpl::new( secret )?;
    /// # let client = Client::build( env )?;
    /// let request = ChatCompletionRequest::former()
    ///   .model( "grok-2-1212".to_string() )
    ///   .messages( vec![
    ///     Message::system( "You are a helpful assistant" ),
    ///     Message::user( "What is 2 + 2?" ),
    ///   ] )
    ///   .temperature( 0.7 )
    ///   .max_tokens( 100u32 )
    ///   .form();
    ///
    /// let response = client.chat().create( request ).await?;
    ///
    /// for choice in response.choices {
    ///   if let Some( content ) = choice.message.content {
    ///     println!( "Assistant : {}", content );
    ///   }
    /// }
    /// # Ok( () )
    /// # }
    /// ```
    pub async fn create(
      &self,
      request : ChatCompletionRequest
    ) -> Result< ChatCompletionResponse >
    {
      self.client.post( "chat/completions", &request ).await
    }

    /// Creates a streaming chat completion.
    ///
    /// Returns a stream of completion chunks via Server-Sent Events (SSE).
    /// The request is automatically modified to enable streaming.
    ///
    /// # Feature Gate
    ///
    /// This method requires the `streaming` feature to be enabled.
    ///
    /// # Arguments
    ///
    /// * `request` - Chat completion request (stream flag is set automatically)
    ///
    /// # Returns
    ///
    /// A pinned stream of `Result< ChatCompletionChunk >` events.
    ///
    /// # Errors
    ///
    /// Returns errors for network failures, API errors, or SSE parsing failures.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[ cfg( feature = "streaming" ) ]
    /// # {
    /// use api_xai::{ Client, XaiEnvironmentImpl, Secret, ChatCompletionRequest, Message, ClientApiAccessors };
    /// use futures_util::StreamExt;
    ///
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// # let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
    /// # let env = XaiEnvironmentImpl::new( secret )?;
    /// # let client = Client::build( env )?;
    /// let request = ChatCompletionRequest::former()
    ///   .model( "grok-2-1212".to_string() )
    ///   .messages( vec![ Message::user( "Count to 5" ) ] )
    ///   .form();
    ///
    /// let chat = client.chat();
    /// let mut stream = chat.create_stream( request ).await?;
    ///
    /// while let Some( chunk_result ) = stream.next().await {
    ///   let chunk = chunk_result?;
    ///   if let Some( delta ) = chunk.choices.first().map( |c| &c.delta ) {
    ///     if let Some( content ) = &delta.content {
    ///       print!( "{}", content );
    ///     }
    ///   }
    /// }
    /// # Ok( () )
    /// # }
    /// # }
    /// ```
    #[ cfg( feature = "streaming" ) ]
    pub async fn create_stream(
      &self,
      mut request : ChatCompletionRequest
    ) -> Result< Pin< Box< dyn Stream< Item = Result< ChatCompletionChunk > > + Send + 'static > > >
    {
      // Enable streaming
      request.stream = Some( true );

      self.client.post_stream( "chat/completions", &request ).await
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    Chat,
  };
}
