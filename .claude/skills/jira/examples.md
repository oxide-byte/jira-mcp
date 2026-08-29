# Examples & Troubleshooting

This document provides real-world usage examples and solutions for common issues.

## Quick Start Examples

### Example 1: Checking Issue Status

**Scenario:** You want to know the status of a specific issue.

**What You Do:**
```
Tell Claude: "What's the status of TEST-1?"
```

**What Happens Behind the Scenes:**
1. Skill detects `TEST-1`
2. Calls `GET /health` → Server responds OK
3. Calls `GET /jira/TEST-1` → Retrieves issue data
4. Extracts: status=Open, priority=High, summary="Implement Jira MCP integration"
5. Injects into context
6. Claude responds with enriched information

**Claude's Response:**
```
TEST-1 is currently **Open** and marked as **High** priority. 
It's about implementing Jira MCP integration. The issue has been in 
the system since January 15, 2024, and was last updated on August 29, 2024.
```

---

### Example 2: Prioritizing Work

**Scenario:** Multiple tasks to choose from, need guidance.

**What You Do:**
```
Tell Claude: "Should I work on TEST-1, TEST-2, or TEST-3?"
```

**What Happens:**
1. Skill detects `TEST-1`, `TEST-2`, `TEST-3`
2. Calls `GET /jira/{id}` for each issue
3. Extracts priorities and statuses:
   - TEST-1: Open, High
   - TEST-2: In Progress, Medium
   - TEST-3: Done, Low
4. Injects comparison into context
5. Claude analyzes and recommends

**Claude's Response:**
```
Based on the current status:

1. **TEST-1** (Open, High Priority) - Start this first
   This should be your top priority as it's unstarted and marked High.

2. **TEST-2** (In Progress, Medium Priority) - Might already be in progress
   If no one is working on it, this is your second choice.

3. **TEST-3** (Done, Low Priority) - Already complete
   No action needed here.

I'd recommend starting with TEST-1 if it's not assigned to anyone else.
```

---

### Example 3: Understanding Issue Description

**Scenario:** You need details about what an issue involves.

**What You Do:**
```
Tell Claude: "Tell me what TEST-1 involves"
```

**What Happens:**
1. Skill detects `TEST-1`
2. Fetches full issue including description
3. Injects complete context
4. Claude analyzes and summarizes

**Claude's Response:**
```
TEST-1 involves creating a skill that integrates with your local Jira server 
running on localhost:3030. Specifically, you need to:

1. Fetch issue data via REST endpoints
2. Extract metadata (status, priority, assignee, etc.)
3. Enhance Claude's conversation context with this information
4. Format data for injection into conversations

The skill should gracefully handle server unavailability and provide 
helpful error messages when issues can't be retrieved.
```

---

### Example 4: Timeline Analysis

**Scenario:** Understanding how long something has been waiting.

**What You Do:**
```
Tell Claude: "How old is TEST-1 and when was it last updated?"
```

**What Happens:**
1. Skill fetches issue with timestamps
2. Injects: created=2024-01-15, updated=2024-08-29
3. Claude calculates duration

**Claude's Response:**
```
TEST-1 was created on January 15, 2024, and last updated on August 29, 2024.

That means:
- **Age:** 226 days in the system
- **Last updated:** 0 days ago (just now)

This issue has been waiting quite a while. If it's still in "Open" status, 
it's definitely ready to be worked on.
```

---

### Example 5: Team Coordination

**Scenario:** Understanding who's responsible for work.

**What You Do:**
```
Tell Claude: "Who's assigned to TEST-1?"
```

**What Happens:**
1. Skill fetches issue with assignee info
2. Injects: assignee="Jane Doe"
3. Claude provides coordination guidance

**Claude's Response:**
```
TEST-1 is currently unassigned. This means:
- The work hasn't been claimed by anyone yet
- You can pick it up without coordination
- Consider assigning yourself when you start

If it WAS assigned to someone, I'd recommend checking with them first 
before starting to avoid duplicate effort.
```

---

## Troubleshooting Guide

### Problem 1: "Server Connection Failed"

**Symptoms:**
```
Error: Connection refused to localhost:3030
Jira context unavailable
```

**Causes:**
- Jira MCP server not running
- Server crashed
- Port 3030 is blocked/in use by another process
- Network issue

**Solutions:**

**Step 1: Check if server is running**
```bash
# In a terminal, check if the process exists
ps aux | grep "cargo run"

# Or try to connect manually
curl http://localhost:3030/health
```

**Step 2: Start the server**
```bash
# Navigate to project directory
cd /Users/qdart/projects/jira-mcp

# Start the server
cargo run --release
```

**Step 3: Verify it started**
```bash
# In another terminal, test the endpoint
curl http://localhost:3030/health
# Expected output: {"status":"ok","timestamp":"..."}
```

**Step 4: Check port conflicts**
```bash
# See what's using port 3030
lsof -i :3030

# If something else is using it, either:
# 1. Stop that process: kill <PID>
# 2. Or change the server port in src/main.rs
```

---

### Problem 2: "Issue Not Found (404)"

**Symptoms:**
```
Error: Issue TEST-1 not found
Status: 404
```

**Causes:**
- Issue key is misspelled
- Case sensitivity issue (TEST-1 vs test-1)
- Issue was deleted
- Wrong project key

**Solutions:**

**Check the issue key format:**
```bash
# Jira keys are typically:
# PROJECT-NUMBER

# Examples:
✓ TEST-1      (correct)
✓ PROJ-123    (correct)
✓ ABC-456     (correct)
✗ test-1      (wrong case - try TEST-1)
✗ TEST1       (missing dash)
✗ -1          (missing project)
```

**Verify the issue exists:**
```bash
# Try fetching in browser or curl
curl http://localhost:3030/jira/TEST-1

# If you get 404, the issue doesn't exist in the mock data
```

**For development/mock data:**

If using the mock Jira client, only hardcoded issues work. To add more:

1. Edit `src/jira.rs`
2. Find the `get_issue()` method
3. Add your issue key to the match statement:

```rust
pub async fn get_issue(&self, issue_key: &str) -> Result<JiraIssue> {
    let issue = match issue_key {
        "TEST-1" => JiraIssue { /* existing */ },
        "TEST-2" => JiraIssue {  // Add this
            key: "TEST-2".to_string(),
            id: "12346".to_string(),
            fields: JiraFields {
                summary: "Your issue title".to_string(),
                // ... more fields
            }
        },
        _ => return Err("Issue not found".into()),
    };
    Ok(issue)
}
```

4. Rebuild and restart:
```bash
cargo build --release
cargo run --release
```

---

### Problem 3: "Partial Data Received"

**Symptoms:**
```
Some fields missing from context
Assignee shows as "(Unavailable)"
Description not included
```

**Causes:**
- Server partially crashed
- Network timeout during fetch
- Malformed JSON response
- Server upgrade/compatibility issue

**Solutions:**

**Step 1: Check server logs**
```bash
# With debug logging enabled
RUST_LOG=jira_mcp=debug cargo run
# Look for error messages related to the request
```

**Step 2: Manually test the endpoint**
```bash
# Test the raw endpoint
curl -v http://localhost:3030/jira/TEST-1 | jq .

# Check if response has all expected fields
# Expected: key, id, fields.summary, fields.status, fields.priority, etc.
```

**Step 3: Restart the server**
```bash
# Stop current server (Ctrl+C)
# Wait 2-3 seconds
# Restart
cargo run --release
```

**Step 4: Clear TUI cache (if needed)**
```bash
# The Ratatui TUI maintains a call log
# You can clear it by restarting the server
# Or pressing 'c' if that's implemented (check your implementation)
```

---

### Problem 4: "Server Responds but No Context Injected"

**Symptoms:**
```
Server is running (curl /health works)
But Claude doesn't receive Jira context
```

**Causes:**
- Issue key not detected in message
- Wrong format (PROJ-123 vs proj-123)
- Skill not enabled in Claude settings
- Issue is in code block (ignored by detection)

**Solutions:**

**Check your message format:**
```
✓ "What's TEST-1?"
✓ "Work on TEST-1 next"
✓ "TEST-1 is important"
✗ "What's `TEST-1`?" (in backticks - might be ignored)
✗ "test-1" (lowercase - try TEST-1)
✗ "TEST 1" (with space - try TEST-1)
```

**Verify the skill is enabled:**
```bash
# Check project .claude/settings.json
cat .claude/settings.json | jq '.skills'

# Should see jira in the list
# If not, activate it with:
# /skill jira
```

**Try explicit activation:**
```
Tell Claude: "/skill jira"
Then: "Can you tell me about TEST-1?"
```

---

### Problem 5: "Timeout / Slow Response"

**Symptoms:**
```
Request takes > 10 seconds
Claude times out waiting for Jira data
```

**Causes:**
- Server is overwhelmed with requests
- Network latency
- Large issue responses
- Server is building (cargo compile)

**Solutions:**

**Check server CPU/memory:**
```bash
# In another terminal
top
# Look for the cargo/server process
# Check if CPU is at 100% or memory is maxed
```

**Reduce concurrent requests:**
```bash
# If fetching multiple issues, try one at a time
Tell Claude: "Tell me about TEST-1"
# Wait for response
Tell Claude: "Tell me about TEST-2"
```

**Optimize issue response:**

In `src/jira.rs`, reduce description length:

```rust
fields: JiraFields {
    summary: "...".to_string(),
    description: Some("Brief description only".to_string()), // Shorter
    // ...
}
```

**Check network connectivity:**
```bash
# Test latency to localhost
ping -c 3 localhost

# If > 50ms, there's a network issue
```

---

### Problem 6: "Mock Data Not Returning"

**Symptoms:**
```
Fetch succeeds (200 OK)
But response has null/empty fields
```

**Causes:**
- Mock data needs to be properly initialized
- JSON serialization issue
- Wrong endpoint path

**Solutions:**

**Verify the mock implementation:**

In `src/jira.rs`, the `get_issue()` function should return valid data:

```rust
pub async fn get_issue(&self, issue_key: &str) -> Result<JiraIssue> {
    // For development, return mock data
    let issue = match issue_key {
        "TEST-1" => JiraIssue {
            key: "TEST-1".to_string(),
            id: "12345".to_string(),
            fields: JiraFields {
                summary: "Example issue".to_string(),
                description: Some("Full description".to_string()),
                status: JiraStatus {
                    name: "Open".to_string(),
                },
                priority: JiraPriority {
                    name: "High".to_string(),
                },
                assignee: Some(JiraUser {
                    display_name: "Jane Doe".to_string(),
                    email_address: "jane@example.com".to_string(),
                }),
                created: "2024-01-15T10:30:00Z".to_string(),
                updated: "2024-08-29T14:22:00Z".to_string(),
            },
        },
        _ => return Err("Issue not found".into()),
    };
    Ok(issue)
}
```

**Test the endpoint directly:**
```bash
curl http://localhost:3030/jira/TEST-1 | jq .fields
# Should see all fields populated
```

---

## Advanced Troubleshooting

### Enable Debug Logging

```bash
# Set debug logging before running
RUST_LOG=jira_mcp=debug RUST_BACKTRACE=1 cargo run
```

**Look for:**
- Connection logs showing requests/responses
- Any panic messages
- Stack traces for errors

### Monitor Call Log in TUI

While the server runs, watch the Ratatui TUI:

```
┌─────────────────────────────────────┐
│ 📊 Jira MCP Call Tracker            │
├─────────────────────────────────────┤
│ Timestamp    | Method | Path | Code │
│ 14:23:45.123 | GET    | /jira/TEST-1 | 200 │
│ 14:23:44.567 | GET    | /health     | 200 │
├─────────────────────────────────────┤
```

**Status Code Guide:**
- 🟢 200-299: Success
- 🟡 300-399: Redirect
- 🔴 400-499: Client error (bad request, not found)
- 🔴 500-599: Server error

### Test with curl

```bash
# Health check
curl -v http://localhost:3030/health
# Expected: 200 OK

# Fetch issue with headers
curl -v -H "Content-Type: application/json" \
  http://localhost:3030/jira/TEST-1
# Expected: 200 OK with JSON body

# Pretty-print response
curl http://localhost:3030/jira/TEST-1 | jq .

# Check specific field
curl http://localhost:3030/jira/TEST-1 | jq .fields.status.name
```

---

## Performance Optimization Tips

### 1. Batch Multiple Issues Efficiently

**Instead of:**
```
Tell Claude: "What's TEST-1?"
Wait for response
Tell Claude: "What's TEST-2?"
Wait for response
```

**Do this:**
```
Tell Claude: "Summarize these issues: TEST-1, TEST-2, TEST-3"
# Skill detches all 3 in parallel
```

### 2. Cache Recent Issues

Track issues you've recently asked about:
```
Fetched: TEST-1, TEST-2, TEST-3 (in this session)
Claude can reference them without refetching
```

### 3. Minimize Description Lengths

In `src/jira.rs`, keep descriptions concise:
```rust
description: Some("Brief summary (avoid long texts)".to_string()),
```

### 4. Use Health Checks Wisely

```
First call in session: Check /health
Subsequent calls: Skip if known healthy (within last minute)
On error: Re-check /health before retrying
```

---

## Integration Checklist

Before considering the skill production-ready:

- [ ] Server starts without errors: `cargo run --release`
- [ ] Health check works: `curl http://localhost:3030/health`
- [ ] Fetch issue works: `curl http://localhost:3030/jira/TEST-1`
- [ ] Claude detects issue keys correctly
- [ ] Context is injected when issues mentioned
- [ ] Claude provides informed responses based on context
- [ ] Skill handles server unavailability gracefully
- [ ] Error messages are helpful
- [ ] No sensitive data in responses (check mock data)
- [ ] TUI shows call history without errors

---

## See Also

- [SKILL.md](SKILL.md) - Main skill overview
- [mcp-integration.md](mcp-integration.md) - API details
- [context-enhancement.md](context-enhancement.md) - Context formatting
