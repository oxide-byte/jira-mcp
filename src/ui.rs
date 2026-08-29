use crate::config::{AuthMethod, SharedConfig};
use crate::logs::SharedLogCollector;
use crate::modal::{
    draw_auth_method_selection_modal, draw_config_edit_modal, ConfigField, Modal, ModalState,
};
use crate::state::CallLog;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use parking_lot::Mutex;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Application state for the TUI
struct AppState {
    modal: Modal,
    login_url: String,
    login_username: String,
    login_password: String,
    login_field: LoginField,  // Track which field is being edited
    selected_auth_method: u8, // 0 = UserPassword, 1 = ApiKey
    auth_method: AuthMethod,  // Currently selected auth method
}

/// Which field is being edited in the login modal
#[derive(Debug, Clone, PartialEq)]
enum LoginField {
    Url,
    Username,
    Password,
}

impl AppState {
    fn new() -> Self {
        Self {
            modal: Modal::new(),
            login_url: String::new(),
            login_username: String::new(),
            login_password: String::new(),
            login_field: LoginField::Url,
            selected_auth_method: 0, // Default to UserPassword
            auth_method: AuthMethod::UserPassword,
        }
    }

    fn show_auth_method_selection(&mut self) {
        self.selected_auth_method = 0; // Reset selection
        self.modal.show_auth_method_selection();
    }

    fn show_login(&mut self, config: &crate::config::JiraConfig) {
        self.login_url = config.url.clone();
        self.login_username = config.username.clone();
        self.login_password = config.password.clone();
        self.login_field = LoginField::Url;
        self.auth_method = config.auth_method;
        self.selected_auth_method = match config.auth_method {
            AuthMethod::UserPassword => 0,
            AuthMethod::ApiKey => 1,
        };
        self.modal.show_login();
    }
}

/// Runs the Ratatui TUI application.
///
/// # Arguments
///
/// * `call_log` - Shared call log to display in the UI
/// * `config` - Shared Jira configuration
/// * `log_collector` - Shared log collector for application logs
///
/// # Errors
///
/// Returns an error if terminal setup fails or an unrecoverable event occurs.
pub async fn run_tui(
    call_log: Arc<Mutex<CallLog>>,
    config: SharedConfig,
    log_collector: SharedLogCollector,
) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Run the app
    let res = run_app(terminal, call_log, config, log_collector).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;

    if let Err(err) = res {
        error!("TUI error: {}", err);
        return Err(err);
    }

    Ok(())
}

/// Main application loop for the TUI.
async fn run_app(
    mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    call_log: Arc<Mutex<CallLog>>,
    config: SharedConfig,
    log_collector: SharedLogCollector,
) -> Result<()> {
    let mut app_state = AppState::new();

    // Show login if not configured
    let cfg = config.lock();
    if !cfg.is_configured() {
        app_state.show_login(&cfg);
    }
    drop(cfg);

    loop {
        // Render the UI
        let log = call_log.lock();
        let calls = log.get_calls();
        let cfg = config.lock();
        let logs = log_collector.lock();
        let log_entries = logs.get_entries();

        terminal.draw(|f| {
            draw_ui(f, &calls, &log_entries, &app_state, &cfg);
        })?;
        drop(cfg);
        drop(log);
        drop(logs);

        // Handle events with a timeout so we can refresh the display
        if crossterm::event::poll(std::time::Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                if app_state.modal.is_visible() {
                    if !handle_modal_input(key, &mut app_state, &config) {
                        app_state.modal.close();
                    }
                } else if should_quit(key) {
                    return Ok(());
                } else {
                    handle_key_input(key, &mut app_state, &config);
                }
            }
        }
    }
}

/// Draws the entire UI.
fn draw_ui(
    f: &mut Frame,
    calls: &[crate::state::Call],
    log_entries: &[crate::logs::LogEntry],
    app_state: &AppState,
    config: &crate::config::JiraConfig,
) {
    let size = f.area();

    // Create main layout: header, call log, logs, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
            Constraint::Length(3),
        ])
        .split(size);

    // Draw header with login status
    draw_header(f, chunks[0], config);

    // Draw call log table
    draw_call_log(f, chunks[1], calls);

    // Draw application logs panel
    draw_logs_panel(f, chunks[2], log_entries);

    // Draw footer
    draw_footer(f, chunks[3]);

    // Draw modal if visible
    if app_state.modal.is_visible() {
        match &app_state.modal.state {
            ModalState::AuthMethodSelection => {
                draw_auth_method_selection_modal(f, size, app_state.selected_auth_method);
            }
            ModalState::Login => draw_login_modal_with_data(
                f,
                size,
                &app_state.login_url,
                &app_state.login_username,
                &app_state.login_password,
                &app_state.login_field,
                &app_state.auth_method,
            ),
            ModalState::ConfigEdit { field, buffer } => {
                draw_config_edit_modal(f, size, field, buffer);
            }
            ModalState::None => {}
        }
    }
}

/// Draws the header section with title and status.
fn draw_header(f: &mut Frame, area: Rect, config: &crate::config::JiraConfig) {
    let status_icon = if config.is_configured() {
        "🟢"
    } else {
        "🔴"
    };

    let status_text = if config.is_configured() {
        "Logged in"
    } else {
        "Not logged in"
    };

    let header_text = vec![
        Line::from(vec![Span::styled(
            "📊 Jira MCP Call Tracker",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::raw(format!("{} Status: {} | ", status_icon, status_text)),
            Span::styled("(L)ogin", Style::default().fg(Color::Yellow)),
            Span::raw(" • "),
            Span::styled("(C)onfigure", Style::default().fg(Color::Yellow)),
        ]),
    ];

    let header = Paragraph::new(header_text).block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, area);
}

/// Draws the call log table.
fn draw_call_log(f: &mut Frame, area: Rect, calls: &[crate::state::Call]) {
    let rows: Vec<Row> = calls
        .iter()
        .map(|call| {
            let status_color = if call.status_code >= 400 {
                Color::Red
            } else if call.status_code >= 300 {
                Color::Yellow
            } else {
                Color::Green
            };

            Row::new(vec![
                call.timestamp.format("%H:%M:%S%.3f").to_string(),
                call.method.clone(),
                call.path.clone(),
                Span::styled(
                    call.status_code.to_string(),
                    Style::default().fg(status_color),
                )
                .to_string(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(6),
            Constraint::Min(20),
            Constraint::Length(4),
        ],
    )
    .header(
        Row::new(vec!["Timestamp", "Method", "Path", "Code"])
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .title(format!("MCP Calls ({})", calls.len()))
            .borders(Borders::ALL),
    );

    f.render_widget(table, area);
}

/// Draws the logs panel showing application logs and errors.
fn draw_logs_panel(f: &mut Frame, area: Rect, log_entries: &[crate::logs::LogEntry]) {
    let rows: Vec<Row> = log_entries
        .iter()
        .map(|entry| {
            let level_color = match entry.level.as_str() {
                "ERROR" => Color::Red,
                "WARN" => Color::Yellow,
                "INFO" => Color::Cyan,
                "DEBUG" => Color::Gray,
                _ => Color::White,
            };

            Row::new(vec![
                entry.timestamp.format("%H:%M:%S%.3f").to_string(),
                Span::styled(
                    entry.level.clone(),
                    Style::default().fg(level_color),
                )
                .to_string(),
                entry.message.clone(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(7),
            Constraint::Min(40),
        ],
    )
    .header(
        Row::new(vec!["Timestamp", "Level", "Message"])
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .title(format!("Logs ({})", log_entries.len()))
            .borders(Borders::ALL),
    );

    f.render_widget(table, area);
}

/// Draws the footer section with help text.
fn draw_footer(f: &mut Frame, area: Rect) {
    let footer_text = vec![Line::from(vec![
        Span::styled("(Q)uit", Style::default().fg(Color::Yellow)),
        Span::raw(" • "),
        Span::styled("(L)ogin", Style::default().fg(Color::Yellow)),
        Span::raw(" • "),
        Span::styled("(C)onfigure", Style::default().fg(Color::Yellow)),
    ])];

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::TOP))
        .alignment(Alignment::Center);
    f.render_widget(footer, area);
}

/// Draws the login modal with actual field values
fn draw_login_modal_with_data(
    f: &mut Frame,
    area: Rect,
    url: &str,
    username: &str,
    password: &str,
    active_field: &LoginField,
    auth_method: &AuthMethod,
) {
    use ratatui::widgets::Clear;

    // Create a centered popup
    let popup_width = (area.width as f32 * 0.8) as u16;
    let popup_height = (area.height as f32 * 0.65) as u16;

    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x,
        y,
        width: popup_width,
        height: popup_height,
    };

    // Clear the area behind the modal
    f.render_widget(Clear, popup_area);

    // Create modal content
    let title = match auth_method {
        AuthMethod::UserPassword => " 🔐 Jira Login (Username/Password) ",
        AuthMethod::ApiKey => " 🔐 Jira Login (API Key) ",
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Create layout for modal content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .margin(1)
        .split(inner);

    // URL field
    let url_label = Span::styled(
        "URL: ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(Paragraph::new(Line::from(vec![url_label])), chunks[0]);

    let url_style = if active_field == &LoginField::Url {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White).bg(Color::Black)
    };
    let url_input = Block::default().borders(Borders::ALL).style(url_style);
    f.render_widget(Paragraph::new(url.to_string()).block(url_input), chunks[1]);

    // Username or Email field label
    let username_label_text = match auth_method {
        AuthMethod::UserPassword => "Username: ",
        AuthMethod::ApiKey => "Email: ",
    };
    let username_label = Span::styled(
        username_label_text,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(Paragraph::new(Line::from(vec![username_label])), chunks[2]);

    let username_style = if active_field == &LoginField::Username {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White).bg(Color::Black)
    };
    let username_input = Block::default().borders(Borders::ALL).style(username_style);
    f.render_widget(
        Paragraph::new(username.to_string()).block(username_input),
        chunks[3],
    );

    // Password or API Key field label
    let password_label_text = match auth_method {
        AuthMethod::UserPassword => "Password: ",
        AuthMethod::ApiKey => "API Key: ",
    };
    let password_label = Span::styled(
        password_label_text,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(Paragraph::new(Line::from(vec![password_label])), chunks[4]);

    let password_style = if active_field == &LoginField::Password {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White).bg(Color::Black)
    };
    let password_masked = "*".repeat(password.len());
    let password_input = Block::default().borders(Borders::ALL).style(password_style);
    f.render_widget(
        Paragraph::new(password_masked).block(password_input),
        chunks[5],
    );

    // Instructions
    let instructions = vec![Line::from(vec![
        Span::styled(
            "Tab",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to switch fields • "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to confirm • "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to cancel"),
    ])];

    f.render_widget(
        Paragraph::new(instructions).style(Style::default().fg(Color::Gray)),
        chunks[7],
    );
}

/// Handles keyboard input for the main screen
fn handle_key_input(key: KeyEvent, app_state: &mut AppState, config: &SharedConfig) {
    match key.code {
        KeyCode::Char('l') | KeyCode::Char('L') => {
            // Open auth method selection or login directly if already configured
            info!("Opening login modal");
            let cfg = config.lock();
            if !cfg.is_configured() {
                // Show auth method selection for first-time setup
                app_state.show_auth_method_selection();
            } else {
                // Show login with current auth method
                app_state.show_login(&cfg);
            }
            drop(cfg);
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            // Open config editor - start with URL
            info!("Opening config editor");
            let cfg = config.lock();
            app_state.modal.start_edit_url(cfg.url.clone());
            drop(cfg);
        }
        _ => {}
    }
}

/// Handles keyboard input for modals. Returns true if still editing, false if closed.
fn handle_modal_input(key: KeyEvent, app_state: &mut AppState, config: &SharedConfig) -> bool {
    match key.code {
        KeyCode::Esc => false,
        KeyCode::Enter => {
            // Save or move to next field
            match &app_state.modal.state {
                ModalState::AuthMethodSelection => {
                    // Select auth method and move to login
                    app_state.auth_method = match app_state.selected_auth_method {
                        0 => AuthMethod::UserPassword,
                        1 => AuthMethod::ApiKey,
                        _ => AuthMethod::UserPassword,
                    };
                    app_state.login_field = LoginField::Url;
                    app_state.modal.show_login();
                    true
                }
                ModalState::Login => {
                    // Move to next field or save
                    match app_state.login_field {
                        LoginField::Url => {
                            app_state.login_field = LoginField::Username;
                        }
                        LoginField::Username => {
                            app_state.login_field = LoginField::Password;
                        }
                        LoginField::Password => {
                            // Save login credentials
                            let mut cfg = config.lock();
                            cfg.url = app_state.login_url.clone();
                            cfg.username = app_state.login_username.clone();
                            cfg.password = app_state.login_password.clone();
                            cfg.auth_method = app_state.auth_method;
                            drop(cfg);
                            return false;
                        }
                    }
                    true
                }
                ModalState::ConfigEdit { field, .. } => {
                    // Move to next field or save
                    match field {
                        ConfigField::Url => {
                            let cfg = config.lock();
                            app_state.modal.start_edit_username(cfg.username.clone());
                            drop(cfg);
                            true
                        }
                        ConfigField::Username => {
                            let cfg = config.lock();
                            app_state.modal.start_edit_password(cfg.password.clone());
                            drop(cfg);
                            true
                        }
                        ConfigField::Password => {
                            // Save all credentials
                            if let ModalState::ConfigEdit { buffer, .. } = &app_state.modal.state {
                                let mut cfg = config.lock();
                                cfg.password = buffer.clone();
                                drop(cfg);
                            }
                            false
                        }
                    }
                }
                ModalState::None => false,
            }
        }
        KeyCode::Backspace => {
            if let ModalState::Login = &app_state.modal.state {
                // Handle backspace in login modal
                match app_state.login_field {
                    LoginField::Url => app_state.login_url.pop(),
                    LoginField::Username => app_state.login_username.pop(),
                    LoginField::Password => app_state.login_password.pop(),
                };
            } else if let ModalState::ConfigEdit { buffer, .. } = &mut app_state.modal.state {
                buffer.pop();
            }
            true
        }
        KeyCode::Char(c) => {
            debug!(
                "Character input: '{}', Modal state: {:?}, Active field: {:?}",
                c, app_state.modal.state, app_state.login_field
            );
            if let ModalState::Login = &app_state.modal.state {
                // Handle character input in login modal
                debug!("Adding '{}' to login modal", c);
                match app_state.login_field {
                    LoginField::Url => {
                        app_state.login_url.push(c);
                        debug!("URL field now: {}", app_state.login_url);
                    }
                    LoginField::Username => {
                        app_state.login_username.push(c);
                        debug!("Username field now: {}", app_state.login_username);
                    }
                    LoginField::Password => {
                        app_state.login_password.push(c);
                        debug!(
                            "Password field updated (len: {})",
                            app_state.login_password.len()
                        );
                    }
                }
            } else if let ModalState::ConfigEdit { buffer, field } = &mut app_state.modal.state {
                debug!("Adding '{}' to config editor", c);
                buffer.push(c);
                // Update config as we type
                match field {
                    ConfigField::Url => {
                        app_state.login_url = buffer.clone();
                    }
                    ConfigField::Username => {
                        app_state.login_username = buffer.clone();
                    }
                    ConfigField::Password => {
                        app_state.login_password = buffer.clone();
                    }
                }
            } else {
                debug!("Character input ignored - no modal active");
            }
            true
        }
        KeyCode::Tab => {
            // Move to next field
            if let ModalState::Login = &app_state.modal.state {
                app_state.login_field = match app_state.login_field {
                    LoginField::Url => LoginField::Username,
                    LoginField::Username => LoginField::Password,
                    LoginField::Password => LoginField::Url,
                };
            } else if let ModalState::ConfigEdit { field, .. } = &app_state.modal.state {
                match field {
                    ConfigField::Url => {
                        app_state
                            .modal
                            .start_edit_username(app_state.login_username.clone());
                    }
                    ConfigField::Username => {
                        app_state
                            .modal
                            .start_edit_password(app_state.login_password.clone());
                    }
                    ConfigField::Password => {
                        app_state.modal.start_edit_url(app_state.login_url.clone());
                    }
                }
            }
            true
        }
        KeyCode::Up => {
            // Navigate up in auth method selection
            if let ModalState::AuthMethodSelection = &app_state.modal.state {
                if app_state.selected_auth_method > 0 {
                    app_state.selected_auth_method -= 1;
                }
            }
            true
        }
        KeyCode::Down => {
            // Navigate down in auth method selection
            if let ModalState::AuthMethodSelection = &app_state.modal.state {
                if app_state.selected_auth_method < 1 {
                    app_state.selected_auth_method += 1;
                }
            }
            true
        }
        _ => true,
    }
}

/// Checks if the user wants to quit the application.
fn should_quit(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    #[test]
    fn test_should_quit_with_q() {
        let key = KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(should_quit(key));
    }

    #[test]
    fn test_should_not_quit_with_esc() {
        let key = KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(!should_quit(key));
    }

    #[test]
    fn test_should_not_quit_with_other_key() {
        let key = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(!should_quit(key));
    }
}