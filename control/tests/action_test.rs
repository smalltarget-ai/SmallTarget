#[cfg(test)]
mod tests {
    use anyhow::Result;
    use control::action::{ActionControl, InputAction};
    use enigo::{Enigo, Mouse, Settings};

    #[tokio::test]
    async fn test_input_control_handler() -> Result<()> {
        let mut action_control = ActionControl::new();
        action_control.handle_action(InputAction::MouseMove { x: 916, y: 1078 })?;
        let enigo = Enigo::new(&Settings::default()).unwrap();
        println!("screen dimensions: {:?}", enigo.main_display().unwrap());
        println!("mouse location: {:?}", enigo.location().unwrap());
        Ok(())
    }
}
