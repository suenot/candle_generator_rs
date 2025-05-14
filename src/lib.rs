pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

mod types;

pub use types::*;
use chrono::{DateTime, Utc, Timelike};
use std::collections::HashMap;

pub struct CandleGenerator {
    pub config: CandleConfig,
}

impl Default for CandleGenerator {
    fn default() -> Self {
        Self { config: CandleConfig::default() }
    }
}

impl CandleGenerator {
    pub fn aggregate<'a, I>(&self, trades: I, timeframe: Timeframe) -> Vec<Candle>
    where
        I: Iterator<Item = &'a Trade>,
    {
        let mut candles = Vec::new();
        let mut current: Option<Candle> = None;
        for trade in trades {
            let ts = truncate_to_tf(trade.timestamp, &timeframe);
            match &mut current {
                Some(c) if c.timestamp == ts => {
                    c.high = c.high.max(trade.price);
                    c.low = c.low.min(trade.price);
                    c.close = trade.price;
                    c.volume += trade.amount;
                    c.trade_count += 1;
                    // USDT volume
                    if let Some(vu) = calc_volume_usdt(trade, &self.config.volume_in_usdt) {
                        c.volume_usdt = Some(c.volume_usdt.unwrap_or(0.0) + vu);
                    }
                    // Кастомные метрики
                    for m in &self.config.custom_metrics {
                        m.update(trade, c);
                    }
                }
                Some(c) => {
                    candles.push(c.clone());
                    current = Some(new_candle(trade, timeframe.clone(), ts, &self.config));
                }
                None => {
                    current = Some(new_candle(trade, timeframe.clone(), ts, &self.config));
                }
            }
        }
        if let Some(c) = current {
            candles.push(c);
        }
        candles
    }

    /// Строит цепочку агрегации: m1→m5→m15→m30→h1→h4→d1
    pub fn aggregate_chain<'a, I>(&self, trades: I) -> HashMap<Timeframe, Vec<Candle>>
    where
        I: Iterator<Item = &'a Trade> + Clone,
    {
        let mut result = HashMap::new();
        let tf_order = vec![Timeframe::m1, Timeframe::m5, Timeframe::m15, Timeframe::m30, Timeframe::h1, Timeframe::h4, Timeframe::d1];
        let mut prev = self.aggregate(trades.clone(), Timeframe::m1);
        result.insert(Timeframe::m1, prev.clone());
        for tf in tf_order.into_iter().skip(1) {
            let higher = aggregate_from_lower(&prev, &tf);
            result.insert(tf.clone(), higher.clone());
            prev = higher;
        }
        result
    }
}

fn new_candle(trade: &Trade, tf: Timeframe, ts: DateTime<Utc>, config: &CandleConfig) -> Candle {
    let mut c = Candle {
        instrument: trade.instrument.clone(),
        interval: tf,
        timestamp: ts,
        open: trade.price,
        high: trade.price,
        low: trade.price,
        close: trade.price,
        volume: trade.amount,
        trade_count: 1,
        volume_usdt: calc_volume_usdt(trade, &config.volume_in_usdt),
        custom: HashMap::new(),
    };
    for m in &config.custom_metrics {
        m.update(trade, &mut c);
    }
    c
}

fn calc_volume_usdt(trade: &Trade, src: &UsdtVolumeSource) -> Option<f64> {
    let quote = &trade.instrument.pair.quote_id;
    if quote == "USDT" {
        Some(trade.price * trade.amount)
    } else {
        match src {
            UsdtVolumeSource::Fixed(rate) => Some(trade.price * trade.amount * rate),
            UsdtVolumeSource::Callback(cb) => cb(&trade.instrument.pair, trade.timestamp).map(|r| trade.price * trade.amount * r),
            UsdtVolumeSource::None => None,
        }
    }
}

fn truncate_to_tf(ts: DateTime<Utc>, tf: &Timeframe) -> DateTime<Utc> {
    match tf {
        Timeframe::m1 => ts.date_naive().and_hms_opt(ts.hour(), ts.minute(), 0).unwrap().and_utc(),
        Timeframe::m5 => {
            let m = ts.minute() / 5 * 5;
            ts.date_naive().and_hms_opt(ts.hour(), m, 0).unwrap().and_utc()
        },
        Timeframe::m15 => {
            let m = ts.minute() / 15 * 15;
            ts.date_naive().and_hms_opt(ts.hour(), m, 0).unwrap().and_utc()
        },
        Timeframe::m30 => {
            let m = ts.minute() / 30 * 30;
            ts.date_naive().and_hms_opt(ts.hour(), m, 0).unwrap().and_utc()
        },
        Timeframe::h1 => ts.date_naive().and_hms_opt(ts.hour(), 0, 0).unwrap().and_utc(),
        Timeframe::h4 => {
            let h = ts.hour() / 4 * 4;
            ts.date_naive().and_hms_opt(h, 0, 0).unwrap().and_utc()
        },
        Timeframe::d1 => ts.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
        _ => ts,
    }
}

fn aggregate_from_lower(lower: &[Candle], tf: &Timeframe) -> Vec<Candle> {
    let count = match tf {
        Timeframe::m5 => 5,
        Timeframe::m15 => 3,
        Timeframe::m30 => 2,
        Timeframe::h1 => 2,
        Timeframe::h4 => 4,
        Timeframe::d1 => 6,
        _ => 1,
    };
    let mut result = Vec::new();
    let mut i = 0;
    while i + count <= lower.len() {
        let slice = &lower[i..i+count];
        let open = slice.first().unwrap().open;
        let close = slice.last().unwrap().close;
        let high = slice.iter().map(|c| c.high).fold(f64::MIN, f64::max);
        let low = slice.iter().map(|c| c.low).fold(f64::MAX, f64::min);
        let volume = slice.iter().map(|c| c.volume).sum();
        let trade_count = slice.iter().map(|c| c.trade_count).sum();
        let volume_usdt = if slice.iter().all(|c| c.volume_usdt.is_some()) {
            Some(slice.iter().map(|c| c.volume_usdt.unwrap()).sum())
        } else {
            None
        };
        let mut candle = Candle {
            instrument: slice[0].instrument.clone(),
            interval: tf.clone(),
            timestamp: truncate_to_tf(slice[0].timestamp, tf),
            open, high, low, close, volume, trade_count, volume_usdt,
            custom: HashMap::new(),
        };
        // Кастомные метрики (агрегация по цепочке)
        i += count;
        result.push(candle);
    }
    result
}

pub struct CandleAggregator {
    pub config: CandleConfig,
}

impl Default for CandleAggregator {
    fn default() -> Self {
        Self { config: CandleConfig::default() }
    }
}

impl CandleAggregator {
    pub fn aggregate<'a, I>(&self, trades: I, timeframe: Timeframe) -> Vec<Candle>
    where
        I: Iterator<Item = &'a Trade>,
    {
        // Stateless: агрегируем поток трейдов в свечи
        let mut candles = Vec::new();
        let mut current: Option<Candle> = None;
        for trade in trades {
            let ts = truncate_to_tf(trade.timestamp, &timeframe);
            match &mut current {
                Some(c) if c.timestamp == ts => {
                    c.high = c.high.max(trade.price);
                    c.low = c.low.min(trade.price);
                    c.close = trade.price;
                    c.volume += trade.amount;
                    c.trade_count += 1;
                    // volume_usdt и кастомные метрики — через config
                }
                Some(c) => {
                    candles.push(c.clone());
                    current = Some(Candle {
                        instrument: trade.instrument.clone(),
                        interval: timeframe.clone(),
                        timestamp: ts,
                        open: trade.price,
                        high: trade.price,
                        low: trade.price,
                        close: trade.price,
                        volume: trade.amount,
                        trade_count: 1,
                        volume_usdt: None, // через config
                        custom: HashMap::new(),
                    });
                }
                None => {
                    current = Some(Candle {
                        instrument: trade.instrument.clone(),
                        interval: timeframe.clone(),
                        timestamp: ts,
                        open: trade.price,
                        high: trade.price,
                        low: trade.price,
                        close: trade.price,
                        volume: trade.amount,
                        trade_count: 1,
                        volume_usdt: None, // через config
                        custom: HashMap::new(),
                    });
                }
            }
        }
        if let Some(c) = current {
            candles.push(c);
        }
        candles
    }
}

// Конфиг и трейты для расширяемости
pub struct CandleConfig {
    pub basic_ohlcv: bool,
    pub volume_in_usdt: UsdtVolumeSource,
    pub custom_metrics: Vec<Box<dyn CandleMetric>>,
}

impl Default for CandleConfig {
    fn default() -> Self {
        Self {
            basic_ohlcv: true,
            volume_in_usdt: UsdtVolumeSource::None,
            custom_metrics: vec![],
        }
    }
}

pub trait CandleMetric {
    fn update(&self, trade: &Trade, candle: &mut Candle);
    fn aggregate(&self, src: &[Candle], dst: &mut Candle);
}
