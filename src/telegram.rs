use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;

#[derive(Clone)]
pub struct TelegramBot {
    token: String,
    chat_id: String,
    client: Client,
}

impl TelegramBot {
    pub fn new(token: String, chat_id: String) -> Self {
        Self {
            token,
            chat_id,
            client: Client::new(),
        }
    }

    pub async fn send_notification(&self, message: &str) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);

        let payload = json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": "HTML"
        });

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send Telegram request")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Telegram API error: {}", error_text);
        }

        Ok(())
    }
}
