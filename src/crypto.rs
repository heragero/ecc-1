use itertools::join;
use num_bigint::{BigUint, RandBigInt};
use num_traits::{Num, One, Zero};
use rand::{CryptoRng, Rng, thread_rng};
use sha2::{Digest, Sha256};
use std::hint::black_box;

use crate::{
    affine::AffinePoint,
    field::Field,
    modulus::{EllipticCurve, P521},
    projective::ProjectivePoint,
};

#[derive(Clone, Debug)]
pub struct PrivateKey<M: EllipticCurve> {
    pub scalar: Field<M>,
}

#[derive(Clone, Debug)]
pub struct PublicKey<M: EllipticCurve> {
    pub point: AffinePoint<M>,
}

impl<M: EllipticCurve> PrivateKey<M> {
    pub fn generate<R: Rng + CryptoRng>(rng: &mut R) -> Self {
        Self {
            scalar: Field::<M>::get_random(rng),
        }
    }

    pub fn public_key(&self) -> PublicKey<M> {
        let generator_affine = AffinePoint::<M>::get_generator();
        let generator_proj: ProjectivePoint<M> = (generator_affine).into();

        let pub_proj = generator_proj.mul_scalar(&self.scalar);

        PublicKey {
            point: pub_proj.into(),
        }
    }
}

pub struct Ecdh;

impl Ecdh {
    pub fn compute_shared_secret<M: EllipticCurve>(
        sender_priv: &PrivateKey<M>,
        receiver_pub: &PublicKey<M>,
    ) -> Result<BigUint, &'static str> {
        let reciever_pub_proj: ProjectivePoint<M> = receiver_pub.point.clone().into();
        let shared_point_proj = reciever_pub_proj.mul_scalar(&sender_priv.scalar);

        let shared_point_affine: AffinePoint<M> = shared_point_proj.into();

        if shared_point_affine.is_infinity {
            return Err("Shared secret is point at infinity");
        }

        Ok(shared_point_affine.x.number)
    }
}

pub struct StreamCipher;

pub type Ciphertext<M> = (PublicKey<M>, Vec<u8>, Vec<u8>);

impl StreamCipher {
    pub fn encrypt<M: EllipticCurve, R: Rng + CryptoRng>(
        rng: &mut R,
        recievier_pub: &PublicKey<M>,
        message: &[u8],
    ) -> Result<Ciphertext<M>, &'static str> {
        let k: [u8; 32] = rng.r#gen();

        let c_m: Vec<u8> = message
            .iter()
            .zip(k.iter().cycle())
            .map(|(m, k_byte)| m ^ k_byte)
            .collect();

        let ephemeral_priv = PrivateKey::<M>::generate(rng);
        let q_a = ephemeral_priv.public_key();

        let s_x = Ecdh::compute_shared_secret(&ephemeral_priv, recievier_pub)?;

        let mut hasher = Sha256::new();
        hasher.update(s_x.to_bytes_be());
        let wrap_key = hasher.finalize();

        let c_k: Vec<u8> = k
            .iter()
            .zip(wrap_key.iter())
            .map(|(k_byte, w_byte)| k_byte ^ w_byte)
            .collect();

        Ok((q_a, c_k, c_m))
    }

    pub fn decrypt<M: EllipticCurve>(
        sender_private: &PrivateKey<M>,
        receiver_pub: &PublicKey<M>,
        c_k: &[u8],
        c_m: &[u8],
    ) -> Result<Vec<u8>, &'static str> {
        let s_x = Ecdh::compute_shared_secret(sender_private, receiver_pub)?;

        let mut hasher = Sha256::new();
        hasher.update(s_x.to_bytes_be());
        let wrap_key = hasher.finalize();

        let k: Vec<u8> = c_k
            .iter()
            .zip(wrap_key.iter())
            .map(|(c_byte, w_byte)| c_byte ^ w_byte)
            .collect();

        let m: Vec<u8> = c_m
            .iter()
            .zip(k.iter().cycle())
            .map(|(c, k_byte)| c ^ k_byte)
            .collect();

        Ok(m)
    }
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub r: BigUint,
    pub s: BigUint,
}

pub struct Ecdsa;

impl Ecdsa {
    fn hash_message(message: &[u8]) -> BigUint {
        let mut hasher = Sha256::new();
        hasher.update(message);
        let hash_bytes = hasher.finalize();
        BigUint::from_bytes_be(&hash_bytes)
    }

    pub fn sign<M: EllipticCurve, R: Rng + CryptoRng>(
        rng: &mut R,
        private_key: &PrivateKey<M>,
        message: &[u8],
    ) -> Result<Signature, &'static str> {
        let n = M::order();
        let z = Self::hash_message(message);
        let d = &private_key.scalar.number;

        loop {
            let k = rng.gen_biguint_below(n);
            if k.is_zero() {
                continue;
            }

            let gen_affine = AffinePoint::<M>::get_generator();
            let gen_proj: ProjectivePoint<M> = (gen_affine).into();

            let point_proj = gen_proj.mul_scalar(&(&k).into());
            let point_affine: AffinePoint<M> = point_proj.into();

            let r = point_affine.x.number % n;
            if r.is_zero() {
                continue;
            }

            let k_inv = k.modinv(n).expect("k must be invertible");
            let r_times_d = (&r * d) % n;
            let z_plus_rd = (z.clone() + r_times_d) % n;

            let s = (k_inv * z_plus_rd) % n;
            if s.is_zero() {
                continue;
            }

            return Ok(Signature { r, s });
        }
    }

    pub fn verify<M: EllipticCurve>(
        public_key: &PublicKey<M>,
        message: &[u8],
        sig: &Signature,
    ) -> bool {
        let n = M::order();

        if sig.r.is_zero() || &sig.r >= n || sig.s.is_zero() || &sig.s >= n {
            return false;
        }

        let z = Self::hash_message(message);

        let w = match sig.s.modinv(n) {
            Some(inv) => inv,
            None => return false,
        };

        let u1 = (z * &w) % n;
        let u2 = (&sig.r * &w) % n;

        let gen_affine = AffinePoint::<M>::get_generator();
        let gen_proj: ProjectivePoint<M> = (gen_affine).into();
        let pub_proj: ProjectivePoint<M> = public_key.point.clone().into();

        let u1_field = Field::<M>::new(u1);
        let u2_field = Field::<M>::new(u2);

        let point1_proj = gen_proj.mul_scalar(&u1_field);
        let point2_proj = pub_proj.mul_scalar(&u2_field);

        let sum_proj = point1_proj.add_points(&point2_proj);
        let sum_affine: AffinePoint<M> = sum_proj.into();

        if sum_affine.is_infinity {
            return false;
        }

        let x1_mod_n = sum_affine.x.number % n;
        x1_mod_n == sig.r
    }
}

pub fn test_crypto() {
    let mut rng = thread_rng();

    println!("----------------------\n\n----------------------");

    let alice_priv = PrivateKey::<P521>::generate(&mut rng);
    let alice_pub = alice_priv.public_key();

    println!("Alice keys:");
    println!("  private: {}", alice_priv.scalar.to_hex());
    println!("  public:\n{}", alice_pub.point.to_hex());

    let bob_priv = PrivateKey::<P521>::generate(&mut rng);
    let bob_pub = bob_priv.public_key();

    println!("Bob keys:");
    println!("  private: {}", bob_priv.scalar.to_hex());
    println!("  public:\n{}", bob_pub.point.to_hex());

    let shared_alice = Ecdh::compute_shared_secret(&alice_priv, &bob_pub).unwrap();
    println!("shared secret from alice: {shared_alice:0x}");

    let shared_bob = Ecdh::compute_shared_secret(&bob_priv, &alice_pub).unwrap();
    println!("shared secret from bob:   {shared_bob:0x}");

    assert_eq!(shared_alice, shared_bob, "ECDH secrets do not match!");

    println!("----------------------\n\n----------------------");

    let original_message = b"secret message do not read pls!!!";
    println!(
        "original Message: {:?}",
        String::from_utf8_lossy(original_message)
    );

    let (ephemeral_pub, c_k, c_m) =
        StreamCipher::encrypt(&mut rng, &bob_pub, original_message).unwrap();

    let decrypted_message = StreamCipher::decrypt(&bob_priv, &ephemeral_pub, &c_k, &c_m).unwrap();

    assert_eq!(
        original_message.to_vec(),
        decrypted_message,
        "decrypt and original don't match"
    );
    println!(
        "oecrypted message: {:?}",
        String::from_utf8_lossy(&decrypted_message)
    );
    println!("----------------------\n\n----------------------");

    let document = b"data for signing and also signing";

    let signature = Ecdsa::sign(&mut rng, &alice_priv, document).unwrap();
    println!("signature generated:");
    println!("  R: {}", signature.r);
    println!("  S: {}", signature.s);

    let is_valid = Ecdsa::verify(&alice_pub, document, &signature);
    assert!(is_valid, "signature verify failed");
    println!("signature verified");

    let mut tampered_document = document.to_vec();
    tampered_document[0] = b'e';

    let is_tampered_valid = Ecdsa::verify(&alice_pub, &tampered_document, &signature);
    assert!(!is_tampered_valid, "forgery of document");
    println!("changed document with same signature was rejected");
}
