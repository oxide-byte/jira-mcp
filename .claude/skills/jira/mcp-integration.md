# MCP Server Integration

This document details how the Jira skill integrates with the Model Context Protocol (MCP) server running locally on `http://localhost:3030`.

## Server Architecture

The Jira MCP server is a Rust application combining:
- **Axum** web server for HTTP endpoints
- **Ratatui** TUI for real-time monitoring of API calls
- **Tokio** async runtime coordinating both components

**Default Configuration:**
- **Host:** `localhost` (127.0.0.1)
- **Port:** `3030`
- **Base URL:** `http://localhost:3030`
- **Protocol:** HTTP (REST API)

## Available Endpoints

### 1. Health Check

**Endpoint:** `GET /health`

**Purpose:** Verify server is running and healthy

**Request:**
```bash
curl -X GET http://localhost:3030/health
```

**Response (200 OK):**
```json
{
  "status": "ok",
  "timestamp": "2024-08-29T14:23:45.123Z"
}
```

**Use Cases:**
- Initial connectivity check before fetching issues
- Graceful degradation if server is unavailable
- Server status verification in monitoring

---

### 2. Fetch Jira Issue

**Endpoint:** `GET /jira/{id}`

**Purpose:** Retrieve full Jira issue data including fields, status, priority, assignee, etc.

**Parameters:**
- `{id}` (path parameter, required): Issue key in format `PROJECT-NUMBER` (e.g., `TEST-1`, `PROJ-123`)

**Request:**
```bash
curl -X GET http://localhost:3030/jira/TEST-1
```

**Response (200 OK):**
```json
{
  "key": "TEST-1",
  "id": "12345",
  "fields": {
    "summary": "Implement Jira MCP integration",
    "description": "Create a skill that integrates with the local Jira server...",
    "status": {
      "name": "Open"
    },
    "priority": {
      "name": "High"
    },
    "assignee": {
      "displayName": "Jane Doe",
      "emailAddress": "jane@example.com"
    },
    "created": "2024-01-15T10:30:00Z",
    "updated": "2024-08-29T14:22:00Z"
  }
}
```

**Field Descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `key` | string | Unique issue identifier (e.g., "TEST-1") |
| `id` | string | Numeric issue ID |
| `fields.summary` | string | One-line issue title |
| `fields.description` | string | Detailed issue description |
| `fields.status.name` | string | Current status (Open, In Progress, Done, etc.) |
| `fields.priority.name` | string | Priority level (Highest, High, Medium, Low, Lowest) |
| `fields.assignee.displayName` | string | Name of assigned developer |
| `fields.assignee.emailAddress` | string | Email of assigned developer |
| `fields.created` | ISO 8601 timestamp | When issue was created |
| `fields.updated` | ISO 8601 timestamp | Last update time |

**Error Responses:**

**404 Not Found** - Issue doesn't exist:
```json
{
  "error": "Issue not found",
  "key": "INVALID-999"
}
```

**500 Internal Server Error** - Server error:
```json
{
  "error": "Failed to fetch issue",
  "reason": "Connection to Jira failed"
}
```

---

## Integration Patterns

### Pattern 1: Health Check Before Fetching

```
1. GET /health
   ↓ (if 200 OK)
2. GET /jira/{issue-key}
   ↓ (on success)
3. Enrich context with issue data
```

**Pseudocode:**
```
function fetchJiraIssue(issueKey) {
  // Check server health
  healthResponse = GET /health
  if (healthResponse.status != 200) {
    return error("Server unavailable")
  }
  
  // Fetch issue
  issueResponse = GET /jira/{issueKey}
  if (issueResponse.status == 200) {
    return parseJSON(issueResponse.body)
  } else if (issueResponse.status == 404) {
    return error("Issue not found")
  } else {
    return error("Server error")
  }
}
```

### Pattern 2: Extract Context Summary

From full issue response, extract key fields for context injection:

```
Input: Full /jira/{id} response

Extract:
- issue.key
- issue.fields.summary
- issue.fields.status.name
- issue.fields.priority.name
- issue.fields.assignee.displayName (if exists)
- issue.fields.created
- issue.fields.updated

Format as:
**{key}** [{Status}] [{Priority}]
Summary: {summary}
Assignee: {assignee}
Created: {created} | Updated: {updated}
```

### Pattern 3: Batch Issue Fetching

When multiple issues are mentioned:

```
issues = detect_issue_keys(user_message)
// e.g., ["TEST-1", "TEST-2", "PROJ-5"]

for each issue in issues:
  1. GET /jira/{issue}
  2. Extract summary, status, priority
  3. Add to context block

Context Block: 
## Active Issues
- TEST-1 [Open] [High]
- TEST-2 [In Progress] [Medium]
- PROJ-5 [Done] [Low]
```

---

## Call Logging

The MCP server maintains a **real-time call log** visible in the Ratatui TUI:

**Tracked Information:**
- **Timestamp**: Precise time of request
- **Method**: HTTP method (GET, POST, etc.)
- **Path**: Request path (e.g., `/health`, `/jira/TEST-1`)
- **Status Code**: HTTP response code (200, 404, 500, etc.)
- **Response**: JSON response body

**Colors in TUI:**
- **Green**: 2xx status codes (success)
- **Yellow**: 3xx status codes (redirect)
- **Red**: 4xx/5xx status codes (errors)

**Log Capacity:**
- Maximum 100 calls retained (LIFO - newest first)
- Older calls are discarded automatically
- No persistence between server restarts

---

## Request/Response Format

### Content-Type

All requests and responses use:
```
Content-Type: application/json
```

### Character Encoding

UTF-8 for all text fields

### Timestamp Format

ISO 8601 format (RFC 3339):
```
2024-08-29T14:23:45.123Z
```

---

## Performance Characteristics

### Response Times

| Endpoint | Typical Response Time |
|----------|----------------------|
| `GET /health` | 10-50ms |
| `GET /jira/{id}` | 50-500ms (depending on response size) |

### Network Requirements

- **Protocol**: HTTP 1.1
- **Keep-Alive**: Supported
- **Compression**: No automatic compression
- **Timeout**: Default 30 seconds (configurable)

### Concurrency

- Server handles multiple concurrent requests
- No rate limiting (local development server)
- Optimal: < 100 concurrent requests

---

## Error Handling Strategies

### Strategy 1: Retry on Failure

```
function fetchWithRetry(issueKey, maxRetries=3) {
  for attempt in 1..maxRetries:
    try:
      response = GET /jira/{issueKey}
      if response.status == 200:
        return response
      else if response.status == 404:
        return error("Issue not found", permanent=true)
    catch ConnectionError:
      wait(exponential_backoff(attempt))
  
  return error("Max retries exceeded")
}
```

### Strategy 2: Graceful Degradation

```
function enhanceContextWithJira(issueKeys) {
  for key in issueKeys:
    try:
      issue = fetchJiraIssue(key)
      addToContext(issue)
    catch:
      // Continue without Jira data
      log.warning("Could not fetch " + key)
}
```

### Strategy 3: Cache Recent Fetches

```
cache = {}

function fetchJiraIssue(issueKey) {
  if issueKey in cache:
    return cache[issueKey]
  
  response = GET /jira/{issueKey}
  cache[issueKey] = response
  return response
}
```

---

## Debugging & Monitoring

### Enable Server Logging

```bash
RUST_LOG=jira_mcp=debug cargo run
```

Outputs detailed logs to stderr including:
- All HTTP requests/responses
- Processing times
- Error details

### Watch Call Log in TUI

```bash
# While server is running in another terminal:
cargo run
# Watch the Ratatui TUI for live call logging
```

### Test Endpoints Manually

```bash
# Test health
curl -v http://localhost:3030/health

# Test issue fetch
curl -v http://localhost:3030/jira/TEST-1 | jq .

# Pretty-print JSON
curl http://localhost:3030/jira/TEST-1 | jq .fields
```

---

## Starting the Server

### Quick Start

```bash
# Navigate to project directory
cd /Users/qdart/projects/jira-mcp

# Build and run
cargo run --release
```

### Development Mode (with logging)

```bash
RUST_LOG=jira_mcp=debug cargo run
```

### Port Configuration

To use a different port, modify `src/main.rs`:

```rust
// Find this line:
let listener = TcpListener::bind("127.0.0.1:3030").await?;

// Change 3030 to your desired port
let listener = TcpListener::bind("127.0.0.1:8080").await?;
```

Then rebuild:

```bash
cargo build --release
```

---

## See Also

- [Context Enhancement Guide](context-enhancement.md) - How to use fetched data
- [Examples](examples.md) - Real-world usage examples
- [SKILL.md](SKILL.md) - Main skill overview
