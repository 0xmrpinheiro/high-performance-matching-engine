pub const SCALE: u64 = 100_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Price(u64);

impl Price {
    pub const SCALE: u64 = 100_000_000;

    pub fn new(inner: u64) -> Self {
        Self(inner)
    }

    pub fn from_f64(value: f64) -> Self {
        Self((value * SCALE as f64).round() as u64)
    }

    pub fn inner(&self) -> u64 {
        self.0
    }

    pub fn to_f64(&self) -> f64 {
        self.0 as f64 / SCALE as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_construction_and_access() {
        let p = Price::new(100);
        assert_eq!(p.inner(), 100);
    }

    #[test]
    fn price_fixed_point_conversion() {
        let p = Price::from_f64(12.34);
        assert_eq!(p.to_f64(), 12.34);
    }

    #[test]
    fn price_ordering() {
        let p1 = Price::new(100);
        let p2 = Price::new(200);
        assert!(p1 < p2);
        assert!(p2 > p1);
    }
}
