use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::Utc;

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
        // ... добавьте больше трейдов для демонстрации цепочки
    ];
    let generator = CandleGenerator::default();
    let chain = generator.aggregate_chain(trades.iter());
    for tf in [Timeframe::m1, Timeframe::m5, Timeframe::m15, Timeframe::m30, Timeframe::h1, Timeframe::h4, Timeframe::d1] {
        println!("=== {:?} ===", tf);
        for candle in chain.get(&tf).unwrap_or(&vec![]) {
            println!("{:?}", candle);
        }
    }
} 