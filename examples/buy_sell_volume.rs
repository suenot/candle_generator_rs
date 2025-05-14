use candle_generator::{CandleGenerator, CandleConfig, CandleMetric, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::Utc;
use std::collections::HashMap;

struct BuySellVolume;
impl CandleMetric for BuySellVolume {
    fn update(&self, trade: &Trade, candle: &mut candle_generator::Candle) {
        let buy = candle.custom.get("buy_volume").cloned().unwrap_or(0.0);
        let sell = candle.custom.get("sell_volume").cloned().unwrap_or(0.0);
        match trade.side {
            Side::Buy => {
                candle.custom.insert("buy_volume".to_string(), buy + trade.amount);
            }
            Side::Sell => {
                candle.custom.insert("sell_volume".to_string(), sell + trade.amount);
            }
            _ => {}
        }
    }
    fn aggregate(&self, _src: &[candle_generator::Candle], _dst: &mut candle_generator::Candle) {}
}

fn main() {
    let trades = vec![
        Trade {
            instrument: Instrument {
                pair: Pair { base_id: "BTC".into(), quote_id: "USDT".into() },
                exchange: "binance".into(),
                market_type: MarketType::Spot,
            },
            id: "t1".into(),
            price: 50000.0,
            amount: 0.1,
            side: Side::Buy,
            timestamp: Utc.timestamp_millis_opt(1714000000000).unwrap(),
        },
        Trade {
            instrument: Instrument {
                pair: Pair { base_id: "BTC".into(), quote_id: "USDT".into() },
                exchange: "binance".into(),
                market_type: MarketType::Spot,
            },
            id: "t2".into(),
            price: 50100.0,
            amount: 0.2,
            side: Side::Sell,
            timestamp: Utc.timestamp_millis_opt(1714000060000).unwrap(),
        },
    ];
    let mut config = CandleConfig::default();
    config.custom_metrics.push(Box::new(BuySellVolume));
    let gen = CandleGenerator { config };
    let candles = gen.aggregate(trades.iter(), Timeframe::m1);
    for candle in candles {
        println!("buy_volume = {:?}, sell_volume = {:?}", candle.custom.get("buy_volume"), candle.custom.get("sell_volume"));
    }
} 