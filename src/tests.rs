use super::*;
use chrono::{TimeZone, Utc};
use rand::seq::SliceRandom;

fn sample_instrument() -> Instrument {
    Instrument {
        pair: Pair { base_id: "BTC".to_string(), quote_id: "USDT".to_string() },
        exchange: "binance".to_string(),
        market_type: MarketType::Spot,
    }
}

fn sample_trade(ts: i64, price: f64, amount: f64, side: Side) -> Trade {
    Trade {
        instrument: sample_instrument(),
        id: format!("{}", ts),
        price,
        amount,
        side,
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
    let trade = sample_trade(1_700_000_000_000, 42000.0, 0.1, Side::Buy);
    gen.add_trade(trade);
    assert_eq!(gen.get_candles(Timeframe::m1, 10).len(), 1);
    gen.reset();
    assert_eq!(gen.get_candles(Timeframe::m1, 10).len(), 0);
}

#[test]
fn test_basic_ohlcv_aggregation() {
    let trades = vec![
        sample_trade(1_700_000_000_000, 42000.0, 0.1, Side::Buy),
        sample_trade(1_700_000_000_000 + 10_000, 42100.0, 0.2, Side::Sell),
    ];
    let gen = CandleGenerator::default();
    let candles = gen.aggregate(trades.iter(), Timeframe::m1);
    assert_eq!(candles.len(), 1);
    let c = &candles[0];
    assert_eq!(c.open, 42000.0);
    assert_eq!(c.high, 42100.0);
    assert_eq!(c.low, 42000.0);
    assert_eq!(c.close, 42100.0);
    assert_eq!(c.volume, 0.3);
    assert_eq!(c.trade_count, 2);
}

#[test]
fn test_aggregation_chain() {
    let t0 = 1_700_000_000_000;
    let trades: Vec<_> = (0..10)
        .map(|i| sample_trade(t0 + i * 60_000, 100.0 + i as f64, 1.0, Side::Buy))
        .collect();
    let gen = CandleGenerator::default();
    let chain = gen.aggregate_chain(trades.iter());
    assert_eq!(chain[&Timeframe::m1].len(), 10);
    assert_eq!(chain[&Timeframe::m5].len(), 2);
    assert_eq!(chain[&Timeframe::m15].len(), 0); // 10 минут — меньше 15
}

#[test]
fn test_usdt_volume_fixed() {
    let mut config = CandleConfig::default();
    config.volume_in_usdt = UsdtVolumeSource::Fixed(2.0); // удваиваем объём
    let gen = CandleGenerator { config };
    let trades = vec![sample_trade(1_700_000_000_000, 100.0, 1.0, Side::Buy)];
    let candles = gen.aggregate(trades.iter(), Timeframe::m1);
    assert_eq!(candles[0].volume_usdt, Some(200.0));
}

#[test]
fn test_usdt_volume_callback() {
    let mut config = CandleConfig::default();
    config.volume_in_usdt = UsdtVolumeSource::Callback(Box::new(|_pair, _ts| Some(10.0)));
    let gen = CandleGenerator { config };
    let trades = vec![sample_trade(1_700_000_000_000, 5.0, 2.0, Side::Buy)];
    let candles = gen.aggregate(trades.iter(), Timeframe::m1);
    assert_eq!(candles[0].volume_usdt, Some(100.0));
}

#[test]
fn test_usdt_volume_none() {
    let mut config = CandleConfig::default();
    config.volume_in_usdt = UsdtVolumeSource::None;
    let gen = CandleGenerator { config };
    let trades = vec![sample_trade(1_700_000_000_000, 5.0, 2.0, Side::Buy)];
    let candles = gen.aggregate(trades.iter(), Timeframe::m1);
    assert_eq!(candles[0].volume_usdt, None);
}

struct BuySellVolume;
impl CandleMetric for BuySellVolume {
    fn update(&self, trade: &Trade, candle: &mut Candle) {
        let buy = candle.custom.get_mut("buy_volume").unwrap_or(&mut 0.0);
        let sell = candle.custom.get_mut("sell_volume").unwrap_or(&mut 0.0);
        match trade.side {
            Side::Buy => *buy += trade.amount,
            Side::Sell => *sell += trade.amount,
            _ => {}
        }
        candle.custom.insert("buy_volume".to_string(), *buy);
        candle.custom.insert("sell_volume".to_string(), *sell);
    }
    fn aggregate(&self, _src: &[Candle], _dst: &mut Candle) {}
}

#[test]
fn test_buy_sell_volume_metric() {
    let mut config = CandleConfig::default();
    config.custom_metrics.push(Box::new(BuySellVolume));
    let gen = CandleGenerator { config };
    let trades = vec![
        sample_trade(1_700_000_000_000, 100.0, 1.0, Side::Buy),
        sample_trade(1_700_000_000_000 + 10_000, 101.0, 2.0, Side::Sell),
    ];
    let candles = gen.aggregate(trades.iter(), Timeframe::m1);
    let c = &candles[0];
    assert_eq!(*c.custom.get("buy_volume").unwrap(), 1.0);
    assert_eq!(*c.custom.get("sell_volume").unwrap(), 2.0);
}

#[test]
fn test_gaps_and_empty_candles() {
    // Трейды только в 00:00 и 00:05
    let t0 = 1_700_000_000_000;
    let t5 = t0 + 5 * 60_000;
    let trades = vec![
        sample_trade(t0, 100.0, 1.0, Side::Buy),
        sample_trade(t5, 110.0, 2.0, Side::Sell),
    ];
    let gen = CandleGenerator::default();
    let m1 = gen.aggregate(trades.iter(), Timeframe::m1);
    assert_eq!(m1.len(), 2); // Только свечи с трейдами
    assert_eq!(m1[0].open, 100.0);
    assert_eq!(m1[1].open, 110.0);
}

#[test]
fn test_bulk_ingestion_sorted() {
    let t0 = 1_700_000_000_000;
    let trades: Vec<_> = (0..1000)
        .map(|i| sample_trade(t0 + i * 60_000, 100.0 + (i % 10) as f64, 1.0, Side::Buy))
        .collect();
    let gen = CandleGenerator::default();
    let m1 = gen.aggregate(trades.iter(), Timeframe::m1);
    assert_eq!(m1.len(), 1000);
}

struct VWAPMetric;
impl CandleMetric for VWAPMetric {
    fn update(&self, trade: &Trade, candle: &mut Candle) {
        let vwap = candle.custom.get("vwap").cloned().unwrap_or(0.0);
        let total_volume = candle.volume;
        let new_vwap = if total_volume > 0.0 {
            (vwap * (total_volume - trade.amount) + trade.price * trade.amount) / total_volume
        } else {
            trade.price
        };
        candle.custom.insert("vwap".to_string(), new_vwap);
    }
    fn aggregate(&self, _src: &[Candle], _dst: &mut Candle) {}
}

#[test]
fn test_custom_vwap_metric() {
    let mut config = CandleConfig::default();
    config.custom_metrics.push(Box::new(VWAPMetric));
    let gen = CandleGenerator { config };
    let trades = vec![
        sample_trade(1_700_000_000_000, 100.0, 1.0, Side::Buy),
        sample_trade(1_700_000_000_000 + 10_000, 110.0, 2.0, Side::Buy),
    ];
    let candles = gen.aggregate(trades.iter(), Timeframe::m1);
    let c = &candles[0];
    let vwap = c.custom.get("vwap").unwrap();
    assert!((*vwap - 106.6666).abs() < 0.01);
}

#[test]
fn test_out_of_order_trades() {
    let t0 = 1_700_000_000_000;
    let trades = vec![
        sample_trade(t0, 100.0, 1.0, Side::Buy),
        sample_trade(t0 + 60_000, 110.0, 2.0, Side::Sell),
        sample_trade(t0 + 10_000, 120.0, 0.5, Side::Buy), // out-of-order
    ];
    let gen = CandleGenerator::default();
    let mut candles = gen.aggregate(trades.iter(), Timeframe::m1);
    candles.sort_by_key(|c| c.timestamp);
    assert_eq!(candles.len(), 2);
    let c0 = &candles[0];
    assert_eq!(c0.high, 120.0);
    assert_eq!(c0.volume, 1.5);
}

#[test]
fn test_bulk_ingestion_unsorted() {
    let t0 = 1_700_000_000_000;
    let mut trades: Vec<_> = (0..1000)
        .map(|i| sample_trade(t0 + i * 60_000, 100.0 + (i % 10) as f64, 1.0, Side::Buy))
        .collect();
    let mut rng = rand::thread_rng();
    trades.shuffle(&mut rng);
    let gen = CandleGenerator::default();
    let mut m1_unsorted = gen.aggregate(trades.iter(), Timeframe::m1);
    let trades_sorted: Vec<_> = trades.iter().cloned().collect();
    let mut m1_sorted = gen.aggregate(trades_sorted.iter(), Timeframe::m1);
    m1_unsorted.sort_by_key(|c| c.timestamp);
    m1_sorted.sort_by_key(|c| c.timestamp);
    assert_eq!(m1_unsorted, m1_sorted);
} 