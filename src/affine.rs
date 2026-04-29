use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul},
};

use num_traits::Zero;
use rand::Rng;

use crate::{field::Field, modulus::EllipticCurve};

#[derive(Debug, PartialEq, PartialOrd, Eq, Default)]
pub struct AffinePoint<M: EllipticCurve> {
    pub x: Field<M>,
    pub y: Field<M>,

    pub is_infinity: bool,
}

impl<M: EllipticCurve> AffinePoint<M> {
    pub fn new(x: Field<M>, y: Field<M>) -> Result<Self, String> {
        let res = Self {
            x,
            y,
            is_infinity: false,
        };

        if !res.is_on_curve() {
            Err("Point with given coordinates is not on curve".to_string())
        } else {
            Ok(res)
        }
    }

    pub fn new_infinity() -> Self {
        Self {
            x: 0.into(),
            y: 0.into(),
            is_infinity: true,
        }
    }

    fn is_on_curve(&self) -> bool {
        if self.is_infinity {
            return true;
        }

        let x = &self.x;
        let y = &self.y;

        let a = Field::<M>::from(M::a());
        let b = Field::<M>::from(M::b());

        let lhs = x.pow(3) + (&a * x) + b;
        let rhs = y * y;

        lhs == rhs
    }

    pub fn get_generator() -> Self {
        Self {
            x: Field::<M>::new(M::gen_x().clone()),
            y: Field::<M>::new(M::gen_y().clone()),
            is_infinity: false,
        }
    }

    pub fn get_random<R: Rng + Sized>(rng: &mut R) -> Self {
        Self::get_generator() * Field::<M>::get_random(rng)
    }

    pub fn add_points(&self, rhs: &Self) -> Self {
        if rhs.is_infinity {
            return self.clone();
        } else if self.is_infinity {
            return rhs.clone();
        } else if self.x == rhs.x && self.y != rhs.y {
            return Self::new_infinity();
        }

        let two = Field::from(2);
        let three = Field::from(3);

        let lambda: Field<M> = if self == rhs {
            // lambda = (3x^2 + a) / 2y
            if self.y.number.is_zero() {
                return Self::new_infinity();
            }
            let a = Field::<M>::from(M::a());
            (three * self.x.pow_field(&two) + a) / (&two * &self.y)
        } else {
            // lambda = (y2 - y1) / (x2 - x1)
            (&rhs.y - &self.y) / (&rhs.x - &self.x)
        };

        let new_x: Field<M> = &lambda.pow_field(&two) - &self.x - &rhs.x;
        let new_y: Field<M> = &lambda * &(&self.x - &new_x) - &self.y;

        Self::new(new_x, new_y).expect("Addition of points needs to be correct")
    }

    pub fn from_hex(x_hex: &str, y_hex: &str) -> Result<Self, String> {
        let x = Field::<M>::from_hex(x_hex).map_err(|err| err.to_string())?;
        let y = Field::<M>::from_hex(y_hex).map_err(|err| err.to_string())?;

        Self::new(x, y)
    }
}

impl<M: EllipticCurve> Add for &AffinePoint<M> {
    type Output = AffinePoint<M>;

    fn add(self, rhs: Self) -> Self::Output {
        self.add_points(rhs)
    }
}

impl<M: EllipticCurve> Add for AffinePoint<M> {
    type Output = AffinePoint<M>;

    fn add(self, rhs: Self) -> Self::Output {
        self.add_points(&rhs)
    }
}

impl<M: EllipticCurve> AddAssign for AffinePoint<M> {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add_points(&rhs);
    }
}

impl<M: EllipticCurve> Mul<&Field<M>> for &AffinePoint<M> {
    type Output = AffinePoint<M>;

    fn mul(self, rhs: &Field<M>) -> Self::Output {
        let mut result = AffinePoint::<M>::new_infinity();
        let mut add = self.clone();

        let bits = rhs.number.bits();

        for i in 0..bits {
            if rhs.number.bit(i) {
                result = &result + &add;
            }
            add = &add + &add;
        }
        result
    }
}

impl<M: EllipticCurve> Mul<Field<M>> for AffinePoint<M> {
    type Output = AffinePoint<M>;

    fn mul(self, rhs: Field<M>) -> Self::Output {
        &self * &rhs
    }
}

impl<M> Clone for AffinePoint<M>
where
    M: EllipticCurve,
{
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
            is_infinity: self.is_infinity,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.x.clone_from(&source.x);
        self.y.clone_from(&source.y);
        self.is_infinity = source.is_infinity;
    }
}

impl<M: EllipticCurve> Display for AffinePoint<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n  x: {}\n  y: {}",
            M::get_modulo(),
            self.x.number,
            self.y.number
        )
    }
}
