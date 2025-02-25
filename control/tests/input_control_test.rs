#[cfg(test)]
mod tests {
    use anyhow::Result;
    use control::input_control::{input_control_handler, InputAction};
    use enigo::{Enigo, Mouse, Settings};

    #[tokio::test]
    async fn test_input_control_handler() -> Result<()> {
        let action = InputAction::MouseMove { x: 916, y: 1078 };
        let result = input_control_handler(action).await?;
        assert!(result);
        let enigo = Enigo::new(&Settings::default()).unwrap();
        println!("screen dimensions: {:?}", enigo.main_display().unwrap());
        println!("mouse location: {:?}", enigo.location().unwrap());
        Ok(())
    }
}
