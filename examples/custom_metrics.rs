use candle_generator::{CandleGenerator, CandleConfig, CandleMetric, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::Utc;

struct VWAPMetric;
impl CandleMetric for VWAPMetric {
    fn update(&self, trade: &Trade, candle: &mut candle_generator::Candle) {
        let vwap = candle.custom.get("vwap").cloned().unwrap_or(0.0);
        let total_volume = candle.volume;
        let new_vwap = if total_volume > 0.0 {
            (vwap * (total_volume - trade.amount) + trade.price * trade.amount) / total_volume
        } else {
            trade.price
        };
        candle.custom.insert("vwap".to_string(), new_vwap);
    }
    fn aggregate(&self, _src: &[candle_generator::Candle], _dst: &mut candle_generator::Candle) {}
}

fn main() {
    let trades = vec![
        Trade {
            instrument: Instrument {
                pair: Pair { base_id: "BTC".into(), quote_id: "USDT".into() },
                exchange: "binance".into(),
                market_type: MarketType::Spot,
            },
            id: "t1".into(),
            price: 100.0,
            amount: 1.0,
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
            price: 110.0,
            amount: 2.0,
            side: Side::Buy,
            timestamp: Utc.timestamp_millis_opt(1714000060000).unwrap(),
        },
    ];
    let mut config = CandleConfig::default();
    config.custom_metrics.push(Box::new(VWAPMetric));
    let gen = CandleGenerator { config };
    let candles = gen.aggregate(trades.iter(), Timeframe::m1);
    for candle in candles {
        println!("VWAP = {:?}", candle.custom.get("vwap"));
    }
} 