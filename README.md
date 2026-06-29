# High-Performance Matching Engine

An in-process, order-book matching engine library in Rust. Limit-only orders, GTC, partial fills, price-time priority.

## Architecture

### Typed Wrappers

All quantities and prices are fixed-point integers under a single `SCALE = 10^8` factor. This eliminates floating-point drift and guarantees deterministic ordering and arithmetic.

```rust
pub const SCALE: u64 = 100_000_000;
```

### Skip List

A custom skip list indexes price levels to achieve `O(log n)` retrieval of the best bid and ask. Raw pointers (`*mut Node`) are confined to `src/skiplist/` and wrapped in exhaustive `unsafe` blocks with safety comments. Each node stores a `LimitLevel` containing a FIFO `VecDeque<Order>` and a cached `total_qty`. Buy and sell sides share the same implementation via a `Comparator` trait; the predicate is flipped for descending (buy) vs. ascending (sell) order, avoiding any duplication.

### OrderBook and `match_orders`

```
submit(incoming) -> MatchResult
```

1. If `incoming.side == Buy`:
   - Walk asks from cheapest upward until the next ask exceeds `incoming.price`.
   - Fill qty oldest-first within each level.
   - If maker qty > taker qty: deduct, emit trade, taker is filled.
   - If maker qty == taker qty: pop maker, emit trade, remove empty level, continue.
   - If maker qty < taker qty: deduct from taker, emit trade, pop maker, continue.
   - If remaining qty > 0, merge into the bids book at `incoming.price`.

2. If `incoming.side == Sell`: symmetric walk over bids from highest downward.

Key properties enforced:
- No negative quantities.
- Book never crosses after any submission.
- FIFO order is preserved within each `LimitLevel`.

## Quick Start

```bash
cargo test           # unit + property + integration tests
cargo test --test property_tests   # proptest-only
cargo run --example partial_fill_demo
```

## Matching Algorithm Walkthrough

Consider a buy of 50 against an ask of 100 with 100 resting qty:

```
Before:  asks[100] = { order#1 qty=100 }
Input:   buy qty=50 @ 100

Tick 1:  best ask = 100, trade qty = min(100, 50) = 50
         maker residual = 50  → returned to level
         taker residual = 0   → order fully filled

After:   asks[100] = { order#1 qty=50 }
         trades: [ { price: 100, qty: 50 } ]
```

## Crate Structure

```
src/
  lib.rs
  order.rs
  side.rs
  price.rs
  qty.rs
  trade.rs
  match_result.rs
  limit_level.rs
  skiplist/
    mod.rs
    node.rs
    comparator.rs
  order_book.rs
tests/
  integration_tests.rs
  property_tests.rs
examples/
  partial_fill_demo.rs
```

## Benchmarks

Run with `cargo bench`. Results from an x86-64 Windows desktop:

| Benchmark | 1,000 levels | 10,000 levels |
|-----------|-------------|--------------|
| Buy sweep (sweep N ask levels) | 6.9 ms median | 610 ms median |
| Sell sweep (sweep N bid levels) | 5.5 ms median | 645 ms median |

Each benchmark pre-populates the book with N price levels (one order per level) and measures only the `submit()` call. Setup is excluded from the timed region via `std::hint::black_box`.

## Dependencies

- `proptest` (dev) — property-based invariants
- `divan` (dev) — benchmark harness
- No external skiplist, no async runtime

## Invariants (Property-Tested)

1. No negative quantities anywhere in the book.
2. All ask prices <= all bid prices (no crossed book).
3. Sum of remaining book qty + sum of trade qty == sum of input qty.
4. Trade prices lie within the crossed range at the time of execution.
5. Within a `LimitLevel`, the deque preserves FIFO insertion order.

## License

MIT
