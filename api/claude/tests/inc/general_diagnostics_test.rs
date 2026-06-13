//! Diagnostics and secret validation tests
//!
//! Tests verify the diagnostic and validation utilities for secret loading
//! and workspace structure detection exposed by the environment module.

#[ allow( unused_imports ) ]
use super::*;

/// AP-12: `create_embeddings_batch` is a stub that always returns `NotImplemented`
#[ cfg( feature = "embeddings" ) ]
#[ test ]
fn test_ap_12_embeddings_batch_not_implemented()
{
  let client = the_module::Client::new(
    the_module::Secret::new_unchecked( "sk-ant-api03-placeholder".to_string() )
  );
  let result = client.create_embeddings_batch( &[] );
  let err_str = result.expect_err( "create_embeddings_batch() must return Err (not implemented)" ).to_string();
  assert!(
    err_str.to_lowercase().contains( "not" ) || err_str.to_lowercase().contains( "embed" ),
    "Error must reference the not-implemented status, got: {err_str}"
  );
}
