use core::ops::*;

#[derive(Debug, Copy, Clone)]
pub struct Saturating {
    val: usize,
    max: usize,
}

impl Saturating {
    #[inline]
    pub fn new(max: usize) -> Saturating {
        Saturating { val: 0, max }
    }

    #[inline]
    pub fn set(&mut self, val: usize) {
        self.val = self.max.min(val)
    }

    pub fn add_check(&mut self, val: usize) -> (usize, bool) {
        let result = self.val + val;
        (result, result > self.max)
    }
}

impl Add<usize> for Saturating {
    type Output = Self;
    fn add(self, rhs: usize) -> Self {
        Self {
            val: self.max.min(self.val.saturating_add(rhs)),
            max: self.max,
        }
    }
}

impl AddAssign<usize> for Saturating {
    fn add_assign(&mut self, rhs: usize) {
        *self = self.add(rhs)
    }
}

impl Sub<usize> for Saturating {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self {
        Self {
            val: self.val.saturating_sub(rhs),
            max: self.max,
        }
    }
}

impl SubAssign<usize> for Saturating {
    fn sub_assign(&mut self, rhs: usize) {
        *self = self.sub(rhs)
    }
}

impl Deref for Saturating {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}
