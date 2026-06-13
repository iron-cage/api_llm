//! Model definitions and model-related data structures for `HuggingFace` API.

use serde::{ Deserialize, Serialize };

/// Information about a `HuggingFace` model
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ModelInfo
{
  /// Model identifier
  pub id : String,
  
  /// Model repository URL
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub repository_url : Option< String >,
  
  /// Model task type
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub pipeline_tag : Option< String >,
  
  /// Model tags
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub tags : Option< Vec< String > >,
  
  /// Whether the model is private
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub private : Option< bool >,
  
  /// Model author/organization
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub author : Option< String >,
  
  /// Number of likes
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub likes : Option< u32 >,
  
  /// Number of downloads
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub downloads : Option< u32 >,
}

/// Popular `HuggingFace` models
#[ derive( Debug ) ]
pub struct Models;

impl Models
{
  /// Kimi K2 Instruct model (supports function calling on Router API)
  ///
  /// This model provides excellent function calling capabilities and
  /// reasoning performance. Use for tasks requiring tool use.
  ///
  /// Note: Previously included `:groq` suffix which was incorrect.
  #[ inline ]
  #[ must_use ]
  pub const fn kimi_k2_instruct() -> &'static str
  {
  "moonshotai/Kimi-K2-Instruct-0905"
  }

  /// Llama 3.3 70B Instruct model (recommended default)
  ///
  /// This is the current recommended default model.
  #[ inline ]
  #[ must_use ]
  pub const fn llama_3_3_70b_instruct() -> &'static str
  {
  "meta-llama/Llama-3.3-70B-Instruct"
  }

  /// Llama 3.1 8B Instruct model (deprecated - use `llama_3_3_70b_instruct`)
  #[ inline ]
  #[ must_use ]
  #[ deprecated( since = "0.3.0", note = "Use llama_3_3_70b_instruct() instead" ) ]
  pub const fn llama_3_1_8b_instruct() -> &'static str
  {
  "meta-llama/Meta-Llama-3.1-8B-Instruct"
  }

  /// Llama 3.1 70B Instruct model (deprecated - use `llama_3_3_70b_instruct`)
  #[ inline ]
  #[ must_use ]
  #[ deprecated( since = "0.3.0", note = "Use llama_3_3_70b_instruct() instead" ) ]
  pub const fn llama_3_1_70b_instruct() -> &'static str
  {
  "meta-llama/Meta-Llama-3.1-70B-Instruct"
  }
  
  /// Mistral 7B Instruct model
  #[ inline ]
  #[ must_use ]
  pub const fn mistral_7b_instruct() -> &'static str
  {
  "mistralai/Mistral-7B-Instruct-v0.3"
  }
  
  /// Code Llama 7B Instruct model
  #[ inline ]
  #[ must_use ]
  pub const fn code_llama_7b_instruct() -> &'static str
  {
  "codellama/CodeLlama-7b-Instruct-hf"
  }
  
  /// All `MiniLM` L6 v2 embeddings model
  #[ inline ]
  #[ must_use ]
  pub const fn all_minilm_l6_v2() -> &'static str
  {
  "sentence-transformers/all-MiniLM-L6-v2"
  }
  
  /// All `MiniLM` L12 v2 embeddings model
  #[ inline ]
  #[ must_use ]
  pub const fn all_minilm_l12_v2() -> &'static str
  {
  "sentence-transformers/all-MiniLM-L12-v2"
  }
  
  /// BGE Large EN v1.5 embeddings model
  #[ inline ]
  #[ must_use ]
  pub const fn bge_large_en_v1_5() -> &'static str
  {
  "BAAI/bge-large-en-v1.5"
  }

  /// GPT-2 base model (publicly accessible)
  #[ inline ]
  #[ must_use ]
  pub const fn gpt2() -> &'static str
  {
  "gpt2"
  }
}