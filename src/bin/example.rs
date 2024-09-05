use std::fs::File;

use ark_groth16::prepare_verifying_key;
use ark_serialize::CanonicalDeserialize;
use vc_prove::{
    circuit::circom_builder,
    get_zk_task_input, get_zk_task_name,
    groth16::{prove, setup, verify},
    params::load_proving_key,
    sample::Sample,
    types::VcProveInput,
    warmup_current_thread,
};

const LOAD_PARAMS: bool = true;

fn main() {
    warmup_current_thread();
    let task_name = get_zk_task_name();
    let command_input = get_zk_task_input();

    let circom = circom_builder(&"output".into(), &task_name);

    // 1. Setup Params
    let pk;
    let vk;
    if LOAD_PARAMS {
        println!("Load params");

        const CHECK_PARAMS: bool = false;
        pk = load_proving_key::<CHECK_PARAMS>(&"output".into(), &task_name).unwrap();
        vk = CanonicalDeserialize::deserialize_uncompressed(
            File::open(format!("output/{task_name}.vk")).unwrap(),
        )
        .unwrap();
    } else {
        println!("Generate params");
        pk = setup(&circom).unwrap();
        vk = prepare_verifying_key(&pk.vk);
    }

    // 2. Prove
    println!("Prove");
    let input = command_input.clone().unwrap_or_else(|| Sample::input());
    let proof = prove(&pk, &circom, input).unwrap();

    // 3. Verify
    println!("Verify");
    let public_input = command_input
        .as_ref()
        .map_or_else(|| Sample::public_input(), VcProveInput::to_verify_input);
    let success = verify(&vk, &proof, &public_input).unwrap();
    assert!(success);

    println!("Done");
}
