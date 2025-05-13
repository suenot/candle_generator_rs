use chrono::{DateTime, Utc};

// Timeframe codes use lowercase (e.g., m1, h1, d1) to avoid ambiguity with monthly candles (M1), per .cursor/rules/terms.md and industry standards.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Timeframe {
    m1,
    m5,
    m15,
    m30,
    h1,
    h4,
    d1,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pair {
    pub base_id: String,
    pub quote_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarketType {
    Spot,
    Futures,
    Margin,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Instrument {
    pub pair: Pair,
    pub exchange: String,
    pub market_type: MarketType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Side {
    Buy,
    Sell,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trade {
    pub instrument: Instrument,
    pub id: String,
    pub price: f64,
    pub amount: f64,
    pub side: Side,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Candle {
    pub instrument: Instrument,
    pub interval: Timeframe,
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: u64,
    pub volume_usdt: Option<f64>,
}

pub enum UsdtVolumeSource {
    Fixed(f64),
    Callback(Box<dyn Fn(&Pair, DateTime<Utc>) -> Option<f64> + Send + Sync>),
    None,
} 