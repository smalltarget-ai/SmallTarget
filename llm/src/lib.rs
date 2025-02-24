
pub mod openai_request;
pub use openai_request::{OpenAiProtocalCallPayload, openai_request, MAX_PIXELS};

pub mod promps;
pub use promps::get_system_prompt;
