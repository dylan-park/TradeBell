use serde::{Deserialize, Serialize};

// --- GetTradeOffers ---

#[derive(Debug, Deserialize, Serialize)]
pub struct GetTradeOffersResponse {
    pub response: TradeOffersResponseData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradeOffersResponseData {
    #[serde(default)]
    pub trade_offers_received: Vec<TradeOffer>,
    #[serde(default)]
    pub trade_offers_sent: Vec<TradeOffer>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TradeOffer {
    pub tradeofferid: String,
    pub trade_offer_state: i32,
    pub message: Option<String>,
    pub time_created: u64,
    pub time_updated: u64,
    pub accountid_other: i64,
}

// --- GetTradeHistory ---

#[derive(Debug, Deserialize, Serialize)]
pub struct GetTradeHistoryResponse {
    pub response: TradeHistoryResponseData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradeHistoryResponseData {
    pub total_trades: Option<u32>,
    pub more: Option<bool>,
    #[serde(default)]
    pub trades: Vec<TradeHistory>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TradeHistory {
    pub tradeid: String,
    pub steamid_other: String,
    pub time_init: u64,
    pub assets_received: Option<Vec<Asset>>,
    pub assets_given: Option<Vec<Asset>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Asset {
    pub appid: u32,
    pub contextid: String,
    pub assetid: String,
    pub classid: String,
    pub instanceid: String,
    pub amount: String,
    #[serde(default)] // Optional fields for mapping purposes.
    pub new_assetid: Option<String>,
    pub new_contextid: Option<String>,
}

// --- GetAssetClassInfo ---

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AssetClassInfo {
    pub icon_url: Option<String>,
    pub name: String,
    pub market_hash_name: String,
    pub market_name: String,
    pub name_color: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_trade_offer_deserialization() {
        let json_data = json!({
            "tradeofferid": "123456",
            "trade_offer_state": 3,
            "message": "Hello",
            "time_created": 1600000000,
            "time_updated": 1600000060,
            "accountid_other": 987654321
        });

        let offer: TradeOffer =
            serde_json::from_value(json_data).expect("Failed to parse TradeOffer");

        assert_eq!(offer.tradeofferid, "123456");
        assert_eq!(offer.trade_offer_state, 3);
        assert_eq!(offer.message, Some("Hello".to_string()));
        assert_eq!(offer.time_created, 1600000000);
        assert_eq!(offer.accountid_other, 987654321);
    }

    #[test]
    fn test_asset_deserialization() {
        let json_data = json!({
            "appid": 440,
            "contextid": "2",
            "assetid": "5000",
            "classid": "100",
            "instanceid": "1",
            "amount": "1"
        });

        let asset: Asset = serde_json::from_value(json_data).expect("Failed to parse Asset");

        assert_eq!(asset.appid, 440);
        assert_eq!(asset.assetid, "5000");
        assert_eq!(asset.classid, "100");
    }
}
