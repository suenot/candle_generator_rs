use candle_generator::{CandleGenerator, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use std::fs::File;
use csv::Reader;
use chrono::Utc;
use chrono::TimeZone;

fn main() {
    let file = File::open("trades.csv").expect("cannot open trades.csv");
    let mut rdr = Reader::from_reader(file);
    let mut trades = Vec::new();
    for result in rdr.deserialize() {
        let trade: Trade = result.expect("invalid trade row");
        trades.push(trade);
    }
    let generator = CandleGenerator::default();
    let candles = generator.aggregate(trades.iter(), Timeframe::m1);
    for candle in candles {
        println!("{:?}", candle);
    }
} 