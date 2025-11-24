mod private
{
  use serde::{ Serialize, Deserialize };

  /// Information about a specific model.
  ///
  /// Contains metadata about an available XAI model including its ID,
  /// creation date, and ownership.
  ///
  /// # Examples
  ///
  /// ```
  /// use api_xai::Model;
  ///
  /// // Typically received from API, not constructed manually
  /// ```
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct Model
  {
    /// Unique model identifier (e.g., "grok-2-1212", "grok-4").
    pub id : String,

    /// Object type (always "model").
    pub object : String,

    /// Unix timestamp when the model was created.
    pub created : u64,

    /// Organization that owns the model.
    pub owned_by : String,
  }

  /// Response from listing all available models.
  ///
  /// Contains an array of model objects with their metadata.
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
  #[ derive( Debug, Serialize, Deserialize, Clone, PartialEq ) ]
  pub struct ListModelsResponse
  {
    /// Object type (always "list").
    pub object : String,

    /// Array of available models.
    pub data : Vec< Model >,
  }
}

crate::mod_interface!
{
  exposed use
  {
    Model,
    ListModelsResponse,
  };
}
