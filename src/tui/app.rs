use anyhow::Result;
use chrono::Utc;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::time::Duration;
use tokio::sync::mpsc;

use super::components::input_box::InputBox;
use super::event;
use super::ui::render_app;

pub struct App {
    pub messages: Vec<Message>,
    pub input: InputBox,
    pub mode: AppMode,
    pub is_thinking: bool,
    pub active_tool: Option<ToolExecution>,

    scroll_offset: u16,
    auto_scroll: bool,

    event_rx: mpsc::Receiver<AppEvent>,
    agent_tx: mpsc::Sender<AgentCommand>,
    should_exit: bool,
}

#[derive(Debug, Clone)]
pub enum AppMode {
    Normal,
    Input,
}

#[derive(Debug, Clone)]
pub enum Message {
    User {
        content: String,
        timestamp: chrono::DateTime<Utc>,
    },
    Assistant {
        content: String,
        timestamp: chrono::DateTime<Utc>,
        tool_calls: Vec<ToolCall>,
    },
    System {
        content: String,
    },
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub args: serde_json::Value,
    pub result: Option<String>,
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct ToolExecution {
    pub name: String,
    pub args: serde_json::Value,
    pub start_time: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    ResponseChunk(String),
    ToolStart { name: String, args: serde_json::Value },
    ToolComplete { name: String, result: String },
    ToolError { name: String, error: String },
    ThinkingStarted,
    Done,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum AgentCommand {
    SendPrompt { prompt: String },
}

impl App {
    pub fn new(event_rx: mpsc::Receiver<AppEvent>, agent_tx: mpsc::Sender<AgentCommand>) -> Self {
        App {
            messages: Vec::new(),
            input: InputBox::new(),
            mode: AppMode::Normal,
            scroll_offset: 0,
            auto_scroll: true,
            event_rx,
            agent_tx,
            is_thinking: false,
            active_tool: None,
            should_exit: false,
        }
    }

    pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        loop {
            self.handle_events()?;

            if self.should_exit {
                break;
            }

            terminal.draw(|f| render_app(f, self))?;

            tokio::time::sleep(Duration::from_millis(16)).await;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        while let Some(ev) = event::poll_event(Duration::from_millis(0))? {
            if let crossterm::event::Event::Key(key) = ev {
                self.handle_key(key);
            }
        }

        while let Ok(app_event) = self.event_rx.try_recv() {
            self.handle_app_event(app_event);
        }

        Ok(())
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};

        if key.kind != KeyEventKind::Press {
            return;
        }

        match self.mode {
            AppMode::Normal => match key.code {
                KeyCode::Char('i') => self.mode = AppMode::Input,
                KeyCode::Char('q') => {
                    self.should_exit = true;
                }
                KeyCode::Char('c')
                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    self.should_exit = true;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.scroll_offset = self.scroll_offset.saturating_add(1);
                    self.auto_scroll = false;
                }
                KeyCode::PageDown => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(10);
                }
                KeyCode::PageUp => {
                    self.scroll_offset = self.scroll_offset.saturating_add(10);
                    self.auto_scroll = false;
                }
                _ => {}
            },
            AppMode::Input => match key.code {
                KeyCode::Enter => {
                    if !self.input.is_empty() {
                        let prompt = self.input.get_text();
                        self.send_prompt(&prompt);
                        self.input.clear();
                    }
                }
                KeyCode::Esc => {
                    self.mode = AppMode::Normal;
                }
                KeyCode::Char(c) => {
                    self.input.insert_char(c);
                }
                KeyCode::Backspace => {
                    self.input.backspace();
                }
                KeyCode::Delete => {
                    self.input.delete();
                }
                KeyCode::Left => {
                    self.input.move_cursor_left();
                }
                KeyCode::Right => {
                    self.input.move_cursor_right();
                }
                KeyCode::Home => {
                    self.input.move_cursor_to_start();
                }
                KeyCode::End => {
                    self.input.move_cursor_to_end();
                }
                _ => {}
            },
        }
    }

    fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::ThinkingStarted => {
                self.is_thinking = true;
            }
            AppEvent::ResponseChunk(chunk) => {
                if let Some(Message::Assistant { content, .. }) = self.messages.last_mut() {
                    content.push_str(&chunk);
                }
                self.auto_scroll = true;
                self.scroll_offset = 0;
            }
            AppEvent::ToolStart { name, args } => {
                self.active_tool = Some(ToolExecution {
                    name,
                    args,
                    start_time: Utc::now(),
                });
            }
            AppEvent::ToolComplete { name, result } => {
                if let Some(Message::Assistant { tool_calls, .. }) = self.messages.last_mut() {
                    if let Some(active) = self.active_tool.take() {
                        let duration = Utc::now()
                            .signed_duration_since(active.start_time)
                            .to_std()
                            .ok();
                        tool_calls.push(ToolCall {
                            name,
                            args: active.args,
                            result: Some(result),
                            duration,
                        });
                    }
                }
                self.is_thinking = true;
            }
            AppEvent::ToolError { name, error } => {
                if let Some(Message::Assistant { tool_calls, .. }) = self.messages.last_mut() {
                    if let Some(active) = self.active_tool.take() {
                        tool_calls.push(ToolCall {
                            name,
                            args: active.args,
                            result: Some(format!("Error: {}", error)),
                            duration: None,
                        });
                    }
                }
            }
            AppEvent::Done => {
                self.is_thinking = false;
                self.active_tool = None;
            }
            AppEvent::Error(error) => {
                self.messages.push(Message::System {
                    content: format!("Error: {}", error),
                });
                self.is_thinking = false;
                self.active_tool = None;
            }
        }
    }

    fn send_prompt(&mut self, prompt: &str) {
        self.messages.push(Message::User {
            content: prompt.to_string(),
            timestamp: Utc::now(),
        });

        self.messages.push(Message::Assistant {
            content: String::new(),
            timestamp: Utc::now(),
            tool_calls: Vec::new(),
        });

        let _ = self.agent_tx.try_send(AgentCommand::SendPrompt {
            prompt: prompt.to_string(),
        });

        self.mode = AppMode::Normal;
        self.auto_scroll = true;
        self.scroll_offset = 0;
    }

    pub fn scroll_offset(&self) -> u16 {
        self.scroll_offset
    }

    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }
}
