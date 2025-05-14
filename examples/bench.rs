use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::{Utc, TimeZone};
use std::time::Instant;
use std::fs::File;
use std::io::Write;

fn main() {
    let t0 = 1_700_000_000_000;
    let n = 300_000_000; // 300 миллионов трейдов
    let batch = 1000; // 1000 трейдов на одну свечу
    println!("Генерация {} трейдов ({} трейдов на свечу)...", n, batch);
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
            timestamp: Utc.timestamp_millis_opt(t0 + ((i / batch) as i64) * 60_000).unwrap(),
        })
        .collect();
    println!("Агрегация...");
    let generator = CandleGenerator::default();
    let start = Instant::now();
    let candles = generator.aggregate(trades.iter(), Timeframe::m1);
    let elapsed = start.elapsed();
    let candles_len = candles.len();
    let elapsed_secs = elapsed.as_secs_f64();
    let throughput = n as f64 / elapsed_secs;
    println!("Свечей сгенерировано: {}", candles_len);
    println!("Трейдов на свечу: {}", batch);
    println!("Время агрегации: {:.3?}", elapsed);
    println!("Пропускная способность: {:.2} трейдов/сек", throughput);
    // Запись результатов в файл
    let mut file = File::create("bench_output.txt").expect("cannot create bench_output.txt");
    writeln!(file, "trades={}", n).unwrap();
    writeln!(file, "candles={}", candles_len).unwrap();
    writeln!(file, "batch={}", batch).unwrap();
    writeln!(file, "elapsed_secs={:.6}", elapsed_secs).unwrap();
    writeln!(file, "throughput={:.2}", throughput).unwrap();
} 