# Инструкция по проекту intuition-rust

## Структура проекта

- `src/main.rs` — основной бинарник (collector + api-server + платежи)
- `src/collector/` — сбор данных с бирж
- `src/storage/` — in-memory hash map, list
- `src/api/` — REST/WebSocket API
- `src/payments/` — крипто-платежи
- `src/users/` — управление пользователями и токенами
- `src/logger/` — логирование
- `src/metrics/` — метрики
- `src/notifier/` — нотификации
- `src/types/` — единые типы
- `proto/` — protobuf-схемы
- `tasks/` — задачи и reasoning
- `tests/` — интеграционные и unit-тесты
- `Cargo.toml` — описание зависимостей

## Терминология и стандарты

**ВНИМАНИЕ:** Все структуры данных, protobuf-схемы, API-эндпоинты, расчеты и документация должны строго следовать определениям из [`.cursor/rules/terms.md`](../.cursor/rules/terms.md):
- Exchange
- MarketType
- Market
- Pair (baseId, quoteId)
- Instrument

В расчетах объема в USDT использовать алгоритм из этого файла:
- Если quoteId = USDT, объем = price * amount
- Для других пар: объем = (price * amount) * bestBid_baseId_USDT

**Любые отклонения от терминологии и алгоритмов из `.cursor/rules/terms.md` не допускаются.**

### Пример использования терминов

- Trading Pair: BTC/USDT (baseId=BTC, quoteId=USDT)
- Instrument: BTC/USDT/binance/spot (BTC/USDT на Binance Spot Market)
- MarketType: Spot, Futures, Margin (enum)

### Пример структуры protobuf

```proto
message TradingPair {
  string base_id = 1;   // Например, "BTC"
  string quote_id = 2;  // Например, "USDT"
}

enum MarketType {
  MARKET_TYPE_UNSPECIFIED = 0;
  SPOT = 1;
  FUTURES = 2;
  MARGIN = 3;
}

message Instrument {
  TradingPair pair = 1;
  string exchange = 2;      // Например, "binance"
  MarketType market_type = 3;
}
```

## Публикация и модульность

Проект организован как workspace Rust (Cargo workspaces), что позволяет разрабатывать и публиковать каждый crate независимо. Для этого в каждой подпапке может быть свой Cargo.toml и README.md. Используйте workspace для локальной разработки и публикации отдельных пакетов в open source или внутренние реестры.

## Документация и задачи

- Все reasoning, архитектурные решения и задачи ведутся в папке `tasks/`.
- Все изменения и новые требования фиксируются в соответствующих task-файлах.
- `.cursor/rules/terms.md` — единственный стандарт терминологии и расчетов для всех компонентов. 