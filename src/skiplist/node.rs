use crate::limit_level::LimitLevel;
use crate::Price;

pub struct Node {
    pub level: usize,
    pub forward: Vec<*mut Node>,
    pub limit_level: LimitLevel,
    pub price: Price,
}

impl Node {
    pub fn new(level: usize, limit_level: LimitLevel, price: Price) -> Self {
        let forward = vec![std::ptr::null_mut(); level + 1];
        Self {
            level,
            forward,
            limit_level,
            price,
        }
    }
}
