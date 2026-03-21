use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, AppMode, StatusMessage};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    // Main vertical layout: title bar (1), main area (fill), help+status (3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Title bar
    let title = Paragraph::new("layang")
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Main area
    match &app.mode {
        AppMode::Loading(msg) => {
            render_post_table(f, app, chunks[1]);
            // Overlay loading message centered
            let loading_msg = msg.clone();
            let popup_area = centered_rect(40, 3, chunks[1]);
            f.render_widget(Clear, popup_area);
            let loading = Paragraph::new(loading_msg)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(loading, popup_area);
        }
        AppMode::PostList => {
            render_post_table(f, app, chunks[1]);
        }
        AppMode::Form => {
            render_form(f, app, chunks[1]);
        }
        AppMode::ConfirmDelete => {
            render_post_table(f, app, chunks[1]);
            render_confirm_delete(f, app, chunks[1]);
        }
    }

    // Help + status bar
    render_help_status(f, app, chunks[2]);
}

fn render_post_table(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Title", "Slug", "Status", "Date"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let rows: Vec<Row> = app
        .posts
        .iter()
        .map(|post| {
            let is_published = post.published_at.is_some();
            let status_text = if is_published { "Published" } else { "Draft" };
            let status_style = if is_published {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            };

            let date = post
                .published_at
                .as_deref()
                .or(post.created_at.as_deref())
                .unwrap_or("")
                .chars()
                .take(10)
                .collect::<String>();

            Row::new(vec![
                Cell::from(post.title.clone()),
                Cell::from(post.slug.clone()),
                Cell::from(status_text).style(status_style),
                Cell::from(date),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(30),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Posts"))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(table, area, &mut app.list_state.clone());
}

fn render_form(f: &mut Frame, app: &App, area: Rect) {
    let title = if app.form.is_edit {
        "Edit Post"
    } else {
        "Create Post"
    };

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Layout inside form: title, excerpt, body info, help
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title field
            Constraint::Length(3), // excerpt field
            Constraint::Length(1), // body info
            Constraint::Length(1), // keybind help
            Constraint::Min(0),    // padding
        ])
        .split(inner);

    // Title field
    let title_focused = app.form.active_field == 0;
    let title_line = render_text_input(app.form.title.chars(), app.form.title.cursor, title_focused);
    let title_block = Block::default()
        .borders(Borders::ALL)
        .title("Title")
        .border_style(if title_focused { Style::default().fg(Color::Cyan) } else { Style::default() });
    f.render_widget(Paragraph::new(title_line).block(title_block), form_chunks[0]);

    // Excerpt field
    let excerpt_focused = app.form.active_field == 1;
    let excerpt_hint = if app.form.is_edit { "Excerpt" } else { "Excerpt (leave blank to auto-generate)" };
    let excerpt_line = render_text_input(app.form.excerpt.chars(), app.form.excerpt.cursor, excerpt_focused);
    let excerpt_block = Block::default()
        .borders(Borders::ALL)
        .title(excerpt_hint)
        .border_style(if excerpt_focused { Style::default().fg(Color::Cyan) } else { Style::default() });
    f.render_widget(Paragraph::new(excerpt_line).block(excerpt_block), form_chunks[1]);

    // Body info line
    let body_info = Paragraph::new("Body: will open in external editor on submit  |  Slug: auto-generated")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(body_info, form_chunks[2]);

    // Keybind help
    let help = Paragraph::new("Tab/↓: next field   Shift+Tab/↑: prev   Enter: open editor   Esc: cancel")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, form_chunks[3]);
}

fn render_confirm_delete(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(50, 7, area);
    f.render_widget(Clear, popup_area);

    let msg = format!(
        "Delete \"{}\"? This cannot be undone.\n\nY: confirm   N/Esc: cancel",
        app.confirm_slug
    );
    let paragraph = Paragraph::new(msg)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Confirm Delete")
                .border_style(Style::default().fg(Color::Red)),
        );
    f.render_widget(paragraph, popup_area);
}

fn render_help_status(f: &mut Frame, app: &App, area: Rect) {
    let help_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2)])
        .split(area);

    // Status line
    if let Some(status) = &app.status {
        let (text, style) = match status {
            StatusMessage::Error(msg) => (msg.as_str(), Style::default().fg(Color::Red)),
            StatusMessage::Success(msg) => (msg.as_str(), Style::default().fg(Color::Green)),
        };
        let status_para = Paragraph::new(text).style(style);
        f.render_widget(status_para, help_chunks[0]);
    }

    // Help bar
    let help_text = "c:new  e:edit  p:pub  u:unpub  d:del  r:refresh  q:quit";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, help_chunks[1]);
}

pub fn render_text_input(line_chars: &[char], cursor: usize, focused: bool) -> Line<'static> {
    if !focused {
        // When not focused, just render plain text
        let text: String = line_chars.iter().collect();
        return Line::from(text);
    }

    let mut spans: Vec<Span> = Vec::new();

    // Characters before cursor
    if cursor > 0 {
        let before: String = line_chars[..cursor].iter().collect();
        spans.push(Span::raw(before));
    }

    // Character at cursor (reversed) or space if at end
    if cursor < line_chars.len() {
        let at_cursor: String = line_chars[cursor..cursor + 1].iter().collect();
        spans.push(Span::styled(
            at_cursor,
            Style::default().add_modifier(Modifier::REVERSED),
        ));
        // Characters after cursor
        if cursor + 1 < line_chars.len() {
            let after: String = line_chars[cursor + 1..].iter().collect();
            spans.push(Span::raw(after));
        }
    } else {
        // Cursor is at the end — show a space REVERSED
        spans.push(Span::styled(
            " ",
            Style::default().add_modifier(Modifier::REVERSED),
        ));
    }

    Line::from(spans)
}

/// Returns a centered rect of fixed height and percentage width within `r`
fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_width = r.width * percent_x / 100;
    let x = r.x + (r.width.saturating_sub(popup_width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width: popup_width.min(r.width),
        height: height.min(r.height),
    }
}
