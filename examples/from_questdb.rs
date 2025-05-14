use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::Utc;
use chrono::TimeZone;
use reqwest::blocking::get;
use csv::ReaderBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Пример: QuestDB HTTP API (CSV)
    let url = "http://localhost:9000/exec?query=SELECT+*+FROM+trades+ORDER+BY+timestamp&fmt=csv";
    let resp = get(url)?.text()?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(resp.as_bytes());
    let mut trades = Vec::new();
    for result in rdr.records() {
        let row = result?;
        let timestamp: i64 = row[0].parse()?;
        let exchange = row[1].to_string();
        let base_id = row[2].to_string();
        let quote_id = row[3].to_string();
        let market_type = row[4].to_string();
        let id = row[5].to_string();
        let price: f64 = row[6].parse()?;
        let amount: f64 = row[7].parse()?;
        let side = row[8].to_string();
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