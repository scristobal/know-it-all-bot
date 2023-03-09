use crate::openai_client::reply;
use log;

use teloxide::{prelude::*, utils::command::BotCommands, RequestError};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command()]
    Ask(String),
}

pub async fn answer_cmd_repl(
    bot: teloxide::Bot,
    msg: Message,
    cmd: Command,
) -> Result<(), RequestError> {
    log::info!("new cmd from {}", msg.chat.username().unwrap_or("unknown"));

    let results = match cmd {
        Command::Ask(prompt) => reply(prompt).await,
    };

    match results {
        Err(e) => match e {
            async_openai::error::OpenAIError::Reqwest(e) => log::error!("{}", e),
            async_openai::error::OpenAIError::ApiError(e) => log::error!("{}", e.message),
            async_openai::error::OpenAIError::JSONDeserialize(e) => log::error!("{}", e),
            async_openai::error::OpenAIError::FileSaveError(e) => log::error!("{}", e),
            async_openai::error::OpenAIError::FileReadError(e) => log::error!("{}", e),
            async_openai::error::OpenAIError::StreamError(e) => log::error!("{}", e),
            async_openai::error::OpenAIError::InvalidArgument(e) => log::error!("{}", e),
        },
        Ok(results) => {
            for result in results {
                bot.send_message(msg.chat.id, result).await?;
            }
        }
    };
    Ok(())
}
