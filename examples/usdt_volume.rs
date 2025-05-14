use candle_generator::{CandleGenerator, CandleConfig, UsdtVolumeSource, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::Utc;

fn main() {
    let trades = vec![
        Trade {
            instrument: Instrument {
                pair: Pair { base_id: "ETH".into(), quote_id: "BTC".into() },
                exchange: "binance".into(),
                market_type: MarketType::Spot,
            },
            id: "t1".into(),
            price: 0.06, // 0.06 BTC за 1 ETH
            amount: 2.0,
            side: Side::Buy,
            timestamp: Utc.timestamp_millis_opt(1714000000000).unwrap(),
        },
    ];
    // Fixed rate (BTC/USDT = 50000)
    let mut config = CandleConfig::default();
    config.volume_in_usdt = UsdtVolumeSource::Fixed(50000.0);
    let gen = CandleGenerator { config };
    let candles = gen.aggregate(trades.iter(), Timeframe::m1);
    println!("Fixed: volume_usdt = {:?}", candles[0].volume_usdt);

    // Callback (динамический курс)
    let mut config = CandleConfig::default();
    config.volume_in_usdt = UsdtVolumeSource::Callback(Box::new(|pair, _| {
        if pair.quote_id == "BTC" { Some(50000.0) } else { None }
    }));
    let gen = CandleGenerator { config };
    let candles = gen.aggregate(trades.iter(), Timeframe::m1);
    println!("Callback: volume_usdt = {:?}", candles[0].volume_usdt);

    // None (не считать)
    let mut config = CandleConfig::default();
    config.volume_in_usdt = UsdtVolumeSource::None;
    let gen = CandleGenerator { config };
    let candles = gen.aggregate(trades.iter(), Timeframe::m1);
    println!("None: volume_usdt = {:?}", candles[0].volume_usdt);
} 