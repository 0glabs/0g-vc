use std::{collections::HashMap, iter::repeat};

use ark_bn254::Fr;
use chrono::NaiveDate;
use keccak_hash::H256;
use num_bigint::BigInt as CircomBigInt;
use serde::{Deserialize, Serialize};

use super::birthdate_format;
use super::vc::VC;
use crate::{signal::Signal, utils::keccak_tuple};

macro_rules! signal_map {
    ($($k:literal => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k.to_string(), $v.to_signal()),)*]))
    }};
}

pub const MERKLE_DEPTH: usize = 32;

#[derive(Serialize, Deserialize)]
pub struct ProveInput {
    data: VC,
    #[serde(with = "birthdate_format")]
    birthdate_threshold: NaiveDate,
    merkle_proof: Vec<H256>,
    path_index: usize,
}

impl ProveInput {
    pub fn new(
        data: VC,
        birthdate_threshold: NaiveDate,
        merkle_proof: Vec<H256>,
        path_index: usize,
    ) -> Self {
        Self {
            data,
            birthdate_threshold,
            merkle_proof,
            path_index,
        }
    }

    pub fn to_inputs(&self) -> HashMap<String, Vec<CircomBigInt>> {
        signal_map! {
            "encodedVC" => self.data,
            "birthDateThreshold" => self.birthdate_threshold,
            "pathElements" => self.merkle_proof(),
            "pathIndex" => self.path_index,
            "pathLength" => self.merkle_length(),
        }
    }

    pub fn merkle_root(&self) -> H256 {
        let mut hash = self.data.file_hash();
        for (i, &proof) in self.merkle_proof.iter().enumerate() {
            hash = if self.path_index & (0x1 << i) != 0 {
                keccak_tuple(proof, hash)
            } else {
                keccak_tuple(hash, proof)
            };
        }
        hash
    }

    fn merkle_proof(&self) -> Vec<H256> {
        assert!(self.merkle_proof.len() <= MERKLE_DEPTH);
        self.merkle_proof
            .iter()
            .cloned()
            .chain(repeat(H256::default()))
            .take(MERKLE_DEPTH)
            .collect()
    }

    pub fn merkle_length(&self) -> usize {
        self.merkle_proof.len()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifyInput {
    #[serde(with = "birthdate_format")]
    birthdate_threshold: NaiveDate,
    root: H256,
}

impl VerifyInput {
    pub fn new(birthdate_threshold: NaiveDate, root: H256) -> Self {
        Self {
            birthdate_threshold,
            root,
        }
    }

    pub fn to_public_inputs(&self) -> Vec<Fr> {
        [&self.root as &dyn Signal, &self.birthdate_threshold]
            .into_iter()
            .flat_map(Signal::to_signal_fr)
            .collect()
    }
}
