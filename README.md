# TUI Calculator

A terminal-based calculator built with Rust and ratatui. This calculator provides a clean, interactive interface for performing mathematical calculations directly in your terminal.

## Features

- **Expression Evaluation**: Supports basic arithmetic operations (+, -, *, /)
- **Advanced Operations**: Exponentiation (^)
- **Parentheses**: Full support for parentheses and proper operator precedence
- **History**: Keeps track of your calculations
- **Error Handling**: Clear error messages for invalid expressions
- **Keyboard Navigation**: Full keyboard control with intuitive shortcuts

## Supported Operations

- Addition: `+`
- Subtraction: `-`
- Multiplication: `*`
- Division: `/`
- Exponentiation: `^`
- Parentheses: `(` and `)`
- Decimal numbers: `3.14`

## Installation

Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/).

```bash
# Clone or navigate to the project directory
cd tui-calculator

# Build the project
cargo build --release

# Run the calculator
cargo run --release
```

## Usage

### Controls

- **Type numbers and operators**: Just start typing your expression
- **Enter**: Calculate the current expression
- **C**: Clear current input
- **Ctrl+C**: Clear all (input, result, and history)
- **Backspace**: Delete last character
- **q** or **Esc**: Quit the calculator

### Example Calculations

- Simple arithmetic: `2 + 3 * 4`
- With parentheses: `(2 + 3) * 4`
- Decimals: `3.14 * 2`
- Exponents: `2^3`
- Complex expressions: `(5 + 3) * 2^2 / 4`

## Interface

The calculator interface is divided into several sections:

- **Title**: Shows "TUI Calculator"
- **History**: Displays your previous calculations
- **Input**: Shows your current expression
- **Output**: Displays results or error messages
- **Help**: Shows available keyboard shortcuts

## Error Handling

The calculator will show helpful error messages for:
- Invalid expressions
- Division by zero
- Mismatched parentheses
- Unknown operators

## Building from Source

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run directly
cargo run
```

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal user interface library
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling

## License

This project is available under the MIT license.
