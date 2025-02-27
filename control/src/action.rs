use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use enigo::{Axis, Button, Coordinate, Direction, InputError, Key};
use enigo::{Enigo, Keyboard, Mouse, Settings};
use serde::{Deserialize, Serialize};

use crate::key_parser::parse_key_from_str;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum InputAction {
    KeyClick(Key),
    WriteText(String),

    MouseMove { x: i32, y: i32 },
    MouseLeftClick { x: i32, y: i32 },
    MouseLeftDoubleClick { x: i32, y: i32 },
    MouseRightClick { x: i32, y: i32 },
    MouseMiddleClick { x: i32, y: i32 },

    Drag { x1: i32, y1: i32, x2: i32, y2: i32 },
    Select { x1: i32, y1: i32, x2: i32, y2: i32 },
    Scroll { x: i32, y: i32, length: i32, direction: Axis },

    Hotkey { hot_keys: Vec<Key> },

    Wait { milliseconds: u64 },
}
impl InputAction {
    pub fn new(action_type: String, action_inputs: HashMap<String, String>) -> Result<InputAction> {
        Self::parse_from_action_type_and_inputs(action_type, action_inputs)
    }
    fn parse_direction(direction: &str, length: i32) -> Result<(Axis, i32), InputError> {
        match direction.to_lowercase().as_str() {
            "up" => Ok((Axis::Vertical, -length)),
            "down" => Ok((Axis::Vertical, length)),
            "left" => Ok((Axis::Horizontal, -length)),
            "right" => Ok((Axis::Horizontal, length)),
            _ => Err(InputError::Unmapping(format!("invalid direction: {} and length: {}", direction, length))),
        }
    }
    fn parse_hotkeys(key_str: &str) -> Result<Vec<Key>, InputError> {
        let mut keys = Vec::new();
        for part in key_str.split('+') {
            let key = parse_key_from_str(part.to_lowercase().trim());
            keys.push(key);
        }
        Ok(keys)
    }

    fn parse_box(box_name: &str, action_inputs: &HashMap<String, String>) -> Result<(i32, i32)> {
        let start_box = action_inputs.get(box_name).ok_or_else(|| anyhow!("missing start_box in inputs: {:?}", action_inputs))?;
        let start_box_values = serde_json::from_str::<Vec<f32>>(start_box).context("parse value failed,invalid start_box json value")?;
        if start_box_values.len() < 2 {
            return Err(anyhow!("invalid start_box value: {:?}", start_box_values));
        }
        let x = start_box_values[0] as i32;
        let y = start_box_values[1] as i32;
        Ok((x, y))
    }

    pub fn parse_from_action_type_and_inputs(action_type: String, action_inputs: HashMap<String, String>) -> Result<InputAction> {
        match action_type.as_str() {
            "click" | "click_left" => {
                let (x, y) = Self::parse_box("start_box", &action_inputs)?;
                Ok(InputAction::MouseLeftClick { x, y })
            }
            "left_double" | "left_double_click" | "double_click" => {
                let (x, y) = Self::parse_box("start_box", &action_inputs)?;
                Ok(InputAction::MouseLeftDoubleClick { x, y })
            }
            "right_single" | "right_click" => {
                let (x, y) = Self::parse_box("start_box", &action_inputs)?;
                Ok(InputAction::MouseRightClick { x, y })
            }
            "mouse_move" => {
                let (x, y) = Self::parse_box("start_box", &action_inputs)?;
                Ok(InputAction::MouseMove { x, y })
            }
            "drag" => {
                let (x1, y1) = Self::parse_box("start_box", &action_inputs)?;
                let (x2, y2) = Self::parse_box("end_box", &action_inputs)?;
                Ok(InputAction::Drag { x1, y1, x2, y2 })
            }
            "scroll" => {
                let (x, y) = Self::parse_box("start_box", &action_inputs)?;
                let direction = action_inputs.get("direction").ok_or_else(|| anyhow!("missing direction in inputs: {:?}", action_inputs))?;
                let (axis, length) = Self::parse_direction(direction, 100)?;
                Ok(InputAction::Scroll { x, y, length, direction: axis })
            }
            "hotkey" => {
                let hot_keys = action_inputs.get("key").ok_or_else(|| anyhow!("missing key in inputs: {:?}", action_inputs))?;
                let hot_keys = Self::parse_hotkeys(hot_keys)?;
                Ok(InputAction::Hotkey { hot_keys })
            }
            "wait" => {
                let milliseconds = action_inputs.get("milliseconds").map(|s| s.as_str()).unwrap_or("5000");
                Ok(InputAction::Wait {
                    milliseconds: milliseconds.parse::<u64>().context("parse value failed,invalid milliseconds value")?,
                })
            }
            "type" => {
                let text = action_inputs.get("content").ok_or_else(|| anyhow!("missing text in inputs: {:?}", action_inputs))?;
                Ok(InputAction::WriteText(text.to_string()))
            }
            "key_click" => {
                let key = action_inputs.get("key").ok_or_else(|| anyhow!("missing key in inputs: {:?}", action_inputs))?;
                let parse_key = parse_key_from_str(key);
                Ok(InputAction::KeyClick(parse_key))
            }
            _ => Err(anyhow!("invalid action type: {}", action_type)),
        }
    }
}

pub struct ActionControl {
    pub enigo: Enigo,
}

impl ActionControl {
    pub fn new(settings: &Settings) -> Self {
        Self { enigo: Enigo::new(settings).unwrap() }
    }

    pub fn handle_action(&mut self, action: InputAction) -> Result<()> {
        match action {
            InputAction::KeyClick(key) => {
                self.enigo.key(key, Direction::Click)?;
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
                self.enigo.scroll(length, direction)?;
            }
            InputAction::Hotkey { hot_keys } => {
                for key in &hot_keys {
                    self.enigo.key(*key, Direction::Press)?;
                }
                for key in hot_keys.iter().rev() {
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
