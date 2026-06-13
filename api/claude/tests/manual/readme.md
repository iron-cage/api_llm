# Manual Testing

This directory contains manual test scripts and utilities for testing aspects that cannot be easily automated.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `readme.md` | Directory index and manual test instructions |
| `test_workspace_secrets.rs` | Manual validation of workspace secret loading |


## Available Tests

### test_workspace_secrets.rs

**Purpose:** Manual validation of workspace_tools secret loading mechanism

**How to run:**

```bash
# Make executable (if needed)
chmod +x tests/manual/test_workspace_secrets.rs

# Run with rust-script
rust-script tests/manual/test_workspace_secrets.rs
```

**What it tests:**
- Workspace root discovery
- Secret directory location (`secret/-secrets.sh`)
- Secret key loading from workspace
- Environment variable fallback
- Error messages and paths attempted

**Expected output:**
- ✅ Workspace found at correct root
- ✅ Secret directory exists at `<workspace>/secret/`
- ✅ API key loaded successfully OR clear error with all paths tried
- ✅ Environment variable fallback works

**Troubleshooting:**

If secrets not found, check:
1. Workspace root has `Cargo.toml`
2. `secret/` directory exists at workspace root (NOT `.secret`)
3. `secret/-secrets.sh` file exists with proper format
4. File permissions: `chmod 600 secret/-secrets.sh`

**Example secret file format:**

```bash
# File: <workspace_root>/secret/-secrets.sh
export ANTHROPIC_API_KEY="sk-ant-api03-your-actual-key-here"
```

## Adding New Manual Tests

When adding new manual tests:
1. Place test scripts in this directory
2. Update this readme.md with test documentation
3. Ensure tests are clearly marked as manual (not automated)
4. Include clear instructions for running and expected results
