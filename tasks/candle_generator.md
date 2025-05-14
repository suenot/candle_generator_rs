# candle_generator: Архитектура и задачи

## Цель

Создать stateless-библиотеку для агрегации трейдов в свечи (candles) по индустриальным стандартам, AGI-ready, без хранения истории данных внутри библиотеки.

---

## Основные требования
- **Stateless:** библиотека не хранит историю трейдов или свечей, только агрегирует поток трейдов в свечи на лету.
- **Гибкая архитектура:** поддержка кастомных метрик, форматов экспорта, расширяемость через трейты.
- **Работа с источниками данных (CSV, Parquet, DuckDB, QuestDB, ClickHouse и др.) вынесена в examples/**
- **Документация и тесты должны отражать stateless-подход.**

---

## Основные интерфейсы

### CandleGenerator
```rust
pub struct CandleGenerator {
    pub config: CandleConfig,
}

impl CandleGenerator {
    pub fn aggregate<'a, I>(&self, trades: I, timeframe: Timeframe) -> Vec<Candle>
    where
        I: Iterator<Item = &'a Trade>,
    {
        // агрегирует поток трейдов в свечи, не хранит состояние между вызовами
    }
    pub fn aggregate_chain<'a, I>(&self, trades: I) -> HashMap<Timeframe, Vec<Candle>>
    where
        I: Iterator<Item = &'a Trade> + Clone,
    {
        // строгая цепочка агрегации m1→m5→m15→m30→h1→h4→d1
    }
}
```

### CandleConfig и расширяемость
```rust
pub struct CandleConfig {
    pub basic_ohlcv: bool,
    pub volume_in_usdt: UsdtVolumeSource,
    pub custom_metrics: Vec<Box<dyn CandleMetric>>,
}

pub trait CandleMetric {
    fn update(&self, trade: &Trade, candle: &mut Candle);
    fn aggregate(&self, src: &[Candle], dst: &mut Candle);
}
```

---

## Логика и reasoning
- Все stateful-операции (например, хранение последних свечей, bulk ingestion, history) — вне библиотеки.
- Любые сценарии загрузки/выгрузки данных реализуются в examples/ или внешних утилитах.
- Для расширения поддерживаются трейты TradeParser, CandleMetric, CandleSerializer.
- Для AGI-совместимости используются строгие типы, serde-атрибуты, возможность расширения структуры свечи.
- Все edge-cases (gaps, out-of-order, bulk ingestion, кастомные метрики) покрыты тестами и примерами.
- Экспорт свечей реализован для всех популярных форматов (CSV, JSON, Parquet).

---

## Покрытие и прогресс
- [x] Stateless-агрегатор CandleGenerator
- [x] Переписать тесты под новый интерфейс
- [x] Примеры для CSV, Parquet, DuckDB, QuestDB, ClickHouse
- [x] Примеры экспорта в CSV, JSON, Parquet
- [x] Примеры для всех edge-cases (gaps, out-of-order)
- [x] Примеры кастомных метрик (buy/sell volume, VWAP)
- [x] Примеры bulk ingestion, aggregation chain
- [x] README.md и types.rs полностью отражают архитектуру и возможности
- [x] Документация и тесты синхронизированы

---

## Рекомендации по развитию
- Добавлять новые edge-cases и сценарии через PR/issues и фиксировать их в examples/ и тестах
- Для новых источников и форматов — реализовывать адаптеры и примеры
- Для новых метрик — реализовывать CandleMetric и покрывать тестами
- Регулярно обновлять README.md и tasks/candle_generator.md при изменениях
- Настроить CI для автоматической проверки тестов и сборки примеров (опционально)

---

## Готовность
- Проект полностью готов к демонстрации инвесторам, AGI-экспертам и для реального использования в индустрии
- Все ключевые сценарии покрыты тестами и примерами
- Архитектура максимально прозрачна, расширяема и AGI-friendly 