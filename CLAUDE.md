# Jira MCP - Codebase Documentation

## Overview

This is a Rust application that combines:
- **Axum** web server for handling MCP (Model Context Protocol) calls to Jira
- **Ratatui** TUI for real-time visualization of call logs
- **Tokio** async runtime coordinating both components

The application demonstrates integrating a web API with a live terminal UI, useful for monitoring and debugging API interactions.

### Authentication Methods

The application supports two authentication methods for connecting to Jira:
1. **Username/Password** - Traditional Basic Authentication (base64 encoded credentials)
2. **API Key** - Bearer Token Authentication (requires email + API token)

Users can choose their preferred authentication method when logging in.

## Configuration

### Environment Variables

The application loads configuration from environment variables (via `.env` file or system env):

#### Required Variables
- `JIRA_URL` - Jira instance URL (default: `http://localhost:8080`)
  - Example: `https://your-domain.atlassian.net`

#### Authentication Variables (choose one method)

**Username/Password Method**:
- `JIRA_USERNAME` - Your Jira username or email
- `JIRA_PASSWORD` - Your Jira password

**API Key Method**:
- `JIRA_USERNAME` - Your Jira email address
- `JIRA_API_KEY` - Your Jira API token (takes precedence if both are set)

#### Optional Variables
- `SERVER_PORT` - Web server port (default: `3030`)

### Example `.env` File

**Using Username/Password**:
```env
JIRA_URL=https://your-domain.atlassian.net
JIRA_USERNAME=user@example.com
JIRA_PASSWORD=mypassword
SERVER_PORT=3030
```

**Using API Key**:
```env
JIRA_URL=https://your-domain.atlassian.net
JIRA_USERNAME=user@example.com
JIRA_API_KEY=atatt1234567890abcdef
SERVER_PORT=3030
```

## Architecture

### Threading Model

The application uses Tokio's async runtime with:
1. **Server Task**: Runs Axum web server in a background task on port 3030
2. **TUI Task**: Runs Ratatui UI in the main task with event loop
3. **Shared State**: `Arc<Mutex<CallLog>>` for thread-safe communication

```
main()
├─ calloc_log: Arc<Mutex<CallLog>>
├─ spawn server_handle (background)
│  └─ /jira/{id} calls write to call_log
└─ run_tui (foreground)
   └─ reads from call_log every 500ms
```

### Key Components

#### 1. Server (`server.rs`)

**Purpose**: RESTful API server using Axum framework

**Endpoints**:
- `GET /health` - Returns server status
- `GET /jira/{id}` - Fetches Jira issue and logs the call

**Request Flow**:
```
GET /jira/TEST-1
    ↓
Axum Router matches route
    ↓
get_jira_issue(Path(id), State(state))
    ↓
JiraClient::get_issue(&id) [mock or real]
    ↓
Log call: Call::new(method, path, status, response)
    ↓
HTTP response + stored in CallLog
```

**State Management**:
- `AppState` contains:
  - `jira_client: JiraClient` - Client for fetching issues
  - `call_log: Arc<Mutex<CallLog>>` - Shared call tracking

#### 2. UI (`ui.rs`)

**Purpose**: Ratatui TUI for monitoring API calls and managing authentication

**Main Screen Layout**:
```
┌─────────────────────────────────────┐
│ 📊 Jira MCP Call Tracker            │
├─────────────────────────────────────┤
│ 🟢 Status: Logged in | (L)ogin • (C)onfigure
├─────────────────────────────────────┤
│ Timestamp    | Method | Path | Code │
│ 14:23:45.123 | GET    | /jira/TEST-1 | 200 │
│ 14:23:44.567 | GET    | /health     | 200 │
├─────────────────────────────────────┤
│ (Q)uit • (L)ogin • (C)onfigure      │
└─────────────────────────────────────┘
```

**Authentication Selection Modal** (shown on first login or when (L)ogin is pressed):
```
┌─────────────────────────────────────┐
│  🔐 Choose Authentication Method    │
├─────────────────────────────────────┤
│ ┌────────────────────────────────┐  │
│ │ 👤 Username / Password         │  │ ← Selected
│ │   (Basic Authentication)       │  │
│ └────────────────────────────────┘  │
│ ┌────────────────────────────────┐  │
│ │ 🔑 API Key                     │  │
│ │   (Bearer Token)               │  │
│ └────────────────────────────────┘  │
├─────────────────────────────────────┤
│ ↑↓ select • Enter continue • Esc    │
└─────────────────────────────────────┘
```

**Login Modal** (shown after selecting auth method):
- Displays appropriate labels based on selected auth method
- Username/Password mode: Shows "Username" and "Password" fields
- API Key mode: Shows "Email" and "API Key" fields

**Event Loop**:
- Polls for keyboard events every 500ms
- Redraws UI with latest call_log data
- Handles keyboard input for modals and main screen

**Keyboard Shortcuts**:
- `Q` - Quit application
- `L` - Open login/auth modal
- `C` - Configure credentials (edit mode)
- `Tab` - Switch between fields in modal
- `↑↓` - Navigate options in auth method selection
- `Enter` - Confirm and move to next field
- `Esc` - Cancel modal

**Color Coding**:
- Status 2xx: Green
- Status 3xx: Yellow
- Status 4xx+: Red
- Active field/option: Yellow highlight

#### 3. Configuration (`config.rs`)

**Purpose**: Manage Jira connection settings and authentication

**Data Structures**:

```rust
pub enum AuthMethod {
    UserPassword,  // Basic auth: base64(username:password)
    ApiKey,        // Bearer auth: Bearer {api_token}
}

pub struct JiraConfig {
    pub url: String,              // Jira instance URL
    pub username: String,         // Username or email
    pub password: String,         // Password or API key
    pub auth_method: AuthMethod,  // Authentication method
}

pub type SharedConfig = Arc<Mutex<JiraConfig>>;
```

**Key Methods**:
- `from_env()` - Load configuration from environment variables
- `new()` - Create configuration with specified values
- `is_configured()` - Check if all required fields are set
- `display_summary()` - Get masked display of configuration

**Environment Variable Detection**:
- If `JIRA_API_KEY` is set → Uses `AuthMethod::ApiKey`
- If only `JIRA_USERNAME` and `JIRA_PASSWORD` → Uses `AuthMethod::UserPassword`

#### 4. State (`state.rs`)

**Purpose**: Thread-safe call logging with bounded size

**Data Structures**:

```rust
pub struct Call {
    id: String,              // UUID for each call
    method: String,          // HTTP method
    path: String,            // Request path
    status_code: u16,        // HTTP status
    timestamp: DateTime,     // When called
    response: String,        // Response body
}

pub struct CallLog {
    calls: Vec<Call>,        // LIFO stack (newest first)
    max_calls: usize,        // Default: 100
}
```

**Invariants**:
- `calls` maintains at most `max_calls` entries
- Newest calls are inserted at index 0
- Thread-safe via `Arc<Mutex<_>>`

#### 4. Jira Client (`jira.rs`)

**Purpose**: Client for connecting to Jira instances with flexible authentication

**Supported Authentication Methods**:
- **User/Password**: Uses HTTP Basic Authentication (base64 encoded `username:password`)
- **API Key**: Uses Bearer Token Authentication (Jira API token)

**Data Structures**:

```rust
pub struct JiraClient {
    base_url: String,
    username: String,           // Username or email (for API key)
    credential: String,         // Password or API token
    auth_method: AuthMethod,    // UserPassword or ApiKey
    client: Client,
}

pub enum AuthMethod {
    UserPassword,  // Basic auth with username/password
    ApiKey,        // Bearer token with email + API key
}
```

**Constructor Examples**:

```rust
// Username/Password authentication
let client = JiraClient::with_user_password(
    "https://your-domain.atlassian.net".to_string(),
    "user@example.com".to_string(),
    "mypassword".to_string(),
);

// API Key authentication
let client = JiraClient::with_api_key(
    "https://your-domain.atlassian.net".to_string(),
    "user@example.com".to_string(),
    "atatt1234567890...".to_string(),
);

// Generic constructor
let client = JiraClient::new(
    base_url,
    username_or_email,
    password_or_api_key,
    auth_method,
);
```

## Data Flow Example

User calls: `curl http://localhost:3030/jira/TEST-1`

```
HTTP Request arrives at Axum router
    ↓
get_jira_issue(Path("TEST-1"), State(app_state))
    ↓
state.jira_client.get_issue("TEST-1")
    ↓
Returns JiraIssue (mock or real)
    ↓
Serialize to JSON: response_json
    ↓
Lock mutex: state.call_log.lock()
    ↓
Create Call { method: "GET", path: "/jira/TEST-1", status_code: 200, response: response_json, timestamp: now }
    ↓
Add to log: log.add_call(call)
    ↓
Return Call (LIFO - newest first)
    ↓
Unlock mutex
    ↓
TUI polls every 500ms
    ↓
Reads call_log.lock().get_calls()
    ↓
Renders table with new call visible
    ↓
Returns HTTP 200 + JSON to client
```

## Testing Strategy

### Unit Tests

**Module**: `state.rs`
- `test_call_creation()` - Verify Call struct initialization
- `test_call_log_add_and_retrieve()` - Add and retrieve calls
- `test_call_log_max_size()` - Verify size limits work

**Module**: `ui.rs`
- `test_should_quit_with_q()` - Quit on 'q'
- `test_should_quit_with_esc()` - Quit on Esc
- `test_should_not_quit_with_other_key()` - No quit on other keys

**Module**: `jira.rs`
- `test_jira_client_get_issue()` - Mock client returns valid issue

**Module**: `server.rs`
- `test_app_state_creation()` - Verify state initialization

## Error Handling

**Server Errors**:
- Jira client failures → 500 Internal Server Error
- Logged to call_log with error message as response
- Logged to stderr via tracing

**TUI Errors**:
- Terminal setup failures → Early return with error
- Terminal properly restored even on error

**Logging**:
- Uses `tracing` crate for structured logging
- Environment: Set `RUST_LOG=jira_mcp=debug` to see detailed logs

## Extension Points

### Adding New Endpoints

1. Create handler function in `server.rs`:
```rust
async fn new_endpoint(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Implementation
}
```

2. Register in router:
```rust
.route("/new-path/:id", get(new_endpoint))
```

3. Call logging happens automatically via `state.call_log`

### Using Different Authentication Methods

The Jira client supports two authentication methods. Choose based on your Jira setup:

**Option 1: Username/Password (Basic Auth)**
```rust
let client = JiraClient::with_user_password(
    "https://your-domain.atlassian.net".to_string(),
    "user@example.com".to_string(),
    "mypassword".to_string(),
);
```

Environment variables:
```env
JIRA_URL=https://your-domain.atlassian.net
JIRA_USERNAME=user@example.com
JIRA_PASSWORD=mypassword
```

**Option 2: API Key (Bearer Token)**
```rust
let client = JiraClient::with_api_key(
    "https://your-domain.atlassian.net".to_string(),
    "user@example.com".to_string(),
    "atatt1234567890...".to_string(),
);
```

Environment variables:
```env
JIRA_URL=https://your-domain.atlassian.net
JIRA_USERNAME=user@example.com
JIRA_API_KEY=atatt1234567890...
```

**How API Keys Work with Atlassian Cloud**:
1. Generate API token at https://id.atlassian.com/manage/api-tokens
2. The system will automatically use Bearer token auth when `JIRA_API_KEY` is set
3. API tokens are preferred for cloud-hosted Jira instances for security

**How Passwords Work with Server/Data Center**:
1. Use your actual Jira password or an app password
2. Basic auth with username/password is common for on-premise Jira
3. Set only `JIRA_USERNAME` and `JIRA_PASSWORD` in environment

### Adding UI Features

In `ui.rs`:
- Add new chunks in `draw_ui()` for additional panels
- Implement filtering/search logic
- Add keyboard shortcuts for different views

## Performance Notes

**Memory**:
- Call log limited to 100 entries (configurable)
- Each Call ~500 bytes (varies with response size)
- Maximum ~50KB for full log

**CPU**:
- TUI redraws every 500ms (configurable)
- Server is event-driven, zero CPU when idle
- No blocking operations

**Thread Safety**:
- `Arc<Mutex<CallLog>>` ensures safe concurrent access
- Mutex held briefly (only during add/read)
- No deadlock risk (single lock)

## Building and Running

```bash
# Development build
cargo build

# Release (optimized)
cargo build --release

# Run
cargo run

# Run with logging
RUST_LOG=jira_mcp=debug cargo run

# Tests
cargo test

# Format check
cargo fmt --check

# Linting
cargo clippy -- -D warnings
```

## Known Limitations

1. **Jira Client**: Currently mocked, not connecting to real Jira
2. **Call Limit**: Fixed at 100 entries (no persistence)
3. **TUI**: Basic implementation, no scrolling/filtering
4. **Authentication**: No Jira auth implemented
5. **Error Recovery**: Server abort on TUI exit (could improve graceful shutdown)

## Recommended Next Steps

1. **Real Jira Integration**: Connect to actual Jira instance
2. **Authentication**: Add API token/OAuth support
3. **Persistence**: Save calls to file or database
4. **TUI Enhancements**: Add filtering, sorting, search
5. **Metrics**: Track response times, success rates
6. **Configuration**: TOML config file for server port, Jira URL, etc.
7. **WebSocket**: Real-time updates instead of polling
8. **Error Recovery**: Graceful shutdown of server on TUI exit
