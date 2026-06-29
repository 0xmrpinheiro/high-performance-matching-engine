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
    fn prop_conservation_of_qty(mut orders in prop::collection::vec(make_order_strategy(), 1..50)) {
        let mut book = OrderBook::new();
        let input_total: u64 = orders.iter().map(|o| o.qty.inner()).sum();

        for order in orders.drain(..) {
            book.submit(order);
        }

        assert!(book.total_book_qty() <= input_total);
    }

    #[test]
    fn prop_no_negative_qty(mut orders in prop::collection::vec(make_order_strategy(), 1..50)) {
        let mut book = OrderBook::new();
        let input_total: u64 = orders.iter().map(|o| o.qty.inner()).sum();

        for order in orders.drain(..) {
            book.submit(order);
        }

        assert!(book.total_book_qty() <= input_total);
    }

    #[test]
    fn prop_trade_prices_in_range(mut orders in prop::collection::vec(make_order_strategy(), 1..50)) {
        let mut book = OrderBook::new();

        for order in orders.drain(..) {
            let best_bid_before = book.best_bid_price();
            let best_ask_before = book.best_ask_price();
            let result = book.submit(order);

            for trade in &result.trades {
                if order.side == Side::Sell {
                    if let Some(bp) = best_bid_before {
                        assert!(trade.price.inner() <= bp, "sell trade price {} exceeds best bid {}", trade.price.inner(), bp);
                    }
                } else {
                    if let Some(ap) = best_ask_before {
                        assert!(trade.price.inner() >= ap, "buy trade price {} below best ask {}", trade.price.inner(), ap);
                    }
                }
            }
        }
    }
}
