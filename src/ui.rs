use crate::calculator::{Calculator, CalculatorMode, AngleMode, BaseMode, ComplexMode}; // Added CalculatorMode, AngleMode, BaseMode, ComplexMode
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
    prelude::Stylize,
};

const MAX_DISPLAY_ITEMS: usize = 100; // Limit display to last 100 items
const MAX_DISPLAY_WIDTH: usize = 50; // Limit width of displayed strings

pub fn draw(f: &mut Frame, calculator: &mut Calculator) {
    f.render_widget(Block::default().bg(calculator.current_theme.background), f.area());
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Top row for mode boxes
            Constraint::Min(5),     // Stack display
            Constraint::Length(5),  // History display
            Constraint::Length(3),  // Input
            Constraint::Length(3),  // Status/Error
            Constraint::Length(6),  // Help
        ])
        .split(f.area());

    let mode_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Mode
            Constraint::Percentage(25), // Angle
            Constraint::Percentage(25), // Base
            Constraint::Percentage(25), // Complex
        ])
        .split(main_chunks[0]); // Split the top row

    // Mode Box
    let mode_text = match calculator.mode {
        CalculatorMode::RPN => Span::styled("RPN", Style::default().fg(calculator.current_theme.success).add_modifier(Modifier::BOLD)),
        CalculatorMode::Infix => Span::styled("INFIX", Style::default().fg(calculator.current_theme.warning).add_modifier(Modifier::BOLD)),
    };
    let mode_paragraph = Paragraph::new(Line::from(mode_text)) // Removed Span::raw("Mode: ")
        .block(Block::default().borders(Borders::ALL).title("Mode").border_style(Style::default().fg(calculator.current_theme.border)).title_style(Style::default().fg(calculator.current_theme.title)));
    f.render_widget(mode_paragraph, mode_chunks[0]);

    // Angle Box
    let angle_text = match calculator.angle_mode {
        AngleMode::Radians => Span::styled("RAD", Style::default().fg(calculator.current_theme.info)),
        AngleMode::Degrees => Span::styled("DEG", Style::default().fg(calculator.current_theme.info)),
    };
    let angle_paragraph = Paragraph::new(Line::from(angle_text)) // Removed Span::raw("Angle: ")
        .block(Block::default().borders(Borders::ALL).title("Angle").border_style(Style::default().fg(calculator.current_theme.border)).title_style(Style::default().fg(calculator.current_theme.title)));
    f.render_widget(angle_paragraph, mode_chunks[1]);

    // Base Box
    let base_text = match calculator.base_mode {
        BaseMode::Decimal => Span::styled("DEC", Style::default().fg(calculator.current_theme.success)),
        BaseMode::Hexadecimal => Span::styled("HEX", Style::default().fg(calculator.current_theme.warning)),
        BaseMode::Binary => Span::styled("BIN", Style::default().fg(calculator.current_theme.error)),
    };
    let base_paragraph = Paragraph::new(Line::from(base_text)) // Removed Span::raw("Base: ")
        .block(Block::default().borders(Borders::ALL).title("Base").border_style(Style::default().fg(calculator.current_theme.border)).title_style(Style::default().fg(calculator.current_theme.title)));
    f.render_widget(base_paragraph, mode_chunks[2]);

    // Complex Box
    let complex_text = match calculator.complex_mode {
        ComplexMode::Rectangular => Span::styled("REC", Style::default().fg(calculator.current_theme.warning)),
        ComplexMode::Polar => Span::styled("POL", Style::default().fg(calculator.current_theme.error)),
    };
    let complex_paragraph = Paragraph::new(Line::from(complex_text)) // Removed Span::raw("Complex: ")
        .block(Block::default().borders(Borders::ALL).title("Complex").border_style(Style::default().fg(calculator.current_theme.border)).title_style(Style::default().fg(calculator.current_theme.title)));
    f.render_widget(complex_paragraph, mode_chunks[3]);

    // Stack display
    let stack_display_slice = if calculator.stack.len() > MAX_DISPLAY_ITEMS {
        &calculator.stack[calculator.stack.len() - MAX_DISPLAY_ITEMS..]
    } else {
        &calculator.stack[..]
    };

    let stack_items: Vec<ListItem> = stack_display_slice
        .iter()
        .enumerate()
        .rev() // Still want top at bottom
        .map(|(i, entry)| {
            // The original_index needs to be relative to the full stack, but adjusted for the slice.
            let full_stack_start_index = calculator.stack.len().saturating_sub(stack_display_slice.len());
            let original_index = full_stack_start_index + (stack_display_slice.len() - 1 - i);

            let truncated_expression = truncate_string(&entry.expression, MAX_DISPLAY_WIDTH);
            let truncated_result = truncate_string(&calculator.format_stack_value(&entry.result), MAX_DISPLAY_WIDTH);

            let expression_span = Span::styled(truncated_expression, Style::default().fg(calculator.current_theme.stack_expression));
            let result_span = Span::styled(truncated_result, Style::default().fg(calculator.current_theme.stack_result));

            let mut line_spans = vec![
                Span::styled(format!("{} ", original_index + 1), Style::default().fg(calculator.current_theme.stack_line_number)),
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
        .block(Block::default().borders(Borders::ALL).title(stack_title).border_style(Style::default().fg(calculator.current_theme.border)).title_style(Style::default().fg(calculator.current_theme.title)))
        .highlight_style(Style::default().bg(calculator.current_theme.highlight_bg))
        .style(Style::default().fg(calculator.current_theme.foreground));
    f.render_stateful_widget(stack, main_chunks[1], &mut calculator.stack_list_state);

    // History display
    let history_display_slice = if calculator.history.len() > MAX_DISPLAY_ITEMS {
        &calculator.history[calculator.history.len() - MAX_DISPLAY_ITEMS..]
    } else {
        &calculator.history[..]
    };

    let history_items: Vec<ListItem> = history_display_slice
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let full_history_start_index = calculator.history.len().saturating_sub(history_display_slice.len());
            let original_index = full_history_start_index + i; // Correct index for history

            let truncated_entry = truncate_string(entry, MAX_DISPLAY_WIDTH);
            let mut item = ListItem::new(truncated_entry);
            if original_index == calculator.history_position {
                item = item.style(Style::default().add_modifier(Modifier::REVERSED));
            }
            item
        })
        .collect();

    let history_title = format!("History ({} items)", calculator.history.len());
    let history = List::new(history_items)
        .block(Block::default().borders(Borders::ALL).title(history_title).border_style(Style::default().fg(calculator.current_theme.border)).title_style(Style::default().fg(calculator.current_theme.title)))
        .highlight_style(Style::default().bg(calculator.current_theme.highlight_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ")
        .style(Style::default().fg(calculator.current_theme.history_text));
    f.render_stateful_widget(history, main_chunks[2], &mut calculator.history_list_state);

    // Input
    let input_text = if calculator.input.is_empty() {
        "Enter expression..."
    } else {
        &calculator.input
    };
    
    let input_style = if calculator.input.is_empty() {
        Style::default().fg(calculator.current_theme.input_placeholder)
    } else {
        Style::default().fg(calculator.current_theme.input_text)
    };

    let input = Paragraph::new(input_text)
        .style(input_style)
        .block(Block::default().borders(Borders::ALL).title("Input").border_style(Style::default().fg(calculator.current_theme.border)).title_style(Style::default().fg(calculator.current_theme.title)))
        .wrap(Wrap { trim: true });
    f.render_widget(input, main_chunks[3]);

    // Status: Show current value or error
    let (status_text, status_style) = if let Some(error) = &calculator.error {
        (format!("Error: {}", error), Style::default().fg(calculator.current_theme.error))
    } else if let Some(current) = calculator.get_current_value() {
        (format!("Current: {}", current), Style::default().fg(calculator.current_theme.success))
    } else {
        ("Ready - Enter numbers to start".to_string(), Style::default().fg(calculator.current_theme.warning))
    };

    let status_widget = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL).title("Status").border_style(Style::default().fg(calculator.current_theme.border)).title_style(Style::default().fg(calculator.current_theme.title)))
        .wrap(Wrap { trim: true });
    f.render_widget(status_widget, main_chunks[4]);

    // Help
    let help_text = vec![
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(calculator.current_theme.warning)),
            Span::raw(": Calculate | "),
            Span::styled("C", Style::default().fg(calculator.current_theme.warning)),
            Span::raw(": Clear | "),
            Span::styled("h", Style::default().fg(calculator.current_theme.warning)),
            Span::raw(": Help Dialog"),
        ]),
        Line::from(vec![
            Span::styled("Backspace", Style::default().fg(calculator.current_theme.warning)),
            Span::raw(": Delete | "),
            Span::styled("q/Esc", Style::default().fg(calculator.current_theme.warning)),
            Span::raw(": Quit | "),
            Span::styled("Ctrl+C", Style::default().fg(calculator.current_theme.warning)),
            Span::raw(": Clear All"),
        ]),
        Line::from(vec![
            Span::styled("m", Style::default().fg(calculator.current_theme.warning)),
            Span::raw(": Toggle RPN/Infix Mode | "),
            Span::raw("Operators: "),
            Span::styled("+, -, *, /, ^", Style::default().fg(calculator.current_theme.info)),
            Span::raw(" | Parentheses: "),
            Span::styled("( )", Style::default().fg(calculator.current_theme.info)),
        ]),
        Line::from(vec![
            Span::styled("PageUp/PageDown", Style::default().fg(calculator.current_theme.warning)),
            Span::raw(": Browse History | "),
            Span::styled("Up/Down", Style::default().fg(calculator.current_theme.warning)),
            Span::raw(": Browse Stack"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Quick Help (Press 'h' for more)").border_style(Style::default().fg(calculator.current_theme.border)).title_style(Style::default().fg(calculator.current_theme.title)))
        .wrap(Wrap { trim: true });
    f.render_widget(help, main_chunks[5]);

    // Render help dialog if active
    if calculator.show_help {
        draw_help_dialog(f, calculator);
    } else if calculator.show_theme_selector {
        draw_theme_selector_dialog(f, calculator);
    }
}

fn draw_help_dialog(f: &mut Frame, calculator: &mut Calculator) {
    // Create a centered popup area
    let area = centered_rect(80, 60, f.area());
    
    // Clear the background
    f.render_widget(Clear, area);
    
    let help_content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("🧮 Advanced Calculator Help", Style::default().fg(calculator.current_theme.info).add_modifier(Modifier::BOLD))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("📋 Calculator Modes:", Style::default().fg(calculator.current_theme.warning).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  Mode: RPN/INFIX (toggle with 'm')")
        ]),
        Line::from(vec![
            Span::raw("  Angle: RAD/DEG (toggle with F1)")
        ]),
        Line::from(vec![
            Span::raw("  Base: DEC/HEX/BIN (cycle with F2)")
        ]),
        Line::from(vec![
            Span::raw("  Complex: REC/POL (toggle with F3)")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("⌨️  Common Operations:", Style::default().fg(calculator.current_theme.warning).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Enter", Style::default().fg(calculator.current_theme.success)),
            Span::raw("       RPN: Push number / Duplicate. Infix: Evaluate expression.")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Delete", Style::default().fg(calculator.current_theme.success)),
            Span::raw("      Drop (remove top of stack)")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Insert", Style::default().fg(calculator.current_theme.success)),
            Span::raw("      Swap top two stack items")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Backspace", Style::default().fg(calculator.current_theme.success)),
            Span::raw("   Delete character from input")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("+, -, *, /, ^", Style::default().fg(calculator.current_theme.success)),
            Span::raw("  Basic arithmetic operations")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("n", Style::default().fg(calculator.current_theme.success)),
            Span::raw("           Negation")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("🔧 Miscellaneous:", Style::default().fg(calculator.current_theme.warning).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("m", Style::default().fg(calculator.current_theme.success)),
            Span::raw("           Toggle RPN/Infix Mode")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Space", Style::default().fg(calculator.current_theme.success)),
            Span::raw("       Scientific notation toggle")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("F1/F2/F3", Style::default().fg(calculator.current_theme.success)),
            Span::raw("    Toggle angle/base/complex modes")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("Up/Down", Style::default().fg(calculator.current_theme.success)),
            Span::raw("     Stack browsing mode")
        ]),
        Line::from(vec![
            Span::raw("  • "),
            Span::styled("PageUp/PageDown", Style::default().fg(calculator.current_theme.success)),
            Span::raw("  History browsing mode")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("💡 Usage Tips:", Style::default().fg(calculator.current_theme.warning).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • RPN Mode: Enter numbers, then use operators. Example: '5', Enter, '3', Enter, '+'")
        ]),
        Line::from(vec![
            Span::raw("  • Infix Mode: Type full expression, then Enter. Example: '2 + 3 * 4', Enter")
        ]),
        Line::from(vec![
            Span::raw("  • Switch to HEX mode and enter '0xFF' for hexadecimal")
        ]),
        Line::from(vec![
            Span::raw("  • Switch to BIN mode and enter '0b1010' for binary")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press 'h' or Esc to close this dialog", Style::default().fg(calculator.current_theme.input_placeholder).add_modifier(Modifier::ITALIC))
        ]),
        Line::from(""),
    ];
    
    let help_dialog = Paragraph::new(help_content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(calculator.current_theme.border)))
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left);
    
    f.render_widget(help_dialog, area);
}

fn draw_theme_selector_dialog(f: &mut Frame, calculator: &mut Calculator) {
    let area = centered_rect(60, 50, f.area());

    f.render_widget(Clear, area);

    let theme_items: Vec<ListItem> = calculator.available_themes.iter().map(|theme_name| {
        ListItem::new(Span::raw(theme_name))
    }).collect();

    let theme_list = List::new(theme_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Select Theme ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(calculator.current_theme.border)))
        .highlight_style(Style::default().bg(calculator.current_theme.highlight_bg).fg(calculator.current_theme.highlight_fg))
        .highlight_symbol(">> "); // We can refine this later

    f.render_stateful_widget(theme_list, area, &mut calculator.theme_list_state);
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

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len && max_len >= 3 { // Ensure max_len is at least 3 for "..."
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

