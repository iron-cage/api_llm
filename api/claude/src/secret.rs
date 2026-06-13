//! Secret management for Anthropic API

mod private
{
  use std::path::Path;
  use error_tools::untyped::Result;

  /// Anthropic API key secret
  ///
  /// # Examples
  ///
  /// ```
  /// use api_claude::Secret;
  ///
  /// // Create a secret with valid API key
  /// let secret = Secret::new( "sk-ant-api03-example".to_string() ).unwrap();
  ///
  /// // Invalid keys will return an error
  /// let invalid_secret = Secret::new( "invalid-key".to_string() );
  /// assert!( invalid_secret.is_err() );
  ///
  /// // Empty keys will return an error
  /// let empty_secret = Secret::new( "".to_string() );
  /// assert!( empty_secret.is_err() );
  /// ```
  #[ derive( Clone ) ]
  #[ allow( non_snake_case ) ] // Following workspace pattern for environment variable names
  pub struct Secret
  {
    /// Anthropic API key
    pub ANTHROPIC_API_KEY : String,
  }

  impl std::fmt::Debug for Secret
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      f.debug_struct( "Secret" )
        .field( "ANTHROPIC_API_KEY", &"< REDACTED >" )
        .finish()
    }
  }

  impl Secret
  {
    /// Create new secret with API key
    ///
    /// # Errors
    ///
    /// Returns an error if the API key is empty or has invalid format
    pub fn new( api_key : String ) -> Result< Self >
    {
      if api_key.trim().is_empty()
      {
        return Err( error_tools::Error::msg( "API key cannot be empty" ) );
      }

      if !api_key.starts_with( "sk-ant-" )
      {
        return Err( error_tools::Error::msg( "Invalid Anthropic API key format - must start with 'sk-ant-'" ) );
      }

      Ok( Self
      {
        ANTHROPIC_API_KEY : api_key,
      })
    }

    /// Create secret without validation (for testing)
    #[ inline ]
    #[ must_use ]
    pub fn new_unchecked( api_key : String ) -> Self
    {
      Self
      {
        ANTHROPIC_API_KEY : api_key,
      }
    }

    /// Load secret from environment variable
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable is not found or the API key is invalid
    #[ inline ]
    pub fn load_from_env( env_var : &str ) -> Result< Self >
    {
      let api_key = std::env::var( env_var )
        .map_err( | e | error_tools::Error::msg(
          format!(
            "Missing environment variable '{env_var}'.\n\
             Error : {e}\n\
             Hint : Set the environment variable:\n\
             - Linux/Mac : export {env_var}=\"your-api-key\"\n\
             - Or source workspace secrets : source /path/to/workspace/secret/-secrets.sh"
          )
        ) )?;

      Self::new( api_key )
    }

    /// Load secret from file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or the API key is invalid
    #[ inline ]
    pub fn load_from_file( path : &Path ) -> Result< Self >
    {
      let api_key = std::fs::read_to_string( path )
        .map_err( | e | error_tools::Error::msg( format!( "Failed to read secret file : {e}" ) ) )?;
      
      Self::new( api_key.trim().to_string() )
    }

    /// Load from workspace secrets (`workspace_tools` pattern)
    ///
    /// # Errors
    ///
    /// Returns an error if workspace loading fails or the API key is invalid
    #[ inline ]
    pub fn load_from_workspace( key_name : &str, filename : &str ) -> Result< Self >
    {
      // Try workspace_tools first, fall back to manual search if it fails
      let secret_dir = Self::find_secret_directory()?;
      let secret_file_path = secret_dir.join( filename );
      let secret_file_abs = secret_file_path.canonicalize()
        .unwrap_or_else( | _ | secret_file_path.clone() );

      // Load secret directly from file instead of using workspace_tools
      // to ensure we use secret/ not .secret/
      let content = std::fs::read_to_string( &secret_file_path )
        .map_err( | e | error_tools::Error::msg(
          format!(
            "Failed to read secrets file.\n\
             Tried file : {}\n\
             Secret directory : {}\n\
             Error : {e}\n\
             Hint : Ensure secret/ directory exists at workspace root with {} file",
            secret_file_abs.display(),
            secret_dir.display(),
            filename
          )
        ) )?;

      // Parse the shell-format file for the key
      let api_key = content
        .lines()
        .find_map( | line | {
          let line = line.trim();
          // Match both "export KEY=value" and "KEY=value" formats
          if line.starts_with( "export " )
          {
            let line = line.strip_prefix( "export " )?.trim();
            Self::parse_key_value( line, key_name )
          }
          else
          {
            Self::parse_key_value( line, key_name )
          }
        })
        .ok_or_else( || error_tools::Error::msg(
          format!(
            "Key '{key_name}' not found in secrets file.\n\
             Tried file : {}\n\
             Hint : Add this line to the file:\n\
             export {key_name}=\"your-api-key\"",
            secret_file_abs.display()
          )
        ) )?;

      Self::new( api_key )
    }

    /// Find the secret/ directory using `workspace_tools` or manual search
    ///
    /// # Errors
    ///
    /// Returns an error if secret directory cannot be found
    fn find_secret_directory() -> Result< std::path::PathBuf >
    {
      use workspace_tools::workspace;

      // Try workspace_tools first
      if let Ok( ws ) = workspace()
      {
        let ws_root = ws.root();
        let detected_root = if ws_root.is_absolute()
        {
          // Normalize absolute path to remove trailing ./
          let path_str = ws_root.to_string_lossy();
          if path_str.ends_with( "/." ) || path_str.ends_with( "\\." )
          {
            // Remove trailing /. component
            ws_root.parent().map_or_else( || ws_root.to_path_buf(), std::path::Path::to_path_buf )
          }
          else
          {
            ws_root.to_path_buf()
          }
        }
        else
        {
          // If relative, resolve from current dir
          std::env::current_dir()
            .map_err( | e | error_tools::Error::msg(
              format!( "Failed to get current directory : {e}" )
            ) )?
            .join( ws_root )
            .canonicalize()
            .map_err( | e | error_tools::Error::msg(
              format!( "Failed to canonicalize workspace root : {e}" )
            ) )?
        };

        // Check if secret/ exists at workspace root
        let secret_dir = detected_root.join( "secret" );
        if secret_dir.exists()
        {
          return Ok( secret_dir );
        }

        // Try parent directory (for workspace members)
        if let Some( parent ) = detected_root.parent()
        {
          let parent_secret = parent.join( "secret" );
          if parent_secret.exists()
          {
            return Ok( parent_secret );
          }
        }
      }

      // Fallback : search upward from current directory
      let start_dir = std::env::current_dir()
        .map_err( | e | error_tools::Error::msg(
          format!( "Failed to get current directory : {e}" )
        ) )?;

      let mut current = start_dir.clone();

      // Search upward until we find secret/ or reach filesystem root
      loop
      {
        let secret_dir = current.join( "secret" );
        if secret_dir.exists() && secret_dir.is_dir()
        {
          return Ok( secret_dir );
        }

        // Move up to parent
        match current.parent()
        {
          Some( parent ) => current = parent.to_path_buf(),
          None =>
          {
            // Reached filesystem root without finding secret/
            return Err( error_tools::Error::msg(
              format!(
                "Could not find secret/ directory.\n\
                 Searched from : {}\n\
                 Hint : Ensure secret/ directory exists at workspace root",
                start_dir.display()
              )
            ) );
          }
        }
      }
    }

    /// Parse a key=value line from shell format
    fn parse_key_value( line : &str, key_name : &str ) -> Option< String >
    {
      if line.starts_with( '#' ) || line.is_empty()
      {
        return None;
      }

      let parts : Vec< &str > = line.splitn( 2, '=' ).collect();
      if parts.len() != 2
      {
        return None;
      }

      if parts[ 0 ].trim() == key_name
      {
        // Remove surrounding quotes if present
        let value = parts[ 1 ].trim();
        let value = value.strip_prefix( '"' ).unwrap_or( value );
        let value = value.strip_suffix( '"' ).unwrap_or( value );
        let value = value.strip_prefix( '\'' ).unwrap_or( value );
        let value = value.strip_suffix( '\'' ).unwrap_or( value );
        Some( value.to_string() )
      }
      else
      {
        None
      }
    }

    /// Load secret from workspace with default settings
    ///
    /// # Errors
    ///
    /// Returns an error if workspace loading fails or the API key is invalid
    #[ inline ]
    pub fn from_workspace() -> Result< Self >
    {
      Self::load_from_workspace( "ANTHROPIC_API_KEY", "-secrets.sh" )
    }

    /// Load secret from an explicit shell-format file path
    ///
    /// Unlike `load_from_workspace`, this method does not search for a `secret/` directory —
    /// it reads the named key directly from the file at `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, the key is not found, or the key is invalid
    pub fn load_from_shell_file( path : &Path, key_name : &str ) -> Result< Self >
    {
      let content = std::fs::read_to_string( path )
        .map_err( | e | error_tools::Error::msg(
          format!(
            "Failed to read secrets file : {}\nError : {e}",
            path.display()
          )
        ) )?;

      let api_key = content
        .lines()
        .find_map( | line | {
          let line = line.trim();
          if line.starts_with( "export " )
          {
            let line = line.strip_prefix( "export " )?.trim();
            Self::parse_key_value( line, key_name )
          }
          else
          {
            Self::parse_key_value( line, key_name )
          }
        })
        .ok_or_else( || error_tools::Error::msg(
          format!(
            "Key '{key_name}' not found in file '{}'.\n\
             Hint : Add a line : export {key_name}=\"your-api-key\"",
            path.display()
          )
        ) )?;

      Self::new( api_key )
    }
  }
}

crate::mod_interface!
{
  exposed use Secret;
}