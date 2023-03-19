use async_openai::{
    error::OpenAIError,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
    Client,
};
use tracing::{info, instrument};

#[instrument]
pub async fn reply(msgs: &[ChatCompletionRequestMessage]) -> Result<Vec<String>, OpenAIError> {
    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-4")
        .messages(msgs)
        .build()?;

    let response = client.chat().create(request).await?;

    info!(response = ?response);

    Ok(response
        .choices
        .into_iter()
        .map(|choice| choice.message.content)
        .collect())
}
