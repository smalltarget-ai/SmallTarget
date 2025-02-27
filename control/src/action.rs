use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use enigo::{Axis, Button, Coordinate, Direction, InputError, Key};
use enigo::{Enigo, Keyboard, Mouse, Settings};
use serde::{Deserialize, Serialize};

#[derive(Deserialize,Serialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum InputAction {
    KeyPress(Key),
    WriteText(String),

    MouseMove { x: i32, y: i32 },
    MouseLeftClick { x: i32, y: i32 },
    MouseLeftDoubleClick { x: i32, y: i32 },
    MouseRightClick { x: i32, y: i32 },
    MouseMiddleClick { x: i32, y: i32 },

    Drag { x1: i32, y1: i32, x2: i32, y2: i32 },
    Select { x1: i32, y1: i32, x2: i32, y2: i32 },
    Scroll { x: i32, y: i32, length: i32, direction: String },

    Hotkey { hot_keys: String },

    Wait { milliseconds: u64 },
}

pub struct ActionControl {
    enigo: Enigo,
}

impl ActionControl {
    pub fn new(settings: &Settings) -> Self {
        Self { enigo: Enigo::new(settings).unwrap() }
    }
    fn parse_hotkeys(&self, key_str: &str) -> Result<Vec<Key>, InputError> {
        let mut keys = Vec::new();
        for part in key_str.split('+') {
            let key = match part.to_lowercase().as_str() {
                "ctrl" => Key::Control,
                "shift" => Key::Shift,
                "alt" => Key::Alt,
                "cmd" | "meta" | "command" => Key::Meta,
                "enter" | "return" => Key::Return,
                "space" => Key::Space,
                _ => Key::Unicode(part.chars().next().unwrap()),
            };
            keys.push(key);
        }
        Ok(keys)
    }
    fn parse_direction(&self, direction: &str) -> Result<Axis, InputError> {
        match direction.to_lowercase().as_str() {
            "up" | "down" => Ok(Axis::Vertical),
            "left" | "right" => Ok(Axis::Horizontal),
            _ => Err(InputError::Unmapping(format!("invalid direction: {}", direction))),
        }
    }

    pub fn handle_action(&mut self, action: InputAction) -> Result<(), InputError> {
        match action {
            InputAction::KeyPress(key) => {
                self.enigo.key(key, Direction::Press)?;
            }
            InputAction::WriteText(text) => {
                let stripped = text.trim_end_matches("\\n").trim_end_matches('\n');
                if !stripped.is_empty() {
                    self.enigo.text(&stripped)?;
                }
                if text.ends_with("\\n") || text.ends_with('\n') {
                    self.enigo.key(Key::Return, Direction::Click)?;
                }
            }
            InputAction::MouseMove { x, y } => {
                self.enigo.move_mouse(x, y, Coordinate::Abs)?;
            }
            InputAction::MouseLeftClick { x, y } => {
                self.enigo.move_mouse(x, y, Coordinate::Abs)?;
                self.enigo.button(Button::Left, Direction::Click)?;
            }
            InputAction::MouseLeftDoubleClick { x, y } => {
                self.enigo.move_mouse(x, y, Coordinate::Abs)?;
                self.enigo.button(Button::Left, Direction::Click)?;
                self.enigo.button(Button::Left, Direction::Click)?;
            }
            InputAction::MouseRightClick { x, y } => {
                self.enigo.move_mouse(x, y, Coordinate::Abs)?;
                self.enigo.button(Button::Right, Direction::Click)?;
            }
            InputAction::MouseMiddleClick { x, y } => {
                self.enigo.move_mouse(x, y, Coordinate::Abs)?;
                self.enigo.button(Button::Middle, Direction::Click)?;
            }
            InputAction::Drag { x1, y1, x2, y2 } | InputAction::Select { x1, y1, x2, y2 } => {
                self.enigo.move_mouse(x1, y1, Coordinate::Abs)?;
                self.enigo.button(Button::Left, Direction::Press)?;
                // 添加延迟确保拖动操作可靠性
                sleep(Duration::from_millis(50));
                self.enigo.move_mouse(x2, y2, Coordinate::Abs)?;
                self.enigo.button(Button::Left, Direction::Release)?;
            }
            InputAction::Scroll { x, y, length, direction } => {
                self.enigo.move_mouse(x, y, Coordinate::Abs)?;
                let axis = self.parse_direction(&direction)?;
                self.enigo.scroll(length, axis)?;
            }
            InputAction::Hotkey { hot_keys } => {
                let keys = self.parse_hotkeys(&hot_keys)?;
                for key in &keys {
                    self.enigo.key(*key, Direction::Press)?;
                }
                for key in keys.iter().rev() {
                    self.enigo.key(*key, Direction::Release)?;
                }
            }
            InputAction::Wait { milliseconds } => {
                sleep(Duration::from_millis(milliseconds));
            }
        };
        Ok(())
    }
}
