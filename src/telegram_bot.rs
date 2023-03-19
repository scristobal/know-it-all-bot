use crate::openai_client::reply;

use serde::Serialize;
use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        UpdateHandler,
    },
    filter_command,
    prelude::*,
    types::ParseMode,
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
    Mute,
    Chat,
}

#[derive(Debug, Default, Clone)]
pub enum State {
    #[default]
    Muted,
    Chatting(Vec<Msg>),
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
    let cmd_handler = filter_command::<Command, _>()
        .branch(case![Command::Reset].endpoint(reset))
        .branch(case![Command::Chat].endpoint(start))
        .branch(case![Command::Mute].endpoint(mute));

    let msg_handler = Update::filter_message()
        .branch(cmd_handler)
        .branch(case![State::Muted].endpoint(muted))
        .branch(case![State::Chatting(msgs)].endpoint(new_msg));

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(msg_handler)
}

type InMemDialogue = Dialogue<State, InMemStorage<State>>;

type HandlerResult = Result<(), anyhow::Error>;

async fn start(bot: Bot, dialogue: InMemDialogue, msg: Message) -> HandlerResult {
    dialogue.update(State::Chatting(vec![])).await?;

    info!(?bot);

    let me = bot.get_me().await?.mention();

    info!("{}", me);

    let reply_txt = if msg.chat.is_private() {
        "`Starting private chat mode, REPL`".to_string()
    } else {
        format!(
            "`Starting group chat mode, REPL. Prepend messages with {}`",
            me
        )
    };

    bot.send_message(msg.chat.id, reply_txt)
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

async fn mute(bot: Bot, dialogue: InMemDialogue, msg: Message) -> HandlerResult {
    dialogue.update(State::Muted).await?;

    bot.send_message(msg.chat.id, "`Muted until futher notice`")
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

async fn reset(bot: Bot, dialogue: InMemDialogue, msg: Message) -> HandlerResult {
    dialogue.exit().await?;

    bot.send_message(msg.chat.id, "`Conversation history has been deleted`")
        .parse_mode(ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

async fn muted() -> HandlerResult {
    // if the bot is muted do nothing
    Ok(())
}

async fn new_msg(
    bot: Bot,
    dialogue: InMemDialogue,
    msg: Message,
    mut msgs: Vec<Msg>,
) -> HandlerResult {
    // check if the bot is mentioned in non-private chats (groups, and so)
    // if not mentioned and not in private chat, do nothing
    // otherwise remove metion and go ahead

    let me = bot.get_me().await?.mention();

    let msg_text = msg.text().unwrap();

    if !msg.chat.is_private() && !msg_text.starts_with(&me) {
        return Ok(());
    }

    let msg_text = msg_text.replace(&me, "");

    // end of bot mention check
    // TODO: move this to a .chain method

    if msgs.is_empty() {
        msgs.push(Msg {
            role: Role::System,
            content: "You are GTP-4 a Telegram chat bot".to_string(),
            name: None,
        })
    }

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

            dialogue.update(State::Chatting(msgs)).await.unwrap();
        }
    };

    Ok(())
}
