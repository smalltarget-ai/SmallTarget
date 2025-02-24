#[cfg(test)]
mod tests {
    use llm::call_ai::{get_system_prompt, open_ai_protocal_call, OpenAiProtocalCallPayload};

    #[tokio::test]
    async fn test_call_ai() {
        let system_prompt = get_system_prompt("zh");
        let result = open_ai_protocal_call(OpenAiProtocalCallPayload::new(
            "http://d3:8000".to_string(),
            "ui-tars".to_string(),
            "".to_string(),
            system_prompt,
            true,
        ))
        .await;
        println!("result: {:?}", result);
    }
}
