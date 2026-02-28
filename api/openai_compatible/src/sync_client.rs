//! Blocking wrapper around [`Client`][crate::Client] for synchronous contexts.
//!
//! Each [`SyncClient`] instance owns a `tokio::runtime::Runtime`. Prefer the
//! async `Client` when possible; use `SyncClient` only for legacy or scripting
//! contexts where async is not an option.

mod private
{
  use crate::error::{ OpenAiCompatError, Result };
  use crate::{ Client, OpenAiCompatEnvironment };
  use std::sync::Arc;
  use tokio::runtime::Runtime;

  /// Blocking wrapper around [`Client`] for use in synchronous contexts.
  ///
  /// Each instance owns a `tokio::runtime::Runtime`. Do not create many
  /// instances; prefer the async [`Client`] when possible.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # #[ cfg( all( feature = "enabled", feature = "sync_api" ) ) ]
  /// # {
  /// use api_openai_compatible::{ Client, SyncClient, OpenAiCompatEnvironmentImpl };
  ///
  /// # fn example() -> Result< (), Box< dyn std::error::Error > > {
  /// let env    = OpenAiCompatEnvironmentImpl::new( "sk-..." )?;
  /// let client = Client::build( env )?;
  /// let sync   = SyncClient::new( client )?;
  /// # Ok( () ) }
  /// # }
  /// ```
  #[ derive( Debug ) ]
  pub struct SyncClient< E >
  where
    E : OpenAiCompatEnvironment,
  {
    /// Async client delegated to for actual HTTP operations.
    client  : Client< E >,
    /// Dedicated tokio runtime for blocking execution.
    runtime : Arc< Runtime >,
  }

  impl< E > SyncClient< E >
  where
    E : OpenAiCompatEnvironment,
  {
    /// Wraps an async `Client` in a new dedicated tokio runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if the tokio runtime cannot be created.
    #[ inline ]
    pub fn new( client : Client< E > ) -> Result< Self >
    {
      let runtime = Runtime::new()
        .map_err( | e | OpenAiCompatError::Environment( e.to_string() ) )?;
      Ok( Self { client, runtime : Arc::new( runtime ) } )
    }

    /// Sends a blocking POST request and deserialises the JSON response.
    ///
    /// # Errors
    ///
    /// Returns network, timeout, or deserialisation errors.
    #[ inline ]
    pub fn post< I, O >( &self, path : &str, body : &I ) -> Result< O >
    where
      I : serde::Serialize,
      O : serde::de::DeserializeOwned,
    {
      self.runtime.block_on( self.client.post( path, body ) )
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    SyncClient,
  };
}
