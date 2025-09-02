use crate::calculator::Calculator;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, calculator: &Calculator) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title with modes
            Constraint::Min(5),     // Stack display
            Constraint::Length(5),  // History display
            Constraint::Length(3),  // Input
            Constraint::Length(3),  // Status/Error
            Constraint::Length(6),  // Help
        ])
        .split(f.area());

    // Title with modes
    let title_text = format!("Calculator Modes: {}", calculator.get_mode_string());
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Calculator"));
    f.render_widget(title, chunks[0]);

    // Stack display
    let stack_items: Vec<ListItem> = calculator
        .stack
        .iter()
        .enumerate()
        .rev()
        .map(|(i, entry)| {
            let original_index = calculator.stack.len() - 1 - i;
            let expression_span = Span::styled(&entry.expression, Style::default().fg(Color::LightBlue));
            let result_span = Span::styled(calculator.format_stack_value(&entry.result), Style::default().fg(Color::White));

            let mut line_spans = vec![
                Span::raw(format!("{}: ", original_index + 1)),
                expression_span,
                Span::raw(" = "),
                result_span,
            ];

            if original_index == calculator.stack_position {
                line_spans.push(Span::raw(" ←"));
            }
            
            ListItem::new(Line::from(line_spans))
        })
        .collect();
    
    let stack_title = format!("Stack ({} items)", calculator.stack.len());
    let stack = List::new(stack_items)
        .block(Block::default().borders(Borders::ALL).title(stack_title))
        .style(Style::default().fg(Color::White));
    f.render_widget(stack, chunks[1]);

    // History display
    let history_items: Vec<ListItem> = calculator
        .history
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let mut item = ListItem::new(entry.clone());
            if i == calculator.history_position {
                item = item.style(Style::default().add_modifier(Modifier::REVERSED));
            }
            item
        })
        .collect();

    let history_title = format!("History ({} items)", calculator.history.len());
    let history = List::new(history_items)
        .block(Block::default().borders(Borders::ALL).title(history_title))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(history, chunks[2]);

    // Input
    let input_text = if calculator.input.is_empty() {
        "Enter expression..."
    } else {
        &calculator.input
    };
    
    let input_style = if calculator.input.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(input_text)
        .style(input_style)
        .block(Block::default().borders(Borders::ALL).title("Input"))
        .wrap(Wrap { trim: true });
    f.render_widget(input, chunks[3]);

    // Status: Show current value or error
    let (status_text, status_style) = if let Some(error) = &calculator.error {
        (format!("Error: {}", error), Style::default().fg(Color::Red))
    } else if let Some(current) = calculator.get_current_value() {
        (format!("Current: {}", current), Style::default().fg(Color::Green))
    } else {
        ("Ready - Enter numbers to start".to_string(), Style::default().fg(Color::Yellow))
    };

    let status_widget = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .wrap(Wrap { trim: true });
    f.render_widget(status_widget, chunks[4]);

    // Help
    let help_text = vec![
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Calculate | "),
            Span::styled("C", Style::default().fg(Color::Yellow)),
            Span::raw(": Clear | "),
            Span::styled("h", Style::default().fg(Color::Yellow)),
            Span::raw(": Help Dialog"),
        ]),
        Line::from(vec![
            Span::styled("Backspace", Style::default().fg(Color::Yellow)),
            Span::raw(": Delete | "),
            Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit | "),
            Span::styled("Ctrl+C", Style::default().fg(Color::Yellow)),
            Span::raw(": Clear All"),
        ]),
        Line::from(vec![
            Span::raw("Operators: "),
            Span::styled("+, -, *, /, ^", Style::default().fg(Color::Cyan)),
            Span::raw(" | Parentheses: "),
            Span::styled("( )", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("PageUp/PageDown", Style::default().fg(Color::Yellow)),
            Span::raw(": Browse History | "),
            Span::styled("Up/Down", Style::default().fg(Color::Yellow)),
            Span::raw(": Browse Stack"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Quick Help (Press 'h' for more)"))
        .wrap(Wrap { trim: true });
    f.render_widget(help, chunks[5]);

    // Render help dialog if active
    if calculator.show_help {
        draw_help_dialog(f);
    }
}

fn draw_help_dialog(f: &mut Frame) {
    // Create a centered popup area
    let area = centered_rect(80, 60, f.area());
    
    // Clear the background
    f.render_widget(Clear, area);
    
    let help_content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("🧮 Advanced Calculator Help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("📋 Calculator Modes:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  angle: RAD/DEG  base: HEX/DEC/BIN  complex: REC/POL")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("⌨️  Common Operations:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw("       Push number to stack / Duplicate if empty")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Delete", Style::default().fg(Color::Green)),
            Span::raw("      Drop (remove top of stack)")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Insert", Style::default().fg(Color::Green)),
            Span::raw("      Swap top two stack items")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Backspace", Style::default().fg(Color::Green)),
            Span::raw("   Delete character from input")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("+, -, *, /, ^", Style::default().fg(Color::Green)),
            Span::raw("  Basic arithmetic operations")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("n", Style::default().fg(Color::Green)),
            Span::raw("           Negation")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("🔧 Miscellaneous:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Space", Style::default().fg(Color::Green)),
            Span::raw("       Scientific notation toggle")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("F1/F2/F3", Style::default().fg(Color::Green)),
            Span::raw("    Toggle angle/base/complex modes")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Up/Down", Style::default().fg(Color::Green)),
            Span::raw("     Stack browsing mode")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("PageUp/PageDown", Style::default().fg(Color::Green)),
            Span::raw("  History browsing mode")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("💡 Usage Tips:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • Enter numbers, then use operators for RPN-style calculation")
        ]),
        Line::from(vec![
            Span::raw("  • Example: Type '5', Enter, '3', Enter, '+' to calculate 5+3")
        ]),
        Line::from(vec![
            Span::raw("  • Switch to HEX mode and enter '0xFF' for hexadecimal")
        ]),
        Line::from(vec![
            Span::raw("  • Switch to BIN mode and enter '0b1010' for binary")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press 'h' or Esc to close this dialog", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))
        ]),
        Line::from(""),
    ];
    
    let help_dialog = Paragraph::new(help_content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Cyan)))
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left);
    
    f.render_widget(help_dialog, area);
}

// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

