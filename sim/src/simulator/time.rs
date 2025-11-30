use std::fmt::Display;
use std::iter::Sum;
use std::ops::{Add, Div, Sub, SubAssign};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct STime(pub f64);

impl STime {
    pub const NOW: STime = Self(0.0);
    pub const INFINITY: STime = Self(f64::INFINITY);

    pub fn new(value: f64) -> Self { Self(value)}
}

impl Display for STime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.5}", self.0)
    }
}

impl Add<SDuration> for STime {
    type Output = Self;

    fn add(self, rhs: SDuration) -> Self {
        Self(self.0 + rhs.0)
    }
}


impl Sub<STime> for STime {
    type Output = SDuration;

    fn sub(self, rhs: STime) -> SDuration {
        SDuration(self.0 - rhs.0)
    }
}

impl Sub<&STime> for &STime {
    type Output = SDuration;

    fn sub(self, rhs: &STime) -> Self::Output {
        *self - *rhs
    }
}


impl Sub<SDuration> for STime {
    type Output = Self;

    fn sub(self, rhs: SDuration) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for SDuration {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}


impl Eq for STime {}

impl Ord for STime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
    fn max(self, other: Self) -> Self {
        if self.0 > other.0 {
            self
        } else {
            other
        }
    }
    fn min(self, other: Self) -> Self {
        if self.0 < other.0 {
            self
        } else {
            other
        }
    }
    fn clamp(self, min: Self, max: Self) -> Self {
        if self.0 < min.0 {
            min
        } else if self.0 > max.0 {
            max
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialOrd, PartialEq,  Serialize, Deserialize)]
pub struct SDuration(pub f64);

impl SDuration {
    pub const NOW: SDuration = Self(0.0);
    pub const INFINITY: SDuration = Self(f64::INFINITY);
    pub fn new(value: f64) -> Self { Self(value)}
}

impl Display for SDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "+{:.4}", self.0)
    }
}

impl Eq for SDuration {}

impl<'a> Sum<&'a SDuration> for SDuration {
    fn sum<I: Iterator<Item=&'a Self>>(iter: I) -> Self {
        SDuration::new(iter.map(|d| d.0).sum::<f64>())
    }
}

impl Div<f64> for SDuration {
    type Output = f64;

    fn div(self, rhs: f64) -> Self::Output {
        self.0 / rhs
    }
}


impl Ord for SDuration {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
    fn max(self, other: Self) -> Self {
        if self.0 > other.0 {
            self
        } else {
            other
        }
    }
    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if self.0 < other.0 {
            self
        } else {
            other
        }
    }
    fn clamp(self, min: Self, max: Self) -> Self {
        if self.0 < min.0 {
            min
        } else if self.0 > max.0 {
            max
        } else {
            self
        }

    }
}