use std::{collections::HashMap, iter::repeat};

use super::{
    array::ByteArray,
    ext::{ExtensionSignal, Extensions},
    vc::VC_LEN,
};
use ark_bn254::Fr;
use chrono::NaiveDate;
use keccak_hash::{keccak, H256};
use num_bigint::BigInt as CircomBigInt;
use serde::{Deserialize, Serialize};

use super::vc::VC;
use crate::{
    aes::encrypt,
    signal::{ProveInput, Signal, VerifyInput},
    utils::keccak_tuple,
};

macro_rules! signal_map {
    ($($k:literal => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k.to_string(), $v.to_signal()),)*]))
    }};
}

pub const MERKLE_DEPTH: usize = 32;

#[derive(Serialize, Deserialize, Clone)]
pub struct VcProveInput {
    key: ByteArray<16>,
    iv: ByteArray<16>,
    data: VC,
    merkle_proof: Vec<H256>,
    path_index: usize,
    extensions: Extensions,
}

impl VcProveInput {
    pub fn new(
        data: VC,
        key: ByteArray<16>,
        iv: ByteArray<16>,
        birthdate_threshold: NaiveDate,
        merkle_proof: Vec<H256>,
        path_index: usize,
    ) -> Self {
        let extensions = vec![ExtensionSignal::Date(birthdate_threshold)]
            .try_into()
            .unwrap();
        Self {
            data,
            key,
            iv,
            extensions,
            merkle_proof,
            path_index,
        }
    }

    pub fn to_inputs(&self) -> HashMap<String, Vec<CircomBigInt>> {
        signal_map! {
            "aesKey" => self.key,
            "aesIV" => self.iv,
            "encodedVC" => self.data,
            "extensions" => self.extensions,
            "pathElements" => self.merkle_proof(),
            "pathIndex" => self.path_index,
            "pathLength" => self.merkle_length(),
        }
    }

    pub fn to_verify_input(&self) -> VcVerifyInput {
        VcVerifyInput {
            root: self.merkle_root(),
            extensions: self.extensions.clone(),
        }
    }

    pub fn plaintext(&self) -> [u8; VC_LEN + 32] {
        self.data.plaintext()
    }

    pub fn ciphertext(&self) -> [u8; VC_LEN + 32] {
        let plaintext = self.data.plaintext();
        encrypt(self.key.as_ref(), self.iv.as_ref(), &plaintext)
    }

    pub fn leaf(&self) -> [u8; 256] {
        let mut leaf = [0u8; 256];
        leaf[0..16].copy_from_slice(self.iv.as_ref());
        leaf[16..16 + VC_LEN + 32].copy_from_slice(&self.ciphertext());
        leaf
    }

    pub fn leaf_hash(&self) -> H256 {
        keccak(self.leaf())
    }

    pub fn merkle_root(&self) -> H256 {
        let mut hash = self.leaf_hash();
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

impl ProveInput for VcProveInput {
    fn to_prove_input(&self) -> HashMap<String, Vec<CircomBigInt>> {
        self.to_inputs()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VcVerifyInput {
    root: H256,
    extensions: Extensions,
}

impl VcVerifyInput {
    pub fn new(birthdate_threshold: NaiveDate, root: H256) -> Self {
        let extensions = vec![ExtensionSignal::Date(birthdate_threshold)]
            .try_into()
            .unwrap();
        Self { extensions, root }
    }

    pub fn to_public_inputs(&self) -> Vec<Fr> {
        [&self.root as &dyn Signal, &self.extensions]
            .into_iter()
            .flat_map(Signal::to_signal_fr)
            .collect()
    }
}

impl VerifyInput for VcVerifyInput {
    fn to_verify_input(&self) -> Vec<Fr> {
        self.to_public_inputs()
    }
}
