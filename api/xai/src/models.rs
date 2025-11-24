mod private
{
  use crate::error::Result;
  use crate::environment::XaiEnvironment;
  use crate::client::Client;
  use crate::components::models::{ Model, ListModelsResponse };

  /// Models API accessor.
  ///
  /// Provides methods for listing and retrieving model information.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use api_xai::{ Client, XaiEnvironmentImpl, Secret, ClientApiAccessors };
  ///
  /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
  /// let env = XaiEnvironmentImpl::new( secret )?;
  /// let client = Client::build( env )?;
  ///
  /// let models = client.models().list().await?;
  /// for model in models.data {
  ///   println!( "Model : {}", model.id );
  /// }
  /// # Ok( () )
  /// # }
  /// ```
  #[ derive( Debug ) ]
  pub struct Models< 'a, E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    client : &'a Client< E >,
  }

  impl< 'a, E > Models< 'a, E >
  where
    E : XaiEnvironment + Send + Sync + 'static,
  {
    /// Creates a new Models API accessor.
    ///
    /// Typically not called directly - use `client.models()` instead.
    ///
    /// # Arguments
    ///
    /// * `client` - Reference to the client
    pub fn new( client : &'a Client< E > ) -> Self
    {
      Self { client }
    }

    /// Lists all available models.
    ///
    /// Retrieves a list of all models accessible with the current API key.
    ///
    /// # Errors
    ///
    /// Returns errors for network failures or API errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use api_xai::{ Client, XaiEnvironmentImpl, Secret, ClientApiAccessors };
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// # let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
    /// # let env = XaiEnvironmentImpl::new( secret )?;
    /// # let client = Client::build( env )?;
    /// let response = client.models().list().await?;
    ///
    /// println!( "Available models:" );
    /// for model in response.data {
    ///   println!( "  - {} (created : {})", model.id, model.created );
    /// }
    /// # Ok( () )
    /// # }
    /// ```
    pub async fn list( &self ) -> Result< ListModelsResponse >
    {
      self.client.get( "models" ).await
    }

    /// Retrieves information about a specific model.
    ///
    /// # Arguments
    ///
    /// * `model_id` - Model identifier (e.g., "grok-2-1212", "grok-4")
    ///
    /// # Errors
    ///
    /// Returns errors for network failures, API errors, or if the model is not found.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use api_xai::{ Client, XaiEnvironmentImpl, Secret, ClientApiAccessors };
    /// # async fn example() -> Result< (), Box< dyn std::error::Error > > {
    /// # let secret = Secret::load_with_fallbacks( "XAI_API_KEY" )?;
    /// # let env = XaiEnvironmentImpl::new( secret )?;
    /// # let client = Client::build( env )?;
    /// let model = client.models().get( "grok-2-1212" ).await?;
    ///
    /// println!( "Model : {}", model.id );
    /// println!( "Owned by : {}", model.owned_by );
    /// println!( "Created : {}", model.created );
    /// # Ok( () )
    /// # }
    /// ```
    pub async fn get( &self, model_id : &str ) -> Result< Model >
    {
      let path = format!( "models/{model_id}" );
      self.client.get( &path ).await
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    Models,
  };
}
