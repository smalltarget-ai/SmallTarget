use enigo::Settings;

mod common;
use common::enigo_test::EnigoTest;

#[test]
fn integration_browser_events() {
    println!("start test");
    let mut enigo = EnigoTest::new(&Settings::default());

    enigo.text("TestText❤️").unwrap();

    enigo.key("F12".to_string()).unwrap();
    enigo.key("Control".to_string()).unwrap();
    enigo.key("Backspace".to_string()).unwrap();
    enigo.key("PageUp".to_string()).unwrap();

    enigo.key("Backspace".to_string()).unwrap();
    enigo.key("Backspace".to_string()).unwrap();

    println!("Test mouse");
    // enigo.button(Button::Left, Click).unwrap();
    enigo.move_mouse("mouse_move".to_string(), "[100,100]".to_string()).unwrap();
    enigo.move_mouse("mouse_move".to_string(), "[200,200]".to_string()).unwrap();
    enigo.click("click".to_string(), "[200,200]".to_string()).unwrap();
    enigo.click("left_double".to_string(), "[250,250]".to_string()).unwrap();
    enigo.click("right_single".to_string(), "[300,300]".to_string()).unwrap();
    let (x, y) = enigo.location().unwrap();
    assert_eq!((200, 200), (x, y));

    // Stalls on Windows, macOS and Linux with x11rb
    enigo.scroll("scroll".to_string(), "[300,300]".to_string(), "up".to_string(), 1).unwrap();
    enigo.scroll("scroll".to_string(), "[300,300]".to_string(), "right".to_string(), 1).unwrap();

    enigo.main_display().unwrap();
    enigo.location().unwrap();
}
