use std::collections::VecDeque;
use std::f64::consts::PI;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;
use serde_json;
use anyhow::{Result, anyhow};
use dirs;
use ratatui::widgets::ListState; // Added
use crate::theme::Theme;

const MAX_STACK_SIZE: usize = 1000;
const MAX_HISTORY_SIZE: usize = 1000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AngleMode {
    Radians,
    Degrees,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BaseMode {
    Decimal,
    Hexadecimal,
    Binary,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComplexMode {
    Rectangular,
    Polar,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalculatorMode {
    RPN,
    Infix,
}

#[derive(Debug, Clone)]
pub struct ComplexNumber {
    pub real: f64,
    pub imag: f64,
}

impl ComplexNumber {
    pub fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }
    
    pub fn magnitude(&self) -> f64 {
        (self.real * self.real + self.imag * self.imag).sqrt()
    }
    
    pub fn phase(&self) -> f64 {
        self.imag.atan2(self.real)
    }
    
    pub fn from_polar(magnitude: f64, phase: f64) -> Self {
        Self {
            real: magnitude * phase.cos(),
            imag: magnitude * phase.sin(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StackValue {
    Real(f64),
    Complex(ComplexNumber),
}

impl StackValue {
    pub fn as_real(&self) -> Option<f64> {
        match self {
            StackValue::Real(r) => Some(*r),
            StackValue::Complex(c) if c.imag == 0.0 => Some(c.real),
            _ => None,
        }
    }
    
    pub fn as_complex(&self) -> ComplexNumber {
        match self {
            StackValue::Real(r) => ComplexNumber::new(*r, 0.0),
            StackValue::Complex(c) => c.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Number(f64),
    Operator(char),
    Function(String),
    LeftParen,
    RightParen,
}

#[derive(Debug)]
pub enum CalculatorError {
    InvalidExpression,
    DivisionByZero,
    UnknownOperator,
    MismatchedParentheses,
    StackUnderflow,
    InvalidBase,
    InvalidComplex,
}

impl fmt::Display for CalculatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CalculatorError::InvalidExpression => write!(f, "Invalid expression"),
            CalculatorError::DivisionByZero => write!(f, "Division by zero"),
            CalculatorError::UnknownOperator => write!(f, "Unknown operator"),
            CalculatorError::MismatchedParentheses => write!(f, "Mismatched parentheses"),
            CalculatorError::StackUnderflow => write!(f, "Stack underflow"),
            CalculatorError::InvalidBase => write!(f, "Invalid number for current base"),
            CalculatorError::InvalidComplex => write!(f, "Invalid complex number"),
        }
    }
}

#[derive(Clone)]
pub struct StackEntry {
    pub expression: String,
    pub result: StackValue,
}

pub struct Calculator {
    pub input: String,
    pub stack: Vec<StackEntry>,
    pub error: Option<String>,
    pub history: Vec<String>,
    pub history_position: usize,
    pub show_help: bool,
    pub angle_mode: AngleMode,
    pub base_mode: BaseMode,
    pub complex_mode: ComplexMode,
    pub stack_position: usize,
    pub abbreviation_mode: bool,
    pub mode: CalculatorMode, // New field
    pub stack_list_state: ListState, // New field for stack scrolling
    pub history_list_state: ListState, // New field for history scrolling
    pub current_theme: Theme,
    pub available_themes: Vec<String>,
    pub show_theme_selector: bool,
    pub theme_list_state: ListState,
}

impl Calculator {
    pub fn new() -> Result<Self, anyhow::Error> {
        let mut current_theme_name = "default".to_string();
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not find config directory"))?;
        let app_config_dir = config_dir.join("tui-calculator");
        let theme_config_path = app_config_dir.join("theme.txt");

        if let Ok(theme_name_from_file) = fs::read_to_string(&theme_config_path) {
            current_theme_name = theme_name_from_file.trim().to_string();
        }

        let initial_theme = match fs::read_to_string(format!("themes/{}.json", current_theme_name)) {
            Ok(content) => serde_json::from_str(&content)?,
            Err(_) => {
                // Fallback to default theme if the saved theme is not found or invalid
                let default_theme_path = "themes/default.json";
                let content = fs::read_to_string(default_theme_path)?;
                serde_json::from_str(&content)?
            }
        };

        let mut available_themes = Vec::new();
        let themes_dir = "themes";
        if Path::new(themes_dir).is_dir() {
            for entry in fs::read_dir(themes_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_stem() {
                        if let Some(name_str) = file_name.to_str() {
                            available_themes.push(name_str.to_string());
                        }
                    }
                }
            }
        }

        Ok(Self {
            input: String::new(),
            stack: Vec::new(),
            error: None,
            history: Vec::new(),
            history_position: 0,
            show_help: false,
            angle_mode: AngleMode::Radians,
            base_mode: BaseMode::Decimal,
            complex_mode: ComplexMode::Rectangular,
            stack_position: 0,
            abbreviation_mode: false,
            mode: CalculatorMode::RPN, // Initialize to RPN
            stack_list_state: ListState::default(), // Initialize ListState
            history_list_state: ListState::default(), // Initialize ListState
            current_theme: initial_theme,
            available_themes,
            show_theme_selector: false,
            theme_list_state: ListState::default(),
        })
    }

    pub fn handle_char_input(&mut self, input_char: char) {
        match self.mode {
            CalculatorMode::RPN => {
                match input_char {
                    '0'..='9' | '.' => {
                        // Accumulate digits for the current number
                        self.input.push(input_char);
                        self.error = None;
                    }
                    '+' | '-' | '*' | '/' | '^' => {
                        // If there's a number being typed, push it to the stack first
                        if !self.input.is_empty() {
                            if let Err(e) = self.parse_current_input_to_stack_entry() {
                                self.error = Some(format!("{}", e));
                                return;
                            }
                        }
                        // Now apply the operator
                        self.apply_rpn_operator(input_char);
                        self.error = None;
                    }
                    _ => {
                        // Ignore other characters for now, or handle as invalid input
                        self.error = Some(format!("Invalid input: '{}'", input_char));
                    }
                }
            }
            CalculatorMode::Infix => {
                // In infix mode, just append all valid characters to the input string
                let is_valid_infix_char = "0123456789.+-*/^()".contains(input_char);
                if is_valid_infix_char {
                    self.input.push(input_char);
                    self.error = None;
                } else {
                    self.error = Some(format!("Invalid character '{}' for infix mode.", input_char));
                }
            }
        }
    }

    pub fn backspace(&mut self) {
        self.input.pop();
        self.error = None;
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.error = None;
    }

    pub fn clear_all(&mut self) {
        self.input.clear();
        self.stack.clear();
        self.error = None;
        self.history.clear();
        self.stack_position = 0;
        self.history_position = 0;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    // Mode switching functions
    pub fn toggle_angle_mode(&mut self) {
        self.angle_mode = match self.angle_mode {
            AngleMode::Radians => AngleMode::Degrees,
            AngleMode::Degrees => AngleMode::Radians,
        };
    }

    pub fn cycle_base_mode(&mut self) {
        self.base_mode = match self.base_mode {
            BaseMode::Decimal => BaseMode::Hexadecimal,
            BaseMode::Hexadecimal => BaseMode::Binary,
            BaseMode::Binary => BaseMode::Decimal,
        };
    }

    pub fn toggle_complex_mode(&mut self) {
        self.complex_mode = match self.complex_mode {
            ComplexMode::Rectangular => ComplexMode::Polar,
            ComplexMode::Polar => ComplexMode::Rectangular,
        };
    }

    pub fn toggle_abbreviation(&mut self) {
        self.abbreviation_mode = !self.abbreviation_mode;
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            CalculatorMode::RPN => CalculatorMode::Infix,
            CalculatorMode::Infix => CalculatorMode::RPN,
        };
        self.error = None; // Clear any error when mode changes
        self.input.clear(); // Clear input when mode changes
    }

    pub fn toggle_theme_selector(&mut self) {
        self.show_theme_selector = !self.show_theme_selector;
    }

    pub fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        let theme_path = format!("themes/{}.json", theme_name);
        let content = fs::read_to_string(&theme_path)?;
        let theme: Theme = serde_json::from_str(&content)?;
        self.current_theme = theme;

        // Save selected theme to config file
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not find config directory"))?;
        let app_config_dir = config_dir.join("tui-calculator");
        fs::create_dir_all(&app_config_dir)?;
        let theme_config_path = app_config_dir.join("theme.txt");
        let mut file = fs::File::create(&theme_config_path)?;
        file.write_all(theme_name.as_bytes())?;

        Ok(())
    }

    pub fn preview_theme(&mut self, theme_name: &str) {
        let theme_path = format!("themes/{}.json", theme_name);
        if let Ok(content) = fs::read_to_string(&theme_path) {
            if let Ok(theme) = serde_json::from_str::<Theme>(&content) {
                self.current_theme = theme;
            }
        }
    }

    // Stack operations
    pub fn drop(&mut self) {
        if !self.stack.is_empty() {
            self.stack.pop();
            self.stack_position = self.stack_position.min(self.stack.len().saturating_sub(1));
        }
    }

    pub fn swap(&mut self) {
        if self.stack.len() >= 2 {
            let len = self.stack.len();
            self.stack.swap(len - 1, len - 2);
        }
    }

    pub fn duplicate(&mut self) {
        if let Some(top) = self.stack.last() {
            self.stack.push(top.clone());
        }
    }

    pub fn negate(&mut self) {
        if let Some(top) = self.stack.last_mut() {
            match top {
                StackEntry { expression: _, result: StackValue::Real(r) } => *r = -*r,
                StackEntry { expression: _, result: StackValue::Complex(c) } => {
                    c.real = -c.real;
                    c.imag = -c.imag;
                }
            }
        } else if !self.input.is_empty() {
            if let Ok(num) = self.input.parse::<f64>() {
                self.input = (-num).to_string();
            }
        }
    }

    pub fn browse_stack_up(&mut self) {
        if self.stack_position > 0 {
            self.stack_position -= 1;
        }
        self.stack_list_state.select(Some(self.stack_position));
    }

    pub fn browse_stack_down(&mut self) {
        if self.stack_position < self.stack.len().saturating_sub(1) {
            self.stack_position += 1;
        }
        self.stack_list_state.select(Some(self.stack_position));
    }

    pub fn browse_history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        if self.history_position > 0 {
            self.history_position -= 1;
        }
        self.input = self.history[self.history_position].split(" = ").next().unwrap_or("").to_string();
        self.error = None;
        self.history_list_state.select(Some(self.history_position));
    }

    pub fn browse_history_down(&mut self) {
        if self.history.is_empty() {
            return;
        }
        if self.history_position < self.history.len() - 1 {
            self.history_position += 1;
        } else {
            // If at the end of history, clear input
            self.history_position = self.history.len();
            self.input.clear();
        }
        if self.history_position < self.history.len() {
            self.input = self.history[self.history_position].split(" = ").next().unwrap_or("").to_string();
        } else {
            self.input.clear();
        }
        self.error = None;
        self.history_list_state.select(Some(self.history_position));
    }

    pub fn enter(&mut self) {
        match self.mode {
            CalculatorMode::RPN => {
                if !self.input.is_empty() {
                    // Check if the input matches a history entry's expression part
                    if let Some(history_entry) = self.history.iter().find(|entry| entry.starts_with(&self.input)) {
                        // Extract the result part (after " = ")
                        if let Some(result_str) = history_entry.split(" = ").nth(1) {
                            if let Ok(num) = result_str.parse::<f64>() {
                                let new_entry = StackEntry {
                                    expression: result_str.to_string(),
                                    result: StackValue::Real(num),
                                };
                                if self.stack.len() >= MAX_STACK_SIZE {
                                    self.stack.remove(0);
                                }
                                self.stack.push(new_entry);
                                self.input.clear();
                                self.error = None;
                                return;
                            }
                        }
                    }

                    // If not a history recall, push the current number to the stack
                    if let Err(e) = self.parse_current_input_to_stack_entry() {
                        self.error = Some(format!("{}", e));
                        return;
                    }
                } else {
                    // If input is empty, duplicate the top stack item (RPN behavior)
                    self.duplicate();
                }
            }
            CalculatorMode::Infix => {
                if self.input.is_empty() {
                    // In infix mode, if input is empty, duplicate top stack item
                    self.duplicate();
                    self.error = None;
                    return;
                }

                // Try to evaluate the input as an expression
                match self.evaluate(&self.input) {
                    Ok(result) => {
                        let new_entry = StackEntry {
                            expression: self.input.clone(),
                            result: StackValue::Real(result),
                        };
                        
                        // Enforce MAX_STACK_SIZE
                        if self.stack.len() >= MAX_STACK_SIZE {
                            self.stack.remove(0); // Remove the oldest entry
                        }
                        self.stack.push(new_entry);

                        // Enforce MAX_HISTORY_SIZE
                        if self.history.len() >= MAX_HISTORY_SIZE {
                            self.history.remove(0); // Remove the oldest entry
                        }
                        self.history.push(format!("{} = {}", self.input, self.format_stack_value(&self.stack.last().unwrap().result)));
                        self.history_position = self.history.len(); // Reset history position to the end
                        self.input.clear();
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(format!("{}", e));
                    }
                }
            }
        }
        self.error = None; // Clear error after successful operation
    }

    fn parse_current_input_to_stack_entry(&mut self) -> Result<(), CalculatorError> {
        if self.input.is_empty() {
            return Err(CalculatorError::InvalidExpression); // Or a more specific error
        }

        let stack_value = self.parse_input()?; // Re-use existing parse_input
        let new_entry = StackEntry {
            expression: self.input.clone(),
            result: stack_value,
        };
        
        // Enforce MAX_STACK_SIZE
        if self.stack.len() >= MAX_STACK_SIZE {
            self.stack.remove(0); // Remove the oldest entry
        }
        self.stack.push(new_entry.clone()); // Clone new_entry before moving it

        // Log the pushed number to history
        if self.history.len() >= MAX_HISTORY_SIZE {
            self.history.remove(0); // Remove the oldest entry
        }
        self.history.push(format!("{}", new_entry.expression)); // new_entry is still available here
        
        self.input.clear();
        Ok(())
    }

    fn apply_rpn_operator(&mut self, op_char: char) {
        match op_char {
            '+' => self.add(),
            '-' => self.subtract(),
            '*' => self.multiply(),
            '/' => self.divide(),
            '^' => self.power(),
            _ => self.error = Some("Unknown RPN operator".to_string()),
        }
    }

    fn tokenize(&self, input: &str) -> Result<Vec<Token>, CalculatorError> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' => {
                    chars.next();
                }
                '0'..='9' | '.' => {
                    let mut number = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_digit() || ch == '.' {
                            number.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    let num = number.parse::<f64>().map_err(|_| CalculatorError::InvalidExpression)?;
                    tokens.push(Token::Number(num));
                }
                '+' | '-' | '*' | '/' | '^' => {
                    tokens.push(Token::Operator(chars.next().unwrap()));
                }
                '(' => {
                    tokens.push(Token::LeftParen);
                    chars.next();
                }
                ')' => {
                    tokens.push(Token::RightParen);
                    chars.next();
                }
                _ => {
                    return Err(CalculatorError::InvalidExpression);
                }
            }
        }

        Ok(tokens)
    }

    fn evaluate(&self, input: &str) -> Result<f64, CalculatorError> {
        let tokens = self.tokenize(input)?;
        self.evaluate_tokens(tokens)
    }

    fn evaluate_tokens(&self, tokens: Vec<Token>) -> Result<f64, CalculatorError> {
        let postfix = self.infix_to_postfix(tokens)?;
        self.evaluate_postfix(postfix)
    }

    fn precedence(&self, op: char) -> i32 {
        match op {
            '+' | '-' => 1,
            '*' | '/' => 2,
            '^' => 3,
            _ => 0,
        }
    }

    fn is_right_associative(&self, op: char) -> bool {
        op == '^'
    }

    fn infix_to_postfix(&self, tokens: Vec<Token>) -> Result<Vec<Token>, CalculatorError> {
        let mut output = Vec::new();
        let mut operators = Vec::new();

        for token in tokens {
            match token {
                Token::Number(_) => output.push(token),
                Token::Function(_) => output.push(token),  // Functions for future use
                Token::Operator(op) => {
                    while let Some(Token::Operator(top_op)) = operators.last() {
                        let top_precedence = self.precedence(*top_op);
                        let curr_precedence = self.precedence(op);
                        
                        if top_precedence > curr_precedence ||
                           (top_precedence == curr_precedence && !self.is_right_associative(op)) {
                            output.push(operators.pop().unwrap());
                        } else {
                            break;
                        }
                    }
                    operators.push(token);
                }
                Token::LeftParen => operators.push(token),
                Token::RightParen => {
                    while let Some(op) = operators.pop() {
                        match op {
                            Token::LeftParen => break,
                            _ => output.push(op),
                        }
                    }
                }
            }
        }

        while let Some(op) = operators.pop() {
            match op {
                Token::LeftParen | Token::RightParen => {
                    return Err(CalculatorError::MismatchedParentheses);
                }
                _ => output.push(op),
            }
        }

        Ok(output)
    }

    fn evaluate_postfix(&self, tokens: Vec<Token>) -> Result<f64, CalculatorError> {
        let mut stack = VecDeque::new();

        for token in tokens {
            match token {
                Token::Number(num) => stack.push_back(num),
                Token::Operator(op) => {
                    if stack.len() < 2 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let b = stack.pop_back().unwrap();
                    let a = stack.pop_back().unwrap();
                    
                    let result = match op {
                        '+' => a + b,
                        '-' => a - b,
                        '*' => a * b,
                        '/' => {
                            if b == 0.0 {
                                return Err(CalculatorError::DivisionByZero);
                            }
                            a / b
                        }
                        '^' => a.powf(b),
                        _ => return Err(CalculatorError::UnknownOperator),
                    };
                    
                    stack.push_back(result);
                }
                _ => return Err(CalculatorError::InvalidExpression),
            }
        }

        if stack.len() == 1 {
            Ok(stack.pop_back().unwrap())
        } else {
            Err(CalculatorError::InvalidExpression)
        }
    }

    fn parse_input(&self) -> Result<StackValue, CalculatorError> {
        let input = self.input.trim();
        
        // Handle different number bases
        match self.base_mode {
            BaseMode::Decimal => {
                if let Ok(num) = input.parse::<f64>() {
                    Ok(StackValue::Real(num))
                } else {
                    Err(CalculatorError::InvalidExpression)
                }
            }
            BaseMode::Hexadecimal => {
                let clean_input = input.strip_prefix("0x").unwrap_or(input);
                if let Ok(num) = i64::from_str_radix(clean_input, 16) {
                    Ok(StackValue::Real(num as f64))
                } else {
                    Err(CalculatorError::InvalidBase)
                }
            }
            BaseMode::Binary => {
                let clean_input = input.strip_prefix("0b").unwrap_or(input);
                if let Ok(num) = i64::from_str_radix(clean_input, 2) {
                    Ok(StackValue::Real(num as f64))
                } else {
                    Err(CalculatorError::InvalidBase)
                }
            }
        }
    }

    pub fn format_stack_value(&self, value: &StackValue) -> String {
        match value {
            StackValue::Real(r) => self.format_real(*r),
            StackValue::Complex(c) => self.format_complex(c),
        }
    }

    fn format_real(&self, value: f64) -> String {
        match self.base_mode {
            BaseMode::Decimal => {
                if self.abbreviation_mode && value.abs() >= 1e6 {
                    format!("{:.3e}", value)
                } else {
                    format!("{}", value)
                }
            }
            BaseMode::Hexadecimal => {
                if value.fract() == 0.0 && value.abs() <= i64::MAX as f64 {
                    format!("0x{:X}", value as i64)
                } else {
                    format!("{} (hex: 0x{:X})", value, value as i64)
                }
            }
            BaseMode::Binary => {
                if value.fract() == 0.0 && value.abs() <= i64::MAX as f64 {
                    format!("0b{:b}", value as i64)
                } else {
                    format!("{} (bin: 0b{:b})", value, value as i64)
                }
            }
        }
    }

    fn format_complex(&self, c: &ComplexNumber) -> String {
        match self.complex_mode {
            ComplexMode::Rectangular => {
                if c.imag >= 0.0 {
                    format!("{} + {}i", self.format_real(c.real), self.format_real(c.imag))
                } else {
                    format!("{} - {}i", self.format_real(c.real), self.format_real(-c.imag))
                }
            }
            ComplexMode::Polar => {
                let mag = c.magnitude();
                let phase = c.phase();
                let phase_display = if self.angle_mode == AngleMode::Degrees {
                    phase * 180.0 / PI
                } else {
                    phase
                };
                let unit = if self.angle_mode == AngleMode::Degrees { "°" } else { "rad" };
                format!("{} ∠ {}{}", self.format_real(mag), self.format_real(phase_display), unit)
            }
        }
    }

    // Arithmetic operations on stack
    pub fn add(&mut self) {
        self.binary_operation('+', |a, b| a + b);
    }

    pub fn subtract(&mut self) {
        self.binary_operation('-', |a, b| a - b);
    }

    pub fn multiply(&mut self) {
        self.binary_operation('*', |a, b| a * b);
    }

    pub fn divide(&mut self) {
        // Pop b first
        let b_opt = self.stack.pop();
        // Pop a second
        let a_opt = self.stack.pop();

        if b_opt.is_some() && a_opt.is_some() { // Check if both are Some
            let b = b_opt.unwrap(); // Unwrap here
            let a = a_opt.unwrap(); // Unwrap here

            match (&a.result, &b.result) {
                (StackValue::Real(x), StackValue::Real(y)) => {
                    if *y == 0.0 {
                        self.error = Some("Division by zero".to_string());
                        self.stack.push(a);
                        self.stack.push(b);
                    } else {
                        let new_expression = format!("({} / {})", a.expression, b.expression);
                        let result_value = StackValue::Real(x / y);

                        // Enforce MAX_STACK_SIZE
                        if self.stack.len() >= MAX_STACK_SIZE {
                            self.stack.remove(0); // Remove the oldest entry
                        }
                        self.stack.push(StackEntry { expression: new_expression.clone(), result: result_value.clone() });

                        // Log the operation to history
                        if self.history.len() >= MAX_HISTORY_SIZE {
                            self.history.remove(0); // Remove the oldest entry
                        }
                        self.history.push(format!("{} = {}", new_expression, self.format_stack_value(&result_value)));
                    }
                }
                _ => {
                    self.error = Some("Complex division not yet implemented".to_string());
                    self.stack.push(a);
                    self.stack.push(b);
                }
            }
        } else {
            // Stack underflow: push back any item that was popped
            if let Some(a) = a_opt { self.stack.push(a); }
            if let Some(b) = b_opt { self.stack.push(b); } // b was popped first, so push it back last
            self.error = Some("Stack underflow".to_string());
        }
    }

    pub fn power(&mut self) {
        self.binary_operation('^', |a, b| a.powf(b));
    }

    fn binary_operation<F>(&mut self, op_char: char, op_fn: F)
    where
        F: Fn(f64, f64) -> f64,
    {
        // Pop b first
        let b_opt = self.stack.pop();
        // Pop a second
        let a_opt = self.stack.pop();

        if b_opt.is_some() && a_opt.is_some() { // Check if both are Some
            let b = b_opt.unwrap(); // Unwrap here
            let a = a_opt.unwrap(); // Unwrap here

            match (&a.result, &b.result) {
                (StackValue::Real(x), StackValue::Real(y)) => {
                    let new_expression = format!("({} {} {})", a.expression, op_char, b.expression);
                    let result_value = StackValue::Real(op_fn(*x, *y));
                    
                    // Enforce MAX_STACK_SIZE
                    if self.stack.len() >= MAX_STACK_SIZE {
                        self.stack.remove(0); // Remove the oldest entry
                    }
                    self.stack.push(StackEntry { expression: new_expression.clone(), result: result_value.clone() });

                    // Log the operation to history
                    if self.history.len() >= MAX_HISTORY_SIZE {
                        self.history.remove(0); // Remove the oldest entry
                    }
                    self.history.push(format!("{} = {}", new_expression, self.format_stack_value(&result_value)));
                }
                _ => {
                    self.error = Some("Complex arithmetic not yet implemented".to_string());
                    // Push back a and b if complex arithmetic is not implemented
                    self.stack.push(a);
                    self.stack.push(b);
                }
            }
        } else {
            // Stack underflow: push back any item that was popped
            if let Some(a) = a_opt { self.stack.push(a); }
            if let Some(b) = b_opt { self.stack.push(b); } // b was popped first, so push it back last
            self.error = Some("Stack underflow".to_string());
        }
    }

    pub fn get_current_value(&self) -> Option<String> {
        if !self.input.is_empty() {
            Some(self.input.clone())
        } else if let Some(top) = self.stack.last() {
            Some(self.format_stack_value(&top.result))
        } else {
            None
        }
    }

    pub fn get_mode_string(&self) -> String {
        format!(
            "Mode: {} | Angle: {} | Base: {} | Complex: {}",
            match self.mode {
                CalculatorMode::RPN => "RPN",
                CalculatorMode::Infix => "INFIX",
            },
            match self.angle_mode {
                AngleMode::Radians => "RAD",
                AngleMode::Degrees => "DEG",
            },
            match self.base_mode {
                BaseMode::Decimal => "DEC",
                BaseMode::Hexadecimal => "HEX",
                BaseMode::Binary => "BIN",
            },
            match self.complex_mode {
                ComplexMode::Rectangular => "REC",
                ComplexMode::Polar => "POL",
            }
        )
    }

    
}