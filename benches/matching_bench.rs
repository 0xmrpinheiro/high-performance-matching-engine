use divan::Bencher;
use high_performance_matching_engine::order::Order;
use high_performance_matching_engine::order_book::OrderBook;
use high_performance_matching_engine::price::Price;
use high_performance_matching_engine::qty::Qty;
use high_performance_matching_engine::side::Side;

fn main() {
    divan::main();
}

fn make_order(id: u64, side: Side, price: u64, qty: u64, timestamp: u64) -> Order {
    Order {
        id,
        side,
        price: Price::new(price),
        qty: Qty::new(qty),
        timestamp,
    }
}

/// Build a book with `levels` ask levels (prices 1..=levels, qty=1 each)
/// and return a taker buy order that sweeps all levels.
fn setup_ask_book(levels: usize) -> (OrderBook, Order) {
    let mut book = OrderBook::new();
    for i in 1..=levels {
        book.submit(make_order(i as u64, Side::Sell, i as u64, 1, 1));
    }
    let taker = make_order(
        levels as u64 + 1,
        Side::Buy,
        levels as u64,
        levels as u64,
        0,
    );
    (book, taker)
}

/// Build a book with `levels` bid levels and return a taker sell order.
fn setup_bid_book(levels: usize) -> (OrderBook, Order) {
    let mut book = OrderBook::new();
    for i in 1..=levels {
        book.submit(make_order(
            i as u64,
            Side::Buy,
            (levels + 1 - i) as u64,
            1,
            i as u64,
        ));
    }
    let taker = make_order(levels as u64 + 1, Side::Sell, 1, levels as u64, 0);
    (book, taker)
}

// ---------------------------------------------------------------------------
// Benchmark: buy order sweeps N ask levels
// ---------------------------------------------------------------------------
#[divan::bench(args = [1_000, 10_000])]
fn buy_sweep(b: Bencher, levels: usize) {
    b.bench_local(|| {
        let (mut book, taker) = setup_ask_book(levels);
        let result = book.submit(taker);
        std::hint::black_box(result);
    });
}

// ---------------------------------------------------------------------------
// Benchmark: sell order sweeps N bid levels
// ---------------------------------------------------------------------------
#[divan::bench(args = [1_000, 10_000])]
fn sell_sweep(b: Bencher, levels: usize) {
    b.bench_local(|| {
        let (mut book, taker) = setup_bid_book(levels);
        let result = book.submit(taker);
        std::hint::black_box(result);
    });
}
