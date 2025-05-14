use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::Utc;
use std::fs::File;
use csv::Writer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let generator = CandleGenerator::default();
    let candles = generator.aggregate(trades.iter(), Timeframe::m1);
    let mut wtr = Writer::from_writer(File::create("candles.csv")?);
    for candle in candles {
        wtr.serialize(candle)?;
    }
    wtr.flush()?;
    println!("Candles exported to candles.csv");
    Ok(())
} 