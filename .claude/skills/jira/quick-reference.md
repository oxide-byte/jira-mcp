# Quick Reference

Essential commands and patterns for working with the Jira MCP skill.

## Starting the Server

```bash
# Basic run
cargo run

# Optimized release build
cargo run --release

# With debug logging
RUST_LOG=jira_mcp=debug cargo run

# On custom port (edit src/main.rs, change 3030 to your port)
cargo run --release
```

## Testing the Server

```bash
# Health check
curl http://localhost:3030/health

# Fetch an issue
curl http://localhost:3030/jira/TEST-1

# Pretty-print with jq
curl http://localhost:3030/jira/TEST-1 | jq .

# Check specific field
curl http://localhost:3030/jira/TEST-1 | jq .fields.status.name
```

## Using with Claude

### Activate the Skill

```
/skill jira
```

### Ask About Issues

```
What's TEST-1?
Tell me about TEST-1
What's the status of TEST-1?
Can I work on TEST-1?
```

### Compare Multiple Issues

```
Should I work on TEST-1 or TEST-2?
What's the priority of TEST-1, TEST-2, and TEST-3?
```

### Timeline Analysis

```
How old is TEST-1?
When was TEST-1 last updated?
```

### Team Coordination

```
Who's assigned to TEST-1?
Is TEST-1 assigned to anyone?
```

## Common Issue Keys (Mock Data)

Default mock data in `src/jira.rs`:

```
TEST-1   - Implement Jira MCP integration
TEST-2   - Add error handling
TEST-3   - Documentation updates
PROJ-1   - Other projects (if configured)
```

**To add more issues**, edit `src/jira.rs`:

```rust
pub async fn get_issue(&self, issue_key: &str) -> Result<JiraIssue> {
    let issue = match issue_key {
        "TEST-1" => { /* existing */ },
        "TEST-2" => { /* existing */ },
        "YOUR-KEY" => {
            JiraIssue {
                key: "YOUR-KEY".to_string(),
                // ... fields
            }
        },
        _ => return Err("Issue not found".into()),
    };
    Ok(issue)
}
```

Then rebuild: `cargo build --release`

## Troubleshooting One-Liners

```bash
# Is server running?
curl -s http://localhost:3030/health

# What's using port 3030?
lsof -i :3030

# Kill process using port 3030
kill -9 $(lsof -t -i :3030)

# Watch server logs in real-time
RUST_LOG=jira_mcp=debug cargo run 2>&1 | grep -E "GET|POST|200|404|500"

# Check if cargo needs to rebuild
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test
```

## File Structure

```
.claude/skills/jira/
├── SKILL.md                  ← Main skill documentation
├── mcp-integration.md        ← API endpoints and protocols
├── context-enhancement.md    ← How data is formatted/injected
├── examples.md               ← Real-world usage & troubleshooting
└── quick-reference.md        ← This file
```

## Context Block Examples

### Quick Reference (Minimal)
```
TEST-1: Open | High - "Implement Jira MCP integration"
```

### Standard (Complete)
```
## Jira Issue: TEST-1
**Status:** Open | **Priority:** High
**Summary:** Implement Jira MCP integration
**Assignee:** Jane Doe
**Created:** 2024-01-15 | **Updated:** 2024-08-29
```

### Rich Format (Detailed)
```
## 🔵 TEST-1 [Open] [🟠 High]
**Summary:** Implement Jira MCP integration
**Assignee:** Jane Doe
**Timeline:** Created Jan 15, 2024 • Last updated Aug 29, 2024
**Description:** Create a skill that integrates with the local Jira server...
```

## HTTP Status Codes

| Code | Meaning | Action |
|------|---------|--------|
| 200 | OK | Request successful ✓ |
| 404 | Not Found | Issue doesn't exist - check key |
| 500 | Server Error | Server crashed - restart with `cargo run` |
| (timeout) | No Response | Server not running |

## Jira Field Guide

| Field | Example | Used For |
|-------|---------|----------|
| **Key** | TEST-1 | Issue identifier |
| **Status** | Open, In Progress, Done | Current state |
| **Priority** | High, Medium, Low | Importance |
| **Summary** | Issue title | Quick description |
| **Description** | Full details | Complete context |
| **Assignee** | Jane Doe | Who's working on it |
| **Created** | 2024-01-15 | Historical context |
| **Updated** | 2024-08-29 | Recency indicator |

## Issue Key Format

```
Standard:     PROJECT-NUMBER
Examples:     TEST-1, PROJ-123, ABC-456

Case:         Uppercase required (TEST-1, not test-1)
Format:       PROJECT and NUMBER separated by hyphen (-)
Spaces:       None (TEST-1, not TEST - 1)
```

## Configuration

### Server Port

**Change from 3030 to custom port:**

1. Edit `src/main.rs`
2. Find: `let listener = TcpListener::bind("127.0.0.1:3030")`
3. Replace `3030` with your port
4. Rebuild: `cargo build --release`

### Jira Mock Data

**Add or modify issues:**

1. Edit `src/jira.rs`
2. Find: `pub async fn get_issue()`
3. Add to match statement:
   ```rust
   "YOUR-KEY" => JiraIssue {
       key: "YOUR-KEY".to_string(),
       // ... fields
   }
   ```
4. Rebuild: `cargo build --release`

### Enable Logging

Set environment variable before running:

```bash
# Debug level (verbose)
export RUST_LOG=jira_mcp=debug

# Trace level (very verbose)
export RUST_LOG=jira_mcp=trace

# Then run
cargo run --release
```

## Performance Tips

### Fast Testing
```bash
# Use release build for faster performance
cargo run --release
```

### Batch Requests
```
Ask Claude: "Summarize TEST-1, TEST-2, TEST-3"
# Instead of asking one at a time
```

### Monitor Real-Time
```bash
# Watch TUI while testing
# Keep terminal visible to see call log updates
```

## Common Patterns

### Pattern 1: Get Issue Status
```
User: "What's TEST-1?"
Claude: (fetches context) "TEST-1 is [Status], [Priority]..."
```

### Pattern 2: Pick Next Task
```
User: "Should I work on TEST-1 or TEST-2?"
Claude: (compares) "Start TEST-1 (higher priority)..."
```

### Pattern 3: Timeline Awareness
```
User: "How old is this?"
Claude: (checks dates) "Created 226 days ago..."
```

### Pattern 4: Team Handoff
```
User: "Who's on this?"
Claude: (checks assignee) "Assigned to Jane Doe..."
```

## Debugging Checklist

- [ ] Server running? `curl http://localhost:3030/health`
- [ ] Issue exists? `curl http://localhost:3030/jira/TEST-1`
- [ ] Skill enabled? `/skill jira` in Claude
- [ ] Issue key in message? "Tell me about TEST-1"
- [ ] Network available? `ping localhost`
- [ ] Port 3030 free? `lsof -i :3030`

## Getting Help

1. Check [examples.md](examples.md) - Troubleshooting section
2. Run with debug logging: `RUST_LOG=jira_mcp=debug cargo run`
3. Test endpoint manually: `curl http://localhost:3030/jira/TEST-1 | jq .`
4. Read [mcp-integration.md](mcp-integration.md) - API details
5. Check server output for error messages

## See Also

- [SKILL.md](SKILL.md) - Overview
- [mcp-integration.md](mcp-integration.md) - API details  
- [context-enhancement.md](context-enhancement.md) - Context formatting
- [examples.md](examples.md) - Usage examples
