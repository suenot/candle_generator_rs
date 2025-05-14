use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::Utc;
use rand::seq::SliceRandom;

fn main() {
    let t0 = 1_700_000_000_000;
    let mut trades: Vec<_> = (0..1000)
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
    let mut rng = rand::thread_rng();
    trades.shuffle(&mut rng);
    let generator = CandleGenerator::default();
    let mut candles_unsorted = generator.aggregate(trades.iter(), Timeframe::m1);
    let mut trades_sorted = trades.clone();
    trades_sorted.sort_by_key(|t| t.timestamp);
    let mut candles_sorted = generator.aggregate(trades_sorted.iter(), Timeframe::m1);
    candles_unsorted.sort_by_key(|c| c.timestamp);
    candles_sorted.sort_by_key(|c| c.timestamp);
    println!("Свечей (unsorted): {}", candles_unsorted.len());
    println!("Свечей (sorted):   {}", candles_sorted.len());
    println!("Совпадают ли свечи: {}", candles_unsorted == candles_sorted);
    println!("Первая свеча: {:?}", candles_unsorted.first());
    println!("Последняя свеча: {:?}", candles_unsorted.last());
} 