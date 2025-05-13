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
use chrono::{DateTime, Utc, Timelike, TimeZone, Duration, Datelike};
use std::collections::HashMap;

pub struct CandleGenerator {
    pub instrument: Instrument,
    pub timeframes: Vec<Timeframe>,
    pub candles: HashMap<Timeframe, Vec<Candle>>,
    pub usdt_volume_source: UsdtVolumeSource,
}

impl CandleGenerator {
    pub fn new(timeframes: Vec<Timeframe>, instrument: Instrument, usdt_volume_source: UsdtVolumeSource) -> Self {
        let mut candles = HashMap::new();
        for tf in &timeframes {
            candles.insert(tf.clone(), Vec::new());
        }
        Self { instrument, timeframes, candles, usdt_volume_source }
    }

    pub fn add_trade(&mut self, trade: Trade) {
        // m1 агрегация из трейдов
        let tf = Timeframe::m1;
        let minute = trade.timestamp.date_naive().and_hms_opt(trade.timestamp.hour(), trade.timestamp.minute(), 0).unwrap();
        let mut m1_candles = self.candles.entry(tf.clone()).or_insert_with(Vec::new);

        // Рассчитываем volume_usdt для трейда
        let volume_usdt = calc_volume_usdt(&trade, trade.price * trade.amount, &self.usdt_volume_source);

        if m1_candles.is_empty() {
            m1_candles.push(Candle {
                instrument: trade.instrument.clone(),
                interval: tf.clone(),
                timestamp: minute.and_utc(),
                open: trade.price,
                high: trade.price,
                low: trade.price,
                close: trade.price,
                volume: trade.amount,
                trade_count: 1,
                volume_usdt,
            });
        } else {
            let last = m1_candles.last_mut().unwrap();
            let last_time = last.timestamp;
            if minute.and_utc() == last_time {
                last.high = last.high.max(trade.price);
                last.low = last.low.min(trade.price);
                last.close = trade.price;
                last.volume += trade.amount;
                last.trade_count += 1;
                if let Some(vu) = volume_usdt {
                    last.volume_usdt = Some(last.volume_usdt.unwrap_or(0.0) + vu);
                }
            } else if minute.and_utc() > last_time {
                let mut t = last_time + Duration::minutes(1);
                while t < minute.and_utc() {
                    m1_candles.push(Candle {
                        instrument: trade.instrument.clone(),
                        interval: tf.clone(),
                        timestamp: t,
                        open: last.close,
                        high: last.close,
                        low: last.close,
                        close: last.close,
                        volume: 0.0,
                        trade_count: 0,
                        volume_usdt: last.volume_usdt,
                    });
                    t = t + Duration::minutes(1);
                }
                m1_candles.push(Candle {
                    instrument: trade.instrument.clone(),
                    interval: tf.clone(),
                    timestamp: minute.and_utc(),
                    open: trade.price,
                    high: trade.price,
                    low: trade.price,
                    close: trade.price,
                    volume: trade.amount,
                    trade_count: 1,
                    volume_usdt,
                });
            } else {
                if let Some(c) = m1_candles.iter_mut().find(|c| c.timestamp == minute.and_utc()) {
                    c.high = c.high.max(trade.price);
                    c.low = c.low.min(trade.price);
                    c.close = trade.price;
                    c.volume += trade.amount;
                    c.trade_count += 1;
                    if let Some(vu) = volume_usdt {
                        c.volume_usdt = Some(c.volume_usdt.unwrap_or(0.0) + vu);
                    }
                }
            }
        }
        self.aggregate_higher_timeframes(Timeframe::m1);
    }

    fn aggregate_higher_timeframes(&mut self, changed_tf: Timeframe) {
        let mut tf = changed_tf;
        while let Some(parent_tf) = source_timeframe_parent(&tf) {
            if !self.candles.contains_key(&parent_tf) { break; }
            let src = self.candles.get(&tf).unwrap();
            let count = count_for_tf(&parent_tf);
            if src.len() < count { break; }
            let last_src = &src[src.len() - 1];
            let period_start = truncate_to_tf(last_src.timestamp, &parent_tf);
            let parent_candles = self.candles.get_mut(&parent_tf).unwrap();
            if parent_candles.last().map(|c| c.timestamp) == Some(period_start) { tf = parent_tf; continue; }
            let src_slice = &src[src.len() - count..];
            let candle = aggregate_candles(src_slice, parent_tf.clone(), period_start);
            parent_candles.push(candle);
            tf = parent_tf;
        }
    }

    pub fn get_candles(&self, timeframe: Timeframe, limit: usize) -> Vec<Candle> {
        self.candles.get(&timeframe)
            .map(|v| v.iter().rev().take(limit).cloned().collect())
            .unwrap_or_default()
    }

    pub fn reset(&mut self) {
        for v in self.candles.values_mut() {
            v.clear();
        }
    }
}

fn calc_volume_usdt(trade: &Trade, value: f64, src: &UsdtVolumeSource) -> Option<f64> {
    let quote = &trade.instrument.pair.quote_id;
    if quote == "USDT" {
        Some(value)
    } else {
        match src {
            UsdtVolumeSource::Fixed(rate) => Some(value * rate),
            UsdtVolumeSource::Callback(cb) => cb(&trade.instrument.pair, trade.timestamp).map(|r| value * r),
            UsdtVolumeSource::None => None,
        }
    }
}

fn source_timeframe_parent(tf: &Timeframe) -> Option<Timeframe> {
    match tf {
        Timeframe::m1 => Some(Timeframe::m5),
        Timeframe::m5 => Some(Timeframe::m15),
        Timeframe::m15 => Some(Timeframe::m30),
        Timeframe::m30 => Some(Timeframe::h1),
        Timeframe::h1 => Some(Timeframe::h4),
        Timeframe::h4 => Some(Timeframe::d1),
        _ => None,
    }
}

fn count_for_tf(tf: &Timeframe) -> usize {
    match tf {
        Timeframe::m5 => 5,
        Timeframe::m15 => 3,
        Timeframe::m30 => 2,
        Timeframe::h1 => 2,
        Timeframe::h4 => 4,
        Timeframe::d1 => 6,
        _ => 1,
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

fn aggregate_candles(candles: &[Candle], tf: Timeframe, timestamp: DateTime<Utc>) -> Candle {
    let open = candles.first().unwrap().open;
    let close = candles.last().unwrap().close;
    let high = candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let low = candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let volume = candles.iter().map(|c| c.volume).sum();
    let trade_count = candles.iter().map(|c| c.trade_count).sum();
    let volume_usdt = if candles.iter().all(|c| c.volume_usdt.is_some()) {
        Some(candles.iter().map(|c| c.volume_usdt.unwrap()).sum())
    } else {
        None
    };
    Candle {
        instrument: candles[0].instrument.clone(),
        interval: tf,
        timestamp,
        open,
        high,
        low,
        close,
        volume,
        trade_count,
        volume_usdt,
    }
}

#[cfg(test)]
pub mod tests;
