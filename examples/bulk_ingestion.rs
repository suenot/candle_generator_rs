use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::Utc;

fn main() {
    let t0 = 1_700_000_000_000;
    let trades: Vec<_> = (0..1000)
        .map(|i| Trade {
            instrument: Instrument {
                pair: Pair { base_id: "BTC".into(), quote_id: "USDT".into() },
                exchange: "binance".into(),
                market_type: MarketType::Spot,
            },
            id: format!("t{}", i),
            price: 100.0 + (i % 10) as f64,
            amount: 1.0,
            side: Side::Buy,
            timestamp: Utc.timestamp_millis_opt(t0 + i * 60_000).unwrap(),
        })
        .collect();
    let generator = CandleGenerator::default();
    let candles = generator.aggregate(trades.iter(), Timeframe::m1);
    println!("Всего свечей: {}", candles.len());
    println!("Первая свеча: {:?}", candles.first());
    println!("Последняя свеча: {:?}", candles.last());
} 