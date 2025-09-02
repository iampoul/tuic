mod calculator;
mod ui;

use calculator::Calculator;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create calculator
    let mut calculator = Calculator::new();

    // Run the app
    let res = run_app(&mut terminal, &mut calculator);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    calculator: &mut Calculator,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, calculator))?;

        if let Event::Key(key) = event::read()? {
            if calculator.show_help {
                // Only allow help toggle and exit when help is shown
                match key.code {
                    KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Esc => {
                        calculator.toggle_help();
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(());
                    }
                    KeyCode::Char('h') | KeyCode::Char('H') => {
                        calculator.toggle_help();
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        calculator.clear_all();
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        calculator.clear_input();
                    }
                    KeyCode::Enter => {
                        calculator.enter();
                    }
                    KeyCode::Backspace => {
                        calculator.backspace();
                    }
                    // Stack operations
                    KeyCode::Delete => {
                        calculator.drop();
                    }
                    KeyCode::Insert => {
                        calculator.swap();
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        calculator.negate();
                    }
                    // Arithmetic operations - now handled by push_char and then evaluation on Enter
                    // The individual operator key presses will just add the character to the input string.
                    // The actual calculation will happen when 'Enter' is pressed.
                    KeyCode::Char('+') => {
                        calculator.push_char('+');
                    }
                    KeyCode::Char('-') => {
                        calculator.push_char('-');
                    }
                    KeyCode::Char('*') => {
                        calculator.push_char('*');
                    }
                    KeyCode::Char('/') => {
                        calculator.push_char('/');
                    }
                    KeyCode::Char('^') => {
                        calculator.push_char('^');
                    }
                    // Mode switching (using F-function keys)
                    KeyCode::F(1) => {
                        calculator.toggle_angle_mode();
                    }
                    KeyCode::F(2) => {
                        calculator.cycle_base_mode();
                    }
                    KeyCode::F(3) => {
                        calculator.toggle_complex_mode();
                    }
                    KeyCode::Char(' ') => {
                        calculator.toggle_abbreviation();
                    }
                    // Stack browsing
                    KeyCode::Up => {
                        calculator.browse_stack_up();
                    }
                    KeyCode::Down => {
                        calculator.browse_stack_down();
                    }
                    // History browsing
                    KeyCode::PageUp => {
                        calculator.browse_history_up();
                    }
                    KeyCode::PageDown => {
                        calculator.browse_history_down();
                    }
                    // Number input
                    KeyCode::Char(ch) => {
                        if ch.is_ascii_digit() || ".-abcdefABCDEFx()^*/+-".contains(ch) {
                            calculator.push_char(ch);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
