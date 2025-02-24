#[cfg(test)]
mod tests {
    use anyhow::Result;
    use control::input_control::{input_control_handler, InputAction};

    #[tokio::test]
    async fn test_input_control_handler() -> Result<()> {
        let action = InputAction::MouseMove { x: 0, y: 0 };
        let result = input_control_handler(action).await?;
        assert!(result);
        Ok(())
    }
}
