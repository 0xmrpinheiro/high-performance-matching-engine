use crate::Price;
use crate::Qty;
use crate::Side;

#[derive(Debug, Clone, Copy)]
pub struct Order {
    pub id: u64,
    pub side: Side,
    pub price: Price,
    pub qty: Qty,
    pub timestamp: u64,
}
