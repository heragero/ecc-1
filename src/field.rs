use std::{
    fmt::Display,
    marker::PhantomData,
    ops::{Add, AddAssign, Div, Mul, Sub},
    str::FromStr,
};

use num_bigint::{BigUint, ParseBigIntError, RandBigInt};
use num_traits::{Num, Zero};
use rand::Rng;

use crate::modulus::Modulus;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct Field<M: Modulus> {
    pub number: BigUint,

    modulo: std::marker::PhantomData<M>,
}

impl<M: Modulus> Field<M> {
    pub fn new(number: BigUint) -> Self {
        Self {
            number: number % M::get_modulo(),
            modulo: PhantomData,
        }
    }

    pub fn pow_field(&self, power: &Self) -> Self {
        Self {
            number: self.number.modpow(&power.number, M::get_modulo()),
            modulo: PhantomData,
        }
    }

    pub fn pow(&self, power: u64) -> Self {
        Self::new(self.number.modpow(&BigUint::from(power), M::get_modulo()))
    }

    pub fn get_random<R: Rng>(rng: &mut R) -> Self {
        Self::new(rng.gen_biguint_below(M::get_modulo()))
    }

    pub fn inv(&self) -> Option<Self> {
        self.number.modinv(M::get_modulo()).map(Self::new)
    }

    pub fn one() -> Self {
        Self::from(1)
    }

    // -number by mod
    pub fn inv_additive(&self) -> Self {
        Self::new(M::get_modulo() - &self.number)
    }

    pub fn is_zero(&self) -> bool {
        self.number.is_zero()
    }

    pub const ZERO: Self = Self {
        number: BigUint::ZERO,
        modulo: PhantomData,
    };

    pub fn minus_one() -> Self {
        Self::new(M::get_modulo() - BigUint::from(1_u64))
    }

    pub fn from_hex(hex: &str) -> Result<Self, ParseBigIntError> {
        let clean_hex = hex.trim_start_matches("0x").trim_start_matches("0X");

        let number = BigUint::from_str_radix(clean_hex, 16)?;

        Ok(Self::new(number))
    }

    pub fn to_hex(&self) -> String {
        format!("{:0x}", self.number)
    }
}

impl<M: Modulus> AddAssign for Field<M> {
    fn add_assign(&mut self, rhs: Self) {
        self.number += rhs.number;
        self.number %= M::get_modulo();
    }
}

impl<M: Modulus> Add for Field<M> {
    type Output = Field<M>;

    fn add(self, rhs: Self) -> Self::Output {
        Field::new(self.number + rhs.number)
    }
}

impl<M: Modulus> Add for &Field<M> {
    type Output = Field<M>;

    fn add(self, rhs: Self) -> Self::Output {
        Field::new(&self.number + &rhs.number)
    }
}

impl<M: Modulus> Add<&Field<M>> for Field<M> {
    type Output = Field<M>;

    fn add(self, rhs: &Self) -> Self::Output {
        Field::new(self.number + &rhs.number)
    }
}

impl<M: Modulus> Add<Field<M>> for &Field<M> {
    type Output = Field<M>;

    fn add(self, rhs: Field<M>) -> Self::Output {
        Field::new(&self.number + &rhs.number)
    }
}

impl<M: Modulus> Mul for Field<M> {
    type Output = Field<M>;

    fn mul(self, rhs: Self) -> Self::Output {
        Field::new(self.number * rhs.number)
    }
}

impl<M: Modulus> Mul for &Field<M> {
    type Output = Field<M>;

    fn mul(self, rhs: Self) -> Self::Output {
        Field::new(&self.number * &rhs.number)
    }
}

impl<M: Modulus> Mul<&Field<M>> for Field<M> {
    type Output = Field<M>;

    fn mul(self, rhs: &Field<M>) -> Self::Output {
        Field::new(self.number * &rhs.number)
    }
}

impl<M: Modulus> Mul<Field<M>> for &Field<M> {
    type Output = Field<M>;

    fn mul(self, rhs: Field<M>) -> Self::Output {
        Field::new(&self.number * &rhs.number)
    }
}

impl<M: Modulus> Div for Field<M> {
    type Output = Field<M>;

    fn div(self, rhs: Self) -> Self::Output {
        Field::new(self.number * rhs.number.modinv(M::get_modulo()).unwrap())
    }
}

impl<M: Modulus> Sub for Field<M> {
    type Output = Field<M>;

    fn sub(self, rhs: Self) -> Self::Output {
        Field::new(self.number + M::get_modulo() - rhs.number)
    }
}

impl<M: Modulus> Sub for &Field<M> {
    type Output = Field<M>;

    fn sub(self, rhs: Self) -> Self::Output {
        Field::new(&self.number + M::get_modulo() - &rhs.number)
    }
}

impl<M: Modulus> Sub<&Field<M>> for Field<M> {
    type Output = Field<M>;

    fn sub(self, rhs: &Field<M>) -> Self::Output {
        Field::new(self.number + M::get_modulo() - &rhs.number)
    }
}

impl<M: Modulus> Sub<Field<M>> for &Field<M> {
    type Output = Field<M>;

    fn sub(self, rhs: Field<M>) -> Self::Output {
        Field::new(&self.number + M::get_modulo() - rhs.number)
    }
}

impl<M: Modulus> Clone for Field<M> {
    fn clone(&self) -> Self {
        Self {
            number: self.number.clone(),
            modulo: PhantomData,
        }
    }
}

impl<M: Modulus> Display for Field<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.number))?;
        Ok(())
    }
}

impl<M: Modulus> From<u64> for Field<M> {
    fn from(value: u64) -> Self {
        Field::new(BigUint::from(value))
    }
}

impl<M: Modulus> From<i32> for Field<M> {
    fn from(value: i32) -> Self {
        let unsigned: u32 = value.try_into().unwrap();
        Field::new(BigUint::from(unsigned))
    }
}

impl<M: Modulus> From<&BigUint> for Field<M> {
    fn from(value: &BigUint) -> Self {
        Field::new(value.clone())
    }
}

impl<M: Modulus> FromStr for Field<M> {
    type Err = ParseBigIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Field::new(BigUint::from_str(s)?))
    }
}
