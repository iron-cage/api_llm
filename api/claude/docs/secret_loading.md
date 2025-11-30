# Secret Loading Guide for api_claude

## Overview

The `api_claude` crate provides multiple methods for loading Anthropic API keys, prioritizing security, flexibility, and clear error messages. This guide explains all secret loading mechanisms and troubleshooting steps.

## Critical Requirements

### ⚠️ Environment Variable Name

The environment variable **MUST** be named exactly:
```bash
ANTHROPIC_API_KEY
```

**Common mistakes:**
- ❌ `CLAUDE_TOKEN` - Wrong name
- ❌ `CLAUDE_API_KEY` - Wrong name
- ❌ `ANTHROPIC_KEY` - Wrong name
- ✅ `ANTHROPIC_API_KEY` - Correct!

### ⚠️ API Key Format

Valid Anthropic API keys start with `sk-ant-`:
```
sk-ant-api03-...
```

## Loading Methods

### Method 1: From Environment Variable

**When to use:** Running examples locally, CI/CD pipelines

**Code:**
```rust
use api_claude::Client;

let client = Client::from_env( )?;
```

**Setup:**
```bash
# Option A: Export directly
export ANTHROPIC_API_KEY="sk-ant-api03-..."

# Option B: Source workspace secrets file
source /path/to/workspace/secret/-secrets.sh

# Verify it's set
echo $ANTHROPIC_API_KEY
```

### Method 2: From Workspace Secrets

**When to use:** Integration tests, development within workspace

**Code:**
```rust
use api_claude::Client;

let client = Client::from_workspace( )?;
```

**How it works:**
1. Uses `workspace_tools` to find workspace root (looks for `Cargo.toml`)
2. Looks for `secret/-secrets.sh` relative to workspace root
3. Parses the file for `ANTHROPIC_API_KEY` variable
4. Does NOT require the file to be sourced

**Setup:**
```bash
# 1. Ensure you're in a workspace with Cargo.toml
cd /path/to/workspace

# 2. Create or verify secret/ directory exists at workspace root
ls secret/

# 3. Add API key to secret/-secrets.sh
echo 'export ANTHROPIC_API_KEY="sk-ant-api03-..."' >> secret/-secrets.sh

# 4. Verify the variable name is correct
grep ANTHROPIC_API_KEY secret/-secrets.sh
```

### Method 3: Direct Secret Construction

**When to use:** Testing, custom secret management

**Code:**
```rust
use api_claude::{Client, Secret};

let secret = Secret::new("sk-ant-api03-...".to_string(   ))?;
let client = Client::new(secret);
```

## Workspace Structure Requirements

For `from_workspace()` to work, your directory structure should be:

```
workspace_root/
├── Cargo.toml              # Marks workspace root
├── secret/                 # Secret directory (NO dot prefix)
│   └── -secrets.sh         # Contains API keys
└── api/
    └── claude/
        ├── Cargo.toml
        └── src/
```

**Key points:**
- Workspace root is identified by presence of `Cargo.toml`
- Secret directory MUST be named `secret/` at workspace root (NO dot prefix)
- Secrets file must be named `-secrets.sh`
- workspace_tools 0.6.0 uses `secret/` directly (no symlinks needed)
- See [Secret Directory Policy](../../../secret/readme.md) for authoritative structure

## Secrets File Format

**Template:** `/home/user1/pro/secret/-secrets.sh`

```bash
#!/bin/bash

# === AI SERVICES ===

# Anthropic Claude API key (MUST be named ANTHROPIC_API_KEY)
export ANTHROPIC_API_KEY="sk-ant-api03-..."

# Export is recommended for compatibility with both methods
```

**Important:**
- Use `export` keyword to make variables available to subprocesses
- Variable name must be exactly `ANTHROPIC_API_KEY`
- File should be executable: `chmod +x secret/-secrets.sh`
- File should be in `.gitignore` (files starting with `-` are typically ignored)

## Validation and Diagnostics

### Check Secret Availability

```rust
use api_claude::validate_anthropic_secret;

match validate_anthropic_secret( ) {
    Ok( source ) => println!("✅ Secret loaded from: {}", source),
    Err( e ) => eprintln!("❌ Secret loading failed:\n{}", e),
}
```

### Get Diagnostic Report

```rust
use api_claude::secret_diagnostic_info;

println!("{}", secret_diagnostic_info( ));
```

Output example:
```
🔍 Secret Loading Diagnostic Information

✅ ANTHROPIC_API_KEY environment variable: SET
   Value: sk-ant-api03-JXk-G...

📁 Workspace Information:
   Root: /home/user1/pro/lib/api_llm
   Secret directory: /home/user1/pro/lib/api_llm/secret
   Secret file: /home/user1/pro/lib/api_llm/secret/-secrets.sh
   ✅ Secrets file exists
   ✅ ANTHROPIC_API_KEY found in secrets file

📂 Current Directory: /home/user1/pro/lib/api_llm/api/claude
```

### Validate Workspace Structure

```rust
use api_claude::validate_workspace_structure;

match validate_workspace_structure( ) {
    Ok( path ) => println!("✅ Secrets file: {}", path.display(   )),
    Err( e ) => eprintln!("❌ Workspace validation failed: {}", e),
}
```

## Troubleshooting

### Error: "ANTHROPIC_API_KEY not found"

**Diagnosis:**
```rust
use api_claude::secret_diagnostic_info;
println!("{}", secret_diagnostic_info( ));
```

**Common causes:**

1. **Wrong variable name in secrets file**
   ```bash
   # Check what's in the file
   grep -i "claude\|anthropic" secret/-secrets.sh

   # Should show:
   # ANTHROPIC_API_KEY="sk-ant-api03-..."
   # NOT:
   # CLAUDE_TOKEN="..."
   ```

2. **Secrets file not in expected location**
   ```bash
   # Find where workspace_tools is looking
   cargo run --example secret_diagnostic

   # Ensure secret/ directory exists at workspace root
   ls -la secret/
   ```

3. **Environment variable not exported**
   ```bash
   # Check if it's set
   env | grep ANTHROPIC_API_KEY

   # Source the file
   source secret/-secrets.sh

   # Verify
   echo $ANTHROPIC_API_KEY
   ```

### Error: "Invalid Anthropic API key format"

**Cause:** API key doesn't start with `sk-ant-`

**Fix:**
```bash
# Check key format
echo $ANTHROPIC_API_KEY | head -c 10

# Should output: sk-ant-api

# Get correct key from: https://console.anthropic.com/settings/keys
```

### Error: "Workspace error: No Cargo.toml found"

**Cause:** Not in a Cargo workspace

**Fix for tests:**
```rust
// Use from_env() instead
let client = Client::from_env( )?;
```

**Fix for workspace:**
```bash
# Ensure you're in the correct directory
pwd
ls Cargo.toml

# Or cd to workspace root
cd /path/to/workspace
```

### Error: "Secrets directory does not exist"

**Fix:**
```bash
# Create secret/ directory at workspace root (NO dot prefix)
mkdir -p secret

# Add secrets file
touch secret/-secrets.sh

# Verify
ls -la secret/
```

## Testing Your Setup

Create a test file `test_secrets.rs`:

```rust
use api_claude::{Client, validate_anthropic_secret, secret_diagnostic_info};

fn main( ) -> Result< (), Box< dyn std::error::Error >> {
    println!("=== Secret Loading Test ===\n");

    // 1. Run diagnostics
    println!("{}", secret_diagnostic_info( ));

    // 2. Validate secret
    match validate_anthropic_secret( ) {
        Ok( source ) => println!("\n✅ Secret available from: {}", source),
        Err( e ) => {
            eprintln!("\n❌ Secret validation failed:\n{}", e);
            return Err( e.into(    ));
        }
    }

    // 3. Try creating client
    println!("\n=== Testing Client Creation ===\n");

    match Client::from_workspace( ) {
        Ok( _ ) => println!("✅ Client::from_workspace( ) works"),
        Err( e ) => println!("❌ Client::from_workspace( ) failed: {}", e),
    }

    match Client::from_env( ) {
        Ok( _ ) => println!("✅ Client::from_env( ) works"),
        Err( e ) => println!("❌ Client::from_env( ) failed: {}", e),
    }

    Ok( ( ))
}
```

Run it:
```bash
cargo run --example test_secrets
```

## Best Practices

### ✅ DO

- Use `export` keyword in secrets file for compatibility
- Name the variable exactly `ANTHROPIC_API_KEY`
- Keep secrets file in `.gitignore` (files starting with `-` are auto-ignored)
- Use `from_workspace()` in tests for consistency
- Use `from_env()` in examples for simplicity
- Add validation checks in integration tests
- Use diagnostic functions when debugging

### ❌ DON'T

- Don't hardcode API keys in source code
- Don't commit secrets file to version control
- Don't use wrong variable names (e.g., `CLAUDE_TOKEN`)
- Don't assume environment variables are set without checking
- Don't mix `from_env()` and `from_workspace()` in the same test suite

## Security Notes

1. **File Permissions:** Restrict access to secrets file
   ```bash
   chmod 600 secret/-secrets.sh
   ```

2. **Git Ignore:** Ensure secrets aren't committed
   ```bash
   # .gitignore should contain:
   -secrets.sh
   secret/

   # Note: .secret is a symlink and can be tracked in git
   ```

3. **Environment Isolation:** Don't expose secrets in logs
   ```rust
   // Secret implements Debug with redaction
   println!("{:?}", secret);  // Outputs: Secret { ANTHROPIC_API_KEY: "<REDACTED>" }
   ```

## CI/CD Integration

### GitHub Actions

```yaml
env:
  ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}

steps:
  - name: Run tests
    run: cargo test --features integration
```

### GitLab CI

```yaml
variables:
  ANTHROPIC_API_KEY: $ANTHROPIC_API_KEY

test:
  script:
    - cargo test --features integration
```

## Related Documentation

- [Secret Directory Policy](../../../secret/readme.md)
- [Examples README](../examples/readme.md)
- [Tests README](../../tests/readme.md)

## Support

If you encounter issues not covered here:

1. Run `secret_diagnostic_info()` and save the output
2. Check the [API Documentation](https://docs.rs/api_claude)
3. Report issues at [GitHub Issues](https://github.com/Wandalen/api_llm/issues)
