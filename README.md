# Candle Generator
From private project "Marketmaker.cc"

**Stateless, AGI-ready библиотека для агрегации трейдов в свечи (candles) по индустриальным стандартам.**

---

## Features
- Terminology-strict, AGI-ready, modular, and exchange-agnostic
- Protobuf-first: all types/messages в `/proto`, Rust via `prost`/`tonic`
- Strict aggregation chain: m1→m5→m15→m30→h1→h4→d1 (см. ниже)
- USDT volume calculation (fixed/callback/none)
- High-performance bulk ingestion (sorted by default, реализовано в examples/)
- Edge case handling: gaps, empty candles, out-of-order trades
- Optional metrics, super candle support (planned)
- Fast loading from CSV, Parquet, DuckDB, QuestDB (реализовано в examples/)

---

## Quick Start

```rust
use candle_generator::*;

let generator = CandleGenerator::default();
let candles = generator.aggregate(trades.iter(), Timeframe::m1);
```

---

## Advanced Usage & Examples

- Загрузка из CSV: [from_csv.rs](examples/from_csv.rs)
- Загрузка из Parquet: [from_parquet.rs](examples/from_parquet.rs)
- Загрузка из DuckDB: [from_duckdb.rs](examples/from_duckdb.rs)
- Загрузка из QuestDB: [from_questdb.rs](examples/from_questdb.rs)
- Загрузка из ClickHouse: [from_clickhouse.rs](examples/from_clickhouse.rs)
- Экспорт в CSV: [to_csv.rs](examples/to_csv.rs)
- Экспорт в JSON: [to_json.rs](examples/to_json.rs)
- Экспорт в Parquet: [to_parquet.rs](examples/to_parquet.rs)
- Строгая цепочка агрегации: [aggregation_chain.rs](examples/aggregation_chain.rs) (m1→m5→m15→m30→h1→h4→d1)
- Расчёт объёма в USDT: [usdt_volume.rs](examples/usdt_volume.rs) (Fixed, Callback, None)
- Раздельный учёт объёма покупок и продаж: [buy_sell_volume.rs](examples/buy_sell_volume.rs)
- Кастомные метрики (VWAP, super candle и др.): [custom_metrics.rs](examples/custom_metrics.rs)
- Быстрая агрегация большого объёма трейдов: [bulk_ingestion.rs](examples/bulk_ingestion.rs)
- Bulk ingestion с неотсортированными трейдами: [bulk_ingestion_unsorted.rs](examples/bulk_ingestion_unsorted.rs)
- Свечи с пропусками (gaps): [gaps_and_empty_candles.rs](examples/gaps_and_empty_candles.rs)
- Out-of-order трейды: [out_of_order_trades.rs](examples/out_of_order_trades.rs)

---

## Candle Generation Algorithm

CandleGenerator реализует строгую цепочку агрегации:
- m1 свечи строятся из трейдов
- m5 из m1, m15 из m5, m30 из m15, h1 из m30, h4 из h1, d1 из h4
- Каждая свеча старшего таймфрейма строится только после завершения нужного количества младших

**Edge-cases:**
- Обработка пропусков (gaps), пустых свечей, out-of-order трейдов
- Bulk ingestion поддерживает только отсортированные трейды для максимальной производительности

---

## Metrics & Extensibility

- **USDT volume calculation:**
    - Поддержка расчёта объёма в USDT (fixed rate, callback, none)
- **Custom metrics:**
    - Через CandleMetric можно реализовать любые дополнительные метрики (VWAP, buy/sell volume, super candle и др.)
- **Proto/Arrow extensibility:**
    - Для расширения структуры свечи используйте proto/serde-атрибуты

**Пример кастомной метрики:**
```rust
pub trait CandleMetric {
    fn update(&self, trade: &Trade, candle: &mut Candle);
    fn aggregate(&self, src: &[Candle], dst: &mut Candle);
}
```

---

## Roadmap & Optional Features

| Feature                        | Status     | Details/Link                       |
|------------------------------- |----------- |------------------------------------|
| Bulk ingestion (sorted)        | ✅         | В examples/                        |
| Bulk ingestion (unsorted)      | Planned    |                                    |
| Extra metrics (buy/sell, etc.) | Planned    | Через CandleMetric                 |
| Super candle (100+ metrics)    | ✅         | Через кастомные метрики (см. custom_metrics.rs)         |
| Candle history limit           | Planned    |                                    |
| Thread safety                  | Planned    |                                    |
| Multi-instrument support       | Planned    |                                    |
| Event/callback subscriptions   | Planned    |                                    |
| Raw mode for backfill          | Planned    |                                    |
| CSV/Parquet/DuckDB/QuestDB     | ✅         | В examples/                        |
| Metrics pipeline               | Planned    |                                    |
| Proto extensibility            | ✅         |                                    |
| See `/tasks/candle_generator.md` for full details and progress. |

---

## FAQ / Best Practices

- **Какой основной принцип?** — Stateless: библиотека не хранит историю, только агрегирует поток трейдов в свечи.
- **Как добавить новый источник данных?** — Через пример в examples/ и реализацию TradeParser.
- **Как добавить новую метрику?** — Через CandleMetric и расширение структуры Candle.
- **Как реализовать bulk ingestion?** — Используйте примеры в examples/ для загрузки больших объёмов данных.
- **Как работает strict aggregation chain?** — Каждая свеча старшего таймфрейма строится только после завершения нужного количества младших.
- **Как рассчитать объём в USDT?** — Используйте опции CandleConfig: Fixed, Callback, None.
- **Можно ли использовать неотсортированные трейды?** — Да, начиная с версии 0.2.0: поддерживается bulk ingestion с неотсортированными трейдами (см. пример bulk_ingestion_unsorted.rs). Для максимальной производительности рекомендуется подавать отсортированные трейды.
- **Как расширять структуру свечи?** — Через proto/serde-атрибуты и кастомные метрики.

---

## Contribution Guidelines
- Все архитектурные решения и прогресс фиксируются в `/tasks/candle_generator.md`.
- Для предложений и новых сценариев — создавайте PR или issue.
- Следуйте терминологии из `.cursor/rules/terms.md`.
- Все изменения должны обновлять README и tasks/candle_generator.md.

---

## Контакты и вклад
- Для вопросов, идей и сотрудничества — открывайте issue или PR на GitHub.
- Для стратегического партнёрства и AGI-исследований — пишите напрямую мейнтейнеру.

---

## TODO
- [ ] Stateless-агрегатор
- [ ] Примеры для CSV, Parquet, DuckDB, QuestDB, ClickHouse
- [ ] Кастомные метрики и экспорт
- [ ] Документация и тесты

## How to Run Tests and Benchmarks

### Запуск тестов

```sh
cargo test
```

- Покроет все ключевые сценарии, edge-cases и кастомные метрики.

### Запуск бенчмарка

```sh
cargo run --release --example bench
```

- Бенчмарк генерирует 300 миллионов трейдов, агрегирует их в свечи (по 1000 трейдов на одну свечу) и выводит метрики производительности.
- Все результаты выводятся в stdout и сохраняются в файл `bench_output.txt` в формате:
  - `trades` — всего трейдов
  - `candles` — всего свечей
  - `batch` — трейдов на одну свечу
  - `elapsed_secs` — время агрегации в секундах
  - `throughput` — пропускная способность (трейдов/сек)

**Пример вывода:**
```
Генерация 300000000 трейдов (1000 трейдов на свечу)...
Агрегация...
Свечей сгенерировано: 300000
Трейдов на свечу: 1000
Время агрегации: 10.123s
Пропускная способность: 29644712.34 трейдов/сек
```

**Файл bench_output.txt:**
```
trades=300000000
candles=300000
batch=1000
elapsed_secs=10.123456
throughput=29644712.34
```

Используйте эти метрики для автоматизированного анализа производительности и сравнения разных версий библиотеки.

### Как добавить свой пример или тест
- Добавьте файл в папку `examples/` или новый тест в `src/tests.rs`.
- Используйте существующие шаблоны для интеграции новых источников, метрик или edge-cases.

---
