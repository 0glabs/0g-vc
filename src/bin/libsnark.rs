use std::time::Instant;

use vc_prove::{
    circuit::circom_builder,
    libsnark::{make_constraints, make_input},
    sample::Sample,
};

use libsnark_rust::snark::{prove, setup, verify};
use libsnark_rust::utils::{init_public_params, reset_profile};

fn main() {
    init_public_params();

    let mut circom = circom_builder(&"output".into(), "check_vc");
    let input = Sample::input();
    circom.inputs = input.to_inputs();

    println!("Make constraints and inputs");

    let start = Instant::now();
    let cs = make_constraints(&circom.setup());
    let (primary_input, auxiliary_input) = make_input(&circom.build().unwrap()).unwrap();
    println!("Done. {:?}", start.elapsed());

    assert!(cs.is_satisfied(&primary_input, &auxiliary_input));

    reset_profile();
    let key = setup(&cs);

    reset_profile();
    let proof = prove(&key, &primary_input, &auxiliary_input);

    reset_profile();
    let res = verify(&key, &primary_input, &proof);
    assert!(res);

    println!("Done");
}
