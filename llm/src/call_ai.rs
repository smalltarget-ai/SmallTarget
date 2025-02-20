use anyhow::{Result, Context};
use reqwest::Client;
use serde_json::{json, Value};

pub async fn call_ai(prompt: String, context: String, expect_json: bool,modle_name:String,base_url:String) -> Result<String> {
    let client = Client::new();

    let messages = vec![
        json!({
            "role": "system",
            "content": context
        }),
        json!({
            "role": "user",
            "content": prompt
        })
    ];

    let mut body = json!({
        "model": modle_name,
        "messages": messages,
        "temperature": 0,
        "stream": false
    });

    if expect_json {
        body["response_format"] = json!({"type": "json_object"});
    }

    let response: Value = client.post(format!("{}/v1/chat/completions",base_url))
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

    if expect_json {
        let json_value: Value = serde_json::from_str(&content)
            .context("response content is not valid JSON")?;
        
        if let Some(array) = json_value["response"].as_array() {
            // If "response" is an array, join its elements with newlines
            Ok(array.iter()
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
