use crate::Price;
use crate::Qty;

#[derive(Debug, Clone)]
pub struct Trade {
    pub trade_id: u64,
    pub maker_id: u64,
    pub taker_id: u64,
    pub price: Price,
    pub qty: Qty,
    pub timestamp: u64,
}
