use std::collections::VecDeque;

use crate::Order;
use crate::Price;
use crate::Qty;

#[derive(Debug, Clone)]
pub struct LimitLevel {
    pub price: Price,
    pub orders: VecDeque<Order>,
    pub total_qty: Qty,
}

impl LimitLevel {
    pub fn new(price: Price) -> Self {
        Self {
            price,
            orders: VecDeque::new(),
            total_qty: Qty::new(0),
        }
    }

    pub fn push_order(&mut self, order: Order) {
        self.orders.push_back(order);
        self.total_qty = Qty::new(self.total_qty.inner() + order.qty.inner());
    }

    pub fn pop_order(&mut self) -> Option<Order> {
        let order = self.orders.pop_front()?;
        self.total_qty = Qty::new(self.total_qty.inner() - order.qty.inner());
        Some(order)
    }

    pub fn front_order(&self) -> Option<&Order> {
        self.orders.front()
    }

    /// Fill up to `max_qty` from the oldest order without moving its remainder.
    pub(crate) fn fill_front(&mut self, max_qty: u64) -> Option<(u64, Qty)> {
        let order = self.orders.front_mut()?;
        let maker_id = order.id;
        let filled = max_qty.min(order.qty.inner());
        order.qty = Qty::new(order.qty.inner() - filled);
        if order.qty.inner() == 0 {
            self.orders.pop_front();
        }
        self.total_qty = Qty::new(self.total_qty.inner() - filled);
        Some((maker_id, Qty::new(filled)))
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
}
