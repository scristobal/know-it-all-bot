use crate::openai_client::reply;

use teloxide::{prelude::*, utils::command::BotCommands, RequestError};
use tracing::{
    instrument, {error, info},
};
use uuid::Uuid;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
#[derive(Debug)]
pub enum Command {
    #[command()]
    Ask(String),
}

#[instrument]
pub async fn answer_cmd_repl(
    bot: teloxide::Bot,
    msg: Message,
    cmd: Command,
) -> Result<(), RequestError> {
    info!(user = msg.chat.username().unwrap_or("unknown"));

    let results = match cmd {
        Command::Ask(prompt) => reply(prompt).await,
    };

    match results {
        Err(e) => {
            let error_id = Uuid::new_v4().simple().to_string();

            error!(error_id, ?e);

            bot.send_message(
                msg.chat.id,
                format!("there was an error processing your request, you can use this ID to track the issue `{}`", error_id),
            )
            .await?;
        }
        Ok(results) => {
            for result in results {
                bot.send_message(msg.chat.id, result).await?;
            }
        }
    };
    Ok(())
}
