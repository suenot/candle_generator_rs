use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use duckdb::{Connection, Result};
use chrono::Utc;

fn main() -> Result<()> {
    let conn = Connection::open("trades.db")?;
    let mut stmt = conn.prepare("SELECT timestamp, exchange, base_id, quote_id, market_type, id, price, amount, side FROM trades ORDER BY timestamp")?;
    let mut trades = Vec::new();
    let rows = stmt.query_map([], |row| {
        Ok(Trade {
            instrument: Instrument {
                pair: Pair {
                    base_id: row.get(2)?,
                    quote_id: row.get(3)?
                },
                exchange: row.get(1)?,
                market_type: match row.get::<_, String>(4)?.as_str() {
                    "Spot" => MarketType::Spot,
                    "Futures" => MarketType::Futures,
                    "Margin" => MarketType::Margin,
                    _ => MarketType::Unknown,
                },
            },
            id: row.get(5)?,
            price: row.get(6)?,
            amount: row.get(7)?,
            side: match row.get::<_, String>(8)?.as_str() {
                "Buy" => Side::Buy,
                "Sell" => Side::Sell,
                _ => Side::Unknown,
            },
            timestamp: Utc.timestamp_millis_opt(row.get(0)?).unwrap(),
        })
    })?;
    for trade in rows {
        trades.push(trade?);
    }
    let generator = CandleGenerator::default();
    let candles = generator.aggregate(trades.iter(), Timeframe::m1);
    for candle in candles {
        println!("{:?}", candle);
    }
    Ok(())
} 