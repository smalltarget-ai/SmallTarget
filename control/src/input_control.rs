use anyhow::{anyhow, Context, Result};
use enigo::Key;
use enigo::{Enigo, Keyboard, Mouse, Settings};
use log::info;
use serde::Deserialize;

pub async fn input_control_handler(action: InputAction) -> Result<bool> {
    use enigo::{Keyboard, Mouse};
    info!("input control handler {:?}", action);
    let mut enigo = Enigo::new(&Settings::default()).context("failed to initialize enigo")?;
    match action {
        InputAction::KeyPress(key) => {
            let _ = enigo.key(key_from_string(&key).unwrap(), enigo::Direction::Press);
        }
        InputAction::MouseMove { x, y } => {
            let _ = enigo.move_mouse(x, y, enigo::Coordinate::Abs);
        }
        InputAction::MouseClick(button) => {
            let _ = enigo.button(
                mouse_button_from_string(&button).unwrap(),
                enigo::Direction::Press,
            );
        }
        InputAction::WriteText(text) => {
            let _ = enigo.text(&text);
        }
    }
    Ok(true)
}

fn key_from_string(key: &str) -> Result<Key> {
    match key {
        "enter" => Ok(Key::Return),
        "space" => Ok(Key::Space),
        "tab" => Ok(Key::Tab),
        "escape" => Ok(Key::Escape),
        "backspace" => Ok(Key::Backspace),
        "delete" => Ok(Key::Delete),
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        _ => Err(anyhow!("Unsupported key: {}", key)),
    }
}

fn mouse_button_from_string(button: &str) -> Result<enigo::Button> {
    match button {
        "left" => Ok(enigo::Button::Left),
        "right" => Ok(enigo::Button::Right),
        // Add more button mappings as needed
        _ => Err(anyhow!("Unsupported mouse button: {}", button)),
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum InputAction {
    KeyPress(String),
    MouseMove { x: i32, y: i32 },
    MouseClick(String),
    WriteText(String),
}