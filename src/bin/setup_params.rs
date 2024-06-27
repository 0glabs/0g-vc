use std::{fs::File, time::Instant};

use ark_bn254::Bn254;
use ark_groth16::PreparedVerifyingKey;
use ark_serialize::CanonicalDeserialize;

use vc_prove::{
    circuit::circom_builder,
    groth16::setup,
    params::{load_proving_key, save_key},
};

fn main() {
    let start = Instant::now();
    let circom = circom_builder(&"output".into(), "check_vc");
    println!("Load circuit time {:?}", start.elapsed());

    let (pk, vk) = setup(&circom);

    println!("Start save");
    save_key(&"output".into(), "check_vc", pk.clone()).unwrap();

    println!("Start load");

    let start = Instant::now();
    let reader = File::open("output/check_vc.vk").unwrap();
    let load_vk = PreparedVerifyingKey::<Bn254>::deserialize_uncompressed(reader).unwrap();
    println!("Load verifing key time {:?}", start.elapsed());
    if load_vk != vk {
        panic!("incorrect vk");
    }

    let start = Instant::now();
    let load_pk = load_proving_key::<false>(&"output".into(), "check_vc").unwrap();
    println!("Load proving key time {:?}", start.elapsed());
    if load_pk != pk {
        panic!("incorrect pk");
    }

    std::hint::black_box(load_pk);
    std::hint::black_box(load_vk);
}
