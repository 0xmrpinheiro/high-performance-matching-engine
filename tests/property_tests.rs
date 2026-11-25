use high_performance_matching_engine::order::Order;
use high_performance_matching_engine::order_book::OrderBook;
use high_performance_matching_engine::price::Price;
use high_performance_matching_engine::qty::Qty;
use high_performance_matching_engine::side::Side;
use proptest::prelude::*;

fn make_order_strategy() -> impl Strategy<Value = Order> {
    (
        1..100000u64,
        any::<bool>(),
        1..1000u64,
        1..1000u64,
        1..1000000u64,
    )
        .prop_map(|(id, is_buy, price, qty, timestamp)| Order {
            id,
            side: if is_buy { Side::Buy } else { Side::Sell },
            price: Price::new(price),
            qty: Qty::new(qty),
            timestamp,
        })
}

proptest! {
    #[test]
    fn prop_no_crossed_book(mut orders in prop::collection::vec(make_order_strategy(), 1..50)) {
        let mut book = OrderBook::new();
        for order in orders.drain(..) {
            book.submit(order);
            assert!(!book.is_book_crossed(), "book crossed: bid {:?} >= ask {:?}", book.best_bid_price(), book.best_ask_price());
        }
    }

    #[test]
    fn prop_conservation_of_qty(orders in prop::collection::vec(make_order_strategy(), 1..50)) {
        let mut book = OrderBook::new();
        let mut submitted = 0;
        let mut executed = 0;

        for order in orders {
            submitted += order.qty.inner();
            let result = book.submit(order);
            executed += result.trades.iter().map(|trade| trade.qty.inner()).sum::<u64>();
            // Every trade consumes equal quantities from the maker and taker.
            prop_assert_eq!(book.total_book_qty() + 2 * executed, submitted);
        }
    }

    #[test]
    fn prop_trade_prices_in_range(orders in prop::collection::vec(make_order_strategy(), 1..50)) {
        let mut book = OrderBook::new();
        for order in orders {
            let (best, lower, upper) = match order.side {
                Side::Buy => (book.best_ask_price(), 0, order.price.inner()),
                Side::Sell => (book.best_bid_price(), order.price.inner(), u64::MAX),
            };
            let result = book.submit(order);
            if let Some(first) = result.trades.first() {
                prop_assert_eq!(Some(first.price.inner()), best);
            }
            for (i, trade) in result.trades.iter().enumerate() {
                let price = trade.price.inner();
                prop_assert!((lower..=upper).contains(&price));
                if i > 0 {
                    let previous = result.trades[i - 1].price.inner();
                    match order.side {
                        Side::Buy => prop_assert!(previous <= price),
                        Side::Sell => prop_assert!(previous >= price),
                    }
                }
            }
        }
    }
}
