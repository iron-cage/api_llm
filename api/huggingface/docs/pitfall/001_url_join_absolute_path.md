# Pitfall: URL Join Absolute Path

### Scope

- **Purpose**: Document the `Url::join` absolute-path stripping pitfall in `api_huggingface` endpoint construction.
- **Responsibility**: This pitfall/ instance — the confirmed URL path construction pitfall affecting providers and inference modules.
- **In Scope**: All endpoint path construction using `Url::join` or string concatenation in `src/`.
- **Out of Scope**: External HTTP library behavior beyond the documented pitfall, non-URL string formatting.

### Failure

Requests to HuggingFace endpoints silently target the wrong path — the `/v1/` version prefix is stripped from the base URL. For example, a call intended for `https://api-inference.huggingface.co/v1/chat/completions` instead hits `https://api-inference.huggingface.co/chat/completions`, producing a 404 or unexpected response without any compile-time or runtime error message.

### Trap

`Url::join(base, path)` treats a leading-slash `path` as an absolute path on the host, discarding all path segments from `base` after the host. A base of `https://api-inference.huggingface.co/v1/` joined with `"/chat/completions"` yields `https://api-inference.huggingface.co/chat/completions` — the `/v1/` prefix is silently dropped. Only relative paths (no leading slash) preserve the base path prefix.

### Mitigation

Always pass relative paths (no leading slash) to `Url::join`. Use `"chat/completions"` not `"/v1/chat/completions"`, and `"models/{id}"` not `"/models/{id}"`. Ensure the base URL already contains the API version prefix with a trailing slash: `https://api-inference.huggingface.co/v1/`. To detect violations, search for `"/[a-z]` (a quote immediately followed by a slash and a letter) in any path strings passed to URL join or format calls — any match is a candidate absolute-path error that will silently strip the base URL prefix.

### Sources

| File | Relationship |
|------|--------------|
| `src/providers.rs` | Primary fix site — leading-slash path in Router API endpoint construction |
| `src/inference.rs` | Secondary fix site — leading-slash format strings in model endpoint construction |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/pitfall/01_url_join_absolute_path.md` | GWT spec scenarios verifying correct relative-path endpoint construction |
| `task/verified/003_fix_url_path_inconsistency.md` | Verification matrix T01–T07 covering all known occurrences of this pitfall |
