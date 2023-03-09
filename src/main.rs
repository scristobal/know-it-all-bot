use dotenv::dotenv;
use know_it_all_bot::bot::{answer_cmd_repl, Command};
use std::io::Result;
use teloxide::{types::Message, utils::command::BotCommands, Bot};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    match dotenv() {
        Ok(_) => log::info!("Loaded .env file"),
        Err(_) => log::info!("No .env file found. Falling back to environment variables"),
    }

    log::info!("Starting bot...");
    let bot = teloxide::Bot::from_env();

    tokio::spawn(teloxide::commands_repl(
        bot,
        move |bot: Bot, msg: Message, cmd: Command| async move { answer_cmd_repl(bot, msg, cmd).await },
        Command::ty(),
    ));

    tokio::signal::ctrl_c().await
}
