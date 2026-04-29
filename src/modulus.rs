use lazy_static::lazy_static;
use num_bigint::BigUint;
use num_traits::Num;

pub trait Modulus: PartialEq + Clone {
    fn get_modulo() -> &'static BigUint;
}

pub trait EllipticCurve: Modulus {
    fn a() -> &'static BigUint;
    fn b() -> &'static BigUint;

    fn gen_x() -> &'static BigUint;
    fn gen_y() -> &'static BigUint;
}

#[derive(PartialEq, Clone)]
pub struct P521 {}

lazy_static! {
    static ref P: BigUint = BigUint::from_str_radix(
            "01ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            16,
        )
    .unwrap();

    static ref A: BigUint = BigUint::from_str_radix(
            "01fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc",
            16,
        )
    .unwrap();

    static ref B: BigUint = BigUint::from_str_radix(
            "0051953eb9618e1c9a1f929a21a0b68540eea2da725b99b315f3b8b489918ef109e156193951ec7e937b1652c0bd3bb1bf073573df883d2c34f1ef451fd46b503f00",
            16,
        )
    .unwrap();

    static ref G_X: BigUint = BigUint::from_str_radix(
            "00c6858e06b70404e9cd9e3ecb662395b4429c648139053fb521f828af606b4d3dbaa14b5e77efe75928fe1dc127a2ffa8de3348b3c1856a429bf97e7e31c2e5bd66",
            16,
        )
    .unwrap();

    static ref G_Y: BigUint = BigUint::from_str_radix(
            "011839296a789a3bc0045c8a5fb42c7d1bd998f54449579b446817afbd17273e662c97ee72995ef42640c550b9013fad0761353c7086a272c24088be94769fd16650",
            16,
        )
    .unwrap();
}

impl Modulus for P521 {
    fn get_modulo() -> &'static BigUint {
        &P
    }
}

impl EllipticCurve for P521 {
    fn a() -> &'static BigUint {
        &A
    }

    fn b() -> &'static BigUint {
        &B
    }

    fn gen_x() -> &'static BigUint {
        &G_X
    }

    fn gen_y() -> &'static BigUint {
        &G_Y
    }
}
