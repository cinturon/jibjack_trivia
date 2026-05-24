mod app;
mod ascii;
mod scores;
mod trivia;
mod ui;

use std::{io::{self, Stdout}};
use std::time::Duration;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use app::App;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = setup().await?;
    
    let result = run(&mut terminal).await;
    cleanup(&mut terminal).await?;

    result
}

async fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn std::error::Error>> {  
    let mut app = App::new();

    loop{
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        // Process input
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
        {
            app.handle_key(key)?;
        }

        // Update timers and animations
        app.tick();

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

async fn setup() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    
    let mut  terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    
    Ok(terminal)
}

async fn cleanup(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

