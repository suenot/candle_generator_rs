use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side, Candle};
use chrono::Utc;
use polars::prelude::*;

fn main() -> polars::prelude::PolarsResult<()> {
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
    // Преобразуем свечи в DataFrame
    let timestamps: Vec<_> = candles.iter().map(|c| c.timestamp.timestamp_millis()).collect();
    let exchanges: Vec<_> = candles.iter().map(|c| c.instrument.exchange.clone()).collect();
    let base_ids: Vec<_> = candles.iter().map(|c| c.instrument.pair.base_id.clone()).collect();
    let quote_ids: Vec<_> = candles.iter().map(|c| c.instrument.pair.quote_id.clone()).collect();
    let intervals: Vec<_> = candles.iter().map(|c| format!("{:?}", c.interval)).collect();
    let opens: Vec<_> = candles.iter().map(|c| c.open).collect();
    let highs: Vec<_> = candles.iter().map(|c| c.high).collect();
    let lows: Vec<_> = candles.iter().map(|c| c.low).collect();
    let closes: Vec<_> = candles.iter().map(|c| c.close).collect();
    let volumes: Vec<_> = candles.iter().map(|c| c.volume).collect();
    let trade_counts: Vec<_> = candles.iter().map(|c| c.trade_count as i64).collect();
    let volume_usdts: Vec<_> = candles.iter().map(|c| c.volume_usdt.unwrap_or(0.0)).collect();
    let df = DataFrame::new(vec![
        Series::new("timestamp", timestamps),
        Series::new("exchange", exchanges),
        Series::new("base_id", base_ids),
        Series::new("quote_id", quote_ids),
        Series::new("interval", intervals),
        Series::new("open", opens),
        Series::new("high", highs),
        Series::new("low", lows),
        Series::new("close", closes),
        Series::new("volume", volumes),
        Series::new("trade_count", trade_counts),
        Series::new("volume_usdt", volume_usdts),
    ])?;
    df.write_parquet("candles.parquet", ParquetWriteOptions::default())?;
    println!("Candles exported to candles.parquet");
    Ok(())
} 