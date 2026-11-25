use high_performance_matching_engine::order::Order;
use high_performance_matching_engine::order_book::OrderBook;
use high_performance_matching_engine::price::Price;
use high_performance_matching_engine::qty::Qty;
use high_performance_matching_engine::side::Side;

fn make_order(id: u64, side: Side, price: u64, qty: u64, timestamp: u64) -> Order {
    Order {
        id,
        side,
        price: Price::new(price * Price::SCALE),
        qty: Qty::new(qty * Price::SCALE),
        timestamp,
    }
}

fn main() {
    let mut book = OrderBook::new();

    book.submit(make_order(1, Side::Sell, 100, 100, 1));
    book.submit(make_order(2, Side::Sell, 100, 80, 2));
    println!("Resting asks at 100: #1 (100 units), then #2 (80 units)");

    for (id, qty) in [(3, 40), (4, 75)] {
        println!("Buy #{id}: {qty} @ 100");
        let result = book.submit(make_order(id, Side::Buy, 100, qty, id));
        for trade in result.trades {
            println!(
                "  matched {} @ {} with maker #{}",
                trade.qty.to_f64(),
                trade.price.to_f64(),
                trade.maker_id
            );
        }
    }
    println!(
        "Remaining book quantity: {}",
        Qty::new(book.total_book_qty()).to_f64()
    );
}
