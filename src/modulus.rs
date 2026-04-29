use num_bigint::BigUint;

pub trait Modulus: PartialEq + Clone {
    fn get_modulo() -> &'static BigUint;
}

pub trait EllipticCurve: Modulus {
    fn a() -> &'static BigUint;
    fn b() -> &'static BigUint;

    fn gen_x() -> &'static BigUint;
    fn gen_y() -> &'static BigUint;
}
