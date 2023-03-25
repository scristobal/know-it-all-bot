use std::fmt::Display;

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
    utils::{command::BotCommands, markdown::escape},
};
use tracing::{error, instrument};
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
    State,
    Reset,
    Mute,
    Chat,
    History,
}

#[derive(Debug, Clone)]
pub enum State {
    Chatting(Vec<Msg>),
    Muted,
}

impl Default for State {
    fn default() -> Self {
        Self::Chatting(vec![])
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Muted => write!(f, "Sate: muted"),
            State::Chatting(msgs) => {
                f.write_fmt(format_args!("State: chatting ({} msgs)", msgs.len()))
            }
        }
    }
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
        .branch(case![Command::State].endpoint(state))
        .branch(case![State::Muted].branch(case![Command::Chat].endpoint(start)))
        .branch(
            case![State::Chatting(msgs)]
                .branch(case![Command::History].endpoint(history))
                .branch(case![Command::Reset].endpoint(reset))
                .branch(case![Command::Mute].endpoint(mute)),
        );

    let msg_handler = Update::filter_message()
        .branch(cmd_handler)
        .branch(case![State::Muted].endpoint(muted))
        .branch(case![State::Chatting(msgs)].endpoint(new_msg))
        .endpoint(invalid);

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(msg_handler)
}

type InMemDialogue = Dialogue<State, InMemStorage<State>>;

type HandlerResult = Result<(), anyhow::Error>;

async fn state(bot: Bot, dialogue: InMemDialogue, msg: Message) -> HandlerResult {
    let state = dialogue.get().await?;

    let reply_txt = match state {
        None => "No active conversation".to_string(),
        Some(state) => format!("State: {:}", state),
    };

    bot.send_message(msg.chat.id, reply_txt).await?;

    Ok(())
}

async fn start(bot: Bot, dialogue: InMemDialogue, msg: Message) -> HandlerResult {
    dialogue.update(State::Chatting(vec![])).await?;

    let me = bot.get_me().await?.mention();

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

async fn invalid(bot: Bot, dialogue: InMemDialogue, msg: Message) -> HandlerResult {
    dialogue.exit().await?;

    let error_id = Uuid::new_v4().simple().to_string();

    error!(error_id);

    bot.send_message(
                msg.chat.id,
                format!("there was an error processing your request, you can use this ID to track the issue `{}`", error_id),
            ).parse_mode(ParseMode::MarkdownV2)
            .await?;

    Ok(())
}

async fn history(bot: Bot, dialogue: InMemDialogue, msg: Message) -> HandlerResult {
    let msgs = dialogue.get().await?;

    let chat_id = msg.chat.id;

    match msgs {
        None => bot.send_message(chat_id, "No active conversation").await?,

        Some(state) => match state {
            State::Muted => bot.send_message(chat_id, "Bot is muted").await?,
            State::Chatting(messages) => {
                let mut reply_txt = String::new();

                for msg in messages {
                    let name = match msg.name {
                        Some(name) => name,
                        None => "System".to_string(),
                    };

                    reply_txt.push_str(&format!("{}: {}\n", name, msg.content));
                }

                let reply_txt = if reply_txt.is_empty() {
                    "`there is no history`".to_string()
                } else {
                    reply_txt
                };

                bot.send_message(chat_id, escape(&reply_txt))
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?
            }
        },
    };

    Ok(())
}

async fn reset(bot: Bot, dialogue: InMemDialogue, msg: Message) -> HandlerResult {
    dialogue.update(State::Chatting(vec![])).await?;

    bot.send_message(
        msg.chat.id,
        "`Conversation history has been deleted. Still in chat, REPL mode`",
    )
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
    // GUARDS
    // check if the bot is mentioned in non-private chats (groups, and so)
    // if not mentioned and not in private chat, do nothing
    // otherwise remove metion and go ahead

    let me = bot.get_me().await?.mention();

    let msg_text = msg.text().unwrap();

    if !msg.chat.is_private() && !msg_text.starts_with(&me) {
        return Ok(());
    }

    let msg_text = msg_text.replace(&me, "");

    if msg_text.is_empty() {
        return Ok(());
    }

    // end of bot mention check
    // TODO: move this to a .chain method

    let username = msg.from().and_then(|user| user.username.clone());

    msgs.push(Msg {
        role: Role::User,
        content: msg_text,
        name: username,
    });

    let results = reply(
        &msgs
            .clone()
            .into_iter()
            .map(|m| m.into())
            .collect::<Vec<_>>(),
        None,
        None,
    )
    .await;

    match results {
        Err(e) => {
            let error_id = Uuid::new_v4().simple().to_string();

            error!(error_id, ?e);

            bot.send_message(
                msg.chat.id,
                format!("there was an error processing your request, you can use this ID to track the issue `{}`", error_id),
            ).parse_mode(ParseMode::MarkdownV2)
            .await?;
        }
        Ok(results) => {
            let botname = &bot.get_me().await?.username;

            let mut reply_txt = String::new();

            for choice in results.choices {
                let result = choice.message.content;

                reply_txt.push_str(&result);

                msgs.push(Msg {
                    role: Role::Assistant,
                    content: result,
                    name: botname.clone(),
                });
            }

            dialogue.update(State::Chatting(msgs)).await.unwrap();

            reply_txt = escape(&reply_txt);

            if let Some(usage) = results.usage {
                reply_txt.push_str(&format!(
                    "\n\n`usage {} tokens = {} prompt + {} completion`",
                    usage.total_tokens, usage.prompt_tokens, usage.completion_tokens
                ));

                if usage.total_tokens > 6000 {
                    reply_txt.push_str("\n`Reaching 8k limit, consider running /reset soon`")
                }
            }

            bot.send_message(msg.chat.id, &reply_txt)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        }
    };

    Ok(())
}
