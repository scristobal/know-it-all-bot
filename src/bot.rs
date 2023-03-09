use crate::openai_client::reply;
use log;

use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands, RequestError};

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
    log::info!("new job from {}", msg.chat.username().unwrap_or("unknown"));

    let results = match cmd {
        Command::Ask(prompt) => reply(prompt).await,
    };

    match results {
        Err(e) => match e {
            OpenAIError => log::error!(""),
        },
        Ok(results) => {
            for result in results {
                bot.send_message(msg.chat.id, result).await?;
            }
        }
    };
    Ok(())
}
