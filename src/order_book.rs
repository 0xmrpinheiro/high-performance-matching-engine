use std::sync::atomic::{AtomicU64, Ordering};

use crate::limit_level::LimitLevel;
use crate::match_result::MatchResult;
use crate::order::Order;
use crate::qty::Qty;
use crate::side::Side;
use crate::skiplist::comparator::{BuyComparator, SellComparator};
use crate::skiplist::SkipList;

pub struct OrderBook {
    bids: SkipList<BuyComparator>,
    asks: SkipList<SellComparator>,
    next_order_id: AtomicU64,
    next_trade_id: AtomicU64,
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: SkipList::new(16),
            asks: SkipList::new(16),
            next_order_id: AtomicU64::new(1),
            next_trade_id: AtomicU64::new(1),
        }
    }

    pub fn submit(&mut self, incoming: Order) -> MatchResult {
        let mut result = MatchResult::default();
        let mut remaining_qty = incoming.qty.inner();

        match incoming.side {
            Side::Buy => {
                while remaining_qty > 0 {
                    let best_ask = match self.asks.front() {
                        Some(node) => node,
                        None => break,
                    };
                    let ask_price = unsafe { (*best_ask).price };

                    if ask_price.inner() > incoming.price.inner() {
                        break;
                    }

                    let limit_level = unsafe { &mut (*best_ask).limit_level };
                    let mut maker_order = match limit_level.pop_order() {
                        Some(order) => order,
                        None => {
                            self.asks.remove(ask_price);
                            continue;
                        }
                    };

                    let maker_qty = maker_order.qty.inner();
                    let taker_qty = remaining_qty;

                    let trade_price = ask_price;
                    let trade_qty = std::cmp::min(maker_qty, taker_qty);
                    let timestamp = incoming.timestamp;

                    result.trades.push(crate::Trade {
                        trade_id: self.next_trade_id.fetch_add(1, Ordering::Relaxed),
                        maker_id: maker_order.id,
                        taker_id: incoming.id,
                        price: trade_price,
                        qty: Qty::new(trade_qty),
                        timestamp,
                    });

                    remaining_qty -= trade_qty;
                    maker_order.qty = Qty::new(maker_qty - trade_qty);

                    if maker_order.qty.inner() > 0 {
                        limit_level.push_order(maker_order);
                    }

                    if limit_level.is_empty() {
                        self.asks.remove(ask_price);
                    }
                }

                if remaining_qty > 0 {
                    let mut resting_order = incoming;
                    resting_order.qty = Qty::new(remaining_qty);

                    if let Some(node) = self.bids.find(resting_order.price) {
                        unsafe {
                            (*node).limit_level.push_order(resting_order);
                        }
                    } else {
                        let mut limit_level = LimitLevel::new(resting_order.price);
                        limit_level.push_order(resting_order);
                        self.bids.insert(resting_order.price, limit_level);
                    }

                    result.resting = Some(resting_order);
                }
            }
            Side::Sell => {
                while remaining_qty > 0 {
                    let best_bid = match self.bids.front() {
                        Some(node) => node,
                        None => break,
                    };
                    let bid_price = unsafe { (*best_bid).price };

                    if bid_price.inner() < incoming.price.inner() {
                        break;
                    }

                    let limit_level = unsafe { &mut (*best_bid).limit_level };
                    let mut maker_order = match limit_level.pop_order() {
                        Some(order) => order,
                        None => {
                            self.bids.remove(bid_price);
                            continue;
                        }
                    };

                    let maker_qty = maker_order.qty.inner();
                    let taker_qty = remaining_qty;

                    let trade_price = bid_price;
                    let trade_qty = std::cmp::min(maker_qty, taker_qty);
                    let timestamp = incoming.timestamp;

                    result.trades.push(crate::Trade {
                        trade_id: self.next_trade_id.fetch_add(1, Ordering::Relaxed),
                        maker_id: maker_order.id,
                        taker_id: incoming.id,
                        price: trade_price,
                        qty: Qty::new(trade_qty),
                        timestamp,
                    });

                    remaining_qty -= trade_qty;
                    maker_order.qty = Qty::new(maker_qty - trade_qty);

                    if maker_order.qty.inner() > 0 {
                        limit_level.push_order(maker_order);
                    }

                    if limit_level.is_empty() {
                        self.bids.remove(bid_price);
                    }
                }

                if remaining_qty > 0 {
                    let mut resting_order = incoming;
                    resting_order.qty = Qty::new(remaining_qty);

                    if let Some(node) = self.asks.find(resting_order.price) {
                        unsafe {
                            (*node).limit_level.push_order(resting_order);
                        }
                    } else {
                        let mut limit_level = LimitLevel::new(resting_order.price);
                        limit_level.push_order(resting_order);
                        self.asks.insert(resting_order.price, limit_level);
                    }

                    result.resting = Some(resting_order);
                }
            }
        }

        result
    }

    pub fn generate_order_id(&self) -> u64 {
        self.next_order_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn bid_levels(&self) -> usize {
        self.bids.len()
    }

    pub fn ask_levels(&self) -> usize {
        self.asks.len()
    }

    pub fn best_bid_price(&self) -> Option<u64> {
        self.bids.front().map(|n| unsafe { (*n).price.inner() })
    }

    pub fn best_ask_price(&self) -> Option<u64> {
        self.asks.front().map(|n| unsafe { (*n).price.inner() })
    }

    pub fn total_book_qty(&self) -> u64 {
        self.bids.total_qty() + self.asks.total_qty()
    }

    pub fn is_book_crossed(&self) -> bool {
        match (self.bids.front(), self.asks.front()) {
            (Some(bid), Some(ask)) => unsafe { (*bid).price.inner() >= (*ask).price.inner() },
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::price::Price;

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
    fn test_empty_book_resting_order() {
        let mut book = OrderBook::new();
        let order = make_order(1, Side::Buy, 100, 100, 1);
        let result = book.submit(order);

        assert_eq!(result.trades.len(), 0);
        assert!(result.resting.is_some());
        assert_eq!(book.bid_levels(), 1);
        assert_eq!(book.ask_levels(), 0);
    }

    #[test]
    fn test_marketable_limit_fills() {
        let mut book = OrderBook::new();

        let resting = make_order(1, Side::Sell, 100, 100, 1);
        book.submit(resting);

        let incoming = make_order(2, Side::Buy, 100, 100, 2);
        let result = book.submit(incoming);

        assert_eq!(result.trades.len(), 1);
        assert_eq!(result.trades[0].qty.inner(), 100);
        assert_eq!(result.trades[0].price.inner(), 100);
        assert!(result.resting.is_none());
        assert_eq!(book.bid_levels(), 0);
        assert_eq!(book.ask_levels(), 0);
    }

    #[test]
    fn test_partial_fill_single_level_taker_fully_filled() {
        let mut book = OrderBook::new();

        let resting = make_order(1, Side::Sell, 100, 100, 1);
        book.submit(resting);

        let incoming = make_order(2, Side::Buy, 100, 50, 2);
        let result = book.submit(incoming);

        assert_eq!(result.trades.len(), 1);
        assert_eq!(result.trades[0].qty.inner(), 50);
        assert!(result.resting.is_none());
        assert_eq!(book.ask_levels(), 1);
    }

    #[test]
    fn test_partial_fill_single_level_maker_partially_filled() {
        let mut book = OrderBook::new();

        let resting = make_order(1, Side::Sell, 100, 50, 1);
        book.submit(resting);

        let incoming = make_order(2, Side::Buy, 100, 100, 2);
        let result = book.submit(incoming);

        assert_eq!(result.trades.len(), 1);
        assert_eq!(result.trades[0].qty.inner(), 50);
        assert!(result.resting.is_some());
        assert_eq!(result.resting.unwrap().qty.inner(), 50);
        assert_eq!(book.bid_levels(), 1);
        assert_eq!(book.ask_levels(), 0);
    }

    #[test]
    fn test_partial_fill_multi_level() {
        let mut book = OrderBook::new();

        book.submit(make_order(1, Side::Sell, 100, 50, 1));
        book.submit(make_order(2, Side::Sell, 101, 50, 2));

        let incoming = make_order(3, Side::Buy, 101, 75, 3);
        let result = book.submit(incoming);

        assert_eq!(result.trades.len(), 2);
        assert_eq!(result.trades[0].qty.inner(), 50);
        assert_eq!(result.trades[0].price.inner(), 100);
        assert_eq!(result.trades[1].qty.inner(), 25);
        assert_eq!(result.trades[1].price.inner(), 101);
        assert_eq!(book.ask_levels(), 1);
        assert!(result.resting.is_none());
    }

    #[test]
    fn test_qty_exhaustion_mid_level() {
        let mut book = OrderBook::new();

        book.submit(make_order(1, Side::Sell, 100, 50, 1));
        book.submit(make_order(2, Side::Sell, 100, 50, 2));
        book.submit(make_order(3, Side::Sell, 100, 50, 3));

        let incoming = make_order(4, Side::Buy, 100, 50, 4);
        let result = book.submit(incoming);

        assert_eq!(result.trades.len(), 1);
        assert_eq!(result.trades[0].qty.inner(), 50);
        assert_eq!(book.ask_levels(), 1);
    }

    #[test]
    fn test_book_uncrossed_after_match() {
        let mut book = OrderBook::new();

        book.submit(make_order(1, Side::Buy, 100, 100, 1));
        book.submit(make_order(2, Side::Sell, 101, 100, 2));

        assert_eq!(book.bid_levels(), 1);
        assert_eq!(book.ask_levels(), 1);
    }

    #[test]
    fn test_non_marketable_limit_rests() {
        let mut book = OrderBook::new();

        book.submit(make_order(1, Side::Sell, 101, 100, 1));

        let incoming = make_order(2, Side::Buy, 100, 100, 2);
        let result = book.submit(incoming);

        assert_eq!(result.trades.len(), 0);
        assert!(result.resting.is_some());
        assert_eq!(book.bid_levels(), 1);
        assert_eq!(book.ask_levels(), 1);
    }

    #[test]
    fn test_sell_sweeps_bids() {
        let mut book = OrderBook::new();

        book.submit(make_order(1, Side::Buy, 101, 50, 1));
        book.submit(make_order(2, Side::Buy, 100, 50, 2));

        let incoming = make_order(3, Side::Sell, 100, 75, 3);
        let result = book.submit(incoming);

        assert_eq!(result.trades.len(), 2);
        assert_eq!(result.trades[0].price.inner(), 101);
        assert_eq!(result.trades[1].price.inner(), 100);
        assert_eq!(book.bid_levels(), 1);
    }

    #[test]
    fn test_fifo_within_level() {
        let mut book = OrderBook::new();

        book.submit(make_order(1, Side::Sell, 100, 50, 1));
        book.submit(make_order(2, Side::Sell, 100, 50, 2));
        book.submit(make_order(3, Side::Sell, 100, 50, 3));

        let incoming = make_order(4, Side::Buy, 100, 75, 4);
        let result = book.submit(incoming);

        assert_eq!(result.trades.len(), 2);
        assert_eq!(result.trades[0].maker_id, 1);
        assert_eq!(result.trades[1].maker_id, 2);
    }
}
