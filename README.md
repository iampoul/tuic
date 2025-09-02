# TUI Calculator

A terminal-based calculator built with Rust and ratatui. This calculator provides a clean, interactive interface for performing mathematical calculations directly in your terminal.

## Features

- **Dual Modes (RPN & Infix)**: Switch between Reverse Polish Notation and standard Infix notation.
- **Expression Evaluation**: Supports basic arithmetic operations (+, -, *, /)
- **Advanced Operations**: Exponentiation (^)
- **Parentheses**: Full support for parentheses and proper operator precedence (Infix mode)
- **History**: Keeps track of your calculations and RPN operations.
- **Error Handling**: Clear error messages for invalid expressions.
- **Keyboard Navigation**: Full keyboard control with intuitive shortcuts.
- **Mode Switching**: Toggle between Angle (Radians/Degrees), Base (Decimal/Hexadecimal/Binary), and Complex (Rectangular/Polar) number modes.
- **Scrolling**: Stack and History views are scrollable for long lists.
- **Truncation**: Long expressions/results are truncated with ellipsis for better readability.

## Supported Operations

- Addition: `+`
- Subtraction: `-`
- Multiplication: `*`
- Division: `/`
- Exponentiation: `^`
- Parentheses: `(` and `)` (primarily for Infix mode)
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

- **Type numbers and operators**: Just start typing your expression.
- **Enter**:
    - **RPN Mode**: Pushes the current number to the stack. If input is empty, duplicates the top stack item.
    - **Infix Mode**: Evaluates the current expression.
- **C**: Clear current input.
- **Ctrl+C**: Clear all (input, stack, and history).
- **Backspace**: Delete last character.
- **q** or **Esc**: Quit the calculator.
- **m**: Toggle between RPN and Infix modes.
- **F1**: Toggle Angle mode (Radians/Degrees).
- **F2**: Cycle Base mode (Decimal/Hexadecimal/Binary).
- **F3**: Toggle Complex mode (Rectangular/Polar).
- **Up/Down Arrows**: Browse and scroll the stack.
- **PageUp/PageDown**: Browse and scroll the history.

### Example Calculations

- **RPN Mode**:
    - `5 Enter 3 Enter +` (Calculates 5 + 3 = 8)
    - `10 Enter 2 Enter / 4 Enter *` (Calculates (10 / 2) * 4 = 20)
- **Infix Mode**:
    - `2 + 3 * 4` (Calculates 2 + (3 * 4) = 14)
    - `(2 + 3) * 4` (Calculates (2 + 3) * 4 = 20)
- Decimals: `3.14 * 2`
- Exponents: `2^3`

### Theming

The calculator supports custom themes to personalize its appearance.

- **t**: Toggle the theme selection dialog.
- **Up/Down Arrows**: Navigate through the list of available themes.
- **Enter**: Apply the selected theme.
- **Esc** or **t**: Close the theme selection dialog.

Theme files are located in the `themes/` directory. You can create your own theme files (JSON format) and place them in this directory. The application will automatically detect and list them.

## Interface

The calculator interface is divided into several sections:

- **Mode Boxes (Top Row)**: Displays the current calculator mode (RPN/Infix), Angle mode, Base mode, and Complex mode in separate, colored boxes.
- **Stack**: Shows the current numbers on the stack. Scrollable for long lists.
- **History**: Displays your previous calculations and RPN operations. Scrollable for long lists.
- **Input**: Shows your current expression.
- **Status**: Displays results or error messages.
- **Help**: Shows available keyboard shortcuts (press 'h' to toggle a detailed help dialog).

## Error Handling

The calculator will show helpful error messages for:
- Invalid expressions
- Division by zero
- Mismatched parentheses
- Unknown operators
- Invalid numbers for current base mode

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
