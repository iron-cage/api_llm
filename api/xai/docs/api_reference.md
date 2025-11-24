# X.AI Grok API Reference

**Last Updated:** 2025-11-08
**API Version:** v1
**Official Docs:** https://docs.x.ai

---

## Overview

The X.AI Grok API provides programmatic access to Grok models via a REST API that is **fully compatible with OpenAI's API format**. This allows seamless migration from OpenAI to X.AI with minimal code changes.

**Key Characteristics:**
- OpenAI-compatible request/response formats
- RESTful HTTP API
- Stateless (no automatic conversation history)
- Server-Sent Events (SSE) for streaming
- Agentic server-side tool calling

---

## Authentication

**Method:** Bearer Token Authentication

**Header Format:**
```
Authorization: Bearer YOUR_XAI_API_KEY
```

**API Key Format:**
- Prefix: `xai-` (typically)
- Obtain from: https://console.x.ai (API Keys Page)
- Environment variable: `XAI_API_KEY`

**Security:**
- Never hardcode API keys in source code
- Use environment variables or secure key management
- Keys grant access to: specified endpoints and models (configurable at creation)

---

## Base URL

```
https://api.x.ai/v1
```

**Alternative (legacy):**
- Some sources mention `https://api.grok.xai.com/v1` but `api.x.ai` is the official endpoint

---

## Endpoints

### 1. Chat Completions

**Endpoint:** `POST /v1/chat/completions`

**Purpose:** Generate conversational AI responses

**Request Format:**
```json
{
  "model": "grok-2-1212",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant."
    },
    {
      "role": "user",
      "content": "What is 2 + 2?"
    }
  ],
  "temperature": 0.7,
  "max_tokens": 1000,
  "stream": false,
  "tools": []
}
```

**Key Parameters:**
- `model` (required): Model identifier (e.g., "grok-2-1212", "grok-4")
- `messages` (required): Array of message objects with `role` and `content`
- `temperature` (optional): 0.0-2.0, controls randomness (0.2-0.4 for factual)
- `max_tokens` (optional): Maximum tokens to generate
- `top_p` (optional): 0.0-1.0, nucleus sampling (0.1-0.9 typical)
- `frequency_penalty` (optional): 0.0-2.0, reduces repetition (0.1-0.8 typical)
- `presence_penalty` (optional): 0.0-2.0, encourages topic diversity (0.1-0.8 typical)
- `stream` (optional): Boolean, enable SSE streaming
- `tools` (optional): Array of tool definitions for function calling

**Message Roles:**
- `system`: System instructions/context
- `user`: User messages
- `assistant`: Assistant responses
- **Note:** X.AI allows flexible role ordering (no strict alternation required)

**Response Format:**
```json
{
  "id": "chatcmpl-...",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "grok-2-1212",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "2 + 2 equals 4."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 20,
    "completion_tokens": 10,
    "total_tokens": 30
  }
}
```

### 2. Models List

**Endpoint:** `GET /v1/models`

**Purpose:** List available models

**Response:** Array of model objects with IDs, capabilities, and metadata

### 3. Embeddings (if available)

**Endpoint:** `POST /v1/embeddings`

**Purpose:** Generate text embeddings

**Note:** Availability may vary based on API access tier

### 4. Completions (Legacy)

**Endpoint:** `POST /v1/completions`

**Note:** Legacy endpoint, prefer `/chat/completions` for new implementations

---

## Available Models

### Grok 4 (grok-4)

**Released:** July 9, 2025

**Capabilities:**
- **Context Window:** 256,000 tokens
- **Multimodal:** Text and vision inputs
- **Advanced Reasoning:** Step-by-step reasoning with "think before responding"
- **Tool Use:** Native function calling and tool integration
- **Real-time Search:** Web and X platform search integration
- **Training:** Reinforcement learning at pretraining scale (200K GPU cluster)

**Pricing:**
- Input: $3.00 per 1M tokens
- Output: $15.00 per 1M tokens
- Cached Input: $0.75 per 1M tokens

**Use Cases:** Complex reasoning, mathematical computations, multimodal tasks

### Grok 4 Fast (grok-beta)

**Released:** September 2025

**Variants:**
- `grok-4-fast-reasoning`: With reasoning mode
- `grok-4-fast-non-reasoning`: Without reasoning mode

**Capabilities:**
- **Context Window:** 2,000,000 tokens (2M)
- **Cost-Efficient:** 40% fewer thinking tokens vs Grok 4
- **Unified Architecture:** Blends reasoning and non-reasoning modes
- **Agentic Search:** Web and X search with media ingestion (images, videos)
- **Performance:** Comparable to Grok 4 on benchmarks

**Pricing:** More cost-efficient than Grok 4 (exact pricing varies)

**Use Cases:** Agentic coding, real-time data augmentation, cost-sensitive applications

### Legacy Models

- `grok-1`: Earlier generation (may be deprecated)
- Other models: Check `/v1/models` endpoint for complete list

**Knowledge Cutoff:** November 2024 (but real-time search capabilities extend this)

---

## Streaming (SSE)

**Protocol:** Server-Sent Events (SSE)

**Enable Streaming:**
```json
{
  "stream": true
}
```

**Response Format:**
```
data: {"id":"chatcmpl-...","object":"chat.completion.chunk","created":...,"model":"grok-2-1212","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"chatcmpl-...","object":"chat.completion.chunk","created":...,"model":"grok-2-1212","choices":[{"index":0,"delta":{"content":" there"},"finish_reason":null}]}

data: {"id":"chatcmpl-...","object":"chat.completion.chunk","created":...,"model":"grok-2-1212","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

**Key Points:**
- Each chunk contains a `delta` with incremental content
- Final chunk has `finish_reason` set (e.g., "stop")
- Stream ends with `data: [DONE]`
- Use `eventsource-stream` or similar SSE client libraries

**X.AI SDK Difference:**
- X.AI's Python SDK uses `.stream()` method returning `(response, chunk)` tuples
- Different from OpenAI's `stream=True` parameter pattern

---

## Function Calling & Tools

### Traditional Function Calling

**Define Tools:**
```json
{
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get current weather for a location",
        "parameters": {
          "type": "object",
          "properties": {
            "location": {
              "type": "string",
              "description": "City name"
            },
            "unit": {
              "type": "string",
              "enum": ["celsius", "fahrenheit"]
            }
          },
          "required": ["location"]
        }
      }
    }
  ]
}
```

**Model Response with Tool Call:**
```json
{
  "choices": [
    {
      "message": {
        "role": "assistant",
        "tool_calls": [
          {
            "id": "call_...",
            "type": "function",
            "function": {
              "name": "get_weather",
              "arguments": "{\"location\":\"San Francisco\",\"unit\":\"celsius\"}"
            }
          }
        ]
      }
    }
  ]
}
```

**Client Executes Tool → Send Result:**
```json
{
  "messages": [
    ...,
    {
      "role": "tool",
      "tool_call_id": "call_...",
      "content": "{\"temperature\": 18, \"condition\": \"sunny\"}"
    }
  ]
}
```

### Agentic Server-Side Tool Calling

**Feature:** Model autonomously executes tools on the server without client intervention

**Built-in Server-Side Tools:**

1. **Web Search**
   - Real-time internet search
   - Web page browsing and navigation
   - Link traversal

2. **X Search**
   - Semantic and keyword search across X posts
   - User and thread search
   - Real-time X platform data

3. **Code Execution**
   - Python code writing and execution
   - Mathematical calculations
   - Data analysis and computations

**Usage:**
- Automatically invoked based on query context
- Model manages entire reasoning and tool-execution loop
- No client-side tool handling required

**Streaming with Agentic Tools:**
- Real-time observability of tool calls via `tool_calls` attribute
- See model's search strategy and parameter choices
- Recommended: Use X.AI Python SDK in streaming mode

**Advantages:**
- Autonomous exploration and problem-solving
- No client-side tool orchestration
- Immediate feedback during long-running requests

---

## Error Handling

**Common HTTP Status Codes:**

| Code | Meaning | Recommended Action |
|------|---------|-------------------|
| 200 | Success | Process response |
| 401 | Unauthorized | Check API key validity |
| 429 | Rate Limit | Implement exponential backoff |
| 498 | Capacity Exceeded | Retry after delay |
| 500 | Internal Server Error | Retry with backoff |
| 502/503 | Service Unavailable | Retry with backoff |

**Rate Limiting:**
- Implement exponential backoff for 429 errors
- Start with 1-2 second delay, double on each retry
- Max retries: 3-5 attempts
- Monitor `Retry-After` header if provided

**Error Response Format:**
```json
{
  "error": {
    "message": "Invalid API key",
    "type": "invalid_request_error",
    "code": "invalid_api_key"
  }
}
```

---

## Pricing Tiers

**Public Beta:**
- $25 monthly API credits
- Requires X Premium+ subscription ($40/month)

**Standard Tier:**
- $0.03 per request (legacy pricing, may vary by model)
- Usage-based billing

**Enterprise:**
- Custom pricing
- Volume discounts
- Dedicated support

**Cost Optimization:**
- Use cached inputs for repeated content (75% discount on Grok 4)
- Choose appropriate model (Grok 4 Fast for cost-efficiency)
- Optimize token usage (temperature, max_tokens)

---

## OpenAI Compatibility

**Migration from OpenAI:**

```python
# OpenAI
from openai import OpenAI
client = OpenAI( api_key="sk-..." )

# X.AI (change base_url and api_key)
from openai import OpenAI
client = OpenAI(
  api_key="xai-...",
  base_url="https://api.x.ai/v1"
)
```

**Compatible SDKs:**
- OpenAI Python SDK
- OpenAI Node.js SDK
- Anthropic SDK (with base URL override)

**Differences:**
- Message role ordering is flexible (not strict alternation)
- Stateless (no automatic conversation history)
- Streaming API differs in X.AI SDK (`.stream()` method)
- Agentic server-side tools (X.AI exclusive)

---

## Best Practices

### Security
- Store API keys in environment variables
- Rotate keys periodically
- Use minimal permissions when creating keys
- Never commit keys to version control

### Performance
- Use streaming for long responses
- Implement connection pooling
- Set reasonable timeouts (30-60 seconds)
- Cache responses when appropriate

### Error Handling
- Implement retry logic with exponential backoff
- Handle rate limits gracefully
- Log errors with context
- Monitor API usage and costs

### Cost Optimization
- Use `max_tokens` to limit response length
- Lower `temperature` for deterministic outputs
- Leverage cached inputs when possible
- Choose cost-efficient models (Grok 4 Fast)

### Prompt Engineering
- Use clear, specific instructions
- Provide relevant context in system messages
- Structure multi-turn conversations logically
- Test with different temperature/top_p values

---

## Code Examples

### Basic Chat Completion (curl)

```bash
curl https://api.x.ai/v1/chat/completions \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "grok-2-1212",
    "messages": [
      {"role": "user", "content": "What is the capital of France?"}
    ],
    "max_tokens": 100
  }'
```

### Streaming Chat (curl)

```bash
curl https://api.x.ai/v1/chat/completions \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "grok-2-1212",
    "messages": [
      {"role": "user", "content": "Tell me a story"}
    ],
    "stream": true
  }'
```

### Python with OpenAI SDK

```python
from openai import OpenAI

client = OpenAI(
  api_key=os.environ.get( "XAI_API_KEY" ),
  base_url="https://api.x.ai/v1"
)

response = client.chat.completions.create(
  model="grok-2-1212",
  messages=[
    { "role": "system", "content": "You are a helpful assistant." },
    { "role": "user", "content": "Hello!" }
  ]
)

print( response.choices[ 0 ].message.content )
```

### Python Streaming

```python
stream = client.chat.completions.create(
  model="grok-2-1212",
  messages=[ { "role": "user", "content": "Count to 10" } ],
  stream=True
)

for chunk in stream:
  if chunk.choices[ 0 ].delta.content:
    print( chunk.choices[ 0 ].delta.content, end="" )
```

---

## References

- **Official Docs:** https://docs.x.ai
- **API Console:** https://console.x.ai
- **Pricing:** Check console for latest pricing
- **Status Page:** Monitor for service updates

---

## Notes for Implementation

1. **OpenAI Compatibility:** The API follows OpenAI's schema, making it easy to adapt existing OpenAI client code
2. **Stateless Design:** Unlike some chat APIs, X.AI does not maintain conversation state - you must send full message history
3. **Flexible Roles:** Message role ordering is not enforced (unlike OpenAI which requires alternating user/assistant)
4. **Agentic Tools:** Unique server-side tool execution feature not available in standard OpenAI API
5. **Large Context:** 2M token context window in Grok 4 Fast enables processing of very large documents
6. **Real-time Data:** Built-in web and X search tools provide access to current information beyond knowledge cutoff

---

**Document Version:** 1.0
**Research Date:** 2025-11-08
**Sources:** Official X.AI documentation, developer guides, API reviews, community resources
