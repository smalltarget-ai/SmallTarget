use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct PredictionParsed {
    pub reflection: Option<String>,
    pub thought: String,
    pub action_type: String,
    pub action_inputs: HashMap<String, String>,
}

pub fn parse_action_vlm(text: &str, factor: (f32, f32), mode: &str) -> Vec<PredictionParsed> {
    let text = text.trim();
    let mut reflection = None;
    let mut thought = None;
    let action_str;

    match mode {
        "bc" => {
            if text.starts_with("Thought:") {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"Thought: ([\s\S]+?)(?:\s*Action:|\s*$)").unwrap();
                }
                if let Some(caps) = RE.captures(text) {
                    thought = Some(caps[1].trim().to_string());
                }
            } else if text.starts_with("Reflection:") {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"Reflection: ([\s\S]+?)Action_Summary: ([\s\S]+?)(?:\s*Action:|\s*$)").unwrap();
                }
                if let Some(caps) = RE.captures(text) {
                    thought = Some(caps[2].trim().to_string());
                    reflection = Some(caps[1].trim().to_string());
                }
            } else if text.starts_with("Action_Summary:") {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"Action_Summary: (.+?)(?:\s*Action:|\s*$)").unwrap();
                }
                if let Some(caps) = RE.captures(text) {
                    thought = Some(caps[1].trim().to_string());
                }
            }

            let parts: Vec<&str> = text.split("Action:").collect();
            action_str = parts.last().unwrap_or(&"").to_string();
        }
        "o1" => {
            lazy_static! {
                static ref THOUGHT_RE: Regex = Regex::new(r"<Thought>\s*(.*?)\s*</Thought>").unwrap();
                static ref SUMMARY_RE: Regex = Regex::new(r"(?s)Action_Summary:\s*(.*?)\s*Action:").unwrap();
                static ref ACTION_RE: Regex = Regex::new(r"(?s)Action:\s*(.*?)\s*</Output>").unwrap();
            }

            let thought_content = THOUGHT_RE.captures(text).and_then(|c| c.get(1)).map(|m| m.as_str());
            let action_summary = SUMMARY_RE.captures(text).and_then(|c| c.get(1)).map(|m| m.as_str());
            let action_content = ACTION_RE.captures(text).and_then(|c| c.get(1)).map(|m| m.as_str());

            thought = Some(format!("{}\n<Action_Summary>\n{}", thought_content.unwrap_or(""), action_summary.unwrap_or("")));
            action_str = action_content.unwrap_or("").to_string();
        }
        _ => panic!("Invalid mode"),
    }
    let all_actions: Vec<&str> = action_str.split("\n\n").collect();
    let mut actions = Vec::new();

    for raw_str in all_actions {
        let cleaned_str = raw_str.replace('\n', r"\n").trim_start().to_string();
        let action_instance = parse_action(&cleaned_str);

        let mut action_inputs = HashMap::new();
        let mut action_type = String::new();

        if let Some(act) = action_instance {
            action_type = act.function;
            for (param_name, param_value) in act.args {
                let trimmed = param_value.trim().to_string();
                if param_name.contains("start_box") || param_name.contains("end_box") {
                    let numbers: Vec<f32> = trimmed
                        .trim_matches(|c| c == '(' || c == ')' || c == '[' || c == ']')
                        .split(',')
                        .filter_map(|s| s.parse::<f32>().ok())
                        .enumerate()
                        .map(|(index, num)| num / if index % 2 == 0 { factor.0 } else { factor.1 })
                        .collect();

                    let numbers = if numbers.len() == 2 {
                        vec![numbers[0], numbers[1], numbers[0], numbers[1]]
                    } else {
                        numbers
                    };

                    action_inputs.insert(param_name, serde_json::to_string(&numbers).unwrap());
                } else {
                    action_inputs.insert(param_name, trimmed);
                }
            }
        }

        actions.push(PredictionParsed {
            reflection: reflection.clone(),
            thought: thought.clone().unwrap_or_default(),
            action_type,
            action_inputs,
        });
    }

    actions
}
#[derive(Debug)]
pub struct ParsedAction {
    function: String,
    args: HashMap<String, String>,
}

//for example parse: click(start_box='(530,965)')  
//return ParsedAction { function: "click", args: {"start_box": "(530,965)"} }
fn parse_action(action_str: &str) -> Option<ParsedAction> {
    lazy_static! {
        static ref FUNC_RE: Regex = Regex::new(r"^(\w+)\((.*)\)$").unwrap();
        static ref ARG_RE: Regex = Regex::new(r#"((?:[^,'"]|'[^']*'|"[^"]*")+)"#).unwrap();
    }

    let cleaned = action_str.trim();
    let caps = FUNC_RE.captures(cleaned)?;
    let function_name = caps.get(1)?.as_str().to_string();
    let args_str = caps.get(2)?.as_str().trim();

    let mut args: HashMap<String, String> = HashMap::new();
    if !args_str.is_empty() {
        for pair in ARG_RE.find_iter(args_str) {
            let pair = pair.as_str();
            println!("pair: {:?}", pair);
            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() != 2 {
                continue;
            }
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().trim_matches(|c| c == '\'' || c == '"').to_string();

            args.insert(key, value);
        }
    }
    Some(ParsedAction { function: function_name, args })
}
