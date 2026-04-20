mod app;
mod components;
mod event;
mod ui;

pub use app::App;

use anyhow::Result;
use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::config::LLMConfig;
use crate::llm::agent::LLMAgent;
use rig::tool::ToolDyn;

pub async fn run(config: LLMConfig, tools: Vec<Box<dyn ToolDyn>>) -> Result<()> {
    let (event_tx, event_rx) = tokio::sync::mpsc::channel(100);
    let (command_tx, command_rx) = tokio::sync::mpsc::channel(10);

    let mut app = App::new(event_rx, command_tx);

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let agent = LLMAgent::new(config, tools)?;
    let _agent_task = tokio::spawn(process_agent_commands(command_rx, event_tx, agent));

    let result = app.run(&mut terminal).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn process_agent_commands(
    mut command_rx: tokio::sync::mpsc::Receiver<app::AgentCommand>,
    event_tx: tokio::sync::mpsc::Sender<app::AppEvent>,
    agent: LLMAgent,
) -> Result<()> {
    while let Some(command) = command_rx.recv().await {
        match command {
            app::AgentCommand::SendPrompt { prompt } => {
                event_tx.send(app::AppEvent::ThinkingStarted).await.ok();

                match agent.prompt(&prompt).await {
                    Ok(response) => {
                        event_tx.send(app::AppEvent::ResponseChunk(response)).await.ok();
                        event_tx.send(app::AppEvent::Done).await.ok();
                    }
                    Err(e) => {
                        event_tx.send(app::AppEvent::Error(e.to_string())).await.ok();
                    }
                }
            }
        }
    }
    Ok(())
}
