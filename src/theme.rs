use ratatui::style::Color;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, Clone)]
pub struct Theme {
    pub name: String,
    #[serde(deserialize_with = "deserialize_color")]
    pub background: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub foreground: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub border: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub title: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub highlight_bg: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub highlight_fg: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub error: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub success: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub warning: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub info: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub input_text: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub input_placeholder: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub stack_expression: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub stack_result: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub stack_line_number: Color,
    #[serde(deserialize_with = "deserialize_color")]
    pub history_text: Color,
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_color(&s).map_err(serde::de::Error::custom)
}

fn parse_color(s: &str) -> Result<Color, String> {
    if s.starts_with('#') && s.len() == 7 {
        let r = u8::from_str_radix(&s[1..3], 16).map_err(|_| "Invalid R color component".to_string())?;
        let g = u8::from_str_radix(&s[3..5], 16).map_err(|_| "Invalid G color component".to_string())?;
        let b = u8::from_str_radix(&s[5..7], 16).map_err(|_| "Invalid B color component".to_string())?;
        Ok(Color::Rgb(r, g, b))
    } else if s.starts_with("rgb(") && s.ends_with(")") {
        let parts: Vec<&str> = s[4..s.len() - 1].split(',').map(|s| s.trim()).collect();
        if parts.len() == 3 {
            let r = parts[0].parse::<u8>().map_err(|_| "Invalid R color component".to_string())?;
            let g = parts[1].parse::<u8>().map_err(|_| "Invalid G color component".to_string())?;
            let b = parts[2].parse::<u8>().map_err(|_| "Invalid B color component".to_string())?;
            Ok(Color::Rgb(r, g, b))
        } else {
            Err(format!("Invalid rgb() format: {}", s))
        }
    } else {
        match s.to_lowercase().as_str() {
            "black" => Ok(Color::Black),
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "yellow" => Ok(Color::Yellow),
            "blue" => Ok(Color::Blue),
            "magenta" => Ok(Color::Magenta),
            "cyan" => Ok(Color::Cyan),
            "white" => Ok(Color::White),
            "darkgray" => Ok(Color::DarkGray),
            "lightred" => Ok(Color::LightRed),
            "lightgreen" => Ok(Color::LightGreen),
            "lightyellow" => Ok(Color::LightYellow),
            "lightblue" => Ok(Color::LightBlue),
            "lightmagenta" => Ok(Color::LightMagenta),
            "lightcyan" => Ok(Color::LightCyan),
            "gray" => Ok(Color::Gray),
            _ => Err(format!("Unknown color name: {}", s)),
        }
    }
}
