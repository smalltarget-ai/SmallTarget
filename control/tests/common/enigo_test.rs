use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    thread::sleep,
};

use anyhow::{anyhow, Result};
use control::{ActionControl, InputAction};
use enigo::{Axis, Button, InputResult, Key, Mouse, Settings};
use tungstenite::accept;

use super::browser_events::BrowserEvent;

const TIMEOUT: u64 = 5; // Number of minutes the test is allowed to run before timing out
                        // This is needed, because some of the websocket functions are blocking and
                        // would run indefinitely without a timeout if they don't receive a message
const INPUT_DELAY: u64 = 40; // Number of milliseconds to wait for the input to have an effect
const SCROLL_STEP: (i32, i32) = (16, 16); // (horizontal, vertical)

pub struct EnigoTest {
    action: ActionControl,
    websocket: tungstenite::WebSocket<TcpStream>,
}

impl EnigoTest {
    pub fn new(settings: &Settings) -> Self {
        env_logger::try_init().ok();
        EnigoTest::start_timeout_thread();
        let action = ActionControl::new(settings);
        let _ = &*super::browser::BROWSER_INSTANCE; // Launch Firefox
        let websocket = Self::websocket();

        std::thread::sleep(std::time::Duration::from_secs(5)); // Give Firefox some time to launch
        Self { action, websocket }
    }

    fn websocket() -> tungstenite::WebSocket<TcpStream> {
        let listener = TcpListener::bind("127.0.0.1:26541").unwrap();
        println!("TcpListener was created");
        let (stream, addr) = listener.accept().expect("Unable to accept the connection");
        println!("New connection was made from {addr:?}");
        let websocket = accept(stream).expect("Unable to accept connections on the websocket");
        println!("WebSocket was successfully created");
        websocket
    }

    fn send_message(&mut self, msg: &str) {
        println!("Sending message: {msg}");
        self.websocket.send(tungstenite::Message::Text(tungstenite::Utf8Bytes::from(msg))).expect("Unable to send the message");
        println!("Sent message");
    }

    fn read_message(&mut self) -> BrowserEvent {
        let message = self.websocket.read().unwrap();
        let Ok(browser_event) = BrowserEvent::try_from(message) else {
            panic!("Other text received");
        };
        assert!(!(browser_event == BrowserEvent::Close), "Received a Close event");
        browser_event
    }

    fn start_timeout_thread() {
        // Spawn a thread to handle the timeout
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(TIMEOUT * 60));
            println!("Test suite exceeded the maximum allowed time of {TIMEOUT} minutes.");
            std::process::exit(1); // Exit with error code
        });
    }

    // This does not work for all text or the library does not work properly
    pub fn text(&mut self, text: &str) -> Result<()> {
        println!("---------------------------------enigo.text({text:?}) start");
        self.send_message("ClearText");
        println!("Attempt to clear the text");
        assert_eq!(BrowserEvent::ReadyForText, self.read_message(), "Failed to get ready for the text");
        let res = self.action.handle_action(InputAction::WriteText(text.to_string()));
        std::thread::sleep(std::time::Duration::from_millis(INPUT_DELAY)); // Wait for input to have an effect
        self.send_message("GetText");

        let ev = self.read_message();
        if let BrowserEvent::Text(received_text) = ev {
            println!("received text: {received_text}");
            assert_eq!(text, received_text);
        } else {
            panic!("BrowserEvent was not a Text: {ev:?}");
        }
        println!("---------------------------------enigo.text({text:?}) was a success");
        res
    }

    pub fn key(&mut self, key_str: String) -> Result<()> {
        println!("---------------------------------enigo.key({key_str:?}) start");
        let input_action = InputAction::new("key_click".to_string(), HashMap::from([("key".to_string(), key_str)]))?;

        let key = match input_action {
            InputAction::KeyClick(k) => k.clone(),
            _ => panic!("Invalid action type"),
        };

        let res = self.action.handle_action(input_action);
        let ev: BrowserEvent = self.read_message();

        if let BrowserEvent::KeyDown(name) = ev {
            println!("browser received pressed key: {name}");
            let key_name = if let Key::Unicode(char) = &key {
                format!("{char}")
            } else {
                format!("{key:?}").to_lowercase()
            };
            assert_eq!(key_name, name.to_lowercase());
        } else {
            panic!("BrowserEvent was not a KeyDown: {ev:?}");
        }

        let ev: BrowserEvent = self.read_message();
        if let BrowserEvent::KeyUp(name) = ev {
            println!("browser received released key: {name}");
            let key_name = if let Key::Unicode(char) = &key {
                format!("{char}")
            } else {
                format!("{key:?}").to_lowercase()
            };
            assert_eq!(key_name, name.to_lowercase());
        } else {
            panic!("BrowserEvent was not a KeyUp: {ev:?}");
        }

        println!("---------------------------------enigo.key({key:?}) was a success");
        res
    }

    pub fn click(&mut self, action_type: String, start_box: String) -> Result<()> {
        println!("---------------------------------enigo.button({action_type:?},({start_box})) start");
        let input_action = InputAction::new(action_type, HashMap::from([("start_box".to_string(), start_box)]))?;
        let (x, y, button, count) = match input_action {
            InputAction::MouseLeftClick { x, y } => (x, y, Button::Left, 1),
            InputAction::MouseMiddleClick { x, y } => (x, y, Button::Middle, 1),
            InputAction::MouseRightClick { x, y } => (x, y, Button::Right, 1),
            InputAction::MouseLeftDoubleClick { x, y } => (x, y, Button::Left, 2),
            _ => panic!("Invalid action type"),
        };
        let res = self.action.handle_action(input_action);

        let ev = self.read_message();
        let mouse_position = if let BrowserEvent::MouseMove(_pos_rel, pos_abs) = ev {
            pos_abs
        } else {
            panic!("BrowserEvent was not a MouseMove: {ev:?}");
        };
        assert_eq!(x, mouse_position.0);
        assert_eq!(y, mouse_position.1);

        for _ in 0..count {
            let ev = self.read_message();
            if let BrowserEvent::MouseDown(name) = ev {
                println!("received pressed button: {name}");
                assert_eq!(button as u32, name);
            } else {
                panic!("BrowserEvent was not a MouseUp: {ev:?}");
            }
            let ev = self.read_message();
            if let BrowserEvent::MouseUp(name) = ev {
                println!("received released button: {name}");
                assert_eq!(button as u32, name);
            } else {
                panic!("BrowserEvent was not a MouseUp: {ev:?}");
            }
        }

        println!("---------------------------------enigo.button({button:?},({x},{y})) was a success");
        res
    }

    pub fn move_mouse(&mut self, action_type: String, start_box: String) -> Result<()> {
        println!("---------------------------------enigo.move_mouse() {start_box} start");
        let input_action = InputAction::new(action_type, HashMap::from([("start_box".to_string(), start_box)]))?;
        let (x, y) = match input_action {
            InputAction::MouseMove { x, y } => (x, y),
            _ => panic!("Invalid action type"),
        };
        let res = self.action.handle_action(input_action);

        let ev = self.read_message();
        let mouse_position = if let BrowserEvent::MouseMove(_pos_rel, pos_abs) = ev {
            pos_abs
        } else {
            panic!("BrowserEvent was not a MouseMove: {ev:?}");
        };
        assert_eq!(x, mouse_position.0);
        assert_eq!(y, mouse_position.1);
        println!("---------------------------------enigo.move_mouse() ({x},{y}) was a success");
        res
    }

    pub fn scroll(&mut self, action_type: String, start_box: String, direction_str: String, length: i32) -> Result<()> {
        println!("---------------------------------enigo.scroll({action_type:?},({start_box}),{direction_str},{length}) start");
        let input_action = InputAction::new(
            action_type,
            HashMap::from([
                ("start_box".to_string(), start_box),
                ("length".to_string(), length.to_string()),
                ("direction".to_string(), direction_str),
            ]),
        )?;
        let (x, y, length_with_sign, direction) = match input_action {
            InputAction::Scroll { x, y, length, direction } => (x, y, length, direction),
            _ => panic!("Invalid action type"),
        };
        let res = self.action.handle_action(input_action);
        let ev = self.read_message();
        let mouse_position = if let BrowserEvent::MouseMove(_pos_rel, pos_abs) = ev {
            pos_abs
        } else {
            panic!("BrowserEvent was not a MouseMove: {ev:?}");
        };
        assert_eq!(x, mouse_position.0);
        assert_eq!(y, mouse_position.1);

        sleep(std::time::Duration::from_millis(INPUT_DELAY)); // Wait for input to have an effect

        // On some platforms it is not possible to scroll multiple lines so we
        // repeatedly scroll. In order for this test to work on all platforms, both
        // cases are not differentiated
        let mut mouse_scroll;
        let mut step;
        let mut length = length;
        while length > 0 {
            let ev = self.read_message();
            (mouse_scroll, step) = if let BrowserEvent::MouseScroll(horizontal_scroll, vertical_scroll) = ev {
                match direction {
                    Axis::Horizontal => (horizontal_scroll, SCROLL_STEP.0),
                    Axis::Vertical => (vertical_scroll, SCROLL_STEP.1),
                }
            } else {
                panic!("BrowserEvent was not a MouseScroll: {ev:?}");
            };
            length -= mouse_scroll.abs() / step;
        }
        println!("---------------------------------enigo.scroll ({x}, {y}), {length_with_sign}, {direction:?} was a success");
        res
    }

    pub fn main_display(&self) -> InputResult<(i32, i32)> {
        let res = self.action.enigo.main_display();
        match res {
            Ok((x, y)) => {
                let (rdev_x, rdev_y) = rdev_main_display();
                println!("enigo display: {x},{y}");
                println!("rdev_display: {rdev_x},{rdev_y}");
                assert_eq!(x, rdev_x);
                assert_eq!(y, rdev_y);
            }
            Err(_) => todo!(),
        }
        res
    }

    // Edge cases don't work (mouse is at the left most border and can't move one to
    // the left)
    pub fn location(&self) -> Result<(i32, i32)> {
        let res = self.action.enigo.location().map_err(|e| anyhow!("{}", e));
        match res {
            Ok((x, y)) => {
                let (mouse_x, mouse_y) = mouse_position();
                println!("enigo_position: {x},{y}");
                println!("mouse_position: {mouse_x},{mouse_y}");
                assert_eq!(x, mouse_x);
                assert_eq!(y, mouse_y);
            }
            Err(_) => todo!(),
        }
        res
    }
}

fn rdev_main_display() -> (i32, i32) {
    use rdev::display_size;
    let (x, y) = display_size().unwrap();
    (x.try_into().unwrap(), y.try_into().unwrap())
}

fn mouse_position() -> (i32, i32) {
    use mouse_position::mouse_position::Mouse;

    if let Mouse::Position { x, y } = Mouse::get_mouse_position() {
        (x, y)
    } else {
        panic!("the crate mouse_location was unable to get the position of the mouse");
    }
}
