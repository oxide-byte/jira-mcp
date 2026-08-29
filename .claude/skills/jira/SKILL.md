---
name: jira
description: Integration skill for querying Jira issues via the MCP server running on localhost:3030. Automatically enriches Claude context with Jira issue data, summaries, status, and metadata for more informed development assistance.
license: MIT
---

# Jira MCP Integration Skill

This skill provides seamless integration with the local Jira MCP server (running on `http://localhost:3030`), enabling Claude to fetch and contextualize Jira issue information automatically.

## When to Use

Activate this skill when:
- Working on issues referenced in conversation (e.g., "working on TEST-1")
- Needing issue details for context (status, priority, assignee, description)
- Correlating code changes with Jira tracking
- Debugging based on issue requirements
- Documenting solutions with issue context
- Running the jira-mcp server locally for development

## Prerequisites

- Jira MCP server running on `http://localhost:3030`
- Server must have the `/health` endpoint available
- Network connectivity to localhost

**To start the server:**
```bash
cargo run --release
```

## Quick Start

When you mention a Jira issue key (e.g., `TEST-1`, `PROJ-123`), this skill will:

1. **Validate** the server is healthy
2. **Fetch** the full issue data from `/jira/{id}`
3. **Extract** key metadata (status, priority, assignee, dates)
4. **Enhance** Claude context with a structured summary

Example conversation:
```
User: "Can you help me understand what TEST-1 is about?"
Skill: [Fetches /jira/TEST-1 → returns full issue data]
Claude: (with enriched context) "TEST-1 is a [Status] issue about [Summary]..."
```

## Features

### Issue Data Enrichment

When fetching an issue, the skill retrieves:
- **Key & ID**: Unique identifier (e.g., "TEST-1", "12345")
- **Summary**: One-line issue title
- **Description**: Detailed issue description
- **Status**: Current workflow state (Open, In Progress, Done, etc.)
- **Priority**: Issue severity/importance (Highest, High, Medium, Low, Lowest)
- **Assignee**: Person responsible for the issue
- **Created/Updated**: Timestamps for tracking lifecycle
- **Response Type**: Full JSON for advanced queries

### Automatic Context Injection

Claude automatically receives issue data when:
- Issue key is mentioned in conversation
- User asks about ticket details
- Issue context is needed for decision-making

### Error Handling

- **Server Unavailable**: Gracefully degrades; suggests starting the server
- **Issue Not Found**: Returns 404; suggests checking the key
- **Server Error**: Logs the error; allows fallback to manual entry

## Reference Files

| Topic | File |
|-------|------|
| How to call the MCP server and available endpoints | [mcp-integration.md](mcp-integration.md) |
| How retrieved data is structured and used | [context-enhancement.md](context-enhancement.md) |
| Examples and troubleshooting | [examples.md](examples.md) |

## Key Endpoints

The MCP server provides these endpoints for integration:

| Endpoint | Method | Purpose | Example |
|----------|--------|---------|---------|
| `/health` | GET | Server health check | `curl http://localhost:3030/health` |
| `/jira/{id}` | GET | Fetch issue data | `curl http://localhost:3030/jira/TEST-1` |

## How It Works

### Request Flow

```
User mentions issue key (e.g., "TEST-1")
    ↓
Skill detects pattern
    ↓
Check server health at /health
    ↓
Fetch issue from /jira/{id}
    ↓
Parse JSON response
    ↓
Extract key fields (status, priority, summary, etc.)
    ↓
Format as structured context block
    ↓
Inject into Claude conversation
    ↓
Claude provides informed response
```

### Response Structure

```json
{
  "key": "TEST-1",
  "id": "12345",
  "fields": {
    "summary": "Example issue title",
    "description": "Detailed description of the issue",
    "status": { "name": "Open" },
    "priority": { "name": "High" },
    "assignee": { "displayName": "Jane Doe" },
    "created": "2024-01-15T10:30:00Z",
    "updated": "2024-08-29T14:22:00Z"
  }
}
```

## Usage Patterns

### Pattern 1: Direct Issue Reference
```
User: "I'm working on TEST-1 today"
Skill: Fetches issue details → enhances context
Claude: "Great! TEST-1 is [status] and assigned to [assignee]..."
```

### Pattern 2: Status Check
```
User: "What's the status of PROJ-456?"
Skill: Fetches full issue with status
Claude: "PROJ-456 is currently [status] and [next steps]..."
```

### Pattern 3: Priority Awareness
```
User: "Should I prioritize this differently?"
Skill: Provides priority level in context
Claude: "Given the [Priority] priority, you should [recommendation]..."
```

### Pattern 4: Historical Context
```
User: "When was this created?"
Skill: Includes created/updated dates
Claude: "This was created on [date] and last updated on [date]..."
```

## Configuration

No additional configuration needed—the skill works with the default server running on `localhost:3030`.

**To customize the server:**
Edit your server's `main.rs` or use environment variables:
```bash
SERVER_PORT=3030 JIRA_URL=http://your-jira-instance cargo run
```

## Testing the Integration

### 1. Verify Server is Running
```bash
curl http://localhost:3030/health
```
Expected response: `{"status": "ok"}`

### 2. Test Issue Fetch
```bash
curl http://localhost:3030/jira/TEST-1
```
Expected response: Full Jira issue JSON

### 3. Test in Claude
Mention "TEST-1" in conversation and observe context enrichment.

## Troubleshooting

| Problem | Solution |
|---------|----------|
| "Server unavailable" | Ensure `cargo run` is active on the project; server runs on port 3030 |
| "Issue not found (404)" | Check the issue key spelling (e.g., `TEST-1` vs `test-1`) |
| "Connection refused" | Server may not be running; start with `cargo run --release` |
| No context enrichment | Ensure you mention a valid Jira key in the conversation |

## Performance Notes

- **Response Time**: ~100-500ms per issue (depends on server load)
- **Caching**: Each issue is fetched fresh per mention (no local caching)
- **Limits**: Server maintains call log of ~100 recent requests

## Advanced Usage

### Filtering by Status
```
User: "Show me all TEST issues in progress"
Skill: [Requires multiple calls to /jira/TEST-*]
Claude: (with context from multiple fetches) "Here are active TEST issues..."
```

### Priority-Based Decisions
```
User: "Which issue should I tackle first?"
Skill: Fetches both issues, provides priority context
Claude: "Based on priority [High > Medium], tackle [issue-1] first"
```

### Timeline Tracking
```
User: "How long has this been open?"
Skill: Provides created/updated dates
Claude: "Created on [date], it's been open for [duration]..."
```

## Integration with Claude Context

This skill automatically enhances Claude's context block with:

```
## Active Jira Issues
- **TEST-1** [Status: Open] [Priority: High]
  Summary: Example issue
  Assignee: John Doe
  Created: 2024-01-15 | Last Updated: 2024-08-29
```

This allows Claude to:
- Provide informed recommendations based on issue status
- Prioritize work based on issue priority levels
- Understand team assignments and ownership
- Reference issue timeline for scheduling decisions

## See Also

- [MCP Integration Details](mcp-integration.md)
- [Context Enhancement Guide](context-enhancement.md)
- [Examples & Troubleshooting](examples.md)
- [Main CLAUDE.md](../../CLAUDE.md) - Project architecture