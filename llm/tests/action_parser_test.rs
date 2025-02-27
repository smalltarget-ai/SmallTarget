#[cfg(test)]
mod tests {
    use llm::action_parser::{parse_action_vlm, PredictionParsed};
    use std::collections::HashMap;

    mod bc_mode {
        use llm::promps::FACTOR;
        use super::*;

        #[test]
        fn should_correctly_parse_input_with_thought() {
            let input = "Thought: I need to click this button\nAction: click(start_box='(100,200)')";
            let result = parse_action_vlm(input, FACTOR, "bc");
            let expected = vec![PredictionParsed {
                reflection: None,
                thought: "I need to click this button".to_string(),
                action_type: "click".to_string(),
                action_inputs: {
                    let mut map = HashMap::new();
                    map.insert("start_box".to_string(), "[0.1,0.2,0.1,0.2]".to_string());
                    map
                },
            }];
            assert_eq!(result, expected);
        }

        #[test]
        fn test_thought_with_custom_factors() {
            let input = "Thought: I need to click this button\nAction: click(start_box='(100,200)')";
            let factor = (1366.0, 768.0);
            
            let result = parse_action_vlm(input, factor, "bc");
            
            let mut expected_inputs = HashMap::new();
            expected_inputs.insert(
                "start_box".to_string(),
                "[0.07320644,0.26041666,0.07320644,0.26041666]".to_string()
            );
            
            assert_eq!(result, vec![
                PredictionParsed {
                    reflection: None,
                    thought: "I need to click this button".to_string(),
                    action_type: "click".to_string(),
                    action_inputs: expected_inputs,
                }
            ]);
        }

        #[test]
        fn should_correctly_parse_input_with_reflection_and_action_summary() {
            let input = "Reflection: This is a reflection\nAction_Summary: This is a summary\nAction: type(text='Hello', start_box='(300,400)')";
            let result = parse_action_vlm(input, FACTOR, "bc");

            let expected = vec![PredictionParsed {
                reflection: Some("This is a reflection".to_string()),
                thought: "This is a summary".to_string(),
                action_type: "type".to_string(),
                action_inputs: {
                    let mut map = HashMap::new();
                    map.insert("text".to_string(), "Hello".to_string());
                    map.insert("start_box".to_string(), "[0.3,0.4,0.3,0.4]".to_string());
                    map
                },
            }];
            assert_eq!(result, expected);
        }

        #[test]
        fn should_handle_multiple_actions() {
            let input = "Thought: Perform multiple actions\nAction: click(start_box='(100,200)')\n\ntype(text='Hello', start_box='(300,400)')";
            let result = parse_action_vlm(input, FACTOR, "bc");

            let expected = vec![
                PredictionParsed {
                    thought: "Perform multiple actions".to_string(),
                    reflection: None,
                    action_type: "click".to_string(),
                    action_inputs: {
                        let mut map = HashMap::new();
                        map.insert("start_box".to_string(), "[0.1,0.2,0.1,0.2]".to_string());
                        map
                    },
                },
                PredictionParsed {
                    thought: "Perform multiple actions".to_string(),
                    reflection: None,
                    action_type: "type".to_string(),
                    action_inputs: {
                        let mut map = HashMap::new();
                        map.insert("text".to_string(), "Hello".to_string());
                        map.insert("start_box".to_string(), "[0.3,0.4,0.3,0.4]".to_string());
                        map
                    },
                },
            ];
            assert_eq!(result, expected);
        }
    }

    mod o1_mode {
        use llm::promps::FACTOR;

        use super::*;

        #[test]
        fn should_correctly_parse_o1_format_input() {
            let input = r#"<Thought>I need to perform this action</Thought>
            Action_Summary: Click and type text
            Action: click(start_box='(100,200)')
            </Output>"#;
            let result = parse_action_vlm(input, FACTOR, "o1");

            let expected = vec![PredictionParsed {
                reflection: None,
                thought: "I need to perform this action\n<Action_Summary>\nClick and type text".to_string(),
                action_type: "click".to_string(),
                action_inputs: {
                    let mut map = HashMap::new();
                    map.insert("start_box".to_string(), "[0.1,0.2,0.1,0.2]".to_string());
                    map
                },
            }];
            assert_eq!(result, expected);
        }

        #[test]
        fn should_handle_complex_o1_format_input() {
            let input = r#"<Thought>Complex operation</Thought>
            Action_Summary: Multiple sequential actions
            Action: click(start_box='(100,200)')
            </Output>"#;
            let result = parse_action_vlm(input, FACTOR, "o1");

            let expected = vec![PredictionParsed {
                reflection: None,
                thought: "Complex operation\n<Action_Summary>\nMultiple sequential actions".to_string(),
                action_type: "click".to_string(),
                action_inputs: {
                    let mut map = HashMap::new();
                    map.insert("start_box".to_string(), "[0.1,0.2,0.1,0.2]".to_string());
                    map
                },
            }];
            assert_eq!(result, expected);
        }
    }

    mod edge_cases {
        use llm::promps::FACTOR;

        use super::*;

        #[test]
        fn should_handle_input_without_action_keyword() {
            let input = r#"click(start_box="(100,200)")"#;
            let result = parse_action_vlm(input, FACTOR, "bc");

            let expected = vec![PredictionParsed {
                action_inputs: {
                    let mut map = HashMap::new();
                    map.insert("start_box".to_string(), "[0.1,0.2,0.1,0.2]".to_string());
                    map
                },
                action_type: "click".to_string(),
                reflection: None,
                thought: "".to_string(),
            }];
            assert_eq!(result, expected);
        }

        #[test]
        fn should_handle_empty_action_input() {
            let input = "Thought: Empty action\nAction:";
            let result = parse_action_vlm(input, FACTOR, "bc");

            let expected = vec![PredictionParsed {
                action_inputs: HashMap::new(),
                action_type: "".to_string(),
                reflection: None,
                thought: "Empty action".to_string(),
            }];
            assert_eq!(result, expected);
        }
    }
}
