use dotenv::dotenv;
use know_it_all_bot::{
    health_checker,
    telegram_bot::{answer_cmd_repl, Command},
};
use std::io::Result;
use teloxide::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    match dotenv() {
        Ok(_) => log::info!("Loaded .env file"),
        Err(_) => log::info!("No .env file found. Falling back to environment variables"),
    }

    log::info!("Starting bot...");
    let bot = teloxide::Bot::from_env();

    tokio::spawn(Command::repl(bot, answer_cmd_repl));

    tokio::spawn(health_checker::run(([0, 0, 0, 0], 8080)));

    tokio::signal::ctrl_c().await
}
