# error/

Error type hierarchy and enhanced error services for the Anthropic API client.

| File | Responsibility |
|------|----------------|
| core.rs | Core error enum, result type alias, and display formatting |
| enhanced.rs | Extended error variants for enterprise and auth error scenarios |
| enhanced_services.rs | Backoff calculation for rate-limit error recovery |
