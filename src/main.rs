mod cache;
mod config;
mod models;
mod steam;
mod telegram;

use anyhow::Result;
use chrono::Utc;
use log::{error, info, warn};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use tokio::time::sleep;

use crate::{
    cache::ItemCache,
    config::Config,
    models::{Asset, TradeOffer},
    steam::SteamClient,
    telegram::TelegramBot,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // 1. Load Config
    let config = Config::load()?;
    let polling_interval = Duration::from_secs(config.polling_interval_seconds);

    // 2. Initialize Cache (Shared)
    let cache = ItemCache::new("cache.json")?;
    let cache = Arc::new(cache); // thread-safe wrapper inside ItemCache already uses RwLock, but wrapping struct in Arc is good for cloning

    // 3. Initialize Telegram Bot (Shared)
    let bot = Arc::new(TelegramBot::new(
        config.telegram_token.clone(),
        config.telegram_chat_id.clone(),
    ));

    info!(
        "Starting Steam Trade Watcher with {} accounts...",
        config.accounts.len()
    );

    // 4. Spawn Tasks
    let mut handles = vec![];

    for account in config.accounts {
        let cache_clone = cache.clone();
        let bot_clone = bot.clone();
        let client = SteamClient::new(account.api_key.clone());
        let account_name = account.name.clone();

        let handle = tokio::spawn(async move {
            info!("[{}] Poller started.", account_name);

            // Track processed trade IDs to prevent duplicates within this session.
            let mut processed_trades: HashSet<String> = HashSet::new();

            // Ignore trades processed before program startup.
            let boot_time = Utc::now().timestamp() as u64;
            let mut last_poll_time = boot_time;

            loop {
                // Poll
                match client.get_active_trade_offers(last_poll_time).await {
                    Ok(offers) => {
                        let mut new_trades = Vec::new();

                        // Check received offers
                        for offer in offers.response.trade_offers_received {
                            if offer.trade_offer_state == 3
                                && !processed_trades.contains(&offer.tradeofferid)
                            {
                                if offer.time_updated < boot_time {
                                    continue;
                                }
                                new_trades.push(offer);
                            }
                        }

                        // Also check sent offers (if we care about completed sent trades? usually yes)
                        for offer in offers.response.trade_offers_sent {
                            if offer.trade_offer_state == 3
                                && !processed_trades.contains(&offer.tradeofferid)
                            {
                                // Ignore trades made before program startup
                                if offer.time_updated < boot_time {
                                    continue;
                                }
                                new_trades.push(offer);
                            }
                        }

                        // Process new trades
                        for trade in new_trades {
                            info!(
                                "[{}] Found new completed trade: {}",
                                account_name, trade.tradeofferid
                            );
                            processed_trades.insert(trade.tradeofferid.clone());

                            // Fetch History to get items
                            match process_completed_trade(&client, &cache_clone, &trade).await {
                                Ok(Some(notification_msg)) => {
                                    let full_msg = format!(
                                        "<b>Account: {}</b>\n{}",
                                        account_name, notification_msg
                                    );
                                    if let Err(e) = bot_clone.send_notification(&full_msg).await {
                                        error!(
                                            "[{}] Failed to send notification: {}",
                                            account_name, e
                                        );
                                    }
                                }
                                Ok(None) => {
                                    warn!(
                                        "[{}] Trade history not found for {}, skipping notification.",
                                        account_name, trade.tradeofferid
                                    );
                                }
                                Err(e) => error!(
                                    "[{}] Failed to process trade details: {}",
                                    account_name, e
                                ),
                            }
                        }

                        // Update cutoff timestamp.
                        last_poll_time = Utc::now().timestamp() as u64 - 60;
                    }
                    Err(e) => {
                        error!("[{}] Metadata poll failed: {}", account_name, e);
                    }
                }

                sleep(polling_interval).await;
            }
        });
        handles.push(handle);
    }

    // Wait forever
    for h in handles {
        let _ = h.await;
    }

    Ok(())
}

async fn process_completed_trade(
    client: &SteamClient,
    cache: &ItemCache,
    trade: &TradeOffer,
) -> Result<Option<String>> {
    // 1. Get History
    // We search near the trade update time.
    let history_response = client.get_trade_history(trade.time_updated - 60).await?;

    // Identify the trade history entry closest to the `time_updated` of the trade offer.
    // This heuristic associates the trade offer with its history since `tradeofferid` is not always present in history.

    let mut best_match: Option<crate::models::TradeHistory> = None;
    for hist in history_response.response.trades {
        // time_init is when trade started/completed.
        let time_diff = (hist.time_init as i64 - trade.time_updated as i64).abs();
        if time_diff < 120 {
            // within 2 minutes
            best_match = Some(hist);
            break; // Assume first (newest) one is correct? API returns newest first.
        }
    }

    let hist = match best_match {
        Some(h) => h,
        None => return Ok(None),
    };

    let mut message_lines = Vec::new();
    message_lines.push(format!("Trade ID: {}", hist.tradeid));

    // Process Received
    if let Some(assets) = hist.assets_received
        && !assets.is_empty()
    {
        message_lines.push("\n<b>Received:</b>".to_string());
        let names = resolve_asset_names(client, cache, &assets).await?;
        for name in names {
            message_lines.push(format!("- {}", name));
        }
    }

    // Process Given
    if let Some(assets) = hist.assets_given
        && !assets.is_empty()
    {
        message_lines.push("\n<b>Given:</b>".to_string());
        let names = resolve_asset_names(client, cache, &assets).await?;
        for name in names {
            message_lines.push(format!("- {}", name));
        }
    }

    Ok(Some(message_lines.join("\n")))
}

async fn resolve_asset_names(
    client: &SteamClient,
    cache: &ItemCache,
    assets: &[Asset],
) -> Result<Vec<String>> {
    let mut names = Vec::new();
    let mut to_fetch: Vec<(String, String)> = Vec::new();
    let mut asset_map: HashMap<(String, String), String> = HashMap::new(); // (classid, instanceid) -> Name

    // 1. Check Cache
    for asset in assets {
        if let Some(info) = cache.get(&asset.classid, &asset.instanceid) {
            asset_map.insert(
                (asset.classid.clone(), asset.instanceid.clone()),
                info.market_hash_name,
            );
        } else {
            to_fetch.push((asset.classid.clone(), asset.instanceid.clone()));
        }
    }

    // 2. Fetch missing
    if !to_fetch.is_empty() {
        // Unique pairs only
        to_fetch.sort();
        to_fetch.dedup();

        // Group assets by AppID to batch API requests.
        let mut by_appid: HashMap<u32, Vec<(String, String)>> = HashMap::new();
        for asset in assets {
            // Find if this specific pair needs fetching
            if !asset_map.contains_key(&(asset.classid.clone(), asset.instanceid.clone())) {
                by_appid
                    .entry(asset.appid)
                    .or_default()
                    .push((asset.classid.clone(), asset.instanceid.clone()));
            }
        }

        for (appid, pairs) in by_appid {
            if pairs.is_empty() {
                continue;
            }
            // Dedup again just in case
            let mut unique_pairs = pairs.clone();
            unique_pairs.sort();
            unique_pairs.dedup();

            match client.get_asset_class_info(appid, &unique_pairs).await {
                Ok(info_map) => {
                    for (cid, iid) in unique_pairs {
                        // Try key = cid
                        if let Some(info) = info_map.get(&cid) {
                            cache.insert(&cid, &iid, info.clone()).unwrap_or_default();
                            asset_map
                                .insert((cid.clone(), iid.clone()), info.market_hash_name.clone());
                        }
                        // Fallback: check composite key format
                        else if let Some(info) = info_map.get(&format!("{}_{}", cid, iid)) {
                            cache.insert(&cid, &iid, info.clone()).unwrap_or_default();
                            asset_map
                                .insert((cid.clone(), iid.clone()), info.market_hash_name.clone());
                        }
                    }
                }
                Err(e) => error!("Failed to enrich items for app {}: {}", appid, e),
            }
        }
    }

    // 3. Construct names list
    for asset in assets {
        if let Some(name) = asset_map.get(&(asset.classid.clone(), asset.instanceid.clone())) {
            names.push(name.clone());
        } else {
            names.push(format!(
                "Unknown Asset ({})",
                asset.market_name_or_fallback()
            ));
        }
    }

    Ok(names)
}

trait AssetFallback {
    fn market_name_or_fallback(&self) -> String;
}

impl AssetFallback for Asset {
    fn market_name_or_fallback(&self) -> String {
        // We don't have the name in the Asset struct itself from history?
        // History `assets_received` often just has IDs.
        format!("ID: {}", self.assetid)
    }
}
