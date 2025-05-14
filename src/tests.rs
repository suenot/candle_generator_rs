use super::*;
use chrono::{TimeZone, Utc, Duration};

fn sample_instrument() -> Instrument {
    Instrument {
        pair: Pair { base_id: "BTC".to_string(), quote_id: "USDT".to_string() },
        exchange: "binance".to_string(),
        market_type: MarketType::Spot,
    }
}

fn sample_trade(ts: i64, price: f64, amount: f64) -> Trade {
    Trade {
        instrument: sample_instrument(),
        id: format!("{}", ts),
        price,
        amount,
        side: Side::Buy,
        timestamp: Utc.timestamp_millis_opt(ts).unwrap(),
    }
}

#[test]
fn test_create_generator() {
    let instrument = sample_instrument();
    let gen = CandleGenerator::new(vec![Timeframe::m1, Timeframe::m5], instrument.clone());
    assert_eq!(gen.instrument, instrument);
    assert!(gen.candles.contains_key(&Timeframe::m1));
    assert!(gen.candles.contains_key(&Timeframe::m5));
}

#[test]
fn test_add_and_reset() {
    let instrument = sample_instrument();
    let mut gen = CandleGenerator::new(vec![Timeframe::m1], instrument);
    let trade = sample_trade(1_700_000_000_000, 42000.0, 0.1);
    gen.add_trade(trade);
    assert_eq!(gen.get_candles(Timeframe::m1, 10).len(), 1);
    gen.reset();
    assert_eq!(gen.get_candles(Timeframe::m1, 10).len(), 0);
}

#[test]
fn test_aggregate_basic() {
    let trades = vec![
        sample_trade(1_700_000_000_000, 42000.0, 0.1),
        sample_trade(1_700_000_000_000 + 10_000, 42100.0, 0.2),
    ];
    let aggregator = CandleAggregator::default();
    let candles = aggregator.aggregate(trades.iter(), Timeframe::m1);
    assert_eq!(candles.len(), 1);
    assert_eq!(candles[0].open, 42000.0);
    assert_eq!(candles[0].high, 42100.0);
    assert_eq!(candles[0].low, 42000.0);
    assert_eq!(candles[0].close, 42100.0);
    assert_eq!(candles[0].volume, 0.3);
    assert_eq!(candles[0].trade_count, 2);
}

#[test]
fn test_gap_and_empty_candles() {
    let t0 = 1_700_000_000_000; // 00:00:00
    let t2 = t0 + 2 * 60_000;   // 00:02:00
    let trades = vec![
        sample_trade(t0, 100.0, 1.0),
        sample_trade(t2, 110.0, 2.0),
    ];
    let aggregator = CandleAggregator::default();
    let candles = aggregator.aggregate(trades.iter(), Timeframe::m1);
    assert_eq!(candles.len(), 2); // Только свечи с трейдами
    assert_eq!(candles[0].open, 100.0);
    assert_eq!(candles[1].open, 110.0);
}

#[test]
fn test_out_of_order_trade() {
    let t0 = 1_700_000_000_000; // 00:00:00
    let t1 = t0 + 60_000;       // 00:01:00
    let trades = vec![
        sample_trade(t0, 100.0, 1.0),
        sample_trade(t1, 110.0, 2.0),
        sample_trade(t0 + 10_000, 120.0, 0.5), // out-of-order
    ];
    let aggregator = CandleAggregator::default();
    let mut candles = aggregator.aggregate(trades.iter(), Timeframe::m1);
    candles.sort_by_key(|c| c.timestamp);
    assert_eq!(candles.len(), 2);
    let c0 = &candles[0];
    assert_eq!(c0.high, 120.0);
    assert_eq!(c0.volume, 1.5);
}

#[test]
fn test_candle_sequence() {
    let t0 = 1_700_000_000_000; // 00:00:00
    let trades: Vec<_> = (0..5)
        .map(|i| sample_trade(t0 + i * 60_000, 100.0 + i as f64, 1.0))
        .collect();
    let aggregator = CandleAggregator::default();
    let candles = aggregator.aggregate(trades.iter(), Timeframe::m1);
    assert_eq!(candles.len(), 5);
    for i in 0..5 {
        assert_eq!(candles[i].open, 100.0 + i as f64);
    }
}

#[test]
fn test_all_timeframes_aggregation() {
    let instrument = sample_instrument();
    let mut gen = CandleGenerator::new(vec![Timeframe::m1, Timeframe::m5, Timeframe::m15, Timeframe::m30, Timeframe::h1, Timeframe::h4, Timeframe::d1], instrument);
    let t0 = 1_700_000_000_000; // 00:00:00
    // 6*24*60 = 8640 минут = 6 дней (чтобы покрыть все таймфреймы)
    for i in 0..8640 {
        gen.add_trade(sample_trade(t0 + i * 60_000, 100.0 + (i % 10) as f64, 1.0));
    }
    // Проверяем количество свечей в каждом таймфрейме
    assert_eq!(gen.get_candles(Timeframe::m1, 10000).len(), 8640);
    assert_eq!(gen.get_candles(Timeframe::m5, 10000).len(), 1728);
    assert_eq!(gen.get_candles(Timeframe::m15, 10000).len(), 576);
    assert_eq!(gen.get_candles(Timeframe::m30, 10000).len(), 288);
    assert_eq!(gen.get_candles(Timeframe::h1, 10000).len(), 144);
    assert_eq!(gen.get_candles(Timeframe::h4, 10000).len(), 36);
    assert_eq!(gen.get_candles(Timeframe::d1, 10000).len(), 6);
}

#[test]
fn test_gaps_and_empty_candles_higher_tfs() {
    let instrument = sample_instrument();
    let mut gen = CandleGenerator::new(vec![Timeframe::m1, Timeframe::m5, Timeframe::m15], instrument);
    let t0 = 1_700_000_000_000; // 00:00:00
    // Добавляем трейды только в 00:00, 00:05, 00:15 (разрывы)
    gen.add_trade(sample_trade(t0, 100.0, 1.0));
    gen.add_trade(sample_trade(t0 + 5 * 60_000, 110.0, 2.0));
    gen.add_trade(sample_trade(t0 + 15 * 60_000, 120.0, 3.0));
    // Проверяем, что пустые свечи созданы корректно
    let m5 = gen.get_candles(Timeframe::m5, 10);
    assert_eq!(m5.len(), 4); // 00:00, 00:05, 00:10 (пустая), 00:15
    assert_eq!(m5[1].volume, 2.0);
    assert_eq!(m5[2].volume, 0.0); // Пустая m5
    let m15 = gen.get_candles(Timeframe::m15, 10);
    assert_eq!(m15.len(), 2); // 00:00, 00:15
    assert_eq!(m15[1].volume, 3.0);
}

#[test]
fn test_out_of_order_trades_higher_tfs() {
    let instrument = sample_instrument();
    let mut gen = CandleGenerator::new(vec![Timeframe::m1, Timeframe::m5], instrument);
    let t0 = 1_700_000_000_000; // 00:00:00
    // Сначала добавляем трейды для 00:00, 00:01, 00:02, 00:03, 00:04
    for i in 0..5 {
        gen.add_trade(sample_trade(t0 + i * 60_000, 100.0 + i as f64, 1.0));
    }
    // Теперь out-of-order trade для 00:02
    gen.add_trade(sample_trade(t0 + 2 * 60_000 + 10_000, 200.0, 0.5));
    // Проверяем, что high m1 и m5 обновился
    let m1 = gen.get_candles(Timeframe::m1, 10);
    assert!(m1[2].high == 200.0);
    let m5 = gen.get_candles(Timeframe::m5, 10);
    assert!(m5[0].high == 200.0);
} 