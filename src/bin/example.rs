use std::fs::File;

use ark_groth16::prepare_verifying_key;
use ark_serialize::CanonicalDeserialize;
use vc_prove::{
    circuit::circom_builder,
    groth16::{prove, setup, verify},
    params::load_proving_key,
    sample::Sample,
    warmup_current_thread,
};

const LOAD_PARAMS: bool = true;

fn main() {
    warmup_current_thread();

    let circom = circom_builder(&"output".into(), "check_vc");

    // 1. Setup Params
    let pk;
    let vk;
    if LOAD_PARAMS {
        println!("Load params");

        const CHECK_PARAMS: bool = false;
        pk = load_proving_key::<CHECK_PARAMS>(&"output".into(), "check_vc").unwrap();
        vk = CanonicalDeserialize::deserialize_uncompressed(
            File::open("output/check_vc.vk").unwrap(),
        )
        .unwrap();
    } else {
        println!("Generate params");
        pk = setup(&circom).unwrap();
        vk = prepare_verifying_key(&pk.vk);
    }

    // 2. Prove
    println!("Prove");
    let input = Sample::input();
    let proof = prove(&pk, &circom, input).unwrap();

    // 3. Verify
    println!("Verify");
    let public_input = Sample::public_input();
    let success = verify(&vk, &proof, &public_input).unwrap();
    assert!(success);

    println!("Done");
}
