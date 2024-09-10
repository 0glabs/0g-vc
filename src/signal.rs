use std::collections::HashMap;

use ark_bn254::Fr;
use ark_ff::{PrimeField, Zero};
use ark_std::iterable::Iterable;
use chrono::NaiveDate;
use keccak_hash::H256;
use num_bigint::BigInt as CircomBigInt;
use num_traits::Signed;

use crate::{
    types::{ExtensionSignal, Extensions, NUM_EXTENSIONS, VC},
    utils::date_to_timestamp,
};

pub trait Signal {
    fn to_signal(&self) -> Vec<CircomBigInt>;
    fn to_signal_fr(&self) -> Vec<Fr> {
        self.to_signal().into_iter().map(bitint_to_fr).collect()
    }
}

impl Signal for VC {
    fn to_signal(&self) -> Vec<CircomBigInt> {
        let bytes = self.encode();
        bytes.into_iter().map(CircomBigInt::from).collect()
    }
}

impl Signal for NaiveDate {
    fn to_signal(&self) -> Vec<CircomBigInt> {
        vec![CircomBigInt::from(date_to_timestamp(self))]
    }
}

impl Signal for usize {
    fn to_signal(&self) -> Vec<CircomBigInt> {
        vec![CircomBigInt::from(*self)]
    }
}

impl Signal for u128 {
    fn to_signal(&self) -> Vec<CircomBigInt> {
        vec![CircomBigInt::from(*self)]
    }
}

impl Signal for u8 {
    fn to_signal(&self) -> Vec<CircomBigInt> {
        vec![CircomBigInt::from(*self)]
    }
}

impl Signal for H256 {
    fn to_signal(&self) -> Vec<CircomBigInt> {
        let (lo, hi) = self.0.split_at(16);
        use num_bigint::Sign::Plus as P;
        vec![
            CircomBigInt::from_bytes_le(P, lo),
            CircomBigInt::from_bytes_le(P, hi),
        ]
    }
}

impl Signal for Vec<H256> {
    fn to_signal(&self) -> Vec<CircomBigInt> {
        self.iter().flat_map(Signal::to_signal).collect()
    }
}

impl Signal for Extensions {
    fn to_signal(&self) -> Vec<CircomBigInt> {
        self.iter()
            .map(|x| match x {
                ExtensionSignal::Date(date) => date.to_signal().pop().unwrap(),
                ExtensionSignal::Number(num) => num.to_signal().pop().unwrap(),
            })
            .chain(std::iter::repeat(CircomBigInt::zero()))
            .take(NUM_EXTENSIONS)
            .collect()
    }
}

fn bitint_to_fr(int: CircomBigInt) -> Fr {
    use num_bigint::BigUint;
    let uint = if int.sign() == num_bigint::Sign::Minus {
        let pub_in_abs = int.abs().to_biguint().unwrap();
        BigUint::from(<Fr as PrimeField>::MODULUS) - pub_in_abs
    } else {
        int.to_biguint().unwrap()
    };
    Fr::from(uint)
}

pub trait ProveInput {
    fn to_prove_input(&self) -> HashMap<String, Vec<CircomBigInt>>;
}

pub trait VerifyInput {
    fn to_verify_input(&self) -> Vec<Fr>;
}

impl ProveInput for HashMap<String, Vec<CircomBigInt>> {
    fn to_prove_input(&self) -> HashMap<String, Vec<CircomBigInt>> {
        self.clone()
    }
}

impl ProveInput for HashMap<&'static str, Vec<CircomBigInt>> {
    fn to_prove_input(&self) -> HashMap<String, Vec<CircomBigInt>> {
        self.iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }
}

impl<T: Signal> VerifyInput for T {
    fn to_verify_input(&self) -> Vec<Fr> {
        self.to_signal_fr()
    }
}
