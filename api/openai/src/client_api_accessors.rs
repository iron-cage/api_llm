//! API Accessor Methods for `OpenAI` Client
//!
//! This module provides convenience accessor methods for all `OpenAI` API categories.
//! Each method returns a specialized API client configured with the parent Client.

use mod_interface::mod_interface;

mod private
{
  use crate::
  {
    client ::Client,
    environment ::{ OpenaiEnvironment, EnvironmentInterface },
    assistants ::Assistants,
    chat ::Chat,
    embeddings ::Embeddings,
    files ::Files,
    fine_tuning ::FineTuning,
    images ::Images,
    models ::Models,
    responses ::Responses,
    vector_stores ::VectorStores,
  };

  #[ cfg( feature = "websocket" ) ]
  use crate::realtime ::Realtime;

  #[ cfg( feature = "audio" ) ]
  use crate::audio::Audio;

  #[ cfg( feature = "moderation" ) ]
  use crate::moderations::Moderations;

  /// Extension trait providing API accessor methods for Client
  pub trait ClientApiAccessors< E >
  where
    E : OpenaiEnvironment + EnvironmentInterface + Send + Sync + 'static,
  {
    /// Returns an `Assistants` API client.
    fn assistants( &self ) -> Assistants< '_, E >;

    /// Returns an `Audio` API client.
    #[ cfg( feature = "audio" ) ]
    fn audio( &self ) -> Audio< '_, E >;

    /// Returns a `Chat` API client.
    fn chat( &self ) -> Chat< '_, E >;

    /// Returns an `Embeddings` API client.
    fn embeddings( &self ) -> Embeddings< '_, E >;

    /// Returns a `Files` API client.
    fn files( &self ) -> Files< '_, E >;

    /// Returns a `FineTuning` API client.
    fn fine_tuning( &self ) -> FineTuning< '_, E >;

    /// Returns an `Images` API client.
    fn images( &self ) -> Images< '_, E >;

    /// Returns a `Models` API client.
    fn models( &self ) -> Models< '_, E >;

    /// Returns a `Moderations` API client.
    #[ cfg( feature = "moderation" ) ]
    fn moderations( &self ) -> Moderations< '_, E >;

    /// Returns a `Realtime` API client.
    #[ cfg( feature = "websocket" ) ]
    fn realtime( &self ) -> Realtime< '_, E >;

    /// Returns a `Responses` API client.
    fn responses( &self ) -> Responses< '_, E >;

    /// Returns a `VectorStores` API client.
    fn vector_stores( &self ) -> VectorStores< '_, E >;
  }

  impl< E > ClientApiAccessors< E > for Client< E >
  where
    E : OpenaiEnvironment + EnvironmentInterface + Send + Sync + 'static,
  {
    #[ inline ]
    fn assistants( &self ) -> Assistants< '_, E >
    {
      Assistants::new( self )
    }

    #[ inline ]
    #[ cfg( feature = "audio" ) ]
    fn audio( &self ) -> Audio< '_, E >
    {
      Audio::new( self )
    }

    #[ inline ]
    fn chat( &self ) -> Chat< '_, E >
    {
      Chat::new( self )
    }

    #[ inline ]
    fn embeddings( &self ) -> Embeddings< '_, E >
    {
      Embeddings::new( self )
    }

    #[ inline ]
    fn files( &self ) -> Files< '_, E >
    {
      Files::new( self )
    }

    #[ inline ]
    fn fine_tuning( &self ) -> FineTuning< '_, E >
    {
      FineTuning::new( self )
    }

    #[ inline ]
    fn images( &self ) -> Images< '_, E >
    {
      Images::new( self )
    }

    #[ inline ]
    fn models( &self ) -> Models< '_, E >
    {
      Models::new( self )
    }

    #[ inline ]
    #[ cfg( feature = "moderation" ) ]
    fn moderations( &self ) -> Moderations< '_, E >
    {
      Moderations::new( self )
    }

    #[ inline ]
    #[ cfg( feature = "websocket" ) ]
    fn realtime( &self ) -> Realtime< '_, E >
    {
      Realtime::new( self )
    }

    #[ inline ]
    fn responses( &self ) -> Responses< '_, E >
    {
      Responses::new( self )
    }

    #[ inline ]
    fn vector_stores( &self ) -> VectorStores< '_, E >
    {
      VectorStores::new( self )
    }
  }
}

mod_interface!
{
  exposed use
  {
    ClientApiAccessors,
  };
}
