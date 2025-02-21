#[cfg(test)]
mod tests {
    use anyhow::Result;
    use control::input_control::{InputAction, input_control_handler};

    #[tokio::test]
    async fn test_input_control_handler() -> Result<()> {
        let action = InputAction::KeyPress("enter".to_string());
        let result = input_control_handler(action).await?;
        assert!(result);
        Ok(())
    }
}