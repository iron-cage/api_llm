//! Streaming types for Server-Sent Events
//!
//! `StreamMessage`, `StreamContentBlock`, `StreamDelta`, `StreamEvent`.

#[ cfg( feature = "streaming" ) ]
mod private
{
  #[ cfg( feature = "error-handling" ) ]
  use crate::error::{ AnthropicError, AnthropicResult };
  
  #[ cfg( not( feature = "error-handling" ) ) ]
  type AnthropicError = crate::error_tools::Error;
  #[ cfg( not( feature = "error-handling" ) ) ]
  type AnthropicResult< T > = Result< T, crate::error_tools::Error >;

  use serde::{ Serialize, Deserialize };
  use core::pin::Pin;
  
  #[ cfg( feature = "streaming" ) ]
  use futures::Stream;

  /// Stream message structure for streaming responses
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct StreamMessage
  {
    /// Message ID
    pub id : String,
    /// Message type
    pub r#type : String,
    /// Role of the message
    pub role : String,
    /// Content blocks
    pub content : Vec< StreamContentBlock >,
    /// Model used
    pub model : String,
    /// Stop reason if completed
    pub stop_reason : Option< String >,
    /// Stop sequence if applicable
    pub stop_sequence : Option< String >,
    /// Usage statistics
    pub usage : crate::Usage,
  }

  impl StreamMessage
  {
    /// Create a new stream message
    #[ inline ]
    #[ must_use ]
    pub fn new< S1 : Into< String >, S2 : Into< String >, S3 : Into< String >, S4 : Into< String > >(
      id : S1,
      message_type : S2,
      role : S3,
      model : S4,
      usage : crate::Usage
    ) -> Self
    {
      Self
      {
        id : id.into(),
        r#type : message_type.into(),
        role : role.into(),
        content : Vec::new(),
        model : model.into(),
        stop_reason : None,
        stop_sequence : None,
        usage,
      }
    }

    /// Check if the stream message is complete
    #[ inline ]
    #[ must_use ]
    pub fn is_complete( &self ) -> bool
    {
      self.stop_reason.is_some()
    }

    /// Get the stop reason if available
    #[ inline ]
    #[ must_use ]
    pub fn stop_reason( &self ) -> Option< &str >
    {
      self.stop_reason.as_deref()
    }

    /// Check if message has any content
    #[ inline ]
    #[ must_use ]
    pub fn has_content( &self ) -> bool
    {
      !self.content.is_empty()
    }

    /// Get total content blocks count
    #[ inline ]
    #[ must_use ]
    pub fn content_count( &self ) -> usize
    {
      self.content.len()
    }

    /// Validate stream message structure
    ///
    /// # Errors
    ///
    /// Returns an error if the message ID, type, or model is empty, or if any content block is invalid
    #[ inline ]
    pub fn validate( &self ) -> AnthropicResult< () >
    {
      if self.id.is_empty()
      {
        #[ cfg( feature = "error-handling" ) ]
        return Err( AnthropicError::InvalidArgument( "Message ID cannot be empty".to_string() ) );
        #[ cfg( not( feature = "error-handling" ) ) ]
        return Err( crate::error_tools::Error::msg( "Message ID cannot be empty" ) );
      }

      if self.r#type.is_empty()
      {
        #[ cfg( feature = "error-handling" ) ]
        return Err( AnthropicError::InvalidArgument( "Message type cannot be empty".to_string() ) );
        #[ cfg( not( feature = "error-handling" ) ) ]
        return Err( crate::error_tools::Error::msg( "Message type cannot be empty" ) );
      }

      if self.model.is_empty()
      {
        #[ cfg( feature = "error-handling" ) ]
        return Err( AnthropicError::InvalidArgument( "Model cannot be empty".to_string() ) );
        #[ cfg( not( feature = "error-handling" ) ) ]
        return Err( crate::error_tools::Error::msg( "Model cannot be empty" ) );
      }

      for ( index, content_block ) in self.content.iter().enumerate()
      {
        content_block.validate()
          .map_err( | e | 
          {
            #[ cfg( feature = "error-handling" ) ]
            return AnthropicError::InvalidArgument( format!( "Invalid content block at index {index}: {e}" ) );
            #[ cfg( not( feature = "error-handling" ) ) ]
            return crate::error_tools::Error::msg( format!( "Invalid content block at index {index}: {e}" ) );
          } )?;
      }

      Ok( () )
    }
  }

  /// Content block in streaming response
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ serde( untagged ) ]
  pub enum StreamContentBlock
  {
    /// Text content block
    Text
    {
      /// Type field
      r#type : String,
      /// Text content
      text : String,
    },
    /// Tool use content block
    #[ cfg( feature = "tools" ) ]
    ToolUse
    {
      /// Type field
      r#type : String,
      /// Tool use ID
      id : String,
      /// Tool name
      name : String,
      /// Tool input
      input : serde_json::Value,
    },
  }

  impl StreamContentBlock
  {
    /// Create a new text content block
    #[ inline ]
    #[ must_use ]
    pub fn new_text< S : Into< String > >( text : S ) -> Self
    {
      Self::Text
      {
        r#type : "text".to_string(),
        text : text.into(),
      }
    }

    /// Create a new tool use content block
    #[ cfg( feature = "tools" ) ]
    #[ inline ]
    #[ must_use ]
    pub fn new_tool_use< S1 : Into< String >, S2 : Into< String > >( id : S1, name : S2, input : serde_json::Value ) -> Self
    {
      Self::ToolUse
      {
        r#type : "tool_use".to_string(),
        id : id.into(),
        name : name.into(),
        input,
      }
    }

    /// Get the content type
    #[ inline ]
    #[ must_use ]
    #[ allow( clippy::match_same_arms ) ] // Different enum variants with conditional compilation
    pub fn content_type( &self ) -> &str
    {
      match self
      {
        StreamContentBlock::Text { r#type, .. } => r#type,
        #[ cfg( feature = "tools" ) ]
        StreamContentBlock::ToolUse { r#type, .. } => r#type,
      }
    }

    /// Check if this is a text content block
    #[ inline ]
    #[ must_use ]
    pub fn is_text( &self ) -> bool
    {
      matches!( self, StreamContentBlock::Text { .. } )
    }

    /// Check if this is a tool use content block
    #[ cfg( feature = "tools" ) ]
    #[ inline ]
    #[ must_use ]
    pub fn is_tool_use( &self ) -> bool
    {
      matches!( self, StreamContentBlock::ToolUse { .. } )
    }

    /// Get text content if this is a text block
    #[ inline ]
    #[ must_use ]
    pub fn text( &self ) -> Option< &str >
    {
      match self
      {
        StreamContentBlock::Text { text, .. } => Some( text ),
        #[ cfg( feature = "tools" ) ]
        StreamContentBlock::ToolUse { .. } => None,
      }
    }

    /// Get tool name if this is a tool use block
    #[ cfg( feature = "tools" ) ]
    #[ inline ]
    #[ must_use ]
    pub fn tool_name( &self ) -> Option< &str >
    {
      match self
      {
        StreamContentBlock::Text { .. } => None,
        StreamContentBlock::ToolUse { name, .. } => Some( name ),
      }
    }

    /// Validate content block structure
    ///
    /// # Errors
    ///
    /// Returns an error if the content block type is invalid or required fields are missing
    #[ inline ]
    pub fn validate( &self ) -> AnthropicResult< () >
    {
      match self
      {
        StreamContentBlock::Text { r#type, text : _ } =>
        {
          if r#type != "text"
          {
            #[ cfg( feature = "error-handling" ) ]
            return Err( AnthropicError::InvalidArgument( format!( "Invalid text content type : '{type}'" ) ) );
            #[ cfg( not( feature = "error-handling" ) ) ]
            return Err( crate::error_tools::Error::msg( format!( "Invalid text content type : '{type}'" ) ) );
          }

          // Note : Empty text is allowed in streaming scenarios as content starts empty and gets deltas
        },
        #[ cfg( feature = "tools" ) ]
        StreamContentBlock::ToolUse { r#type, id, name, .. } =>
        {
          if r#type != "tool_use"
          {
            #[ cfg( feature = "error-handling" ) ]
            return Err( AnthropicError::InvalidArgument( format!( "Invalid tool use content type : '{type}'" ) ) );
            #[ cfg( not( feature = "error-handling" ) ) ]
            return Err( crate::error_tools::Error::msg( format!( "Invalid tool use content type : '{type}'" ) ) );
          }

          if id.is_empty()
          {
            #[ cfg( feature = "error-handling" ) ]
            return Err( AnthropicError::InvalidArgument( "Tool use ID cannot be empty".to_string() ) );
            #[ cfg( not( feature = "error-handling" ) ) ]
            return Err( crate::error_tools::Error::msg( "Tool use ID cannot be empty" ) );
          }

          if name.is_empty()
          {
            #[ cfg( feature = "error-handling" ) ]
            return Err( AnthropicError::InvalidArgument( "Tool name cannot be empty".to_string() ) );
            #[ cfg( not( feature = "error-handling" ) ) ]
            return Err( crate::error_tools::Error::msg( "Tool name cannot be empty" ) );
          }
        }
      }

      Ok( () )
    }
  }

  /// Delta updates for streaming content
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ serde( untagged ) ]
  pub enum StreamDelta
  {
    /// Text delta update
    TextDelta
    {
      /// Type field
      r#type : String,
      /// Text delta
      text : String,
    },
    /// Input JSON delta for tools
    #[ cfg( feature = "tools" ) ]
    InputJsonDelta
    {
      /// Type field
      r#type : String,
      /// Partial JSON input
      partial_json : String,
    },
  }

  impl StreamDelta
  {
    /// Create a new text delta
    #[ inline ]
    #[ must_use ]
    pub fn new_text< S : Into< String > >( text : S ) -> Self
    {
      Self::TextDelta
      {
        r#type : "text_delta".to_string(),
        text : text.into(),
      }
    }

    /// Create a new input JSON delta
    #[ cfg( feature = "tools" ) ]
    #[ inline ]
    #[ must_use ]
    pub fn new_input_json< S : Into< String > >( partial_json : S ) -> Self
    {
      Self::InputJsonDelta
      {
        r#type : "input_json_delta".to_string(),
        partial_json : partial_json.into(),
      }
    }

    /// Get the delta type
    #[ inline ]
    #[ must_use ]
    #[ allow( clippy::match_same_arms ) ] // Different enum variants with conditional compilation
    pub fn delta_type( &self ) -> &str
    {
      match self
      {
        StreamDelta::TextDelta { r#type, .. } => r#type,
        #[ cfg( feature = "tools" ) ]
        StreamDelta::InputJsonDelta { r#type, .. } => r#type,
      }
    }

    /// Check if this is a text delta
    #[ inline ]
    #[ must_use ]
    pub fn is_text_delta( &self ) -> bool
    {
      matches!( self, StreamDelta::TextDelta { .. } )
    }

    /// Check if this is an input JSON delta
    #[ cfg( feature = "tools" ) ]
    #[ inline ]
    #[ must_use ]
    pub fn is_input_json_delta( &self ) -> bool
    {
      matches!( self, StreamDelta::InputJsonDelta { .. } )
    }

    /// Get text content if this is a text delta
    #[ inline ]
    #[ must_use ]
    pub fn text( &self ) -> Option< &str >
    {
      match self
      {
        StreamDelta::TextDelta { text, .. } => Some( text ),
        #[ cfg( feature = "tools" ) ]
        StreamDelta::InputJsonDelta { .. } => None,
      }
    }

    /// Get partial JSON if this is an input JSON delta
    #[ cfg( feature = "tools" ) ]
    #[ inline ]
    #[ must_use ]
    pub fn partial_json( &self ) -> Option< &str >
    {
      match self
      {
        StreamDelta::TextDelta { .. } => None,
        StreamDelta::InputJsonDelta { partial_json, .. } => Some( partial_json ),
      }
    }

    /// Validate delta structure
    ///
    /// # Errors
    ///
    /// Returns an error if the delta type is invalid or required fields are missing
    #[ inline ]
    pub fn validate( &self ) -> AnthropicResult< () >
    {
      match self
      {
        StreamDelta::TextDelta { r#type, text : _ } =>
        {
          if r#type != "text_delta"
          {
            #[ cfg( feature = "error-handling" ) ]
            return Err( AnthropicError::InvalidArgument( format!( "Invalid text delta type : '{type}'" ) ) );
            #[ cfg( not( feature = "error-handling" ) ) ]
            return Err( crate::error_tools::Error::msg( format!( "Invalid text delta type : '{type}'" ) ) );
          }

          // Note : Empty text deltas are allowed in streaming scenarios (e.g., whitespace-only deltas)
        },
        #[ cfg( feature = "tools" ) ]
        StreamDelta::InputJsonDelta { r#type, partial_json } =>
        {
          if r#type != "input_json_delta"
          {
            #[ cfg( feature = "error-handling" ) ]
            return Err( AnthropicError::InvalidArgument( format!( "Invalid input JSON delta type : '{type}'" ) ) );
            #[ cfg( not( feature = "error-handling" ) ) ]
            return Err( crate::error_tools::Error::msg( format!( "Invalid input JSON delta type : '{type}'" ) ) );
          }

          if partial_json.is_empty()
          {
            #[ cfg( feature = "error-handling" ) ]
            return Err( AnthropicError::InvalidArgument( "Partial JSON cannot be empty".to_string() ) );
            #[ cfg( not( feature = "error-handling" ) ) ]
            return Err( crate::error_tools::Error::msg( "Partial JSON cannot be empty" ) );
          }
        }
      }

      Ok( () )
    }
  }

  /// Streaming events from Server-Sent Events
  #[ derive( Debug, Clone ) ]
  pub enum StreamEvent
  {
    /// Message start event
    MessageStart
    {
      /// The message being started
      message : StreamMessage,
    },
    /// Content block start event
    ContentBlockStart
    {
      /// Index of the content block
      index : usize,
      /// The content block being started
      content_block : StreamContentBlock,
    },
    /// Content block delta event
    ContentBlockDelta
    {
      /// Index of the content block
      index : usize,
      /// The delta update
      delta : StreamDelta,
    },
    /// Content block stop event
    ContentBlockStop
    {
      /// Index of the content block
      index : usize,
    },
    /// Message stop event
    MessageStop,
    /// Error event
    Error
    {
      /// Error details
      error : AnthropicError,
    },
  }

  impl StreamEvent
  {
    /// Create a message start event
    #[ inline ]
    #[ must_use ]
    pub fn message_start( message : StreamMessage ) -> Self
    {
      Self::MessageStart { message }
    }

    /// Create a content block start event
    #[ inline ]
    #[ must_use ]
    pub fn content_block_start( index : usize, content_block : StreamContentBlock ) -> Self
    {
      Self::ContentBlockStart { index, content_block }
    }

    /// Create a content block delta event
    #[ inline ]
    #[ must_use ]
    pub fn content_block_delta( index : usize, delta : StreamDelta ) -> Self
    {
      Self::ContentBlockDelta { index, delta }
    }

    /// Create a content block stop event
    #[ inline ]
    #[ must_use ]
    pub fn content_block_stop( index : usize ) -> Self
    {
      Self::ContentBlockStop { index }
    }

    /// Create a message stop event
    #[ inline ]
    #[ must_use ]
    pub fn message_stop() -> Self
    {
      Self::MessageStop
    }

    /// Create an error event
    #[ inline ]
    #[ must_use ]
    pub fn from_error( error : AnthropicError ) -> Self
    {
      Self::Error { error }
    }

    /// Check if this is a message start event
    #[ inline ]
    #[ must_use ]
    pub fn is_message_start( &self ) -> bool
    {
      matches!( self, StreamEvent::MessageStart { .. } )
    }

    /// Check if this is a content block start event
    #[ inline ]
    #[ must_use ]
    pub fn is_content_block_start( &self ) -> bool
    {
      matches!( self, StreamEvent::ContentBlockStart { .. } )
    }

    /// Check if this is a content block delta event
    #[ inline ]
    #[ must_use ]
    pub fn is_content_block_delta( &self ) -> bool
    {
      matches!( self, StreamEvent::ContentBlockDelta { .. } )
    }

    /// Check if this is a content block stop event
    #[ inline ]
    #[ must_use ]
    pub fn is_content_block_stop( &self ) -> bool
    {
      matches!( self, StreamEvent::ContentBlockStop { .. } )
    }

    /// Check if this is a message stop event
    #[ inline ]
    #[ must_use ]
    pub fn is_message_stop( &self ) -> bool
    {
      matches!( self, StreamEvent::MessageStop )
    }

    /// Check if this is an error event
    #[ inline ]
    #[ must_use ]
    pub fn is_error( &self ) -> bool
    {
      matches!( self, StreamEvent::Error { .. } )
    }

    /// Get the content block index if applicable
    #[ inline ]
    #[ must_use ]
    pub fn content_block_index( &self ) -> Option< usize >
    {
      match self
      {
        StreamEvent::ContentBlockStart { index, .. } |
        StreamEvent::ContentBlockDelta { index, .. } |
        StreamEvent::ContentBlockStop { index, .. } => Some( *index ),
        _ => None,
      }
    }

    /// Get the stream message if this is a message start event
    #[ inline ]
    #[ must_use ]
    pub fn message( &self ) -> Option< &StreamMessage >
    {
      match self
      {
        StreamEvent::MessageStart { message } => Some( message ),
        _ => None,
      }
    }

    /// Get the delta if this is a delta event
    #[ inline ]
    #[ must_use ]
    pub fn delta( &self ) -> Option< &StreamDelta >
    {
      match self
      {
        StreamEvent::ContentBlockDelta { delta, .. } => Some( delta ),
        _ => None,
      }
    }

    /// Get the error if this is an error event
    #[ inline ]
    #[ must_use ]
    pub fn error( &self ) -> Option< &AnthropicError >
    {
      match self
      {
        StreamEvent::Error { error } => Some( error ),
        _ => None,
      }
    }

    /// Validate the stream event structure
    ///
    /// # Errors
    ///
    /// Returns an error if any contained message, content block, or delta is invalid
    #[ inline ]
    pub fn validate( &self ) -> AnthropicResult< () >
    {
      match self
      {
        StreamEvent::MessageStart { message } =>
        {
          message.validate()
        },
        StreamEvent::ContentBlockStart { content_block, .. } =>
        {
          content_block.validate()
        },
        StreamEvent::ContentBlockDelta { delta, .. } =>
        {
          delta.validate()
        },
        StreamEvent::ContentBlockStop { .. } |
        StreamEvent::MessageStop |
        StreamEvent::Error { .. } =>
        {
          // These events don't need validation
          Ok( () )
        }
      }
    }
  }

  /// Parse Server-Sent Events data into stream events
  ///
  /// # Errors
  ///
  /// Returns an error if the SSE data is malformed or contains invalid JSON
  pub fn parse_sse_events( data : &str ) -> AnthropicResult< Vec< StreamEvent > >
  {
    let mut events = Vec::new();
    let mut current_event : Option< String > = None;
    let mut current_data = String::new();

    for line in data.lines()
    {
      let line = line.trim();
      
      if line.is_empty()
      {
        // Empty line indicates end of event
        if let Some( event_type ) = current_event.take()
        {
          if let Ok( event ) = parse_single_event( &event_type, &current_data )
          {
            events.push( event );
          }
          current_data.clear();
        }
        continue;
      }

      if let Some( event_line ) = line.strip_prefix( "event: " )
      {
        current_event = Some( event_line.to_string() );
      }
      else if let Some( data_line ) = line.strip_prefix( "data: " )
      {
        if !current_data.is_empty()
        {
          current_data.push( '\n' );
        }
        current_data.push_str( data_line );
      }
    }

    // Handle final event if no trailing empty line
    if let Some( event_type ) = current_event
    {
      if let Ok( event ) = parse_single_event( &event_type, &current_data )
      {
        events.push( event );
      }
    }

    Ok( events )
  }

  /// Parse a single SSE event with enhanced error handling
  #[ allow( clippy::too_many_lines ) ] // Complex parsing logic with multiple event types
  fn parse_single_event( event_type : &str, data : &str ) -> AnthropicResult< StreamEvent >
  {
    validate_event_input( event_type, data )?;

    match event_type
    {
      "message_start" => parse_message_start( data ),
      "content_block_start" => parse_content_block_start( data ),
      "content_block_delta" => parse_content_block_delta( data ),
      "content_block_stop" => parse_content_block_stop( data ),
      "message_stop" => Ok( StreamEvent::MessageStop ),
      "error" => parse_error_event( data ),
      _ => parse_unknown_event( event_type ),
    }
  }

  /// Validate input parameters for event parsing
  #[ inline ]
  fn validate_event_input( event_type : &str, data : &str ) -> AnthropicResult< () >
  {
    if event_type.is_empty()
    {
      #[ cfg( feature = "error-handling" ) ]
      return Err( AnthropicError::InvalidArgument( "Event type cannot be empty".to_string() ) );
      #[ cfg( not( feature = "error-handling" ) ) ]
      return Err( crate::error_tools::Error::msg( "Event type cannot be empty" ) );
    }

    if data.is_empty() && event_type != "message_stop"
    {
      #[ cfg( feature = "error-handling" ) ]
      return Err( AnthropicError::InvalidArgument( format!( "Event data cannot be empty for event type : {event_type}" ) ) );
      #[ cfg( not( feature = "error-handling" ) ) ]
      return Err( crate::error_tools::Error::msg( format!( "Event data cannot be empty for event type : {event_type}" ) ) );
    }

    Ok( () )
  }

  /// Parse `message_start` event
  fn parse_message_start( data : &str ) -> AnthropicResult< StreamEvent >
  {
    let message : StreamMessage = serde_json::from_str( data )
      .map_err( | e | 
      {
        #[ cfg( feature = "error-handling" ) ]
        return AnthropicError::Parsing( format!( "Failed to parse message_start : {e}" ) );
        #[ cfg( not( feature = "error-handling" ) ) ]
        return crate::error_tools::Error::msg( format!( "Failed to parse message_start : {e}" ) );
      } )?;

    message.validate()?;
    Ok( StreamEvent::MessageStart { message } )
  }

  /// Parse `content_block_start` event
  fn parse_content_block_start( data : &str ) -> AnthropicResult< StreamEvent >
  {
    #[ derive( Deserialize ) ]
    struct ContentBlockStartData
    {
      index : usize,
      content_block : StreamContentBlock,
    }

    let event_data : ContentBlockStartData = serde_json::from_str( data )
      .map_err( | e | 
      {
        #[ cfg( feature = "error-handling" ) ]
        return AnthropicError::Parsing( format!( "Failed to parse content_block_start : {e}" ) );
        #[ cfg( not( feature = "error-handling" ) ) ]
        return crate::error_tools::Error::msg( format!( "Failed to parse content_block_start : {e}" ) );
      } )?;

    event_data.content_block.validate()?;
    Ok( StreamEvent::ContentBlockStart
    {
      index : event_data.index,
      content_block : event_data.content_block,
    } )
  }

  /// Parse `content_block_delta` event
  fn parse_content_block_delta( data : &str ) -> AnthropicResult< StreamEvent >
  {
    #[ derive( Deserialize ) ]
    struct ContentBlockDeltaData
    {
      index : usize,
      delta : StreamDelta,
    }

    let event_data : ContentBlockDeltaData = serde_json::from_str( data )
      .map_err( | e | 
      {
        #[ cfg( feature = "error-handling" ) ]
        return AnthropicError::Parsing( format!( "Failed to parse content_block_delta : {e}" ) );
        #[ cfg( not( feature = "error-handling" ) ) ]
        return crate::error_tools::Error::msg( format!( "Failed to parse content_block_delta : {e}" ) );
      } )?;

    event_data.delta.validate()?;
    Ok( StreamEvent::ContentBlockDelta
    {
      index : event_data.index,
      delta : event_data.delta,
    } )
  }

  /// Parse `content_block_stop` event
  fn parse_content_block_stop( data : &str ) -> AnthropicResult< StreamEvent >
  {
    #[ derive( Deserialize ) ]
    struct ContentBlockStopData
    {
      index : usize,
    }

    let event_data : ContentBlockStopData = serde_json::from_str( data )
      .map_err( | e | 
      {
        #[ cfg( feature = "error-handling" ) ]
        return AnthropicError::Parsing( format!( "Failed to parse content_block_stop : {e}" ) );
        #[ cfg( not( feature = "error-handling" ) ) ]
        return crate::error_tools::Error::msg( format!( "Failed to parse content_block_stop : {e}" ) );
      } )?;
    
    Ok( StreamEvent::ContentBlockStop { index : event_data.index } )
  }

  /// Parse error event
  fn parse_error_event( data : &str ) -> AnthropicResult< StreamEvent >
  {
    #[ cfg( feature = "error-handling" ) ]
    {
      let api_error : crate::error::AnthropicApiError = serde_json::from_str( data )
        .map_err( | e | AnthropicError::Parsing( format!( "Failed to parse error : {e}" ) ) )?;
      Ok( StreamEvent::Error { error : AnthropicError::Api( api_error ) } )
    }
    #[ cfg( not( feature = "error-handling" ) ) ]
    {
      let error_msg = format!( "API error : {data}" );
      Ok( StreamEvent::Error { error : crate::error_tools::Error::msg( error_msg ) } )
    }
  }

  /// Handle unknown event type
  fn parse_unknown_event( event_type : &str ) -> AnthropicResult< StreamEvent >
  {
    #[ cfg( feature = "error-handling" ) ]
    return Err( AnthropicError::Parsing( format!( "Unknown event type : '{event_type}'. Supported types : message_start, content_block_start, content_block_delta, content_block_stop, message_stop, error" ) ) );
    #[ cfg( not( feature = "error-handling" ) ) ]
    return Err( crate::error_tools::Error::msg( format!( "Unknown event type : '{event_type}'. Supported types : message_start, content_block_start, content_block_delta, content_block_stop, message_stop, error" ) ) );
  }

  /// Stream of Server-Sent Events
  #[ cfg( feature = "streaming" ) ]
  pub type EventStream = Pin< Box< dyn Stream< Item = AnthropicResult< StreamEvent > > + Send + 'static > >;

}

#[ cfg( feature = "streaming" ) ]
crate::mod_interface!
{
  exposed use StreamMessage;
  exposed use StreamContentBlock;
  exposed use StreamDelta;
  exposed use StreamEvent;
  exposed use EventStream;
  exposed use parse_sse_events;
}
