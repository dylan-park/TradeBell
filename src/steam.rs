use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

use crate::models::*;

pub struct SteamClient {
    api_key: String,
    client: Client,
}

impl SteamClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    pub async fn get_active_trade_offers(
        &self,
        time_historical_cutoff: u64,
    ) -> Result<GetTradeOffersResponse> {
        let url = "https://api.steampowered.com/IEconService/GetTradeOffers/v1/";

        let response = self
            .client
            .get(url)
            .query(&[
                ("key", &self.api_key),
                ("get_received_offers", &"1".to_string()),
                ("get_sent_offers", &"1".to_string()),
                ("active_only", &"0".to_string()),
                ("historical_only", &"0".to_string()),
                (
                    "time_historical_cutoff",
                    &time_historical_cutoff.to_string(),
                ),
                ("format", &"json".to_string()),
            ])
            .send()
            .await
            .context("Failed to fetch trade offers")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Steam API error (GetTradeOffers): {}", error_text);
        }

        let result = response
            .json::<GetTradeOffersResponse>()
            .await
            .context("Failed to parse trade offers response")?;

        Ok(result)
    }

    pub async fn get_trade_history(&self, start_time: u64) -> Result<GetTradeHistoryResponse> {
        let url = "https://api.steampowered.com/IEconService/GetTradeHistory/v1/";

        let response = self
            .client
            .get(url)
            .query(&[
                ("key", &self.api_key),
                ("max_trades", &"10".to_string()),
                ("start_time", &start_time.to_string()),
                ("get_descriptions", &"0".to_string()),
                ("format", &"json".to_string()),
            ])
            .send()
            .await
            .context("Failed to fetch trade history")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Steam API error (GetTradeHistory): {}", error_text);
        }

        let text = response
            .text()
            .await
            .context("Failed to get response text")?;

        // Parse the response, logging the raw body if it fails.
        let result = serde_json::from_str::<GetTradeHistoryResponse>(&text)
            .with_context(|| format!("Failed to parse trade history response. Body: {}", text))?;

        Ok(result)
    }

    pub async fn get_asset_class_info(
        &self,
        appid: u32,
        class_instance_pairs: &[(String, String)],
    ) -> Result<HashMap<String, AssetClassInfo>> {
        if class_instance_pairs.is_empty() {
            return Ok(HashMap::new());
        }

        // Construct parameters: class_count is required, followed by numbered classid/instanceid pairs.

        let mut params: Vec<(String, String)> = vec![
            ("key".to_string(), self.api_key.clone()),
            ("format".to_string(), "json".to_string()),
            (
                "class_count".to_string(),
                class_instance_pairs.len().to_string(),
            ),
        ];

        for (i, (classid, instanceid)) in class_instance_pairs.iter().enumerate() {
            params.push((format!("classid{}", i), classid.clone()));
            params.push((format!("instanceid{}", i), instanceid.clone()));
        }

        let url = "https://api.steampowered.com/ISteamEconomy/GetAssetClassInfo/v0001/";
        // Use ISteamEconomy with explicit appid parameter.
        params.push(("appid".to_string(), appid.to_string()));

        let response = self
            .client
            .get(url)
            .query(&params)
            .send()
            .await
            .context("Failed to fetch asset class info")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Steam API error (GetAssetClassInfo): {}", error_text);
        }

        // Custom parsing because the result is flexible
        #[derive(Deserialize)]
        struct RawResponse {
            result: Option<HashMap<String, serde_json::Value>>,
        }

        let raw: RawResponse = response
            .json()
            .await
            .context("Failed to parse raw asset info")?;

        let mut final_map = HashMap::new();

        if let Some(result_map) = raw.result {
            for (k, v) in result_map {
                // Determine if valid AssetClassInfo.
                if let Ok(info) = serde_json::from_value::<AssetClassInfo>(v.clone()) {
                    final_map.insert(k, info);
                }
            }
        }

        Ok(final_map)
    }
}
