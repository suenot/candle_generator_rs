use super::super::Args;
use anyhow::Result;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side, Candle};
use std::time::Instant;
use crate::chain;

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

fn parse_intervals(interval_str: &str) -> Vec<Timeframe> {
    if interval_str.to_uppercase() == "ALL" {
        return vec![Timeframe::m1, Timeframe::m5, Timeframe::m15, Timeframe::m30, Timeframe::h1, Timeframe::h4, Timeframe::d1];
    }
    interval_str
        .split(',')
        .filter_map(|s| match s.trim() {
            "1" => Some(Timeframe::m1),
            "5" => Some(Timeframe::m5),
            "15" => Some(Timeframe::m15),
            "30" => Some(Timeframe::m30),
            "60" => Some(Timeframe::h1),
            "240" => Some(Timeframe::h4),
            "1440" => Some(Timeframe::d1),
            _ => None,
        })
        .collect()
}

pub fn process_csv_batch(args: &Args) -> Result<()> {
    let start = Instant::now();
    let intervals = parse_intervals(&args.interval);
    let symbols: Vec<String> = if args.symbol.to_uppercase() == "ALL" {
        // Все поддиректории в input
        fs::read_dir(&args.input)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect()
    } else {
        args.symbol.split(',').map(|s| s.trim().to_string()).collect()
    };
    println!("Batch symbols: {:?}", symbols);
    for symbol in &symbols {
        let symbol_dir = args.input.join(symbol);
        if !symbol_dir.exists() { continue; }
        let files: Vec<_> = fs::read_dir(&symbol_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |ext| ext == "csv"))
            .collect();
        println!("\nProcessing symbol: {} ({} files)", symbol, files.len());
        for file_path in files {
            println!("  File: {:?}", file_path.file_name().unwrap());
            let file = File::open(&file_path)?;
            let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
            let mut trades = Vec::new();
            for result in rdr.deserialize() {
                let csv_trade: CsvTrade = result?;
                trades.push(csv_trade.to_trade());
            }
            println!("    Trades: {}", trades.len());
            // Агрегация цепочкой
            let generator = CandleGenerator::default();
            let base_tf = *intervals.iter().min().unwrap_or(&Timeframe::m1);
            let base_candles = generator.aggregate(trades.iter(), base_tf.clone());
            let chain = chain::aggregate_chain(&base_candles, &intervals)?;
            for (tf, candles) in chain {
                let out_dir = args.output.clone().unwrap_or_else(|| PathBuf::from("candles"));
                let out_dir = out_dir.join(format!("{}_{}", symbol, format!("{:?}", tf)));
                fs::create_dir_all(&out_dir)?;
                let out_file = out_dir.join(format!("{}_{}.csv", file_path.file_stem().unwrap().to_string_lossy(), format!("{:?}", tf)));
                let mut wtr = WriterBuilder::new().has_headers(true).from_path(&out_file)?;
                for candle in &candles {
                    wtr.serialize(SimpleCandle::from(candle))?;
                }
                wtr.flush()?;
                println!("    [{:?}] Candles: {} -> {:?}", tf, candles.len(), out_file);
            }
        }
    }
    println!("\nBatch completed in {:.2?}", start.elapsed());
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