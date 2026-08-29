# Context Enhancement Guide

This document explains how data retrieved from the Jira MCP server is structured, formatted, and injected into Claude's conversation context to provide more informed assistance.

## Context Injection Overview

When a Jira issue key is detected in the conversation, the skill:

1. **Detects** issue keys using pattern matching (e.g., `TEST-1`, `PROJ-123`)
2. **Validates** the server is healthy via `/health` endpoint
3. **Fetches** full issue data via `/jira/{id}` endpoint
4. **Extracts** key fields (status, priority, assignee, dates)
5. **Formats** data into structured context blocks
6. **Injects** into Claude's context before generating responses

## Context Block Structure

### Minimal Context (Quick Reference)

Used when only brief information is needed:

```
## Jira Issue: {KEY}
**Status:** {status} | **Priority:** {priority}
**Summary:** {summary}
**Assignee:** {assignee}
```

**Example:**
```
## Jira Issue: TEST-1
**Status:** Open | **Priority:** High
**Summary:** Implement Jira MCP integration
**Assignee:** Jane Doe
```

### Standard Context (Complete Issue Overview)

Used in most conversations:

```
## Jira Issue: {KEY}
**Status:** {status} | **Priority:** {priority}
**Summary:** {summary}
**Assignee:** {assignee}
**Created:** {created_date} | **Updated:** {updated_date}
**Description:** 
{description}
```

**Example:**
```
## Jira Issue: TEST-1
**Status:** Open | **Priority:** High
**Summary:** Implement Jira MCP integration
**Assignee:** Jane Doe
**Created:** 2024-01-15 | **Updated:** 2024-08-29
**Description:** 
Create a skill that integrates with the local Jira server running on localhost:3030.
This skill should fetch issue data, extract key metadata, and enhance Claude's context
with issue-specific information for informed development assistance.
```

### Multi-Issue Context (Batch Overview)

Used when multiple issues are discussed:

```
## Jira Issues
| Key | Status | Priority | Assignee | Summary |
|-----|--------|----------|----------|---------|
| {KEY1} | {status} | {priority} | {assignee} | {summary} |
| {KEY2} | {status} | {priority} | {assignee} | {summary} |
| {KEY3} | {status} | {priority} | {assignee} | {summary} |

### Details

**{KEY1}**: {description}
**{KEY2}**: {description}
**{KEY3}**: {description}
```

**Example:**
```
## Jira Issues
| Key | Status | Priority | Assignee | Summary |
|-----|--------|----------|----------|---------|
| TEST-1 | Open | High | Jane Doe | Implement Jira MCP integration |
| TEST-2 | In Progress | Medium | John Smith | Add error handling |
| PROJ-5 | Done | Low | Jane Doe | Documentation updates |

### Details

**TEST-1**: Create a skill that integrates with the local Jira server...
**TEST-2**: Enhance error handling in server responses...
**PROJ-5**: Update all documentation files...
```

---

## Field Mapping & Formatting

### Issue Key
```
Raw: "TEST-1"
Display: **TEST-1**
Usage: Unique identifier, always included
```

### Status
```
Raw: { "name": "Open" }
Display: Open
Formatting:
  - Open → 🔵 Open (or just "Open")
  - In Progress → 🟡 In Progress
  - Done → 🟢 Done
```

### Priority
```
Raw: { "name": "High" }
Display: High
Mapping:
  - Highest → 🔴 Highest
  - High → 🟠 High
  - Medium → 🟡 Medium
  - Low → 🔵 Low
  - Lowest → ⚫ Lowest
```

### Summary
```
Raw: "Implement Jira MCP integration"
Display: Implement Jira MCP integration
Length: Truncate to 80 chars if > 80
Format: Plain text, no markdown
```

### Description
```
Raw: "Create a skill that integrates with the local Jira server..."
Display: (full text)
Format: Plain text or markdown (if contains formatting)
Handling: 
  - Empty → "(No description)"
  - > 500 chars → Include ellipsis and note "[Description truncated...]"
  - Markdown → Preserve formatting (**, -, #, etc.)
```

### Assignee
```
Raw: { "displayName": "Jane Doe", "emailAddress": "jane@example.com" }
Display: Jane Doe
Alternative: Jane Doe (jane@example.com) - if email available
Handling:
  - Null/missing → "(Unassigned)"
  - Multiple assignees → comma-separated list
```

### Timestamps
```
Raw: "2024-08-29T14:23:45.123Z" (ISO 8601)
Display: 2024-08-29 14:23:45 (human-readable)
Alternative: Aug 29, 2024 (short format)
Relative: "Created 226 days ago" (optional)
```

---

## Usage Patterns for Claude

### Pattern 1: Status-Aware Recommendations

**Input:**
```
User: "Should I start working on TEST-1?"

Context Injected:
## Jira Issue: TEST-1
**Status:** Open | **Priority:** High
**Summary:** Implement Jira MCP integration
**Assignee:** (Unassigned)
```

**Claude Response Behavior:**
Claude understands:
- Issue is **Open** (not yet started)
- Priority is **High** (should be prioritized)
- **Unassigned** (user can take it)

Generates response:
```
Yes, TEST-1 is a great choice. It's marked as High priority and currently Open, 
so you can start immediately. The issue focuses on implementing Jira MCP integration...
```

### Pattern 2: Priority-Based Prioritization

**Input:**
```
User: "What should I work on?"

Context (Multiple Issues):
## Jira Issues
| Key | Status | Priority |
|-----|--------|----------|
| TEST-1 | Open | High |
| TEST-2 | Open | Medium |
| TEST-3 | Open | Low |
```

**Claude Response Behavior:**
Claude understands priority ordering and generates:
```
Based on priority levels:
1. **TEST-1** (High) - Implement Jira MCP integration - Start here
2. **TEST-2** (Medium) - Add error handling - Next
3. **TEST-3** (Low) - Documentation updates - After higher priorities
```

### Pattern 3: Timeline & History Awareness

**Input:**
```
User: "How long has this been in progress?"

Context Injected:
**Created:** 2024-01-15 | **Updated:** 2024-08-29
```

**Claude Response Behavior:**
Claude can calculate and explain:
```
TEST-1 was created on January 15, 2024, and last updated on August 29, 2024.
That's about 226 days in the system. If it's still Open (not started), 
it's been waiting quite a while—definitely time to tackle it.
```

### Pattern 4: Team Assignment Awareness

**Input:**
```
User: "Who's working on this?"

Context Injected:
**Assignee:** Jane Doe
```

**Claude Response Behavior:**
Claude provides informed response:
```
TEST-1 is assigned to Jane Doe. You might want to coordinate with her 
before starting work, or check if she needs assistance.
```

---

## Context Enrichment Examples

### Example 1: Simple Question

**User Query:**
```
"What's TEST-1 about?"
```

**Context Injected:**
```
## Jira Issue: TEST-1
**Status:** Open | **Priority:** High
**Summary:** Implement Jira MCP integration
**Assignee:** (Unassigned)
**Created:** 2024-01-15 | **Updated:** 2024-08-29
**Description:** 
Create a skill that integrates with the local Jira server running on localhost:3030.
This skill should fetch issue data, extract key metadata, and enhance Claude's context
with issue-specific information for informed development assistance.
```

**Claude Response:**
```
TEST-1 is about implementing Jira MCP integration. Specifically, you need to create 
a skill that connects to the local Jira server on localhost:3030 to fetch issue data 
and enhance Claude's context. It's marked as High priority and currently unassigned, 
so it's available to work on.
```

---

### Example 2: Dependency Analysis

**User Query:**
```
"Can I start TEST-2 or should I finish TEST-1 first?"
```

**Context Injected:**
```
## Jira Issues
| Key | Status | Priority | Summary |
|-----|--------|----------|---------|
| TEST-1 | Open | High | Implement Jira MCP integration |
| TEST-2 | Open | Medium | Add error handling |

**TEST-1 Details:**
Create a skill that integrates with the local Jira server...

**TEST-2 Details:**
Enhance error handling in server responses to gracefully degrade when 
Jira server is unavailable...
```

**Claude Response:**
```
TEST-2 depends on TEST-1. You'll want to complete the basic Jira MCP integration 
(TEST-1) before adding comprehensive error handling (TEST-2). 

Since TEST-1 is High priority and unstarted, start there. Once the basic integration 
is working, then tackle TEST-2's error handling enhancements.
```

---

### Example 3: Progress Tracking

**User Query:**
```
"I just finished the implementation. What's next?"
```

**Context Injected:**
```
## Jira Issue: TEST-1
**Status:** Open | **Priority:** High
**Summary:** Implement Jira MCP integration
**Assignee:** Jane Doe
**Created:** 2024-01-15 | **Updated:** 2024-08-29
```

**Claude Response:**
```
Great! Now that you've completed the implementation, you should:

1. Update TEST-1 status to "Done" or "In Review" in Jira
2. Assign it to Jane Doe (the original assignee) if needed
3. Move to the next high-priority item: TEST-2 (Add error handling)

The context shows you can leverage the integration you just built for TEST-2's 
error handling requirements.
```

---

## Context Priority Levels

The skill injects context at different levels based on conversation content:

### Level 1: Minimal (Issue Mentioned in Passing)
```
STATUS: Low context injection
FIELDS: Key, Status, Priority (only)
USE CASE: Issue mentioned but not primary focus
```

### Level 2: Standard (Issue is Main Topic)
```
STATUS: Normal context injection
FIELDS: Key, Status, Priority, Summary, Assignee, Dates, Description
USE CASE: User asks questions about specific issue
```

### Level 3: Extended (Multiple Issues or Deep Analysis)
```
STATUS: Full context injection
FIELDS: All fields from Level 2 + Full descriptions, timelines
FORMAT: Table for quick overview + details section
USE CASE: Priority analysis, dependency tracking, timeline analysis
```

---

## Special Formatting Rules

### Code in Descriptions

If description contains code blocks (markdown ```), preserve them:

```markdown
## Description
To fix this, update src/main.rs:

```rust
pub async fn handle_jira_request(id: String) {
    // Implementation
}
```
```

### Links in Descriptions

If description contains links, preserve them:

```markdown
**Description:**
See the architecture diagram: [link-to-diagram](https://...)
For more details, check: https://docs.jira.com/...
```

### Lists in Descriptions

Preserve bullet/numbered lists:

```markdown
**Description:**
This issue covers:
- Implement REST endpoints
- Add error handling
- Write unit tests
- Update documentation
```

---

## Error Handling in Context

### Server Unavailable

```
## Jira Issue: TEST-1 [⚠️ Data unavailable]

Context fetched from cache or last known state:
**Last Known Status:** Open | **Priority:** High
**Summary:** Implement Jira MCP integration
**Note:** Live data unavailable - Jira server not responding.
If you see this, ensure the server is running: cargo run
```

### Issue Not Found

```
## Jira Issue: TEST-1 [❌ Not found]

The issue could not be retrieved from the server.
Possible reasons:
- Issue key spelling is incorrect (check TEST vs test)
- Issue was deleted
- Server is not running

Did you mean: TEST-123 or PROJ-1?
```

### Partial Data

```
## Jira Issue: TEST-1
**Status:** Open | **Priority:** High
**Summary:** Implement Jira MCP integration
**Assignee:** (Unavailable)
**Created:** 2024-01-15 | **Updated:** (Not retrieved)

⚠️ Some fields could not be retrieved. The core information above is available.
```

---

## Customization Options

### Minimal Format (for context efficiency)

```
TEST-1: Open | High - "Implement Jira MCP integration"
```

### Verbose Format (for detailed discussion)

```
**TEST-1: Implement Jira MCP integration**
- Status: Open
- Priority: High  
- Assignee: Jane Doe
- Created: 2024-01-15 14:23:45
- Updated: 2024-08-29 09:15:22
- Description: [Full description text]
```

### Rich Format (for comprehensive context)

```
## 🔵 TEST-1 [Open] [🟠 High]
**Summary:** Implement Jira MCP integration
**Assignee:** Jane Doe (jane@example.com)
**Timeline:** Created Jan 15, 2024 • Updated Aug 29, 2024 (226 days)
**Description:** [Full description with markdown preserved]
```

---

## See Also

- [MCP Integration](mcp-integration.md) - How to call the server
- [Examples](examples.md) - Real-world usage examples
- [SKILL.md](SKILL.md) - Main skill overview
