use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::Utc;

fn main() {
    let t0 = 1_700_000_000_000;
    let t1 = t0 + 60_000;
    let trades = vec![
        Trade {
            instrument: Instrument {
                pair: Pair { base_id: "BTC".into(), quote_id: "USDT".into() },
                exchange: "binance".into(),
                market_type: MarketType::Spot,
            },
            id: "t1".into(),
            price: 100.0,
            amount: 1.0,
            side: Side::Buy,
            timestamp: Utc.timestamp_millis_opt(t0).unwrap(),
        },
        Trade {
            instrument: Instrument {
                pair: Pair { base_id: "BTC".into(), quote_id: "USDT".into() },
                exchange: "binance".into(),
                market_type: MarketType::Spot,
            },
            id: "t2".into(),
            price: 110.0,
            amount: 2.0,
            side: Side::Sell,
            timestamp: Utc.timestamp_millis_opt(t1).unwrap(),
        },
        Trade {
            instrument: Instrument {
                pair: Pair { base_id: "BTC".into(), quote_id: "USDT".into() },
                exchange: "binance".into(),
                market_type: MarketType::Spot,
            },
            id: "t3".into(),
            price: 120.0,
            amount: 0.5,
            side: Side::Buy,
            timestamp: Utc.timestamp_millis_opt(t0 + 10_000).unwrap(), // out-of-order
        },
    ];
    let generator = CandleGenerator::default();
    let mut m1 = generator.aggregate(trades.iter(), Timeframe::m1);
    m1.sort_by_key(|c| c.timestamp);
    for candle in m1 {
        println!("{:?}", candle);
    }
} 