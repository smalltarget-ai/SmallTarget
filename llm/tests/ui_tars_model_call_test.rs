#[cfg(test)]
mod tests {
    use anyhow::Result;
    use llm::{get_system_prompt, openai_request, parse_action_vlm, promps::FACTOR, OpenAiProtocalCallPayload, MAX_PIXELS};
    use openai_api_rs::v1::chat_completion::{ChatCompletionMessage, Content, ContentType, ImageUrl, ImageUrlType, MessageRole};
    use small_target_image::{image_from_path, image_resize, image_to_base64};
    use std::{fs::create_dir_all, path::PathBuf};

    #[tokio::test]
    async fn test_call_ai() -> Result<()> {
        // test image path
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("screen_shot_macos.png");
        println!("Path to screen_shot_macos.png: {:?}", path);

        // resize image to max pixels and convert to base64
        let img = image_from_path(&path.to_str().unwrap())?;
        let width = img.width() as f32;
        let height = img.height() as f32;
        println!("origin image width: {:?}, height: {:?}", width, height);
        let resized_img = image_resize(img, MAX_PIXELS)?;

        //save resized image to file
        let mut resized_img_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        resized_img_path.push("target");
        resized_img_path.push("test_call_ai");
        //create test_call_ai directory if not exists
        if !resized_img_path.exists() {
            create_dir_all(resized_img_path.to_str().unwrap())?;
        }
        resized_img_path.push("resized_screen_shot_macos.jpeg");
        resized_img.save(resized_img_path.to_str().unwrap())?;
        println!("resized image saved to: {:?}", resized_img_path);

        let image_base64 = image_to_base64(resized_img)?;

        // construct user message content
        let input_content = "打开微信app".to_string();
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
        let payload = OpenAiProtocalCallPayload::new("http://d3:8000/v1".to_string(), "ui-tars".to_string(), "api_token".to_string(), messages, history, 0.0, 0.7, 1000);
        let result = openai_request(payload).await?;
        println!("action_parser_result: {:?}", result);
        let action_parser_result = parse_action_vlm(result.choices[0].message.content.as_ref().unwrap(), FACTOR, "bc");
        for action in action_parser_result {
            println!("action: {:?}", action.action_type);
            println!("thought: {:?}", action.thought);
            println!("reflection: {:?}", action.reflection);
            for (key, value) in action.action_inputs {
                if key == "start_box" || key == "end_box" {
                    //convert string to json array
                    let json_value: Vec<f32> = serde_json::from_str(&value)?;
                    //macos retina display, so need to divide by 2
                    println!(
                        "key: {:?}, x1: {:?}, y1: {:?}, x2: {:?}, y2: {:?}",
                        key,
                        (json_value[0] * width / 2.0).round(),
                        (json_value[1] * height / 2.0).round(),
                        (json_value[2] * width / 2.0).round(),
                        (json_value[3] * height / 2.0).round()
                    );
                }
            }
        }
        Ok(())
    }
}
