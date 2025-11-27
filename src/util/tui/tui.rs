use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame, style::{Color, Style}, widgets::{Block, Paragraph}};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

/// Run the TUI application loop.
pub fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        // Draw the UI
        terminal.draw(render)?;

        // Handle input events
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    let greeting = Paragraph::new(" Create Client Binary ");
    
    let scroll = 0;

    let input = Paragraph::new("")
            .scroll((0, scroll as u16))
            .block(Block::bordered());

    frame.render_widget(input, frame.area());
    frame.render_widget(greeting.centered(), frame.area());
}