use crate::Price;

pub trait Comparator {
    fn less_than(a: &Price, b: &Price) -> bool;
}

pub struct BuyComparator;

impl Comparator for BuyComparator {
    fn less_than(a: &Price, b: &Price) -> bool {
        a > b
    }
}

pub struct SellComparator;

impl Comparator for SellComparator {
    fn less_than(a: &Price, b: &Price) -> bool {
        a < b
    }
}
