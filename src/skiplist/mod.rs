pub mod comparator;
pub mod node;

use std::marker::PhantomData;
use std::ptr;

use crate::limit_level::LimitLevel;
use crate::price::Price;
use crate::skiplist::comparator::Comparator;
use crate::skiplist::node::Node;

pub struct SkipList<C: Comparator> {
    max_level: usize,
    current_level: usize,
    head: *mut Node,
    rng: u64,
    _phantom: PhantomData<C>,
}

#[allow(clippy::needless_range_loop)]
impl<C: Comparator> SkipList<C> {
    pub fn new(max_level: usize) -> Self {
        let head = Box::into_raw(Box::new(Node::new(
            max_level,
            LimitLevel::new(Price::new(0)),
            Price::new(0),
        )));
        Self {
            max_level,
            current_level: 0,
            head,
            rng: 1,
            _phantom: PhantomData,
        }
    }

    pub fn insert(&mut self, price: Price, limit_level: LimitLevel) {
        let level = self.random_level();
        let update = &mut vec![ptr::null_mut(); self.max_level + 1];
        let mut current = self.head;

        for i in (0..=self.current_level).rev() {
            unsafe {
                while !(&(*current).forward)[i].is_null()
                    && C::less_than(&(*(&(*current).forward)[i]).price, &price)
                {
                    current = (&(*current).forward)[i];
                }
            }
            update[i] = current;
        }

        current = unsafe { (&(*current).forward)[0] };

        if !current.is_null() && unsafe { (*current).price == price } {
            unsafe { (*current).limit_level = limit_level };
            return;
        }

        if level > self.current_level {
            for i in self.current_level + 1..=level {
                update[i] = self.head;
            }
            self.current_level = level;
        }

        let new_node = Box::into_raw(Box::new(Node::new(level, limit_level, price)));

        for i in 0..=level {
            unsafe {
                (&mut (*new_node).forward)[i] = (&(*update[i]).forward)[i];
                (&mut (*update[i]).forward)[i] = new_node;
            }
        }
    }

    pub fn remove(&mut self, price: Price) -> Option<LimitLevel> {
        let update = &mut vec![ptr::null_mut(); self.max_level + 1];
        let mut current = self.head;

        for i in (0..=self.current_level).rev() {
            unsafe {
                while !(&(*current).forward)[i].is_null()
                    && C::less_than(&(*(&(*current).forward)[i]).price, &price)
                {
                    current = (&(*current).forward)[i];
                }
            }
            update[i] = current;
        }

        current = unsafe { (&(*current).forward)[0] };

        if current.is_null() || unsafe { (*current).price != price } {
            return None;
        }

        for i in 0..=self.current_level {
            unsafe {
                if (&(*update[i]).forward)[i] != current {
                    break;
                }
                (&mut (*update[i]).forward)[i] = (&(*current).forward)[i];
            }
        }

        unsafe {
            let limit_level = (*current).limit_level.clone();
            let _ = Box::from_raw(current);
            Some(limit_level)
        }
    }

    pub fn find(&self, price: Price) -> Option<*mut Node> {
        let mut current = self.head;

        for i in (0..=self.current_level).rev() {
            unsafe {
                while !(&(*current).forward)[i].is_null()
                    && C::less_than(&(*(&(*current).forward)[i]).price, &price)
                {
                    current = (&(*current).forward)[i];
                }
            }
        }

        let candidate = unsafe { (&(*current).forward)[0] };
        if !candidate.is_null() && unsafe { (*candidate).price == price } {
            Some(candidate)
        } else {
            None
        }
    }

    pub fn front(&self) -> Option<*mut Node> {
        if self.head.is_null() {
            return None;
        }
        let first = unsafe { (&(*self.head).forward)[0] };
        if first.is_null() {
            None
        } else {
            Some(first)
        }
    }

    pub fn pop_front(&mut self) -> Option<LimitLevel> {
        let node_ptr = self.front()?;
        let price = unsafe { (*node_ptr).price };
        self.remove(price)
    }

    pub fn is_empty(&self) -> bool {
        self.front().is_none()
    }

    pub fn len(&self) -> usize {
        let mut count = 0;
        let mut current = unsafe { (&(*self.head).forward)[0] };
        while !current.is_null() {
            count += 1;
            current = unsafe { (&(*current).forward)[0] };
        }
        count
    }

    pub fn total_qty(&self) -> u64 {
        let mut total = 0u64;
        let mut current = unsafe { (&(*self.head).forward)[0] };
        while !current.is_null() {
            total += unsafe { (*current).limit_level.total_qty.inner() };
            current = unsafe { (&(*current).forward)[0] };
        }
        total
    }

    fn random_level(&mut self) -> usize {
        // Advance on every insert. Using high bits avoids the simple cycle of
        // the low bits of a linear congruential generator.
        self.rng = self
            .rng
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.rng.leading_zeros() as usize).min(self.max_level)
    }
}

impl<C: Comparator> Drop for SkipList<C> {
    fn drop(&mut self) {
        let mut current = unsafe { (&(*self.head).forward)[0] };
        while !current.is_null() {
            let next = unsafe { (&(*current).forward)[0] };
            let _ = unsafe { Box::from_raw(current) };
            current = next;
        }
        let _ = unsafe { Box::from_raw(self.head) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::Order;
    use crate::qty::Qty;
    use crate::side::Side;
    use crate::skiplist::comparator::{BuyComparator, SellComparator};

    #[test]
    fn test_insert_creates_upper_links() {
        let mut levels = SkipList::<SellComparator>::new(4);
        for price in 1..=64 {
            let price = Price::new(price);
            levels.insert(price, LimitLevel::new(price));
        }

        assert!(levels.current_level > 0);
        assert!(levels.current_level <= levels.max_level);
        for price in (1..=64).rev() {
            let price = Price::new(price);
            assert!(levels.find(price).is_some());
            assert_eq!(levels.remove(price).unwrap().price, price);
        }
        assert!(levels.is_empty());
    }

    #[test]
    fn test_insert_and_find_buy() {
        let mut sl: SkipList<BuyComparator> = SkipList::new(4);
        sl.insert(Price::new(100), LimitLevel::new(Price::new(100)));
        sl.insert(Price::new(200), LimitLevel::new(Price::new(200)));
        sl.insert(Price::new(150), LimitLevel::new(Price::new(150)));

        assert!(sl.find(Price::new(100)).is_some());
        assert!(sl.find(Price::new(150)).is_some());
        assert!(sl.find(Price::new(200)).is_some());
        assert!(sl.find(Price::new(50)).is_none());
    }

    #[test]
    fn test_insert_and_find_sell() {
        let mut sl: SkipList<SellComparator> = SkipList::new(4);
        sl.insert(Price::new(100), LimitLevel::new(Price::new(100)));
        sl.insert(Price::new(200), LimitLevel::new(Price::new(200)));
        sl.insert(Price::new(150), LimitLevel::new(Price::new(150)));

        assert!(sl.find(Price::new(100)).is_some());
        assert!(sl.find(Price::new(150)).is_some());
        assert!(sl.find(Price::new(200)).is_some());
        assert!(sl.find(Price::new(50)).is_none());
    }

    #[test]
    fn test_buy_front_returns_highest() {
        let mut sl: SkipList<BuyComparator> = SkipList::new(4);
        sl.insert(Price::new(100), LimitLevel::new(Price::new(100)));
        sl.insert(Price::new(200), LimitLevel::new(Price::new(200)));
        sl.insert(Price::new(150), LimitLevel::new(Price::new(150)));

        let front = sl.front().unwrap();
        unsafe {
            assert_eq!((*front).price.inner(), 200);
        }
    }

    #[test]
    fn test_sell_front_returns_lowest() {
        let mut sl: SkipList<SellComparator> = SkipList::new(4);
        sl.insert(Price::new(100), LimitLevel::new(Price::new(100)));
        sl.insert(Price::new(200), LimitLevel::new(Price::new(200)));
        sl.insert(Price::new(150), LimitLevel::new(Price::new(150)));

        let front = sl.front().unwrap();
        unsafe {
            assert_eq!((*front).price.inner(), 100);
        }
    }

    #[test]
    fn test_remove_buy() {
        let mut sl: SkipList<BuyComparator> = SkipList::new(4);
        sl.insert(Price::new(100), LimitLevel::new(Price::new(100)));
        sl.insert(Price::new(200), LimitLevel::new(Price::new(200)));

        assert_eq!(sl.len(), 2);
        sl.remove(Price::new(200));
        assert_eq!(sl.len(), 1);
        assert!(sl.find(Price::new(200)).is_none());
        assert!(sl.find(Price::new(100)).is_some());

        let front = sl.front().unwrap();
        unsafe {
            assert_eq!((*front).price.inner(), 100);
        }
    }

    #[test]
    fn test_remove_sell() {
        let mut sl: SkipList<SellComparator> = SkipList::new(4);
        sl.insert(Price::new(100), LimitLevel::new(Price::new(100)));
        sl.insert(Price::new(200), LimitLevel::new(Price::new(200)));

        sl.remove(Price::new(100));
        assert_eq!(sl.len(), 1);
        assert!(sl.find(Price::new(100)).is_none());

        let front = sl.front().unwrap();
        unsafe {
            assert_eq!((*front).price.inner(), 200);
        }
    }

    #[test]
    fn test_duplicate_price_insert_replaces() {
        let mut sl: SkipList<BuyComparator> = SkipList::new(4);
        let mut level1 = LimitLevel::new(Price::new(100));
        level1.push_order(Order {
            id: 1,
            side: Side::Buy,
            price: Price::new(100),
            qty: Qty::new(100),
            timestamp: 1,
        });

        let mut level2 = LimitLevel::new(Price::new(100));
        level2.push_order(Order {
            id: 2,
            side: Side::Buy,
            price: Price::new(100),
            qty: Qty::new(200),
            timestamp: 2,
        });

        sl.insert(Price::new(100), level1);
        sl.insert(Price::new(100), level2);

        assert_eq!(sl.len(), 1);
        let node = sl.find(Price::new(100)).unwrap();
        unsafe {
            assert_eq!((*node).limit_level.total_qty.inner(), 200);
        }
    }

    #[test]
    fn test_pop_front_buy() {
        let mut sl: SkipList<BuyComparator> = SkipList::new(4);
        sl.insert(Price::new(100), LimitLevel::new(Price::new(100)));
        sl.insert(Price::new(200), LimitLevel::new(Price::new(200)));

        let popped = sl.pop_front().unwrap();
        assert_eq!(popped.price.inner(), 200);
        assert_eq!(sl.len(), 1);
    }

    #[test]
    fn test_pop_front_sell() {
        let mut sl: SkipList<SellComparator> = SkipList::new(4);
        sl.insert(Price::new(100), LimitLevel::new(Price::new(100)));
        sl.insert(Price::new(200), LimitLevel::new(Price::new(200)));

        let popped = sl.pop_front().unwrap();
        assert_eq!(popped.price.inner(), 100);
        assert_eq!(sl.len(), 1);
    }

    #[test]
    fn test_is_empty() {
        let mut sl: SkipList<BuyComparator> = SkipList::new(4);
        assert!(sl.is_empty());
        sl.insert(Price::new(100), LimitLevel::new(Price::new(100)));
        assert!(!sl.is_empty());
        sl.remove(Price::new(100));
        assert!(sl.is_empty());
    }
}
