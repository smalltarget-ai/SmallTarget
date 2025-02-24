use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{json, Value};

pub struct OpenAiProtocalCallPayload {
    base_url: String,
    model_name: String,
    user_content: String,
    system_content: String,
    expect_json: bool,
    api_key: String,
}
impl OpenAiProtocalCallPayload {
    pub fn new(
        base_url: String,
        model_name: String,
        user_content: String,
        system_content: String,
        expect_json: bool,
        api_key: String,
    ) -> Self {
        Self {
            base_url,
            model_name,
            user_content,
            system_content,
            expect_json,
            api_key,
        }
    }
}
pub fn get_system_prompt(language: &str) -> String {
    format!(
        r#"You are a GUI agent. You are given a task and your action history, with screenshots. You need to perform the next action to complete the task.
            ## Output Format
            Thought: ...
            Action: ...
            ## Action Space
            click(start_box='[x1, y1, x2, y2]')
            left_double(start_box='[x1, y1, x2, y2]')
            right_single(start_box='[x1, y1, x2, y2]')
            drag(start_box='[x1, y1, x2, y2]', end_box='[x3, y3, x4, y4]')
            hotkey(key='')
            type(content='') #If you want to submit your input, use "\\n" at the end of `content`.
            scroll(start_box='[x1, y1, x2, y2]', direction='down or up or right or left')
            wait() #Sleep for 5s and take a screenshot to check for any changes.
            finished()
            call_user() # Submit the task and call the user when the task is unsolvable, or when you need the user's help.

            ## Note
            - Use {} in `Thought` part.
            - Write a small plan and finally summarize your next action (with its target element) in one sentence in `Thought` part.

            ## User Instruction
        "#,
        if language == "zh" {
            "Chinese"
        } else {
            "English"
        }
    )
}

pub async fn open_ai_protocal_call(payload: OpenAiProtocalCallPayload) -> Result<String> {
    let client = Client::new();

    let messages = vec![
        json!({
            "role": "system",
            "content": payload.system_content
        }),
        json!({
            "role": "user",
            "content": payload.user_content
        }),
    ];

    let mut body = json!({
        "model": payload.model_name,
        "messages": messages,
        "temperature": 0,
        "stream": false
    });

    if payload.expect_json {
        body["response_format"] = json!({"type": "json_object"});
    }

    let response: Value = client
        .post(format!("{}/v1/chat/completions", payload.base_url))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", payload.api_key))
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    // Log the entire response for debugging
    // println!("raw api response: {}", serde_json::to_string_pretty(&response)?);

    if response.get("error").is_some() {
        // If there's an error, return it as a string
        let error_message = response["error"]["message"]
            .as_str()
            .unwrap_or("Unknown error")
            .to_string();
        return Err(anyhow::anyhow!(error_message));
    }

    let content = response["choices"]
        .get(0)
        .and_then(|choice| choice["message"]["content"].as_str())
        .context("failed to extract content from response")?
        .to_string();

    if payload.expect_json {
        let json_value: Value =
            serde_json::from_str(&content).context("response content is not valid JSON")?;

        if let Some(array) = json_value["response"].as_array() {
            // If "response" is an array, join its elements with newlines
            Ok(array
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("\n"))
        } else if let Some(response_str) = json_value["response"].as_str() {
            // If "response" is a string, return it directly
            Ok(response_str.to_string())
        } else {
            // If "response" is neither an array nor a string, return the whole JSON
            Ok(content)
        }
    } else {
        Ok(content)
    }
}
