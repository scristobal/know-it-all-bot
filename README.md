# know-it-all-bot

> **Warning**
> This is a legacy project, it has been replaced by [Chatlize](https://github.com/scristobal/chatlyze)

A Telegram bot that knows it all, powered by OpenAI's ChatGPT

The bot retains memory of previous interactions/messages as uses it as context for the next interactions. However, there is a limit of 8k per request context. The bot will notify when the context is close to the limit. In that case `/reset` will clear the conversation history.

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
