use super::Price;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Qty(u64);

impl Qty {
    pub fn new(inner: u64) -> Self {
        Self(inner)
    }

    pub fn from_f64(value: f64) -> Self {
        Self((value * Price::SCALE as f64).round() as u64)
    }

    pub fn inner(&self) -> u64 {
        self.0
    }

    pub fn to_f64(&self) -> f64 {
        self.0 as f64 / Price::SCALE as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qty_construction_and_access() {
        let q = Qty::new(50);
        assert_eq!(q.inner(), 50);
    }

    #[test]
    fn qty_fixed_point
        let q = Qty::from_f64(3.14);
        assert_eq!(q.to_f64(), 3.14);
    }

    #[test]
    fn qty_ordering() {
        let q1 = Qty::new(100);
        let q2 = Qty::new(200);
        assert!(q1 < q2);
    }
}
