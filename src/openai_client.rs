use async_openai::{
    error::OpenAIError,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
    Client,
};

pub async fn reply(msgs: &[ChatCompletionRequestMessage]) -> Result<Vec<String>, OpenAIError> {
    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo")
        .messages(msgs)
        .build()?;

    let response = client.chat().create(request).await?;

    Ok(response
        .choices
        .into_iter()
        .map(|choice| choice.message.content)
        .collect())
}
