use crate::Order;
use crate::Trade;

#[derive(Debug, Clone, Default)]
pub struct MatchResult {
    pub trades: Vec<Trade>,
    pub resting: Option<Order>,
}
