# Centralized Secrets Management

## ⚠️ CRITICAL: Official Secret Directory Location

**This directory (`secret/`) is the ONLY official location for workspace secrets.**

### Directory Structure

```
workspace_root/
├── secret/                 # ✅ OFFICIAL: Secret directory (NO dot prefix)
│   ├── readme.md          # This file (public documentation)
│   └── -secrets.sh        # Single secrets file containing all API keys (hidden from git)
```

### Important Notes

- **Directory Name**: MUST be `secret/` (NO dot prefix, NOT `.secret/`)
- **Location**: MUST be at workspace root
- **workspace_tools 0.6.0+**: Uses `secret/` directly (no symlinks needed)
- **Secret File**: Uses hyphen prefix (`-secrets.sh`) to auto-hide from git via `-*` gitignore pattern

**This document defines the complete secret directory policy.**

## Usage

All crates should use `workspace_tools::workspace()?.load_secret_key()` to access secrets:

```rust
use workspace_tools::workspace;

// All secrets come from single file
let openai_key = workspace()?.load_secret_key("OPENAI_API_KEY", "-secrets.sh")?;
let gemini_key = workspace()?.load_secret_key("GEMINI_API_KEY", "-secrets.sh")?;
let anthropic_key = workspace()?.load_secret_key("ANTHROPIC_API_KEY", "-secrets.sh")?;
let huggingface_key = workspace()?.load_secret_key("HUGGINGFACE_API_KEY", "-secrets.sh")?;
let xai_key = workspace()?.load_secret_key("XAI_API_KEY", "-secrets.sh")?;
```

## Environment Variables

### OpenAI
- `OPENAI_API_KEY`: API key for OpenAI services

### Gemini
- `GEMINI_API_KEY`: API key for Google Gemini services

### Anthropic
- `ANTHROPIC_API_KEY`: API key for Anthropic Claude services

### Hugging Face
- `HUGGINGFACE_API_KEY`: API key for Hugging Face Inference API services

### X.AI
- `XAI_API_KEY`: API key for X.AI Grok services

### Ollama
- No API key required (local runtime)

## Shell Usage

To source all secrets into your shell environment:

```bash
# Source the single secrets file
. ./secret/-secrets.sh
```

## Security

- The secret file with hyphen prefix (`-`) is automatically hidden from git via the `-*` gitignore pattern
- Never commit actual secret values to version control
- Use environment variables or secure secret management systems in production  
- Each developer should maintain their own local secrets
- Prefer hiding sensitive files with `-` prefix instead of deleting them when relevant