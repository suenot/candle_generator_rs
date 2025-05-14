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
- [x] Реализовать bulk ingestion для неотсортированных трейдов (edge-case, тесты, пример в examples/)
- [x] Реализовать Super Candle (расширенная свеча с множеством метрик) через кастомный CandleMetric (пример custom_metrics.rs)
- [ ] Проектировать архитектурный скелет SuperCandleMetric для поддержки всех метрик из ТЗ (TradeStats, OrderStats, OBStats, FuturesStats) и событий (trade, order, orderbook, futures)

### 2024-06-XX: Начат этап проектирования архитектурного скелета SuperCandleMetric. Будет реализована структура с промежуточными буферами для всех групп метрик и интерфейсами для приёма событий и финализации свечи.

---

## Рекомендации по развитию
- Добавлять новые edge-cases и сценарии через PR/issues и фиксировать их в examples/ и тестах
- Для новых источников и форматов — реализовывать адаптеры и примеры
- Для новых метрик — реализовывать CandleMetric и покрывать тестами
- Регулярно обновлять README.md и tasks/candle_generator.md при изменениях
- Настроить CI для автоматической проверки тестов и сборки примеров (опционально)
    - **GitHub Actions**: используйте workflow `.github/workflows/ci.yml`:
      ```yaml
      name: CI
      on: [push, pull_request]
      jobs:
        test:
          runs-on: ubuntu-latest
          steps:
            - uses: actions/checkout@v3
            - uses: actions-rs/toolchain@v1
              with:
                toolchain: stable
                override: true
            - run: cargo test --all
            - run: cargo run --release --example bench
      ```
    - Добавьте бейджи статуса сборки и тестов в README.md
- Следить за производительностью: периодически запускать бенчмарки и фиксировать результаты
- Принимать обратную связь от пользователей и инвесторов, интегрировать новые сценарии

---

## Готовность
- Проект полностью готов к демонстрации инвесторам, AGI-экспертам и для реального использования в индустрии
- Все ключевые сценарии покрыты тестами и примерами
- Архитектура максимально прозрачна, расширяема и AGI-friendly 

---

## Сводка всей сессии (июнь 2024)

1. **Исходная задача:**  
   Пользователь разрабатывает библиотеку для агрегации трейдов в свечи (candles) — stateless, AGI-ready, с поддержкой кастомных метрик, строгой цепочкой агрегации (m1→m5→m15→m30→h1→h4→d1), экспортом в разные форматы и примерами для всех источников (CSV, Parquet, DuckDB, QuestDB, ClickHouse).

2. **Требования к архитектуре:**
   - Библиотека не хранит данные, только агрегирует поток трейдов.
   - Все сценарии загрузки/выгрузки реализуются в examples/.
   - Поддержка кастомных метрик через CandleMetric.
   - Экспорт свечей в CSV, JSON, Parquet.
   - Edge-cases: gaps, out-of-order trades, bulk ingestion.
   - Полное покрытие тестами и примерами.

3. **Реализация:**
   - Переписан README.md: отражает stateless-архитектуру, все фичи, примеры, FAQ, roadmap.
   - CandleGenerator реализован как stateless-агрегатор с поддержкой кастомных метрик и aggregation chain.
   - В Candle добавлено поле custom: HashMap<String, f64> для любых метрик.
   - Все тесты переписаны под новую архитектуру, покрывают все фичи и edge-cases.
   - В examples/ созданы рабочие примеры для:
     - Загрузки из CSV, Parquet, DuckDB, QuestDB, ClickHouse
     - Экспорта в CSV, JSON, Parquet
     - Кастомных метрик (VWAP, buy/sell volume)
     - Bulk ingestion, aggregation chain
     - Edge-cases: gaps, out-of-order trades
   - Добавлен бенчмарк (bench.rs) для оценки производительности на 1 млн трейдов.

4. **Документация и поддержка:**
   - README.md содержит инструкции по запуску тестов, бенчмарков, добавлению новых примеров.
   - tasks/candle_generator.md фиксирует архитектуру, прогресс, рекомендации по развитию, CI, поддержке качества.
   - Даны рекомендации по настройке CI (GitHub Actions), поддержке производительности, расширяемости.

5. **Дальнейшее развитие:**
   - Проект готов к масштабированию, автоматизации, интеграции новых сценариев.
   - Следующий этап — поддержка новых edge-cases, метрик, источников, интеграция с внешними системами, развитие API и визуализации.
   - Пользователь предоставил ТЗ на систему Super Candles для Binance (расширенные свечи с множеством метрик, сбор с REST/WebSocket, хранение в Parquet, API, визуализация, масштабируемость).
   - Я предложил архитектурный план реализации Super Candles: схемы данных, ingestion pipeline, хранение, API, визуализация, тестирование, масштабирование.

**Проект полностью готов для демонстрации, развития и интеграции новых задач.** 

## Edge-case: Bulk ingestion для неотсортированных трейдов

### Цель
Проверить и гарантировать, что агрегатор корректно собирает свечи при поступлении трейдов в произвольном порядке (bulk ingestion unsorted), результат идентичен агрегации отсортированного потока.

### Логика
- Трейды могут приходить не по порядку (shuffle).
- Агрегатор обязан корректно собирать свечи независимо от порядка.
- Необходимо покрыть edge-cases:
  - Дубликаты трейдов
  - Трейды с одинаковым timestamp
  - Трейды на границе свечей (ровно на границе таймфрейма)
  - Пропуски (gaps)
  - Out-of-order trades

### Прогресс
- [x] Пример bulk_ingestion_unsorted.rs реализован (examples/bulk_ingestion_unsorted.rs)
- [x] Тест test_bulk_ingestion_unsorted реализован (src/tests.rs)
- [x] Покрытие out-of-order trades (test_out_of_order_trades)
- [x] Тесты на дубликаты, одинаковые timestamp, boundary trades добавлены (test_bulk_ingestion_duplicates, test_bulk_ingestion_same_timestamp, test_bulk_ingestion_boundary_trades)

### Следующие шаги
1. Зафиксировать архитектурную базу для Super Candles (multi-source ingestion, расширенные метрики).
2. Обновить README.md и документацию.
3. Продолжить покрытие edge-cases и развитие примеров.

### Архитектурные решения
- Stateless-агрегация: результат не зависит от порядка трейдов.
- Все edge-cases должны быть покрыты тестами и примерами.
- Примеры и тесты должны быть максимально наглядными для AGI-ready архитектуры. 

---

## Архитектурный скелет SuperCandleMetric (июнь 2024)

### Цель
Реализовать расширяемую структуру для Super Candle, поддерживающую агрегацию метрик из разных источников событий:
- TradeStats (трейды)
- OrderStats (ордера)
- OBStats (orderbook snapshots)
- FuturesStats (фьючерсные события)

### Архитектура
- Каждая группа метрик реализуется отдельной структурой (TradeStats, OrderStats, OBStats, FuturesStats)
- SuperCandleMetric агрегирует все группы и реализует трейт CandleMetric
- Поддерживаются методы on_trade, on_order, on_orderbook, on_futures для приёма событий
- finalize(&self, candle: &mut Candle) записывает все агрегированные метрики в custom-поле свечи
- Stateless: pipeline допускает поступление событий в любом порядке

### Пример интерфейса
```rust
pub struct SuperCandleMetric {
    pub trade_stats: TradeStats,
    pub order_stats: OrderStats,
    pub ob_stats: OBStats,
    pub futures_stats: FuturesStats,
}

impl SuperCandleMetric {
    pub fn new() -> Self { /* ... */ }
    pub fn on_trade(&mut self, trade: &Trade) { /* ... */ }
    pub fn on_order(&mut self, order: &OrderEvent) { /* ... */ }
    pub fn on_orderbook(&mut self, ob: &OrderBookSnapshot) { /* ... */ }
    pub fn on_futures(&mut self, fut: &FuturesEvent) { /* ... */ }
    pub fn finalize(&self, candle: &mut Candle) { /* ... */ }
}
```

### План реализации
- [x] TradeStats реализован (см. custom_metrics.rs)
- [ ] OrderStats: stub-структура и методы
- [ ] OBStats: stub-структура и методы
- [ ] FuturesStats: stub-структура и методы
- [ ] finalize агрегирует все группы в custom-поле свечи
- [ ] Пример в examples/super_candle.rs: агрегация всех типов событий в одну свечу
- [ ] Обновить README.md: добавить раздел Super Candle, архитектуру, пример
- [ ] Покрыть edge-cases для multi-source ingestion

### Принципы
- Максимальная расширяемость: новые группы метрик и событий добавляются без изменения ядра
- Stateless: pipeline допускает любые сценарии поступления событий
- AGI-ready: структура и интерфейсы прозрачны для автоматического анализа и расширения 