use std::default;

use crate::openai_client::reply;

use serde::Serialize;
use teloxide::{
    dispatching::{
        dialogue::{self, GetChatId, InMemStorage, InMemStorageError},
        UpdateHandler,
    },
    dptree::di,
    filter_command,
    prelude::*,
    types::{MediaKind, MediaText, MessageKind, UpdateKind},
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
}

#[derive(Debug, Default, Clone)]
pub enum State {
    #[default]
    Start,
    Chat(Vec<Msg>),
}

#[derive(Debug, Clone, Serialize)]
enum Role {
    System,
    User,
    Assistant,
}

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Role::System => "system".to_string(),
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Msg {
    role: Role,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[instrument]
pub fn schema() -> UpdateHandler<anyhow::Error> {
    use dptree::case;

    let cmd_handler = filter_command::<Command, _>().branch(case![Command::Reset].endpoint(reset));

    let msg_handler = Update::filter_message()
        .filter(|m: Update| {
            if let UpdateKind::Message(msg) = m.kind {
                if let MessageKind::Common(msg) = msg.kind {
                    if let MediaKind::Text(msg) = msg.media_kind {
                        info!(  ?msg.text)
                    }
                }
            }

            true
        })
        .branch(cmd_handler)
        .branch(case![State::Start].endpoint(start))
        .branch(case![State::Chat(msgs)].endpoint(new_msg));

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(msg_handler)
}

type InMemDialogue = Dialogue<State, InMemStorage<State>>;

type HandlerResult = Result<(), anyhow::Error>;

async fn reset(bot: Bot, dialogue: InMemDialogue, msg: Message) -> HandlerResult {
    dialogue.exit().await?;
    bot.send_message(msg.chat.id, "Conversation history has been deleted")
        .await?;
    Ok(())
}

async fn start(bot: Bot, dialogue: InMemDialogue, msg: Message) -> HandlerResult {
    dialogue
        .update(State::Chat(vec![Msg {
            role: Role::System,
            content: "You are a chat bot".to_string(),
            name: None,
        }]))
        .await?;
    bot.send_message(msg.chat.id, "Starting a new conversation")
        .await?;
    Ok(())
}

async fn new_msg(
    bot: Bot,
    dialogue: InMemDialogue,
    msg: Message,
    mut msgs: Vec<Msg>,
) -> HandlerResult {
    msgs.push(Msg {
        role: Role::User,
        content: msg.text().unwrap().to_string(),
        name: msg.chat.username().map(|user| user.to_string()),
    });

    bot.send_message(msg.chat.id, format!("got so far {:?}", &msgs))
        .await?;

    dialogue.update(State::Chat(msgs)).await.unwrap();

    Ok(())
}

/*
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
*/
