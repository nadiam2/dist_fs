// Double sided modular arithmetic
use std::ops::{Add, Deref, DerefMut, Sub};

#[derive(Debug, Clone)]
pub struct Modular {
    remainder: u32,
    modulus: u32
}

impl Modular {
    pub fn new(remainder: i32, modulus: u32) -> Self {
        if modulus == 0 {
            panic!("Modular instantiated with modulus = 0");
        }
        Modular {
            remainder: Self::normalize_remainder(remainder, modulus),
            modulus: modulus
        }
    }
    fn normalize_remainder(remainder: i32, modulo: u32) -> u32 {
        match remainder < 0 {
            true  => {
                let close_remainder = remainder % (modulo as i32);
                (modulo as i32 + close_remainder) as u32 % modulo
            },
            false => remainder as u32 % modulo
        }
    }
    fn assert_same_modulus(&self, other: &Self) {
        if self.modulus != other.modulus {
            panic!("Cannot do operation on Modulars with different moduluses");
        }
    }
}

impl Deref for Modular {
    type Target = u32;
    fn deref(&self) -> &u32 {
        &self.remainder
    }
}

impl Add for Modular {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.assert_same_modulus(&rhs);
        Modular {
            remainder: Self::normalize_remainder((self.remainder + rhs.remainder) as i32, self.modulus),
            modulus: self.modulus
        }
    }
}

impl Add<i32> for Modular {
    type Output = Self;
    fn add(self, rhs: i32) -> Self {
        Modular {
            remainder: Self::normalize_remainder(self.remainder as i32 + rhs, self.modulus),
            modulus: self.modulus
        }
    }
}

impl Add<u32> for Modular {
    type Output = Self;
    fn add(self, rhs: u32) -> Self {
        Modular {
            remainder: Self::normalize_remainder((self.remainder + rhs) as i32, self.modulus),
            modulus: self.modulus
        }
    }
}

impl Sub for Modular {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.assert_same_modulus(&rhs);
        Modular {
            remainder: Self::normalize_remainder((self.remainder as i32) - (rhs.remainder as i32), self.modulus),
            modulus: self.modulus
        }
    }
}

impl Sub<i32> for Modular {
    type Output = Self;
    fn sub(self, rhs: i32) -> Self {
        Modular {
            remainder: Self::normalize_remainder((self.remainder as i32) - rhs, self.modulus),
            modulus: self.modulus
        }
    }
}


