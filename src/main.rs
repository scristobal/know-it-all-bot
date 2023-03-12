use dotenv::dotenv;
use know_it_all_bot::{
    health_checker,
    telegram_bot::{answer_cmd_repl, Command},
};
use std::io::Result;
use teloxide::prelude::*;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    match dotenv() {
        Ok(_) => info!("Loaded .env file"),
        Err(_) => info!("No .env file found. Falling back to environment variables"),
    }

    info!("Starting bot...");
    let bot = teloxide::Bot::from_env();

    tokio::spawn(Command::repl(bot, answer_cmd_repl));

    tokio::spawn(health_checker::run(([0, 0, 0, 0], 8080)));

    tokio::signal::ctrl_c().await
}
