# know-it-all-bot

A Telegram bot that knows it all, powered by OpenAI's ChatGPT

## Usage

There is only one command `/ask <prompt>`

## Limitations

The bot has no memory of previous interactions/messages. So each interaction starts a blank new context.
There is no feedback while the answer is generated, neither the answers are streamed.

## Setup

You need Rust 1.67, optionally Docker, docker compose and a fly.io account for deployment.

## Config

The bot needs two secrets, as environment variables to configure:

- get your `OPENAI_API_KEY` from [OpenAI](https://platform.openai.com/account/api-keys)
- ask the [@BotFather](https://telegram.me/BotFather) to get your `TELOXIDE_TOKEN`

The program will try to read the environment variables from a `.env` file at the root of the repository.

```bash
TELOXIDE_TOKEN=<your-telegram-bot-token>
OPENAI_API_KEY=<your-openai-token>
```

## Run

Simply use cargo ❤️
>`cargo run`
