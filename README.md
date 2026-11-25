# Rust Limit Order Book & Matching Engine

An in-memory Rust limit order book with **price-time priority** and partial fills.
Run the demo to see two orders at the same price and verify that the first order
keeps its place after a partial fill. The library has no runtime dependencies.

## Quick start

With Rust 1.85 or newer and a working Rust linker toolchain:

```bash
git clone https://github.com/0xmrpinheiro/high-performance-matching-engine.git
cd high-performance-matching-engine
cargo run --locked --example partial_fill_demo
cargo test --locked
```

```text
Resting asks at 100: #1 (100 units), then #2 (80 units)
Buy #3: 40 @ 100
  matched 40 @ 100 with maker #1
Buy #4: 75 @ 100
  matched 60 @ 100 with maker #1
  matched 15 @ 100 with maker #2
Remaining book quantity: 65
```

## Matching behavior

- Accepts limit orders only. Unfilled quantities rest at the order's limit price.
- Buys match the lowest eligible ask first; sells match the highest eligible bid
  first. Trades execute at the resting order's price.
- At a given price, orders fill in submission order. A partially filled resting
  order retains its position until its remaining quantity is filled.
- Prices and quantities use fixed-point integers scaled by `10^8`. `Price::new`
  and `Qty::new` accept already-scaled integers.

Each side uses a deterministic skip list for price levels. Reading the best price
uses the first node; a full sweep may consume many resting orders and remove
multiple levels, so matching is **not** a constant-time operation. Each level
stores its resting orders in a FIFO queue with a cached total quantity.

This is a single-process library, not an exchange service. There is no cancel or
amend API, persistence, networking, or risk management.

## Benchmarks

```bash
cargo bench --locked --bench matching_bench -- --sample-count 100 --sample-size 1
```

Divan builds a fresh book for each iteration outside the timed section. Each book
has one resting order per price level. The benchmark times one `submit()` that
sweeps the entire opposite side; this includes trade creation and the removal of
filled price levels, but not book construction or destruction of the final result.

Example medians from one run on Windows 11 x86-64, AMD Ryzen 7 3800X,
Rust 1.99.1, default optimized bench profile:

| Workload | 1,000 levels | 10,000 levels |
| --- | ---: | ---: |
| Buy sweeps asks | 299.6 µs | 2.717 ms |
| Sell sweeps bids | 286.2 µs | 1.885 ms |

These are times for the whole sweep on that machine, not per-trade latencies,
end-to-end exchange latency, or a throughput guarantee.

## Tests and source

- [`src/order_book.rs`](src/order_book.rs) — submission and matching
- [`src/skiplist/`](src/skiplist/) — ordered index of price levels
- [`tests/integration_tests.rs`](tests/integration_tests.rs) — partial-fill and FIFO regressions
- [`tests/property_tests.rs`](tests/property_tests.rs) — no crossed book, trade price ordering, and quantity conservation: `resting + 2 × executed = submitted`

## License

[MIT](LICENSE)
