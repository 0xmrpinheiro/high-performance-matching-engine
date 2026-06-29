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

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
}
