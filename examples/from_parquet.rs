use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use polars::prelude::*;
use chrono::Utc;

fn main() -> polars::prelude::PolarsResult<()> {
    let df = LazyFrame::scan_parquet("trades.parquet", Default::default())?.collect()?;
    let mut trades = Vec::new();
    for row in df.iter_rows() {
        let (timestamp, exchange, base_id, quote_id, market_type, id, price, amount, side): (i64, String, String, String, String, String, f64, f64, String) = (
            row[0].try_extract()?,
            row[1].try_extract()?,
            row[2].try_extract()?,
            row[3].try_extract()?,
            row[4].try_extract()?,
            row[5].try_extract()?,
            row[6].try_extract()?,
            row[7].try_extract()?,
            row[8].try_extract()?,
        );
        trades.push(Trade {
            instrument: Instrument {
                pair: Pair { base_id, quote_id },
                exchange,
                market_type: match market_type.as_str() {
                    "Spot" => MarketType::Spot,
                    "Futures" => MarketType::Futures,
                    "Margin" => MarketType::Margin,
                    _ => MarketType::Unknown,
                },
            },
            id,
            price,
            amount,
            side: match side.as_str() {
                "Buy" => Side::Buy,
                "Sell" => Side::Sell,
                _ => Side::Unknown,
            },
            timestamp: Utc.timestamp_millis_opt(timestamp).unwrap(),
        });
    }
    let generator = CandleGenerator::default();
    let candles = generator.aggregate(trades.iter(), Timeframe::m1);
    for candle in candles {
        println!("{:?}", candle);
    }
    Ok(())
} 