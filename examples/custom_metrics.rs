use candle_generator::{CandleGenerator, CandleConfig, CandleMetric, Timeframe, Trade, Instrument, Pair, MarketType, Side};
use chrono::Utc;

// --- TRADE STATS STRUCTURE AND METRICS ---
#[derive(Default)]
pub struct TradeStats {
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    pub volume: f64,
    pub quote_volume: f64,
    pub trades_count: usize,
    pub prices: Vec<f64>,
    pub buy_count: usize,
    pub sell_count: usize,
    pub vol_buy: f64,
    pub vol_sell: f64,
    pub quote_vol_buy: f64,
    pub quote_vol_sell: f64,
    pub buy_prices: Vec<f64>,
    pub sell_prices: Vec<f64>,
    pub buy_amounts: Vec<f64>,
    pub sell_amounts: Vec<f64>,
    pub taker_buy_vol: f64,
    pub taker_sell_vol: f64,
    pub maker_buy_vol: f64,
    pub maker_sell_vol: f64,
    pub open_ts: Option<i64>,
    pub high_ts: Option<i64>,
    pub low_ts: Option<i64>,
    pub close_ts: Option<i64>,
    pub start_ts: Option<i64>,
}

impl TradeStats {
    pub fn on_trade(&mut self, trade: &Trade) {
        let price = trade.price;
        let amount = trade.amount;
        let quote = price * amount;
        let ts = trade.timestamp.timestamp();
        if self.open.is_none() {
            self.open = Some(price);
            self.open_ts = Some(ts);
            self.start_ts = Some(ts);
        }
        self.close = Some(price);
        self.close_ts = Some(ts);
        self.high = Some(self.high.map_or(price, |h| h.max(price)));
        if self.high == Some(price) { self.high_ts = Some(ts); }
        self.low = Some(self.low.map_or(price, |l| l.min(price)));
        if self.low == Some(price) { self.low_ts = Some(ts); }
        self.volume += amount;
        self.quote_volume += quote;
        self.trades_count += 1;
        self.prices.push(price);
        match trade.side {
            Side::Buy => {
                self.buy_count += 1;
                self.vol_buy += amount;
                self.quote_vol_buy += quote;
                self.buy_prices.push(price);
                self.buy_amounts.push(amount);
            },
            Side::Sell => {
                self.sell_count += 1;
                self.vol_sell += amount;
                self.quote_vol_sell += quote;
                self.sell_prices.push(price);
                self.sell_amounts.push(amount);
            },
            _ => {}
        }
        // TODO: определить тейкер/мейкер по данным trade (если есть флаг)
    }
    pub fn finalize(&self, candle: &mut candle_generator::Candle) {
        // OHLCV
        if let Some(open) = self.open { candle.custom.insert("open".into(), open); }
        if let Some(high) = self.high { candle.custom.insert("high".into(), high); }
        if let Some(low) = self.low { candle.custom.insert("low".into(), low); }
        if let Some(close) = self.close { candle.custom.insert("close".into(), close); }
        candle.custom.insert("volume".into(), self.volume);
        candle.custom.insert("quote_volume".into(), self.quote_volume);
        candle.custom.insert("trades_count".into(), self.trades_count as f64);
        // Стандартное отклонение цены
        let mean = if self.prices.is_empty() { 0.0 } else { self.prices.iter().sum::<f64>() / self.prices.len() as f64 };
        let var = if self.prices.is_empty() { 0.0 } else { self.prices.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / self.prices.len() as f64 };
        candle.custom.insert("pr_std".into(), var.sqrt());
        // VWAP
        let vwap = if self.volume > 0.0 {
            self.prices.iter().zip(self.buy_amounts.iter().chain(self.sell_amounts.iter())).map(|(p, a)| p * a).sum::<f64>() / self.volume
        } else { 0.0 };
        candle.custom.insert("pr_vwap".into(), vwap);
        // Изменение цены за период, %
        let pr_change = if let (Some(open), Some(close)) = (self.open, self.close) {
            if open.abs() > 1e-8 { (close - open) / open * 100.0 } else { 0.0 }
        } else { 0.0 };
        candle.custom.insert("pr_change".into(), pr_change);
        // Buy/Sell counts & volumes
        candle.custom.insert("trades_buy".into(), self.buy_count as f64);
        candle.custom.insert("trades_sell".into(), self.sell_count as f64);
        candle.custom.insert("vol_buy".into(), self.vol_buy);
        candle.custom.insert("vol_sell".into(), self.vol_sell);
        candle.custom.insert("quote_vol_buy".into(), self.quote_vol_buy);
        candle.custom.insert("quote_vol_sell".into(), self.quote_vol_sell);
        // Дисбаланс объёма
        let disbalance = if (self.vol_buy + self.vol_sell).abs() > 1e-8 {
            (self.vol_buy - self.vol_sell) / (self.vol_buy + self.vol_sell)
        } else { 0.0 };
        candle.custom.insert("disbalance".into(), disbalance);
        // VWAP по сторонам
        let vwap_buy = if self.vol_buy > 0.0 {
            self.buy_prices.iter().zip(self.buy_amounts.iter()).map(|(p, a)| p * a).sum::<f64>() / self.vol_buy
        } else { 0.0 };
        let vwap_sell = if self.vol_sell > 0.0 {
            self.sell_prices.iter().zip(self.sell_amounts.iter()).map(|(p, a)| p * a).sum::<f64>() / self.vol_sell
        } else { 0.0 };
        candle.custom.insert("pr_vwap_buy".into(), vwap_buy);
        candle.custom.insert("pr_vwap_sell".into(), vwap_sell);
        // TODO: taker/maker volumes (если есть флаг)
        // Временные метки для open/high/low/close
        if let (Some(start), Some(open_ts)) = (self.start_ts, self.open_ts) {
            candle.custom.insert("sec_pr_open".into(), (open_ts - start) as f64);
        }
        if let (Some(start), Some(high_ts)) = (self.start_ts, self.high_ts) {
            candle.custom.insert("sec_pr_high".into(), (high_ts - start) as f64);
        }
        if let (Some(start), Some(low_ts)) = (self.start_ts, self.low_ts) {
            candle.custom.insert("sec_pr_low".into(), (low_ts - start) as f64);
        }
        if let (Some(start), Some(close_ts)) = (self.start_ts, self.close_ts) {
            candle.custom.insert("sec_pr_close".into(), (close_ts - start) as f64);
        }
    }
}

// --- ARCHITECTURAL SKELETON FOR SUPERCANDLEMETRIC ---
#[derive(Default)]
pub struct SuperCandleMetric {
    pub trade_stats: TradeStats,
    pub orders: Vec<OrderEvent>,
    pub orderbooks: Vec<OrderBookSnapshot>,
    pub futures: Vec<FuturesEvent>,
}

impl SuperCandleMetric {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn on_trade(&mut self, trade: Trade) {
        self.trade_stats.on_trade(&trade);
    }
    pub fn on_order(&mut self, order: OrderEvent) {
        self.orders.push(order);
    }
    pub fn on_orderbook(&mut self, ob: OrderBookSnapshot) {
        self.orderbooks.push(ob);
    }
    pub fn on_futures(&mut self, fut: FuturesEvent) {
        self.futures.push(fut);
    }
    pub fn finalize(&self, candle: &mut candle_generator::Candle) {
        self.trade_stats.finalize(candle);
        // TODO: вызвать finalize для OrderStats, OBStats, FuturesStats
    }
}

// --- STUB STRUCTS FOR EVENTS ---
pub struct OrderEvent {/* TODO: fields for order events */}
pub struct OrderBookSnapshot {/* TODO: fields for orderbook snapshot */}
pub struct FuturesEvent {/* TODO: fields for futures events */}

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
    config.custom_metrics.push(Box::new(SuperCandleMetric::new()));
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