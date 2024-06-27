use ark_bn254::{G1Affine, G2Affine};
use ark_ec::AffineRepr;
use ark_serialize::CanonicalSerialize;
use chrono::NaiveDate;
use keccak_hash::H256;

use rand::SeedableRng;
use rand_xorshift::XorShiftRng;

use vc_prove::{
    circuit::circom_builder,
    groth16::{prove, setup, verify},
    types::{Input, PublicInput, VC},
    warmup_current_thread,
};

const MERKLE_DEPTH: usize = 32;

struct Sample;

impl Sample {
    fn vc() -> VC {
        let vc_json = r#"{"name": "Alice", "age": 25, "birth_date": "20000101", "edu_level": 4, "serial_no": "1234567890"}"#;
        VC::from_json(vc_json).unwrap()
    }

    fn threshold() -> NaiveDate {
        NaiveDate::parse_from_str("20000304", "%Y%m%d").unwrap()
    }

    fn merkle_path(depth: usize) -> Vec<H256> {
        let mut rng = XorShiftRng::seed_from_u64(22);
        let mut random_hash = || {
            let mut x = H256::random_using(&mut rng);
            let (lo, hi) = x.0.split_at_mut(16);
            lo.copy_from_slice(hi);
            x
        };
        (0..depth).map(|_| random_hash()).collect()
    }

    fn input() -> Input {
        Input::new(
            Self::vc(),
            Self::threshold(),
            Self::merkle_path(MERKLE_DEPTH),
            0,
        )
    }

    fn public_input() -> PublicInput {
        PublicInput::new(
            Self::threshold(),
            Self::vc().hash(),
            Self::input().merkle_root(),
        )
    }
}

fn main() {
    warmup_current_thread();

    let circom = circom_builder(&"output".into(), "check_vc");

    // 1. Setup Params
    println!("Generate params");
    let (pk, vk) = setup(&circom);

    // 2. Prove
    println!("Prove");
    let input = Sample::input();
    let proof = prove(&pk, &circom, input);

    // 3. Verify
    println!("Verify");
    let public_input = Sample::public_input();
    let result = verify(&vk, &proof, &public_input);
    assert!(result == true);

    println!("Done");
}
