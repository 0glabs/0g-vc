use std::collections::HashMap;

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

#[derive(Serialize, Deserialize)]
pub struct Input {
    data: VC,
    #[serde(with = "birthdate_format")]
    birthdate_threshold: NaiveDate,
    merkle_proof: Vec<H256>,
    path_index: usize,
}

impl Input {
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
            "pathElements" => self.merkle_proof,
            "pathIndex" => self.path_index,
        }
    }

    pub fn merkle_root(&self) -> H256 {
        let mut hash = self.data.hash();
        for i in 0..3 {
            hash = if self.path_index & (0x1 << i) != 0 {
                keccak_tuple(self.merkle_proof[i], hash)
            } else {
                keccak_tuple(hash, self.merkle_proof[i])
            };
        }
        hash
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicInput {
    #[serde(with = "birthdate_format")]
    birthdate_threshold: NaiveDate,
    pub leaf_hash: H256,
    pub root: H256,
}

impl PublicInput {
    pub fn new(birthdate_threshold: NaiveDate, leaf_hash: H256, root: H256) -> Self {
        Self {
            birthdate_threshold,
            leaf_hash,
            root,
        }
    }

    pub fn to_public_inputs(&self) -> Vec<Fr> {
        [
            &self.leaf_hash as &dyn Signal,
            &self.root,
            &self.birthdate_threshold,
        ]
        .into_iter()
        .flat_map(Signal::to_signal_fr)
        .collect()
    }
}
