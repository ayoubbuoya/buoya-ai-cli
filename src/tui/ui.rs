use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::app::{App, AppMode, Message};

pub fn render_app(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_status_bar(f, app, chunks[0]);
    render_chat_history(f, app, chunks[1]);
    render_input_box(f, app, chunks[2]);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status = if app.is_thinking {
        "Thinking..."
    } else if let Some(tool) = &app.active_tool {
        &tool.name
    } else {
        match app.mode {
            AppMode::Normal => "Normal (press 'i' to input, 'q' to quit)",
            AppMode::Input => "Input (Enter to send, Esc to cancel)",
        }
    };

    let text = Text::from(vec![Line::from(vec![
        Span::styled(
            " buoya-ai-cli ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(status, Style::default().fg(Color::Green)),
    ])]);

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_chat_history(f: &mut Frame, app: &App, area: Rect) {
    let mut text_lines = Vec::new();

    for message in &app.messages {
        match message {
            Message::User { content, .. } => {
                text_lines.push(Line::from(vec![
                    Span::styled(
                        "You: ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(content),
                ]));
            }
            Message::Assistant {
                content,
                tool_calls,
                ..
            } => {
                text_lines.push(Line::from(vec![Span::styled(
                    "Agent: ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )]));

                for line in content.lines() {
                    text_lines.push(Line::from(Span::raw(line)));
                }

                for tool_call in tool_calls {
                    text_lines.push(Line::from(vec![
                        Span::styled("  -> ", Style::default().fg(Color::Yellow)),
                        Span::styled(&tool_call.name, Style::default().fg(Color::Cyan)),
                    ]));
                    if let Some(result) = &tool_call.result {
                        for result_line in result.lines() {
                            text_lines.push(Line::from(Span::raw(format!("    {}", result_line))));
                        }
                    }
                }
            }
            Message::System { content } => {
                text_lines.push(Line::from(vec![
                    Span::styled(
                        "System: ",
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(content),
                ]));
            }
        }

        text_lines.push(Line::from(""));
    }

    let content_width = area.width.saturating_sub(2) as usize;
    let text = Text::from(text_lines);
    let total_lines = wrap_line_count(&text, content_width);
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    let visible_height = area.height.saturating_sub(2);

    let scroll = if app.auto_scroll() {
        total_lines.saturating_sub(visible_height as usize)
    } else {
        total_lines
            .saturating_sub(visible_height as usize)
            .saturating_sub(app.scroll_offset() as usize)
    };

    f.render_widget(paragraph.scroll((scroll as u16, 0)), area);
}

fn render_input_box(f: &mut Frame, app: &App, area: Rect) {
    let input_text = match app.mode {
        AppMode::Input => {
            let text = app.input.get_text();
            format!("> {}", text)
        }
        AppMode::Normal => "> ".to_string(),
    };

    let paragraph = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);

    if let AppMode::Input = app.mode {
        f.set_cursor_position((
            area.x + 2 + app.input.cursor_position() as u16,
            area.y + 1,
        ));
    }
}

fn wrap_line_count(text: &Text, max_width: usize) -> usize {
    if max_width == 0 {
        return text.lines.len();
    }
    text.lines
        .iter()
        .map(|line| {
            let line_width = line.width();
            if line_width == 0 {
                1
            } else {
                (line_width + max_width - 1) / max_width
            }
        })
        .sum()
}
