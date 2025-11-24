
//! Comprehensive Function Calling & Tool Integration Example
//!
//! This example demonstrates advanced AI agent capabilities including:
//! - Dynamic function/tool registration and execution
//! - Multi-step workflow orchestration
//! - External API integration simulation
//! - Security controls and validation
//! - Comprehensive error handling and logging
//! - Interactive and automated execution modes
//!
//! Usage:
//! ```bash
//! # Interactive mode with specific tools
//! cargo run --example gemini_function_calling -- --agent-mode interactive --tools weather,calculator,search
//!
//! # Execute specific task with all tools
//! cargo run --example gemini_function_calling -- --task "Plan a trip to Paris" --use-tools all
//!
//! # API integration demo
//! cargo run --example gemini_function_calling -- --demo api_integration --service weather_api
//! ```

use api_gemini::{ client::Client, models::* };
use serde_json::{ json, Value };
use std::collections::HashMap;
use std::env;
use std::time::{ Duration, Instant };
use tokio::time::timeout;

/// Configuration for the AI agent
#[ derive( Debug, Clone ) ]
pub struct AgentConfig
{
  /// Agent execution mode
  pub agent_mode: AgentMode,
  /// List of available tools for the agent
  pub available_tools: Vec< String >,
  /// Task description for automated mode
  pub task_description: Option< String >,
  /// Service name for demo mode
  pub demo_service: Option< String >,
  /// Maximum number of workflow iterations
  pub max_iterations: usize,
  /// Function timeout in seconds
  pub timeout_seconds: u64,
  /// Enable detailed logging
  pub logging_enabled: bool,
}

/// Agent execution mode
#[ derive( Debug, Clone ) ]
pub enum AgentMode
{
  /// Interactive mode with user input
  Interactive,
  /// Automated mode with predefined task
  Automated,
  /// Demo mode with predefined scenarios
  Demo( String ),
}

impl Default for AgentConfig
{
  fn default() -> Self
  {
    Self
    {
      agent_mode: AgentMode::Interactive,
      available_tools: vec![ "weather".to_string(), "calculator".to_string() ],
      task_description: None,
      demo_service: None,
      max_iterations: 10,
      timeout_seconds: 30,
      logging_enabled: true,
    }
  }
}

/// Function execution context with security and logging
#[ derive( Debug ) ]
pub struct FunctionContext
{
  /// Name of the executed function
  pub function_name: String,
  /// Arguments passed to the function
  pub arguments: Value,
  /// When function execution started
  pub execution_time: Instant,
  /// Maximum execution time allowed
  pub timeout: Duration,
  /// Whether parameter validation passed
  pub validation_passed: bool,
}

impl FunctionContext
{
  /// Create new function execution context
  pub fn new( name: String, args: Value, timeout_secs: u64 ) -> Self
  {
    Self
    {
      function_name: name,
      arguments: args,
      execution_time: Instant::now(),
      timeout: Duration::from_secs( timeout_secs ),
      validation_passed: false,
    }
  }

  /// Get elapsed execution time
  pub fn elapsed( &self ) -> Duration
  {
    self.execution_time.elapsed()
  }
}

/// Comprehensive tool registry with built-in functions
#[ derive( Debug ) ]
pub struct ToolRegistry
{
  available_functions: HashMap<  String, FunctionDeclaration  >,
  execution_log: Vec< FunctionContext >,
}

impl ToolRegistry
{
  /// Create new tool registry with default tools
  pub fn new() -> Self
  {
    let mut registry = Self
    {
      available_functions: HashMap::new(),
      execution_log: Vec::new(),
    };

    registry.register_default_tools();
    registry
  }

  fn register_default_tools( &mut self )
  {
    // Weather API tool
    self.available_functions.insert(
    "get_weather".to_string(),
    FunctionDeclaration
    {
      name: "get_weather".to_string(),
      description: "Get current weather conditions for a specific location".to_string(),
      parameters : Some( json!({
        "type": "object",
        "properties": {
          "location": {
            "type": "string",
            "description": "City name or location (e.g., 'San Francisco', 'Tokyo')"
          },
          "unit": {
            "type": "string",
            "enum": ["celsius", "fahrenheit"],
            "description": "Temperature unit preference",
            "default": "celsius"
          }
        },
        "required": ["location"]
      })),
    }
    );

    // Calculator tool
    self.available_functions.insert(
    "calculate".to_string(),
    FunctionDeclaration
    {
      name: "calculate".to_string(),
      description: "Perform mathematical calculations with support for basic arithmetic, percentages, and common functions".to_string(),
      parameters : Some( json!({
        "type": "object",
        "properties": {
          "expression": {
            "type": "string",
            "description": "Mathematical expression to evaluate (e.g., '2 + 3 * 4', 'sqrt(16)', '15% of 200')"
          },
          "precision": {
            "type": "integer",
            "description": "Number of decimal places for the result",
            "default": 2,
            "minimum": 0,
            "maximum": 10
          }
        },
        "required": ["expression"]
      })),
    }
    );

    // Search tool
    self.available_functions.insert(
    "web_search".to_string(),
    FunctionDeclaration
    {
      name: "web_search".to_string(),
      description: "Search the web for information on a specific topic or query".to_string(),
      parameters : Some( json!({
        "type": "object",
        "properties": {
          "query": {
            "type": "string",
            "description": "Search query or keywords"
          },
          "max_results": {
            "type": "integer",
            "description": "Maximum number of search results to return",
            "default": 5,
            "minimum": 1,
            "maximum": 20
          },
          "language": {
            "type": "string",
            "description": "Preferred language for results",
            "default": "en"
          }
        },
        "required": ["query"]
      })),
    }
    );

    // Flight search tool
    self.available_functions.insert(
    "search_flights".to_string(),
    FunctionDeclaration
    {
      name: "search_flights".to_string(),
      description: "Search for available flights between two locations".to_string(),
      parameters : Some( json!({
        "type": "object",
        "properties": {
          "from": {
            "type": "string",
            "description": "Departure city or airport code"
          },
          "to": {
            "type": "string",
            "description": "Arrival city or airport code"
          },
          "date": {
            "type": "string",
            "description": "Flight date in YYYY-MM-DD format"
          },
          "passengers": {
            "type": "integer",
            "description": "Number of passengers",
            "default": 1,
            "minimum": 1,
            "maximum": 9
          },
          "class": {
            "type": "string",
            "enum": ["economy", "business", "first"],
            "description": "Flight class preference",
            "default": "economy"
          }
        },
        "required": ["from", "to", "date"]
      })),
    }
    );

    // Database query tool
    self.available_functions.insert(
    "query_database".to_string(),
    FunctionDeclaration
    {
      name: "query_database".to_string(),
      description: "Execute queries against a simulated database for user data, orders, or analytics".to_string(),
      parameters : Some( json!({
        "type": "object",
        "properties": {
          "query_type": {
            "type": "string",
            "enum": ["users", "orders", "analytics", "inventory"],
            "description": "Type of data to query"
          },
          "filters": {
            "type": "object",
            "description": "Query filters and conditions"
          },
          "limit": {
            "type": "integer",
            "description": "Maximum number of results",
            "default": 10,
            "minimum": 1,
            "maximum": 100
          }
        },
        "required": ["query_type"]
      })),
    }
    );
  }

  /// Get tools filtered by name list
  pub fn get_tools_for_names( &self, tool_names: &[ String ] ) -> Vec< Tool >
  {
    let mut function_declarations = Vec::new();

    for tool_name in tool_names
    {
      if let Some( func_decl ) = self.available_functions.get( tool_name )
      {
        function_declarations.push( func_decl.clone() );
      }
    }

    if function_declarations.is_empty()
    {
      Vec::new() // Return empty vec if no functions
    }
    else
    {
      vec!
      [
      Tool
      {
        function_declarations: Some( function_declarations ),
        code_execution: None,
        google_search_retrieval: None,
        code_execution_tool: None,
      }
      ]
    }
  }

  /// Get all available tools
  pub fn get_all_tools( &self ) -> Vec< Tool >
  {
    let function_declarations: Vec< FunctionDeclaration > = self.available_functions
    .values()
    .cloned()
    .collect();

    if function_declarations.is_empty()
    {
      Vec::new() // Return empty vec if no functions
    }
    else
    {
      vec!
      [
      Tool
      {
        function_declarations: Some( function_declarations ),
        code_execution: None,
        google_search_retrieval: None,
        code_execution_tool: None,
      }
      ]
    }
  }

  /// Execute function with validation and logging
  pub async fn execute_function( &mut self, name: &str, args: &Value ) -> Result< Value, Box< dyn std::error::Error > >
  {
    let mut context = FunctionContext::new( name.to_string(), args.clone(), 30 );

    // Validate function exists
    if !self.available_functions.contains_key( name )
    {
    return Err( format!( "Function '{}' not found in registry", name ).into() );
    }

    // Validate parameters (basic validation)
    context.validation_passed = self.validate_parameters( name, args )?;

    // Execute with timeout
    let result = timeout( context.timeout, self.execute_function_impl( name, args ) ).await
.map_err( |_| format!( "Function '{}' timed out after {} seconds", name, context.timeout.as_secs() ) )?;

    // Log execution
    self.execution_log.push( context );

    result
  }

  fn validate_parameters( &self, name: &str, args: &Value ) -> Result< bool, Box< dyn std::error::Error > >
  {
    let func_decl = self.available_functions.get( name )
  .ok_or( format!( "Function '{}' not found", name ) )?;

    if let Some( schema ) = &func_decl.parameters
    {
      if let Some( required ) = schema.get( "required" ).and_then( |r| r.as_array() )
      {
        for req_field in required
        {
          if let Some( field_name ) = req_field.as_str()
          {
            if !args.get( field_name ).is_some()
            {
          return Err( format!( "Required parameter '{}' missing for function '{}'", field_name, name ).into() );
            }
          }
        }
      }
    }

    Ok( true )
  }

  async fn execute_function_impl( &self, name: &str, args: &Value ) -> Result< Value, Box< dyn std::error::Error > >
  {
    match name
    {
      "get_weather" => self.execute_weather( args ).await,
      "calculate" => self.execute_calculator( args ).await,
      "web_search" => self.execute_web_search( args ).await,
      "search_flights" => self.execute_flight_search( args ).await,
      "query_database" => self.execute_database_query( args ).await,
    _ => Err( format!( "Unknown function : {}", name ).into() ),
    }
  }

  async fn execute_weather( &self, args: &Value ) -> Result< Value, Box< dyn std::error::Error > >
  {
    let location = args.get( "location" )
    .and_then( |l| l.as_str() )
    .ok_or( "Missing location parameter" )?;

    let unit = args.get( "unit" )
    .and_then( |u| u.as_str() )
    .unwrap_or( "celsius" );

    // Simulate API call delay
    tokio ::time::sleep( Duration::from_millis( 100 ) ).await;

    // Simulated weather data with more comprehensive information
    let weather_data: HashMap<  &str, ( f32, &str, u8, f32, &str )  > = HashMap::from([
    ( "Tokyo", ( 22.0, "Partly cloudy", 65, 15.5, "Light breeze from southeast" ) ),
    ( "London", ( 15.0, "Rainy", 85, 18.0, "Moderate rain with strong winds" ) ),
    ( "New York", ( 18.0, "Sunny", 55, 12.0, "Clear skies with light winds" ) ),
    ( "Paris", ( 20.0, "Overcast", 70, 14.0, "Cloudy with occasional sun breaks" ) ),
    ( "Sydney", ( 25.0, "Clear", 45, 8.0, "Bright sunshine with calm conditions" ) ),
    ]);

    let ( temp, condition, humidity, wind_speed, description ) = weather_data
    .get( location )
    .unwrap_or( &( 20.0, "Unknown conditions", 50, 10.0, "Weather data unavailable" ) );

    let temp_converted = match unit
    {
      "fahrenheit" => temp * 9.0 / 5.0 + 32.0,
      _ => *temp,
    };

    Ok( json!({
      "location": location,
      "temperature": temp_converted,
      "unit": unit,
      "condition": condition,
      "humidity": humidity,
      "wind_speed": wind_speed,
      "description": description,
      "timestamp": "2024-01-15T10:00:00Z",
      "source": "WeatherAPI Simulation"
    }))
  }

  async fn execute_calculator( &self, args: &Value ) -> Result< Value, Box< dyn std::error::Error > >
  {
    let expression = args.get( "expression" )
    .and_then( |e| e.as_str() )
    .ok_or( "Missing expression parameter" )?;

    let precision = args.get( "precision" )
    .and_then( |p| p.as_u64() )
    .unwrap_or( 2 ) as usize;

    // Simulate calculation processing
    tokio ::time::sleep( Duration::from_millis( 50 ) ).await;

    // Simple expression evaluator (for demo purposes)
    let result = self.evaluate_expression( expression )?;

    Ok( json!({
      "expression": expression,
    "result": format!( "{:.1$}", result, precision ),
      "precision": precision,
      "calculation_time": "0.001s",
      "status": "success"
    }))
  }

  fn evaluate_expression( &self, expr: &str ) -> Result< f64, Box< dyn std::error::Error > >
  {
    // Basic calculator implementation for demo
    match expr.trim()
    {
      e if e.contains( "+" ) =>
      {
        let parts: Vec< &str > = e.split( '+' ).collect();
        if parts.len() == 2
        {
          let a: f64 = parts[ 0 ].trim().parse()?;
          let b: f64 = parts[ 1 ].trim().parse()?;
          Ok( a + b )
        }
        else
        {
          Err( "Invalid addition expression".into() )
        }
      }
      e if e.contains( "-" ) =>
      {
        let parts: Vec< &str > = e.split( '-' ).collect();
        if parts.len() == 2
        {
          let a: f64 = parts[ 0 ].trim().parse()?;
          let b: f64 = parts[ 1 ].trim().parse()?;
          Ok( a - b )
        }
        else
        {
          Err( "Invalid subtraction expression".into() )
        }
      }
      e if e.contains( "*" ) =>
      {
        let parts: Vec< &str > = e.split( '*' ).collect();
        if parts.len() == 2
        {
          let a: f64 = parts[ 0 ].trim().parse()?;
          let b: f64 = parts[ 1 ].trim().parse()?;
          Ok( a * b )
        }
        else
        {
          Err( "Invalid multiplication expression".into() )
        }
      }
      e if e.contains( "/" ) =>
      {
        let parts: Vec< &str > = e.split( '/' ).collect();
        if parts.len() == 2
        {
          let a: f64 = parts[ 0 ].trim().parse()?;
          let b: f64 = parts[ 1 ].trim().parse()?;
          if b == 0.0
          {
            Err( "Division by zero".into() )
          }
          else
          {
            Ok( a / b )
          }
        }
        else
        {
          Err( "Invalid division expression".into() )
        }
      }
      e if e.starts_with( "sqrt(" ) && e.ends_with( ')' ) =>
      {
        let num_str = &e[ 5..e.len() - 1 ];
        let num: f64 = num_str.parse()?;
        if num < 0.0
        {
          Err( "Cannot take square root of negative number".into() )
        }
        else
        {
          Ok( num.sqrt() )
        }
      }
      e if e.contains( "% of " ) =>
      {
        let parts: Vec< &str > = e.split( "% of " ).collect();
        if parts.len() == 2
        {
          let percentage: f64 = parts[ 0 ].trim().parse()?;
          let base: f64 = parts[ 1 ].trim().parse()?;
          Ok( ( percentage / 100.0 ) * base )
        }
        else
        {
          Err( "Invalid percentage expression".into() )
        }
      }
      e =>
      {
        // Try to parse as simple number
        match e.parse::< f64 >()
        {
          Ok( num ) => Ok( num ),
        Err( _ ) => Err( format!( "Unsupported expression : {}", e ).into() ),
        }
      }
    }
  }

  async fn execute_web_search( &self, args: &Value ) -> Result< Value, Box< dyn std::error::Error > >
  {
    let query = args.get( "query" )
    .and_then( |q| q.as_str() )
    .ok_or( "Missing query parameter" )?;

    let max_results = args.get( "max_results" )
    .and_then( |m| m.as_u64() )
    .unwrap_or( 5 ) as usize;

    // Simulate search API delay
    tokio ::time::sleep( Duration::from_millis( 200 ) ).await;

    // Simulated search results
    let mut results = Vec::new();
    for i in 1..=max_results.min( 10 )
    {
      results.push( json!({
    "title": format!( "Search Result {} for '{}'", i, query ),
      "url": format!( "https://example{}.com/search-results", i ),
      "snippet": format!( "This is a simulated search result snippet for query '{}'. Contains relevant information about the topic.", query ),
        "relevance_score": 0.9 - ( i as f64 * 0.1 )
      }));
    }

    Ok( json!({
      "query": query,
      "total_results": results.len(),
      "results": results,
      "search_time": "0.2s",
      "source": "WebSearch API Simulation"
    }))
  }

  async fn execute_flight_search( &self, args: &Value ) -> Result< Value, Box< dyn std::error::Error > >
  {
    let from = args.get( "from" )
    .and_then( |f| f.as_str() )
    .ok_or( "Missing from parameter" )?;

    let to = args.get( "to" )
    .and_then( |t| t.as_str() )
    .ok_or( "Missing to parameter" )?;

    let date = args.get( "date" )
    .and_then( |d| d.as_str() )
    .ok_or( "Missing date parameter" )?;

    let passengers = args.get( "passengers" )
    .and_then( |p| p.as_u64() )
    .unwrap_or( 1 );

    let class = args.get( "class" )
    .and_then( |c| c.as_str() )
    .unwrap_or( "economy" );

    // Simulate flight search API delay
    tokio ::time::sleep( Duration::from_millis( 300 ) ).await;

    let base_price = match ( from, to )
    {
      ( from, to ) if from.contains( "New York" ) && to.contains( "Tokyo" ) => 850.0,
      ( from, to ) if from.contains( "London" ) && to.contains( "Paris" ) => 180.0,
      _ => 450.0,
    };

    let class_multiplier = match class
    {
      "business" => 2.5,
      "first" => 4.0,
      _ => 1.0,
    };

    let flights = vec![
    json!({
      "flight_number": "AA123",
      "airline": "American Airlines",
      "departure": from,
      "arrival": to,
      "date": date,
      "departure_time": "08:00",
      "arrival_time": "22:30",
      "duration": "14h 30m",
      "price": (base_price * class_multiplier * passengers as f64) as u32,
      "class": class,
      "passengers": passengers,
      "stops": 1,
      "available_seats": 24
    }),
    json!({
      "flight_number": "UA456",
      "airline": "United Airlines",
      "departure": from,
      "arrival": to,
      "date": date,
      "departure_time": "14:00",
      "arrival_time": "04:30+1",
      "duration": "15h 30m",
      "price": ((base_price - 50.0) * class_multiplier * passengers as f64) as u32,
      "class": class,
      "passengers": passengers,
      "stops": 0,
      "available_seats": 18
    })
    ];

    Ok( json!({
      "search_params": {
        "from": from,
        "to": to,
        "date": date,
        "passengers": passengers,
        "class": class
      },
      "flights": flights,
      "search_time": "0.3s",
      "source": "FlightSearch API Simulation"
    }))
  }

  async fn execute_database_query( &self, args: &Value ) -> Result< Value, Box< dyn std::error::Error > >
  {
    let query_type = args.get( "query_type" )
    .and_then( |q| q.as_str() )
    .ok_or( "Missing query_type parameter" )?;

    let limit = args.get( "limit" )
    .and_then( |l| l.as_u64() )
    .unwrap_or( 10 ) as usize;

    // Simulate database query delay
    tokio ::time::sleep( Duration::from_millis( 150 ) ).await;

    let data = match query_type
    {
      "users" => json!({
        "total_count": 1250,
        "data": (1..=limit.min( 5 )).map( |i| json!({
          "user_id": i,
        "name": format!( "User {}", i ),
        "email": format!( "user{}@example.com", i ),
          "created_at": "2024-01-15T10:00:00Z",
          "status": "active"
        })).collect::< Vec< _ > >()
      }),
      "orders" => json!({
        "total_count": 5420,
        "data": (1..=limit.min( 5 )).map( |i| json!({
        "order_id": format!( "ORD-{:06}", i ),
          "customer_id": i,
          "amount": 150.00 + ( i as f64 * 25.0 ),
          "status": "completed",
          "created_at": "2024-01-15T10:00:00Z"
        })).collect::< Vec< _ > >()
      }),
      "analytics" => json!({
        "metrics": {
          "daily_revenue": 12500.00,
          "active_users": 850,
          "conversion_rate": 3.2,
          "avg_order_value": 185.50
        },
        "trends": {
          "revenue_growth": "+15%",
          "user_growth": "+8%",
          "conversion_trend": "stable"
        }
      }),
      "inventory" => json!({
        "total_items": 850,
        "data": (1..=limit.min( 5 )).map( |i| json!({
        "product_id": format!( "PROD-{:04}", i ),
        "name": format!( "Product {}", i ),
          "stock_quantity": 50 + i * 10,
          "price": 25.00 + ( i as f64 * 5.0 ),
          "category": "electronics"
        })).collect::< Vec< _ > >()
      }),
    _ => return Err( format!( "Unknown query type : {}", query_type ).into() ),
    };

    Ok( json!({
      "query_type": query_type,
      "execution_time": "0.15s",
      "result": data,
      "source": "Database Simulation"
    }))
  }

  /// Get execution statistics summary
  pub fn get_execution_summary( &self ) -> Value
  {
    json!({
      "total_executions": self.execution_log.len(),
      "functions_used": self.execution_log.iter()
      .map( |ctx| ctx.function_name.clone() )
      .collect::< std::collections::HashSet<  _  > >()
      .into_iter()
      .collect::< Vec< _ > >(),
      "average_execution_time": if self.execution_log.is_empty()
      {
        0.0
      } else {
        self.execution_log.iter()
        .map( |ctx| ctx.elapsed().as_secs_f64() )
        .sum::< f64 >() / self.execution_log.len() as f64
      },
      "validation_success_rate": if self.execution_log.is_empty()
      {
        100.0
      } else {
        self.execution_log.iter()
        .filter( |ctx| ctx.validation_passed )
        .count() as f64 / self.execution_log.len() as f64 * 100.0
      }
    })
  }
}

/// AI Agent that can execute multi-step workflows
#[ derive( Debug ) ]
pub struct FunctionCallingAgent
{
  client: Client,
  tool_registry: ToolRegistry,
  config: AgentConfig,
  conversation_history: Vec< Content >,
}

impl FunctionCallingAgent
{
  /// Create new function calling agent
  pub fn new( config: AgentConfig ) -> Result< Self, Box< dyn std::error::Error > >
  {
    Ok( Self
    {
      client: Client::new()?,
      tool_registry: ToolRegistry::new(),
      config,
      conversation_history: Vec::new(),
    })
  }

  /// Execute task using available tools
  pub async fn execute_task( &mut self, task: &str ) -> Result< String, Box< dyn std::error::Error > >
  {
    if self.config.logging_enabled
    {
    println!( "Starting task execution : '{}'", task );
    println!( "Available tools : {:?}", self.config.available_tools );
    }

    // Initialize conversation
    self.conversation_history.clear();
    self.conversation_history.push( Content
    {
      role: "user".to_string(),
      parts: vec![ Part
      {
      text : Some( format!( "{} Please use the available tools to get accurate, up-to-date information.", task ) ),
        inline_data: None,
        function_call: None,
        function_response: None,
        ..Default::default()
      }],
    });

    let mut iteration = 0;
    let mut final_response = String::new();

    while iteration < self.config.max_iterations
    {
      iteration += 1;

      if self.config.logging_enabled
      {
    println!( "\nIteration {}/{}", iteration, self.config.max_iterations );
      }

      let tools = if self.config.available_tools.contains( &"all".to_string() )
      {
        self.tool_registry.get_all_tools()
      }
      else
      {
        self.tool_registry.get_tools_for_names( &self.config.available_tools )
      };

      let request = GenerateContentRequest
      {
        contents: self.conversation_history.clone(),
        generation_config: Some( GenerationConfig
        {
          temperature: Some( 0.1 ), // Low temperature for reliable function calling
          top_k: Some( 40 ),
          top_p: Some( 0.95 ),
          candidate_count: Some( 1 ),
          max_output_tokens: Some( 2048 ),
          stop_sequences: None,
        }),
        safety_settings: None,
        tools: Some( tools ),
        tool_config: None,
        system_instruction: None,
        cached_content: None,
      };

      let response = timeout(
      Duration::from_secs( self.config.timeout_seconds ),
      self.client.models().by_name( "gemini-2.5-flash" ).generate_content( &request )
      ).await??;

      if let Some( candidate ) = response.candidates.first()
      {
        // Add model's response to conversation
        self.conversation_history.push( candidate.content.clone() );

        let mut has_function_calls = false;
        let mut function_responses = Vec::new();

        // Process function calls
        for part in &candidate.content.parts
        {
          if let Some( function_call ) = &part.function_call
          {
            has_function_calls = true;

            if self.config.logging_enabled
            {
          println!( "Function call : {} with args : {}", 
              function_call.name, 
              serde_json ::to_string_pretty( &function_call.args )? 
              );
            }

            match self.tool_registry.execute_function( &function_call.name, &function_call.args ).await
            {
              Ok( result ) =>
              {
                if self.config.logging_enabled
                {
                println!( "Function result : {}", serde_json::to_string_pretty( &result )? );
                }

                function_responses.push( Part
                {
                  text: None,
                  inline_data: None,
                  function_call: None,
                  function_response: Some( FunctionResponse
                  {
                    name: function_call.name.clone(),
                    response: result,
                  }),
                  ..Default::default()
                });
              }
              Err( error ) =>
              {
                if self.config.logging_enabled
                {
                println!( "Function error : {}", error );
                }

                function_responses.push( Part
                {
                  text: None,
                  inline_data: None,
                  function_call: None,
                  function_response: Some( FunctionResponse
                  {
                    name: function_call.name.clone(),
                  response : json!({ "error": error.to_string() }),
                  }),
                  ..Default::default()
                });
              }
            }
          }
          else if let Some( text ) = &part.text
          {
            final_response = text.clone();
          }
        }

        // If there are function responses, add them to conversation and continue
        if has_function_calls
        {
          if !function_responses.is_empty()
          {
            self.conversation_history.push( Content
            {
              role: "user".to_string(),
              parts: function_responses,
            });
          }
        }
        else
        {
          // No function calls, task is complete
          break;
        }
      }
      else
      {
        return Err( "No response candidate received".into() );
      }
    }

    if iteration >= self.config.max_iterations
    {
      if self.config.logging_enabled
      {
        println!( "Maximum iterations reached" );
      }
    }

    if self.config.logging_enabled
    {
      println!( "\nExecution Summary:" );
    println!( "{}", serde_json::to_string_pretty( &self.tool_registry.get_execution_summary() )? );
    }

    Ok( final_response )
  }

  /// Run agent in interactive mode
  pub async fn run_interactive_mode( &mut self ) -> Result< (), Box< dyn std::error::Error > >
  {
    println!( "Interactive AI Agent Mode" );
  println!( "Available tools : {:?}", self.config.available_tools );
    println!( "Type 'exit' to quit, 'tools' to list available tools" );

    loop
    {
      println!( "\nEnter your task or question:" );
      
      let mut input = String::new();
      std ::io::stdin().read_line( &mut input )?;
      let input = input.trim();

      match input
      {
        "exit" => break,
        "tools" =>
        {
        println!( "Available tools : {:?}", self.config.available_tools );
          continue;
        }
        task if !task.is_empty() =>
        {
          match self.execute_task( task ).await
          {
          Ok( response ) => println!( "\nAgent : {}", response ),
          Err( error ) => println!( "Error : {}", error ),
          }
        }
        _ => continue,
      }
    }

    Ok( () )
  }

  /// Run agent in demo mode
  pub async fn run_demo_mode( &mut self, service: &str ) -> Result< (), Box< dyn std::error::Error > >
  {
    match service
    {
      "weather_api" =>
      {
        let demo_task = "Get the weather for Tokyo and New York, then compare them and tell me which city has better weather for outdoor activities today.";
        println!( "Weather API Integration Demo" );
      println!( "Task : {}", demo_task );
    
        let response = self.execute_task( demo_task ).await?;
      println!( "\nFinal Response:\n{}", response );
      }
      "multi_step" =>
      {
        let demo_task = "I want to plan a trip to Paris. Check the weather there, search for flights from New York for January 25th, and calculate what 15% tip would be on a $150 restaurant bill.";
        println!( "Multi-step Workflow Demo" );
      println!( "Task : {}", demo_task );
    
        let response = self.execute_task( demo_task ).await?;
      println!( "\nFinal Response:\n{}", response );
      }
      "data_analysis" =>
      {
        let demo_task = "Query the database for recent user analytics, then calculate the percentage increase if revenue grew by 15%, and search for information about industry benchmarks for conversion rates.";
        println!( "Data Analysis Demo" );
      println!( "Task : {}", demo_task );
    
        let response = self.execute_task( demo_task ).await?;
      println!( "\nFinal Response:\n{}", response );
      }
    _ => return Err( format!( "Unknown demo service : {}", service ).into() ),
    }

    Ok( () )
  }
}

fn parse_args() -> AgentConfig
{
  let args: Vec< String > = env::args().collect();
  let mut config = AgentConfig::default();

  let mut i = 1;
  while i < args.len()
  {
    match args[ i ].as_str()
    {
      "--agent-mode" =>
      {
        if i + 1 < args.len()
        {
          match args[ i + 1 ].as_str()
          {
            "interactive" => config.agent_mode = AgentMode::Interactive,
            "automated" => config.agent_mode = AgentMode::Automated,
            mode => config.agent_mode = AgentMode::Demo( mode.to_string() ),
          }
          i += 1;
        }
      }
      "--tools" =>
      {
        if i + 1 < args.len()
        {
          if args[ i + 1 ] == "all"
          {
            config.available_tools = vec![ "all".to_string() ];
          }
          else
          {
            config.available_tools = args[ i + 1 ]
            .split( ',' )
            .map( |s| s.trim().to_string() )
            .collect();
          }
          i += 1;
        }
      }
      "--task" =>
      {
        if i + 1 < args.len()
        {
          config.task_description = Some( args[ i + 1 ].clone() );
          config.agent_mode = AgentMode::Automated;
          i += 1;
        }
      }
      "--demo" =>
      {
        if i + 1 < args.len()
        {
          config.agent_mode = AgentMode::Demo( "demo".to_string() );
          i += 1;
        }
      }
      "--service" =>
      {
        if i + 1 < args.len()
        {
          config.demo_service = Some( args[ i + 1 ].clone() );
          i += 1;
        }
      }
      "--max-iterations" =>
      {
        if i + 1 < args.len()
        {
          if let Ok( max_iter ) = args[ i + 1 ].parse::< usize >()
          {
            config.max_iterations = max_iter;
          }
          i += 1;
        }
      }
      "--timeout" =>
      {
        if i + 1 < args.len()
        {
          if let Ok( timeout ) = args[ i + 1 ].parse::< u64 >()
          {
            config.timeout_seconds = timeout;
          }
          i += 1;
        }
      }
      "--quiet" =>
      {
        config.logging_enabled = false;
      }
    _ => {}
    }
    i += 1;
  }

  config
}

fn print_usage()
{
  println!( "Comprehensive Function Calling & Tool Integration Example" );
  println!();
  println!( "Usage:" );
  println!( "  cargo run --example gemini_function_calling [OPTIONS]" );
  println!();
  println!( "Options:" );
  println!( "  --agent-mode MODE     Set agent mode: interactive, automated" );
  println!( "  --tools TOOLS         Comma-separated list of tools or 'all'" );
  println!( "  --task TASK          Execute specific task (sets mode to automated)" );
  println!( "  --demo               Run demonstration mode" );
  println!( "  --service SERVICE    Demo service: weather_api, multi_step, data_analysis" );
  println!( "  --max-iterations N   Maximum workflow iterations (default: 10)" );
  println!( "  --timeout SECONDS    Function timeout in seconds (default: 30)" );
  println!( "  --quiet              Disable logging output" );
  println!();
  println!( "Available tools:" );
  println!( "  weather      - Get weather information for locations" );
  println!( "  calculator   - Perform mathematical calculations" );
  println!( "  web_search   - Search the web for information" );
  println!( "  search_flights - Find flight options between cities" );
  println!( "  query_database - Query simulated database for analytics" );
  println!();
  println!( "Examples:" );
  println!( "  # Interactive mode with specific tools" );
  println!( "  cargo run --example gemini_function_calling -- --agent-mode interactive --tools weather,calculator" );
  println!();
  println!( "  # Execute specific task with all tools" );
  println!( "  cargo run --example gemini_function_calling -- --task \"Plan a trip to Tokyo\" --tools all" );
  println!();
  println!( "  # Run weather API demo" );
  println!( "  cargo run --example gemini_function_calling -- --demo --service weather_api" );
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn core::error::Error > >
{
  let config = parse_args();

  if env::args().any( |arg| arg == "--help" || arg == "-h" )
  {
    print_usage();
    return Ok( () );
  }

  let mut agent = FunctionCallingAgent::new( config.clone() )?;

  match config.agent_mode
  {
    AgentMode::Interactive =>
    {
      agent.run_interactive_mode().await?;
    }
    AgentMode::Automated =>
    {
      if let Some( task ) = &config.task_description
      {
        let response = agent.execute_task( task ).await?;
        if !config.logging_enabled
        {
        println!( "{response}" );
        }
      }
      else
      {
        println!( "No task specified for automated mode. Use --task \"your task here\"" );
        print_usage();
      }
    }
    AgentMode::Demo( _ ) =>
    {
      let service = config.demo_service.as_deref().unwrap_or( "weather_api" );
      agent.run_demo_mode( service ).await?;
    }
  }

  Ok( () )
}