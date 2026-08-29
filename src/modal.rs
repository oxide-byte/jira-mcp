use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Application modal/state for showing login or config popup
#[derive(Debug, Clone, PartialEq)]
pub enum ModalState {
    None,
    AuthMethodSelection,
    Login,
    ConfigEdit { field: ConfigField, buffer: String },
}

/// Configuration field being edited
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigField {
    Url,
    Username,
    Password,
}

impl ConfigField {
    pub fn label(&self) -> &'static str {
        match self {
            ConfigField::Url => "Jira URL",
            ConfigField::Username => "Username",
            ConfigField::Password => "Password",
        }
    }

    #[allow(dead_code)]
    pub fn next(&self) -> ConfigField {
        match self {
            ConfigField::Url => ConfigField::Username,
            ConfigField::Username => ConfigField::Password,
            ConfigField::Password => ConfigField::Url,
        }
    }
}

/// Modal controller
pub struct Modal {
    pub state: ModalState,
}

impl Modal {
    pub fn new() -> Self {
        Self {
            state: ModalState::None,
        }
    }

    pub fn show_auth_method_selection(&mut self) {
        self.state = ModalState::AuthMethodSelection;
    }

    pub fn show_login(&mut self) {
        self.state = ModalState::Login;
    }

    pub fn start_edit_url(&mut self, initial_value: String) {
        self.state = ModalState::ConfigEdit {
            field: ConfigField::Url,
            buffer: initial_value,
        };
    }

    pub fn start_edit_username(&mut self, initial_value: String) {
        self.state = ModalState::ConfigEdit {
            field: ConfigField::Username,
            buffer: initial_value,
        };
    }

    pub fn start_edit_password(&mut self, initial_value: String) {
        self.state = ModalState::ConfigEdit {
            field: ConfigField::Password,
            buffer: initial_value,
        };
    }

    pub fn close(&mut self) {
        self.state = ModalState::None;
    }

    pub fn is_visible(&self) -> bool {
        self.state != ModalState::None
    }
}

impl Default for Modal {
    fn default() -> Self {
        Self::new()
    }
}

/// Draws a centered modal popup
#[allow(dead_code)]
pub fn draw_login_modal(f: &mut Frame, area: Rect) {
    // Create a centered popup (80% of screen width, 60% of height)
    let popup_width = (area.width as f32 * 0.8) as u16;
    let popup_height = (area.height as f32 * 0.6) as u16;

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
    let block = Block::default()
        .title(" 🔐 Jira Login ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Create layout for modal content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(2),
        ])
        .margin(1)
        .split(inner);

    // URL input
    let url_label = Span::styled(
        "URL: ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(Paragraph::new(Line::from(vec![url_label])), chunks[0]);

    // Username input
    let username_label = Span::styled(
        "Username: ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(Paragraph::new(Line::from(vec![username_label])), chunks[2]);

    // Password input
    let password_label = Span::styled(
        "Password: ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(Paragraph::new(Line::from(vec![password_label])), chunks[4]);

    // Instructions
    let instructions = vec![
        Line::from(Span::raw("Enter your Jira credentials to proceed")),
        Line::from(""),
        Line::from(vec![
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
        ]),
    ];

    f.render_widget(
        Paragraph::new(instructions).style(Style::default().fg(Color::Gray)),
        chunks[5],
    );
}

/// Draws a config edit modal
pub fn draw_config_edit_modal(f: &mut Frame, area: Rect, field: &ConfigField, buffer: &str) {
    // Create a centered popup
    let popup_width = (area.width as f32 * 0.75) as u16;
    let popup_height = (area.height as f32 * 0.5) as u16;

    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x,
        y,
        width: popup_width,
        height: popup_height,
    };

    // Clear the area
    f.render_widget(Clear, popup_area);

    // Create modal block
    let block = Block::default()
        .title(format!(" Edit {} ", field.label()))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Layout for content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(2),
            Constraint::Length(2),
        ])
        .margin(1)
        .split(inner);

    // Label
    let label = Span::styled(
        format!("{}: ", field.label()),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(Paragraph::new(Line::from(vec![label])), chunks[0]);

    // Input field with border
    let input_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::Black));

    let display_buffer = if field == &ConfigField::Password {
        "*".repeat(buffer.len())
    } else {
        buffer.to_string()
    };

    let input = Paragraph::new(display_buffer)
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(input, chunks[1]);

    // Instructions
    let instructions = vec![
        Line::from("Type to edit this field"),
        Line::from(vec![
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
        ]),
    ];

    f.render_widget(
        Paragraph::new(instructions)
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: true }),
        chunks[3],
    );
}

/// Draws the authentication method selection modal
pub fn draw_auth_method_selection_modal(f: &mut Frame, area: Rect, selected: u8) {
    // Create a centered popup
    let popup_width = (area.width as f32 * 0.7) as u16;
    let popup_height = (area.height as f32 * 0.5) as u16;

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
    let block = Block::default()
        .title(" 🔐 Choose Authentication Method ")
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
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .margin(1)
        .split(inner);

    // Option 1: Username/Password
    let option1_style = if selected == 0 {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White).bg(Color::Black)
    };

    let option1_text = vec![
        Line::from(vec![
            Span::styled("👤 ", Style::default().fg(Color::Cyan)),
            Span::raw("Username / Password"),
        ]),
        Line::from(vec![Span::styled(
            "  (Basic Authentication)",
            Style::default().fg(Color::Gray),
        )]),
    ];
    let option1_block = Block::default().borders(Borders::ALL).style(option1_style);
    f.render_widget(Paragraph::new(option1_text).block(option1_block), chunks[1]);

    // Option 2: API Key
    let option2_style = if selected == 1 {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White).bg(Color::Black)
    };

    let option2_text = vec![
        Line::from(vec![
            Span::styled("🔑 ", Style::default().fg(Color::Cyan)),
            Span::raw("API Key"),
        ]),
        Line::from(vec![Span::styled(
            "  (Bearer Token)",
            Style::default().fg(Color::Gray),
        )]),
    ];
    let option2_block = Block::default().borders(Borders::ALL).style(option2_style);
    f.render_widget(Paragraph::new(option2_text).block(option2_block), chunks[3]);

    // Instructions
    let instructions = vec![Line::from(vec![
        Span::styled(
            "↑↓",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to select • "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to continue • "),
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
        chunks[5],
    );
}
