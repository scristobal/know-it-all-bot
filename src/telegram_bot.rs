use std::default;

use crate::openai_client::reply;

use teloxide::{
    dispatching::{dialogue::InMemStorage, UpdateHandler},
    filter_command,
    prelude::*,
    utils::command::BotCommands,
    RequestError,
};
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
    Reset,
    #[command()]
    Directive(String),
    #[command()]
    Ask(String),
}

#[derive(Debug, Default)]
pub enum State {
    #[default]
    Start,
    Directive(String),
    Chat(Vec<Msg>),
}

#[derive(Debug, serde::Serialize)]
pub struct Msg {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[instrument]
pub fn schema() -> UpdateHandler<()> {
    use dptree::case;

    let cmd_handler = filter_command::<Command, _>().branch(case![Command::Reset].endpoint(reset));

    todo!()
}

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), ()>;

async fn reset(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    todo!()
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
        _ => unimplemented!(),
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
