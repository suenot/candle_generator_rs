use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_files: usize,
    pub total_trades: usize,
    pub total_candles: HashMap<i64, usize>, // interval -> count
    pub processing_time: Duration,
    pub trade_processing_time: Duration,
    pub aggregation_time: Duration,
    pub io_time: Duration,
}

impl ProcessingStats {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_candles(&mut self, interval: i64, count: usize) {
        *self.total_candles.entry(interval).or_default() += count;
    }
}

pub fn print_summary(stats: &ProcessingStats) {
    println!("\n=== Processing Summary ===");
    println!("Total files processed: {}", stats.total_files);
    println!("Total trades processed: {}", stats.total_trades);
    // ... далее по аналогии с bybit-generate-candles ...
} 