mod private
{
  use crate::environment::XaiEnvironment;
  use crate::client::Client;
  use crate::chat::Chat;
  use crate::models::Models;

  /// Trait providing convenient API accessors for the client.
  ///
  /// This trait adds methods like `chat()` and `models()` to the `Client` type,
  /// providing a fluent interface for accessing different API endpoints.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use api_xai::{ Client, XaiEnvironmentImpl, Secret, ClientApiAccessors, Message };
  ///
  /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
  /// let env = XaiEnvironmentImpl::new( secret )?;
  /// let client = Client::build( env )?;
  ///
  /// // Using the chat accessor
  /// let request = api_xai::ChatCompletionRequest::former()
  ///   .model( "grok-2-1212".to_string() )
  ///   .messages( vec![ Message::user( "Hello!" ) ] )
  ///   .form();
  /// let response = client.chat().create( request ).await?;
  ///
  /// // Using the models accessor
  /// let models = client.models().list().await?;
  /// # Ok( () )
  /// # }
  /// ```
  pub trait ClientApiAccessors< E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    /// Returns a chat completions API accessor.
    ///
    /// Provides access to the `/v1/chat/completions` endpoint.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use api_xai::{ Client, XaiEnvironmentImpl, Secret, ClientApiAccessors };
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// # let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
    /// # let env = XaiEnvironmentImpl::new( secret )?;
    /// # let client = Client::build( env )?;
    /// let chat_accessor = client.chat();
    /// # Ok( () )
    /// # }
    /// ```
    fn chat( &self ) -> Chat< '_, E >;

    /// Returns a models API accessor.
    ///
    /// Provides access to the `/v1/models` endpoint.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use api_xai::{ Client, XaiEnvironmentImpl, Secret, ClientApiAccessors };
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// # let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
    /// # let env = XaiEnvironmentImpl::new( secret )?;
    /// # let client = Client::build( env )?;
    /// let models_accessor = client.models();
    /// # Ok( () )
    /// # }
    /// ```
    fn models( &self ) -> Models< '_, E >;
  }

  impl< E > ClientApiAccessors< E > for Client< E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    fn chat( &self ) -> Chat< '_, E >
    {
      Chat::new( self )
    }

    fn models( &self ) -> Models< '_, E >
    {
      Models::new( self )
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    ClientApiAccessors,
  };
}
