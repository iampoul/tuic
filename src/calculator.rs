use std::collections::VecDeque;
use std::f64::consts::PI;
use std::fmt;

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
}

impl Calculator {
    pub fn new() -> Self {
        Self {
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
        }
    }

    pub fn push_char(&mut self, ch: char) {
        let is_operator = "+-*/^()".contains(ch);
        let is_valid_for_mode = match self.base_mode {
            BaseMode::Decimal => ch.is_ascii_digit() || ch == '.',
            BaseMode::Hexadecimal => ch.is_ascii_hexdigit() || ch == '.',
            BaseMode::Binary => ch == '0' || ch == '1',
        };

        if is_operator || is_valid_for_mode {
            self.input.push(ch);
            self.error = None;
        } else {
            self.error = Some(format!("Invalid character '{}' for current base mode.", ch));
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
    }

    pub fn browse_stack_down(&mut self) {
        if self.stack_position < self.stack.len().saturating_sub(1) {
            self.stack_position += 1;
        }
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
    }

    pub fn enter(&mut self) {
        if self.input.is_empty() {
            // If no input, duplicate the top stack item
            self.duplicate();
            return;
        }

        // Try to evaluate the input as an expression
        match self.evaluate(&self.input) {
            Ok(result) => {
                let new_entry = StackEntry {
                    expression: self.input.clone(),
                    result: StackValue::Real(result),
                };
                self.stack.push(new_entry);
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
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            match (&a.result, &b.result) {
                (StackValue::Real(x), StackValue::Real(y)) => {
                    if *y == 0.0 {
                        self.error = Some("Division by zero".to_string());
                        self.stack.push(a);
                        self.stack.push(b);
                    } else {
                        self.stack.push(StackEntry { expression: format!("({} / {})", a.expression, b.expression), result: StackValue::Real(x / y) });
                    }
                }
                _ => {
                    self.error = Some("Complex division not yet implemented".to_string());
                    self.stack.push(a);
                    self.stack.push(b);
                }
            }
        } else {
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
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            match (&a.result, &b.result) {
                (StackValue::Real(x), StackValue::Real(y)) => {
                    let new_expression = format!("({} {} {})", a.expression, op_char, b.expression);
                    self.stack.push(StackEntry { expression: new_expression, result: StackValue::Real(op_fn(*x, *y)) });
                }
                _ => {
                    self.error = Some("Complex arithmetic not yet implemented".to_string());
                    self.stack.push(a);
                    self.stack.push(b);
                }
            }
        } else {
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
            "angle: {} base: {} complex: {}",
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
}
