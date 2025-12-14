# TradeBell

[![Rust](https://img.shields.io/badge/rust-1.88.0%2B-orange.svg)](https://www.rust-lang.org/) [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT) [![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)


TradeBell is a lightweight Rust utility that monitors Steam trade offers. It polls for completed trades across multiple accounts and sends detailed notifications to a Telegram chat.

## Features

- **Multi-Account Monitoring**: Track trades for as many Steam accounts as you need.
- **Telegram Integration**: Receive instant alerts with lists of items given and received.
- **Smart Caching**: Caches item details locally to reduce API usage and improve performance.
- **Security Focused and Lightweight**: You hold your own API keys, all calls are made directly using each service's respective API with no wrappers or external services.

## Quick Start

### Prerequisites

To build and run this project, you need:

- **Rust**: The Rust toolchain (cargo, rustc) 1.88 or higher. Install it from [rustup.rs](https://rustup.rs).
- **Steam Web API Key**: A valid API key for each account you wish to monitor. Obtain one at [steamcommunity.com/dev/apikey](https://steamcommunity.com/dev/apikey).
- **Telegram Bot**: A bot token and chat ID. Talk to [@BotFather](https://t.me/botfather) on Telegram to create a bot.

### Local Installation

1.  Clone this repository:
    ```bash
    git clone https://github.com/dylan-park/TradeBell.git
    cd tradebell
    ```
2.  Run the application:
    ```bash
    cargo build --release
    ```
On startup, the bot will log that it has started polling. When a new trade is completed (accepted), you will receive a message on Telegram listing the account and items exchanged.

### Docker

Build and run with Docker:

```bash
docker build -t tradebell .
docker run -d \
  --name tradebell \
  --hostname tradebell \
  --restart unless-stopped \
  -v ./config.json:/app/config.json \
  -v ./cache.json:/app/cache.json \
  tradebell
```

### Docker Compose

For the easiest deployment:

```bash
docker-compose up -d
```

See [docker-compose.yaml](docker-compose.yaml) for configuration options.

## Configuration

The application requires a `config.json` file in the working directory.

1.  Create a file named `config.json`.
2.  Paste the following structure and fill in your details:

```json
{
  "telegram_token": "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11",
  "telegram_chat_id": "-123456789",
  "polling_interval_seconds": 30,
  "accounts": [
    {
      "name": "Main Account",
      "api_key": "YOUR_STEAM_API_KEY_HERE"
    },
    {
      "name": "Storage Alt",
      "api_key": "ANOTHER_STEAM_API_KEY_HERE"
    }
  ]
}
```
- **telegram_token**: Bot token to a telegram bot you controll.
- **telegram_chat_id**: Telegram chat id to a chat or channel you controll and/or the bot is already in.
- **polling_interval_seconds**: How often (in seconds) the bot checks Steam for updates. 30 seconds is currently recommended to avoid rate limits.
- **accounts**
  - **name**: Name for the account (Only used for logging and notifications, can differ from your actual Steam username).
  - **api_key**: Steam Web API Key for the account you wish to track.

## Troubleshooting

- **No notifications?** Check the console logs. If Steam's API is lagging, the bot will warn you that trade history is missing. It will simply wait for the next successful API call.
- **Rate Limits?** If you see 429 errors, try increasing the `polling_interval_seconds`.

## Future Work

- [ ] Combine duplicate items with count (Item x5)
- [ ] Multiple notification types

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
