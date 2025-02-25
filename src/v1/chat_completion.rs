impl_builder_methods!(
    ChatCompletionRequest,
    temperature: f64,       // 控制输出的随机性（0-2），值越高输出越随机
    top_p: f64,             // 核采样概率阈值（0-1），与temperature二选一
    n: i64,                 // 生成多少个聊天完成选项
    response_format: Value, // 指定响应格式（如JSON模式）
    stream: bool,           // 是否使用流式传输
    stop: Vec<String>,      // 遇到这些字符串时停止生成
    max_tokens: i64,        // 生成的最大token数
    presence_penalty: f64,  // 主题重复惩罚（-2.0-2.0），正值降低重复
    frequency_penalty: f64, // 词汇重复惩罚（-2.0-2.0），正值降低重复
    logit_bias: HashMap<String, i32>, // 特定token的偏差调整（-100到100）
    user: String,           // 终端用户标识（用于滥用检测）
    seed: i64,              // 随机种子（确保确定性输出）
    tools: Vec<Tool>,       // 可用的工具列表（函数调用）
    parallel_tool_calls: bool, // 是否允许并行工具调用
    tool_choice: ToolChoiceType // 控制工具调用策略（自动/强制/指定工具）
); 