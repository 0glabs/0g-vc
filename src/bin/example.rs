use ark_groth16::prepare_verifying_key;
use vc_prove::{
    circuit::circom_builder, groth16::{prove, setup, verify}, sample::Sample, warmup_current_thread
};

fn main() {
    warmup_current_thread();

    let circom = circom_builder(&"output".into(), "check_vc");

    // 1. Setup Params
    println!("Generate params");
    let pk = setup(&circom).unwrap();
    let vk = prepare_verifying_key(&pk.vk);

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
