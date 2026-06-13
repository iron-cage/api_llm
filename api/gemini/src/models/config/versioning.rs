//! Configuration versioning and history tracking with delta compression
//!
//! This module provides version tracking, change history, and delta compression
//! for efficient storage of configuration changes over time.

use super::DynamicConfig;
use serde::{ Deserialize, Serialize };
use std::collections::HashMap;
use std::time::SystemTime;

/// Types of configuration changes
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum ConfigChangeType
{
  /// Configuration was updated
  Update,
  /// Configuration was rolled back
  Rollback,
  /// Configuration was loaded from file
  FileLoad,
  /// Configuration was restored from version
  VersionRestore,
}

/// Event representing a configuration change
#[ derive( Debug, Clone ) ]
pub struct ConfigChangeEvent
{
  /// Unique identifier for this configuration version
  pub version_id : String,
  /// Type of change that occurred
  pub change_type : ConfigChangeType,
  /// When the change occurred
  pub timestamp : SystemTime,
  /// Previous configuration (if any)
  pub previous_config : Option< DynamicConfig >,
  /// New configuration after the change
  pub new_config : DynamicConfig,
}

/// Historical configuration entry for versioning with optimization metadata
#[ derive( Debug, Clone ) ]
pub struct ConfigHistoryEntry
{
  /// Unique version identifier
  pub version_id : String,
  /// Configuration at this version
  pub config : DynamicConfig,
  /// When this version was created
  pub timestamp : SystemTime,
  /// Type of change that created this version
  pub change_type : ConfigChangeType,
  /// Hash of the configuration for quick comparison
  pub config_hash : u64,
  /// Size in bytes of the configuration when serialized (for memory tracking)
  pub size_bytes : usize,
  /// Configuration differences from previous version (for compression)
  pub delta : Option< ConfigDelta >,
  /// User or system that created this version
  pub created_by : Option< String >,
  /// Human-readable description of the change
  pub description : Option< String >,
}

/// Configuration difference data for efficient storage
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ConfigDelta
{
  /// Fields that were changed
  pub changed_fields : HashMap<  String, serde_json::Value  >,
  /// Fields that were added
  pub added_fields : HashMap<  String, serde_json::Value  >,
  /// Fields that were removed
  pub removed_fields : Vec< String >,
  /// Tag changes
  pub tag_changes : HashMap< String, Option< String > >, // None = removed, Some = added/changed
}

impl ConfigDelta
{
  /// Create a delta between two configurations
  pub fn create_delta( old_config : &DynamicConfig, new_config : &DynamicConfig ) -> Self
  {
    let mut changed_fields = HashMap::new();
    let added_fields = HashMap::new();
    let removed_fields = Vec::new();
    let mut tag_changes = HashMap::new();

    // Check basic fields
    if old_config.timeout != new_config.timeout
    {
      changed_fields.insert( "timeout".to_string(), serde_json::to_value( new_config.timeout ).unwrap() );
    }
    if old_config.retry_attempts != new_config.retry_attempts
    {
      changed_fields.insert( "retry_attempts".to_string(), serde_json::to_value( new_config.retry_attempts ).unwrap() );
    }
    if old_config.base_url != new_config.base_url
    {
      changed_fields.insert( "base_url".to_string(), serde_json::to_value( &new_config.base_url ).unwrap() );
    }
    if old_config.enable_jitter != new_config.enable_jitter
    {
      changed_fields.insert( "enable_jitter".to_string(), serde_json::to_value( new_config.enable_jitter ).unwrap() );
    }
    if old_config.max_retry_delay != new_config.max_retry_delay
    {
      changed_fields.insert( "max_retry_delay".to_string(), serde_json::to_value( new_config.max_retry_delay ).unwrap() );
    }
    if old_config.base_retry_delay != new_config.base_retry_delay
    {
      changed_fields.insert( "base_retry_delay".to_string(), serde_json::to_value( new_config.base_retry_delay ).unwrap() );
    }
    if old_config.backoff_multiplier != new_config.backoff_multiplier
    {
      changed_fields.insert( "backoff_multiplier".to_string(), serde_json::to_value( new_config.backoff_multiplier ).unwrap() );
    }
    if old_config.source_priority != new_config.source_priority
    {
      changed_fields.insert( "source_priority".to_string(), serde_json::to_value( new_config.source_priority ).unwrap() );
    }

    // Check tag changes
    for ( key, old_value ) in &old_config.tags
    {
      match new_config.tags.get( key )
      {
        Some( new_value ) => {
          if old_value != new_value
          {
            tag_changes.insert( key.clone(), Some( new_value.clone() ) );
          }
        },
        None => {
          tag_changes.insert( key.clone(), None ); // Tag was removed
        }
      }
    }

    // Check for new tags
    for ( key, new_value ) in &new_config.tags
    {
      if !old_config.tags.contains_key( key )
      {
        tag_changes.insert( key.clone(), Some( new_value.clone() ) );
      }
    }

    Self {
      changed_fields,
      added_fields,
      removed_fields,
      tag_changes,
    }
  }

  /// Apply delta to a configuration to get the new configuration
  pub fn apply_delta( &self, base_config : &DynamicConfig ) -> Result< DynamicConfig, serde_json::Error >
  {
    let mut new_config = base_config.clone();

    // Apply field changes
    for ( field, value ) in &self.changed_fields
    {
      match field.as_str()
      {
        "timeout" => new_config.timeout = serde_json::from_value( value.clone() )?,
        "retry_attempts" => new_config.retry_attempts = serde_json::from_value( value.clone() )?,
        "base_url" => new_config.base_url = serde_json::from_value( value.clone() )?,
        "enable_jitter" => new_config.enable_jitter = serde_json::from_value( value.clone() )?,
        "max_retry_delay" => new_config.max_retry_delay = serde_json::from_value( value.clone() )?,
        "base_retry_delay" => new_config.base_retry_delay = serde_json::from_value( value.clone() )?,
        "backoff_multiplier" => new_config.backoff_multiplier = serde_json::from_value( value.clone() )?,
        "source_priority" => new_config.source_priority = serde_json::from_value( value.clone() )?,
        _ => {} // Ignore unknown fields
      }
    }

    // Apply tag changes
    for ( key, change ) in &self.tag_changes
    {
      match change
      {
        Some( new_value ) => {
          new_config.tags.insert( key.clone(), new_value.clone() );
        },
        None => {
          new_config.tags.remove( key );
        }
      }
    }

    // Invalidate validation cache since configuration changed
    new_config.validation_hash = None;

    Ok( new_config )
  }

  /// Calculate memory footprint of this delta
  pub fn memory_footprint( &self ) -> usize
  {
    serde_json ::to_string( self ).unwrap_or_default().len()
  }
}

impl ConfigHistoryEntry
{
  /// Create a new history entry from a configuration
  pub fn from_config( config : DynamicConfig, change_type : ConfigChangeType, version_id : String ) -> Self
  {
    let config_hash = config.compute_hash();
    let size_bytes = serde_json::to_string( &config ).unwrap_or_default().len();

    Self {
      version_id,
      config,
      timestamp : SystemTime::now(),
      change_type,
      config_hash,
      size_bytes,
      delta : None,
      created_by : None,
      description : None,
    }
  }

  /// Create a new history entry with delta compression
  pub fn from_config_with_delta(
    config : DynamicConfig,
    change_type : ConfigChangeType,
    version_id : String,
    previous_config : Option< &DynamicConfig >,
    created_by : Option< String >,
    description : Option< String >
  ) -> Self
  {
    let config_hash = config.compute_hash();
    let size_bytes = serde_json::to_string( &config ).unwrap_or_default().len();

    let delta = previous_config.map( |prev| ConfigDelta::create_delta( prev, &config ) );

    Self {
      version_id,
      config,
      timestamp : SystemTime::now(),
      change_type,
      config_hash,
      size_bytes,
      delta,
      created_by,
      description,
    }
  }

  /// Get the effective memory footprint including delta compression
  pub fn effective_memory_footprint( &self ) -> usize
  {
    if let Some( delta ) = &self.delta
    {
      // If we have a delta, use its footprint instead of full config
      delta.memory_footprint()
    } else {
      self.size_bytes
    }
  }

  /// Check if this entry can be compressed using delta
  pub fn can_compress( &self ) -> bool
  {
    self.delta.is_some()
  }

  /// Reconstruct configuration from delta if available
  pub fn reconstruct_config( &self, base_config : &DynamicConfig ) -> Result< DynamicConfig, serde_json::Error >
  {
    if let Some( delta ) = &self.delta
    {
      delta.apply_delta( base_config )
    } else {
      Ok( self.config.clone() )
    }
  }
}
