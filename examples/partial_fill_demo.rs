use high_performance_matching_engine::order::Order;
use high_performance_matching_engine::order_book::OrderBook;
use high_performance_matching_engine::price::Price;
use high_performance_matching_engine::qty::Qty;
use high_performance_matching_engine::side::Side;

fn make_order(id: u64, side: Side, price: u64, qty: u64, timestamp: u64) -> Order {
    Order {
        id,
        side,
        price: Price::new(price),
        qty: Qty::new(qty),
        timestamp,
    }
}

fn main() {
    let mut book = OrderBook::new();

    println!("=== Partial-Fill Demo ===\n");

    let maker = make_order(1, Side::Sell, 100, 100, 1);
    println!("Submitted: Sell 100 @ 100 (maker)");
    book.submit(maker);
    println!("Book state: 1 ask level @ 100 (100 qty), 0 bid levels\n");

    let taker = make_order(2, Side::Buy, 100, 50, 2);
    println!("Submitted: Buy 50 @ 100 (taker)");
    let result = book.submit(taker);
    println!("Result:");
    println!("  - {} trade(s)", result.trades.len());
    for (i, trade) in result.trades.iter().enumerate() {
        println!(
            "    Trade {}: {} @ {} (maker={}, taker={})",
            i + 1,
            trade.qty.to_f64(),
            trade.price.to_f64(),
            trade.maker_id,
            trade.taker_id
        );
    }
    if let Some(resting) = result.resting {
        println!(
            "  - Resting: {:?} {} @ {} (id={})",
            resting.side,
            resting.qty.to_f64(),
            resting.price.to_f64(),
            resting.id
        );
    }
    println!(
        "Book state: {} ask level(s), {} bid level(s)\n",
        book.ask_levels(),
        book.bid_levels()
    );
}
