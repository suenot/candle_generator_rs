use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::Utc;
use std::time::Instant;

fn main() {
    let t0 = 1_700_000_000_000;
    let n = 1_000_000; // 1 миллион трейдов
    println!("Генерация {} трейдов...", n);
    let trades: Vec<_> = (0..n)
        .map(|i| Trade {
            instrument: Instrument {
                pair: Pair { base_id: "BTC".into(), quote_id: "USDT".into() },
                exchange: "binance".into(),
                market_type: MarketType::Spot,
            },
            id: format!("t{}", i),
            price: 100.0 + (i % 10) as f64,
            amount: 1.0,
            side: if i % 2 == 0 { Side::Buy } else { Side::Sell },
            timestamp: Utc.timestamp_millis_opt(t0 + i * 60_000).unwrap(),
        })
        .collect();
    println!("Агрегация...");
    let generator = CandleGenerator::default();
    let start = Instant::now();
    let candles = generator.aggregate(trades.iter(), Timeframe::m1);
    let elapsed = start.elapsed();
    println!("Свечей сгенерировано: {}", candles.len());
    println!("Время агрегации: {:.3?}", elapsed);
    println!("Пропускная способность: {:.2} трейдов/сек", n as f64 / elapsed.as_secs_f64());
} 