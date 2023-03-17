use crate::openai_client::reply;

use serde::Serialize;
use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        UpdateHandler,
    },
    filter_command,
    prelude::*,
    types::{MediaKind, MessageKind, UpdateKind},
    utils::command::BotCommands,
};
use tracing::{
    instrument, {error, info},
};
use uuid::Uuid;

use dptree::case;

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

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Role {
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
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[instrument]
pub fn schema() -> UpdateHandler<anyhow::Error> {
    let is_private = |msg: Message| msg.chat.is_private();

    let cmd_handler = filter_command::<Command, _>().branch(case![Command::Reset].endpoint(reset));

    let msg_handler = Update::filter_message()
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

async fn has_mention(bot: Bot, msg: Message) -> HandlerResult {
    let me = bot.get_me().await?.mention();

    let msg_text = msg.text().unwrap();

    if !msg.chat.is_private() && !msg_text.starts_with(&me) {
        return Ok(());
    }

    let msg_text = msg_text.replace(&me, "");

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

    // bot.send_message(msg.chat.id, "Starting a new conversation")
    //    .await?;

    new_msg(
        bot,
        dialogue,
        msg,
        vec![Msg {
            role: Role::System,
            content: "You are GTP-4 a Telegram chat bot ".to_string(),
            name: None,
        }],
    )
    .await?;

    Ok(())
}

async fn new_msg(
    bot: Bot,
    dialogue: InMemDialogue,
    msg: Message,
    mut msgs: Vec<Msg>,
) -> HandlerResult {
    let me = bot.get_me().await?.mention();

    let msg_text = msg.text().unwrap();

    if !msg.chat.is_private() && !msg_text.starts_with(&me) {
        return Ok(());
    }

    let msg_text = msg_text.replace(&me, "");

    msgs.push(Msg {
        role: Role::User,
        content: msg_text,
        name: msg.chat.username().map(|user| user.to_string()),
    });

    let results = reply(
        &msgs
            .clone()
            .into_iter()
            .map(|m| m.into())
            .collect::<Vec<_>>(),
    )
    .await;

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
                bot.send_message(msg.chat.id, &result).await?;
                msgs.push(Msg {
                    role: Role::Assistant,
                    content: result,
                    name: None,
                })
            }
            info!(?msgs);

            dialogue.update(State::Chat(msgs)).await.unwrap();
        }
    };

    Ok(())
}
