#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use llm::{get_system_prompt, openai_request, OpenAiProtocalCallPayload, MAX_PIXELS};
    use openai_api_rs::v1::chat_completion::{
        ChatCompletionMessage, Content, ContentType, ImageUrl, ImageUrlType, MessageRole,
    };
    use small_target_image::{image_from_path, image_resize, image_to_base64};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_call_ai() -> Result<()> {
        // test image path
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("screen_shot_macos.png");
        println!("Path to screen_shot_macos.png: {:?}", path);

        // resize image to max pixels and convert to base64
        let mut img = image_from_path(&path.to_str().unwrap())?;
        img = image_resize(img, MAX_PIXELS)?;
        let image_base64 = image_to_base64(img)?;

        // construct user message content
        let input_content = "open wechat".to_string();
        let system_prompt = get_system_prompt("zh");
        let first_content = format!("{}{}", system_prompt, input_content);
        let user_content = ChatCompletionMessage {
            role: MessageRole::user,
            content: Content::Text(first_content),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };
        let image_message = ChatCompletionMessage {
            role: MessageRole::user,
            content: Content::ImageUrl(vec![ImageUrl {
                text: None,
                r#type: ContentType::image_url,
                image_url: Some(ImageUrlType { url: image_base64 }),
            }]),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };
        let messages = vec![user_content, image_message];
        let history = vec![];
        let payload = OpenAiProtocalCallPayload::new(
            "http://d3:8000/v1".to_string(),
            "ui-tars".to_string(),
            "api_token".to_string(),
            messages,
            history,
            0.0,
            0.7,
            1000,
        );
        let result = openai_request(payload).await?;
        println!("result: {:?}", result);
        Ok(())
    }
}
