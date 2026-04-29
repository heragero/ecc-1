#![allow(clippy::suspicious_arithmetic_impl)]

use std::hint::black_box;
use std::time::Instant;

use rand::thread_rng;

use crate::{
    field::Field,
    modulus::{EllipticCurve, P521},
    projective::ProjectivePoint,
};

mod affine;
mod field;
mod modulus;
mod projective;

const ITERATIONS: u32 = 100;
type MainPoint = ProjectivePoint<P521>;

fn main() {
    println!("Starting Elliptic Curve Operations Benchmark...");
    println!("Curve: P-521");
    println!("Iterations per operation: {}", ITERATIONS);

    let mut rng = thread_rng();

    let order = &Field::<P521>::new(P521::order().clone());
    println!("Generating points...");

    let start = Instant::now();
    let mut points = (0..=(ITERATIONS * 4))
        .map(|_| MainPoint::get_random(&mut rng))
        .collect::<Vec<_>>();
    let time_taken = start.elapsed();

    println!("Time taken: {:?}", time_taken);

    println!("--------------------------------------------------");

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let p1 = points.pop().unwrap();
        let p2 = points.pop().unwrap();

        black_box(p1.add_points(&p2));
    }
    let duration_add = start.elapsed();

    println!("Operation: Point Addition (P1 + P2)");
    println!("Total time: {:?}", duration_add);
    println!("Average time per op: {:?}", duration_add / ITERATIONS);
    println!("--------------------------------------------------");

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let p1 = points.pop().unwrap();

        black_box(p1.double());
    }
    let duration_double = start.elapsed();

    println!("Operation: Point Doubling (2 * P1)");
    println!("Total time: {:?}", duration_double);
    println!("Average time per op: {:?}", duration_double / ITERATIONS);
    println!("--------------------------------------------------");

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let p1 = points.pop().unwrap();

        black_box(p1.mul_scalar(order));
    }
    let duration_mul = start.elapsed();

    println!("Operation: Scalar Multiplication (k3 * P1)");
    println!("Total time: {:?}", duration_mul);
    println!("Average time per op: {:?}", duration_mul / ITERATIONS);
    println!("--------------------------------------------------");

    println!("Benchmark Summary (Average Execution Time):");
    println!("  Addition:       {:?}", duration_add / ITERATIONS);
    println!("  Doubling:       {:?}", duration_double / ITERATIONS);
    println!("  Scalar Mul:     {:?}", duration_mul / ITERATIONS);
}
