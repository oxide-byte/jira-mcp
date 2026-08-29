# Jira MCP - Model Context Protocol Server for Jira

A Rust application that provides a **Model Context Protocol (MCP)** server to integrate Jira ticket data with Claude AI. It combines:

- **Axum Web Server** - RESTful MCP endpoints for retrieving Jira issues with authentication
- **Ratatui TUI** - Real-time monitoring dashboard of all MCP calls and their responses
- **Tokio Async Runtime** - Non-blocking concurrent request handling

## Features

### 🔗 MCP Server (Port 3030)
- **Real Jira API Integration** - Connects to actual Jira instances with authentication
- **Basic Authentication** - Supports username and API token/password
- **/health** - Health check endpoint
- **/jira/{id}** - Fetch Jira issues with full field details

### 📊 Live Monitoring Dashboard
- Real-time table of all MCP calls with timestamps
- HTTP status codes with color coding:
  - 🟢 Green for 2xx (success)
  - 🟡 Yellow for 3xx (redirect)
  - 🔴 Red for 4xx/5xx (errors)
- Live-updating display with 500ms refresh
- Press `q` or `Esc` to exit

### ⚙️ Configuration Management
- **Interactive Login Modal** - Enter Jira credentials via TUI (press `L`)
- **.env File Support** - Load credentials from `JIRA_URL`, `JIRA_USERNAME`, `JIRA_PASSWORD`
- **Configuration Editor** - Edit saved credentials with popup modal
- **Environment Variables** - Fall back to system environment variables

### 🧠 Claude Integration
- **Jira Skill** - Use the bundled skill to automatically fetch Jira ticket data in Claude conversations
- **Pre-configured** - `.claude/settings.local.json` pre-approves localhost API calls
- **Seamless Context** - Mention ticket IDs and the skill fetches live data


A sample: .claude/settings.local.json

json
```
{
  "permissions": {
    "allow": [
      "Bash(curl * http://localhost:3030/health *)",
      "Bash(curl * http://localhost:3030/jira/* *)",
      "Bash(curl -s http://localhost:3030/health *)",
      "Bash(curl -s http://localhost:3030/jira/* *)",
      "WebFetch(domain:localhost)",
    ]
  }
}
```

## Quick Start

### Prerequisites
- Rust 1.70+ ([Install from rustup.rs](https://rustup.rs/))
- Cargo (included with Rust)
- Jira instance with API access (Cloud or Server)

### 1. Configuration

Create a `.env` file in the project root:

```env
JIRA_URL=https://your-domain.atlassian.net
JIRA_USERNAME=your-email@example.com
JIRA_PASSWORD=your-api-token
SERVER_PORT=3030
RUST_LOG=jira_mcp=info
```

**For Jira Cloud:**
- URL: `https://your-domain.atlassian.net`
- Username: Your email address
- Password: [Generate API token](https://id.atlassian.com/manage-profile/security/api-tokens)

**For Jira Server:**
- URL: `http://your-jira-server:port`
- Username: Your username
- Password: Your password or API token

### 2. Build & Run

```bash
# Build
cargo build --release

# Run
cargo run --release
```

The application will:
1. Start the MCP server on `http://localhost:3030`
2. Launch the monitoring TUI
3. Prompt for credentials if not in `.env` (press `L` to open login modal)

### 3. Test the Server

In another terminal:

```bash
# Health check
curl http://localhost:3030/health

# Fetch Jira issue (requires authentication)
curl http://localhost:3030/jira/PROJ-123
```

Each request appears in the TUI dashboard in real-time.

## Project Structure

```
src/
├── main.rs           - Entry point, initializes server and TUI
├── server.rs         - Axum web server with MCP endpoints
├── ui.rs             - Ratatui dashboard for monitoring calls
├── state.rs          - CallLog data structure for tracking requests
├── jira.rs           - Real Jira API client with authentication
├── config.rs         - Configuration loading from .env/env vars
└── modal.rs          - Interactive login and config edit modals

.claude/
└── skills/
    └── jira/         - Claude skill for Jira integration
        ├── SKILL.md
        ├── mcp-integration.md
        ├── context-enhancement.md
        ├── examples.md
        └── quick-reference.md

.claude/
└── settings.local.json  - Pre-approved MCP calls for Claude
```

## Architecture

```
┌──────────────────────────────────────┐
│     Ratatui TUI Dashboard            │
│  • Real-time call monitoring         │
│  • Login modal (press L)              │
│  • Config editor (press C)            │
└──────────────────┬───────────────────┘
                   │
                   ├─ Reads
                   │
                   v
         ┌─────────────────┐
         │ Shared CallLog  │
         │Arc<Mutex<...>>  │
         └─────────────────┘
                   ^
                   │
                   ├─ Writes
                   │
┌──────────────────┴───────────────────┐
│    Axum MCP Server (Port 3030)       │
│  • GET /health                       │
│  • GET /jira/{id}                    │
│  • Basic Authentication              │
└──────────────────┬───────────────────┘
                   │
                   └──→ Real Jira API
                       (via reqwest)
```

## TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `L` | Open login modal to enter Jira credentials |
| `C` | Edit configuration (URL, username, password) |
| `Q` / `Esc` | Exit application |
| `Tab` | Navigate between login fields |
| `Enter` | Confirm login or settings |
| `Backspace` | Delete character in input field |

## Jira API Integration

### Real Jira Connection

The application now makes **actual HTTP requests** to your Jira instance:

```rust
// Jira REST API v3 endpoint
GET https://your-domain.atlassian.net/rest/api/3/issue/{KEY}
Authorization: Basic base64(username:password)
```

### Supported Response Fields

Each Jira issue returns:
- `key` - Issue key (e.g., "PROJ-123")
- `id` - Internal issue ID
- `fields.summary` - Issue title
- `fields.description` - Detailed description
- `fields.status` - Current status (Open, In Progress, Done, etc.)
- `fields.priority` - Priority level
- `fields.assignee` - Assigned user
- `fields.created` - Creation timestamp
- `fields.updated` - Last update timestamp

### Example Response

```json
{
  "key": "PROJ-123",
  "id": "10000",
  "fields": {
    "summary": "Implement user authentication",
    "description": "Add login system with JWT tokens",
    "status": { "name": "In Progress", "id": "3" },
    "priority": { "name": "High", "id": "2" },
    "assignee": { "name": "john.doe", "displayName": "John Doe" },
    "created": "2024-01-15T10:30:00Z",
    "updated": "2024-01-20T14:45:30Z"
  }
}
```

## Claude Integration

### Using the Jira Skill

The skill is pre-configured in `.claude/skills/jira/`. In Claude conversations:

```
Check jira ticket PROJ-123
```

Claude will automatically:
1. Call the local MCP server at `localhost:3030`
2. Fetch the live Jira issue data
3. Enrich the conversation with ticket details
4. You see the call logged in real-time in the TUI

### Configuration

`.claude/settings.local.json` pre-approves:
- `curl http://localhost:3030/health`
- `curl http://localhost:3030/jira/*`
- `WebFetch(domain:localhost)`

No permission prompts needed!

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

Tests cover:
- ✅ Configuration loading
- ✅ Call logging and limits
- ✅ UI event handling
- ✅ Jira client initialization
- ✅ Server state management

## Development

### Code Quality

- Type-safe Rust with strong error handling
- Thread-safe shared state with `Arc<Mutex<T>>`
- Async/await with Tokio runtime
- Comprehensive unit tests
- Structured logging with `tracing`

### Building from Source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run with debug logging
RUST_LOG=jira_mcp=debug cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint with Clippy
cargo clippy -- -D warnings
```

## Performance

- **Memory**: ~500 bytes per logged call, max 100 calls = ~50KB
- **CPU**: Server is event-driven (zero CPU when idle), TUI refreshes every 500ms
- **Concurrency**: Tokio handles multiple simultaneous requests
- **Response Time**: ~500ms-2s depending on Jira network latency

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `axum` | Web framework |
| `reqwest` | HTTP client for Jira API |
| `ratatui` | Terminal UI |
| `crossterm` | Terminal handling |
| `serde/serde_json` | JSON serialization |
| `chrono` | Timestamps |
| `uuid` | Unique call IDs |
| `parking_lot` | Efficient mutex |
| `base64` | Basic auth encoding |
| `dotenvy` | .env file loading |
| `tracing` | Structured logging |

## Troubleshooting

### "Unauthorized" Errors (401)
- Check your `JIRA_USERNAME` and `JIRA_PASSWORD`
- For Jira Cloud, use an [API token](https://id.atlassian.com/manage-profile/security/api-tokens), not your password
- Verify credentials with: `echo "Basic $(echo -n 'user:pass' | base64)"`

### "Not Found" Errors (404)
- Verify the issue key exists in your Jira instance
- Check the Jira URL (should end with `.atlassian.net` for Cloud)
- Confirm permissions to view the issue

### "Access Forbidden" (403)
- Your account doesn't have permission to view this issue
- Contact your Jira administrator

### TUI Not Displaying
- Ensure terminal supports UTF-8
- Try exporting: `export LANG=en_US.UTF-8`
- Check `RUST_LOG=jira_mcp=debug cargo run` for errors

### Connection Refused
- Confirm Jira URL is correct and accessible
- Check network connectivity
- Verify Jira instance is running

## Future Enhancements

- [ ] Multiple Jira instance support
- [ ] OAuth2 authentication
- [ ] Call history export (CSV/JSON)
- [ ] Performance metrics (response times, success rates)
- [ ] WebSocket for real-time updates
- [ ] Database persistence
- [ ] GraphQL API support
- [ ] Advanced search and filtering in TUI

## License

MIT License - Feel free to use and modify!

## Contributing

Issues and PRs welcome. Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- No clippy warnings (`cargo clippy`)

## Resources

- [Jira Cloud REST API Documentation](https://developer.atlassian.com/cloud/jira/rest/v3/)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [Claude AI Capabilities](https://claude.ai)
- [Axum Web Framework](https://github.com/tokio-rs/axum)
- [Ratatui TUI Library](https://ratatui.rs/)