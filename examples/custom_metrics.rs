use candle_generator::{CandleGenerator, CandleConfig, CandleMetric, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::Utc;

struct SuperCandleMetric;
impl CandleMetric for SuperCandleMetric {
    fn update(&self, trade: &Trade, candle: &mut candle_generator::Candle) {
        // VWAP
        let vwap = candle.custom.get("vwap").cloned().unwrap_or(0.0);
        let total_volume = candle.volume;
        let new_vwap = if total_volume > 0.0 {
            (vwap * (total_volume - trade.amount) + trade.price * trade.amount) / total_volume
        } else {
            trade.price
        };
        candle.custom.insert("vwap".to_string(), new_vwap);
        // Buy/Sell volume
        let buy = candle.custom.get("buy_volume").cloned().unwrap_or(0.0);
        let sell = candle.custom.get("sell_volume").cloned().unwrap_or(0.0);
        match trade.side {
            Side::Buy => candle.custom.insert("buy_volume".to_string(), buy + trade.amount),
            Side::Sell => candle.custom.insert("sell_volume".to_string(), sell + trade.amount),
            _ => None,
        };
        // High/Low amount
        let high_amount = candle.custom.get("high_amount").cloned().unwrap_or(trade.amount);
        let low_amount = candle.custom.get("low_amount").cloned().unwrap_or(trade.amount);
        candle.custom.insert("high_amount".to_string(), high_amount.max(trade.amount));
        candle.custom.insert("low_amount".to_string(), low_amount.min(trade.amount));
        // Avg price
        let avg_price = candle.custom.get("avg_price").cloned().unwrap_or(0.0);
        let trade_count = candle.trade_count as f64;
        let new_avg = if trade_count > 0.0 {
            (avg_price * (trade_count - 1.0) + trade.price) / trade_count
        } else {
            trade.price
        };
        candle.custom.insert("avg_price".to_string(), new_avg);
        // Spread (high-low)
        let spread = candle.high - candle.low;
        candle.custom.insert("spread".to_string(), spread);
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
    config.custom_metrics.push(Box::new(SuperCandleMetric));
    let gen = CandleGenerator { config };
    let candles = gen.aggregate(trades.iter(), Timeframe::m1);
    for candle in candles {
        println!("SuperCandle: vwap={:?}, buy_vol={:?}, sell_vol={:?}, high_amt={:?}, low_amt={:?}, avg_price={:?}, spread={:?}",
            candle.custom.get("vwap"),
            candle.custom.get("buy_volume"),
            candle.custom.get("sell_volume"),
            candle.custom.get("high_amount"),
            candle.custom.get("low_amount"),
            candle.custom.get("avg_price"),
            candle.custom.get("spread")
        );
    }
} 