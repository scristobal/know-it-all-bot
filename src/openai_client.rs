use async_openai::{
    error::OpenAIError,
    types::{
        ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, CreateChatCompletionResponse,
    },
    Client,
};
use tracing::instrument;

#[instrument]
pub async fn reply(
    msgs: &[ChatCompletionRequestMessage],
    system: Option<&str>,
    model: Option<&str>,
) -> Result<CreateChatCompletionResponse, OpenAIError> {
    let client = Client::new();

    let system_msg = ChatCompletionRequestMessage {
        role: async_openai::types::Role::System,
        content: system
            .unwrap_or("You are GTP-4 a Telegram chat bot")
            .to_string(),
        name: None,
    };

    let mut req_msgs = vec![system_msg];

    req_msgs.extend_from_slice(msgs);

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model(model.unwrap_or("gpt-4"))
        .messages(msgs)
        .build()?;

    client.chat().create(request).await
}
