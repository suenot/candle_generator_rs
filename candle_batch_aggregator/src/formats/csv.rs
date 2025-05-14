use super::super::Args;
use anyhow::Result;
use std::fs::File;
use std::path::PathBuf;
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side, Candle};

#[derive(Debug, Deserialize)]
struct CsvTrade {
    timestamp: i64,
    price: f64,
    amount: f64,
    side: String,
    #[serde(default)]
    base: String,
    #[serde(default)]
    quote: String,
    #[serde(default)]
    exchange: String,
}

impl CsvTrade {
    fn to_trade(&self) -> Trade {
        Trade {
            instrument: Instrument {
                pair: Pair {
                    base_id: self.base.clone(),
                    quote_id: self.quote.clone(),
                },
                exchange: self.exchange.clone(),
                market_type: MarketType::Spot,
            },
            id: format!("{}", self.timestamp),
            price: self.price,
            amount: self.amount,
            side: match self.side.to_lowercase().as_str() {
                "buy" => Side::Buy,
                "sell" => Side::Sell,
                _ => Side::Unknown,
            },
            timestamp: chrono::Utc.timestamp_millis_opt(self.timestamp).unwrap(),
        }
    }
}

pub fn process_csv_batch(args: &Args) -> Result<()> {
    // MVP: обрабатываем только один файл, одну пару, один таймфрейм
    let input_file = args.input.clone();
    let output_file = args.output.clone().unwrap_or_else(|| PathBuf::from("candles.csv"));
    let interval: i64 = args.interval.split(',').next().unwrap().parse().unwrap_or(1);
    let tf = match interval {
        1 => Timeframe::m1,
        5 => Timeframe::m5,
        15 => Timeframe::m15,
        30 => Timeframe::m30,
        60 => Timeframe::h1,
        240 => Timeframe::h4,
        1440 => Timeframe::d1,
        _ => Timeframe::m1,
    };
    println!("[CSV] Batch processing: input={:?}, output={:?}, interval={:?}", input_file, output_file, tf);
    let file = File::open(&input_file)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    let mut trades = Vec::new();
    for result in rdr.deserialize() {
        let csv_trade: CsvTrade = result?;
        trades.push(csv_trade.to_trade());
    }
    println!("Прочитано трейдов: {}", trades.len());
    let generator = CandleGenerator::default();
    let candles = generator.aggregate(trades.iter(), tf);
    println!("Сгенерировано свечей: {}", candles.len());
    // Запись свечей в CSV
    let mut wtr = WriterBuilder::new().has_headers(true).from_path(&output_file)?;
    for candle in candles {
        wtr.serialize(SimpleCandle::from(&candle))?;
    }
    wtr.flush()?;
    println!("Свечи записаны в {:?}", output_file);
    Ok(())
}

#[derive(Debug, Serialize)]
struct SimpleCandle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl From<&Candle> for SimpleCandle {
    fn from(c: &Candle) -> Self {
        Self {
            timestamp: c.timestamp.timestamp_millis(),
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        }
    }
} 