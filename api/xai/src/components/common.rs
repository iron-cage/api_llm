//! Common wire types shared across `api_xai` components.
//!
//! Re-exported from `api_openai_compatible` to eliminate duplication.

mod private
{
  pub use api_openai_compatible::{ Usage, Role };
}

crate::mod_interface!
{
  exposed use
  {
    Usage,
    Role,
  };
}
