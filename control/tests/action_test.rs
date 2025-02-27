mod action_test {
    use anyhow::Result;
    use control::action::InputAction;
    use std::collections::HashMap;

    #[test]
    fn test_action() -> Result<()> {
        let input_action = InputAction::new("click".to_string(), HashMap::from([("start_box".to_string(), "[100,200,100,200]".to_string())]))?;

        let (x, y) = match input_action {
            InputAction::MouseLeftClick { x, y } => (x, y),
            _ => panic!("Invalid action type"),
        };
        assert_eq!(x, 100);
        assert_eq!(y, 200);
        Ok(())
    }
}
