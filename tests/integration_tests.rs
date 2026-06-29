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

#[test]
fn integration_empty_book_partial_fill_scenario() {
    let mut book = OrderBook::new();

    let maker = make_order(1, Side::Sell, 101, 100, 1);
    let taker = make_order(2, Side::Buy, 101, 150, 2);

    let result = book.submit(maker);
    assert_eq!(result.trades.len(), 0);
    assert!(result.resting.is_some());

    let result2 = book.submit(taker);
    assert_eq!(result2.trades.len(), 1);
    assert_eq!(result2.trades[0].qty.inner(), 100);
    assert!(result2.resting.is_some());
    assert_eq!(result2.resting.unwrap().qty.inner(), 50);
    assert_eq!(book.ask_levels(), 0);
    assert_eq!(book.bid_levels(), 1);
}

#[test]
fn integration_full_cross() {
    let mut book = OrderBook::new();

    book.submit(make_order(1, Side::Sell, 100, 50, 1));
    book.submit(make_order(2, Side::Sell, 101, 50, 2));
    book.submit(make_order(3, Side::Sell, 102, 50, 3));

    let result = book.submit(make_order(4, Side::Buy, 102, 150, 4));
    assert_eq!(result.trades.len(), 3);
    assert_eq!(result.trades[0].price.inner(), 100);
    assert_eq!(result.trades[1].price.inner(), 101);
    assert_eq!(result.trades[2].price.inner(), 102);
    assert_eq!(book.ask_levels(), 0);
    assert!(result.resting.is_none());
}

#[test]
fn integration_mixed_flow() {
    let mut book = OrderBook::new();

    book.submit(make_order(1, Side::Buy, 100, 100, 1));
    book.submit(make_order(2, Side::Sell, 101, 100, 2));

    let result = book.submit(make_order(3, Side::Sell, 100, 150, 3));
    assert_eq!(result.trades.len(), 1);
    assert_eq!(result.trades[0].price.inner(), 100);
    assert!(result.resting.is_some());
    assert_eq!(result.resting.unwrap().qty.inner(), 50);
    assert_eq!(book.bid_levels(), 0);
    assert_eq!(book.ask_levels(), 2);
}
