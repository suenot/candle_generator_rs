use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Timeframe codes use lowercase (e.g., m1, h1, d1) to avoid ambiguity with monthly candles (M1), per .cursor/rules/terms.md and industry standards.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Timeframe {
    m1,
    m5,
    m15,
    m30,
    h1,
    h4,
    d1,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pair {
    pub base_id: String,
    pub quote_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketType {
    Spot,
    Futures,
    Margin,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Instrument {
    pub pair: Pair,
    pub exchange: String,
    pub market_type: MarketType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trade {
    pub instrument: Instrument,
    pub id: String,
    pub price: f64,
    pub amount: f64,
    pub side: Side,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candle {
    pub instrument: Instrument,
    pub interval: Timeframe,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "o")]
    pub open: f64,
    #[serde(rename = "h")]
    pub high: f64,
    #[serde(rename = "l")]
    pub low: f64,
    #[serde(rename = "c")]
    pub close: f64,
    #[serde(rename = "v")]
    pub volume: f64,
    #[serde(rename = "tc")]
    pub trade_count: u64,
    #[serde(rename = "vusdt")]
    pub volume_usdt: Option<f64>,
    /// Кастомные метрики (buy/sell volume, VWAP и др.), AGI-ready
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, f64>,
}

pub enum UsdtVolumeSource {
    Fixed(f64),
    Callback(Box<dyn Fn(&Pair, DateTime<Utc>) -> Option<f64> + Send + Sync>),
    None,
} 