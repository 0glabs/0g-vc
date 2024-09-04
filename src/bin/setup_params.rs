use std::{fs::File, time::Instant};

use ark_bn254::Bn254;
use ark_groth16::{prepare_verifying_key, PreparedVerifyingKey};
use ark_serialize::CanonicalDeserialize;

use vc_prove::{
    circuit::circom_builder,
    get_zk_task_name,
    groth16::setup,
    params::{load_proving_key, save_key},
};

fn main() {
    let name = get_zk_task_name();

    let start = Instant::now();
    let circom = circom_builder(&"output".into(), &name);
    println!("Load circuit time {:?}", start.elapsed());

    let pk = setup(&circom).unwrap();
    let vk = prepare_verifying_key(&pk.vk);

    println!("Start save");
    save_key(&"output".into(), &name, pk.clone()).unwrap();

    println!("Start load");
    let start = Instant::now();
    let reader = File::open(format!("output/{name}.vk")).unwrap();
    let load_vk = PreparedVerifyingKey::<Bn254>::deserialize_uncompressed(reader).unwrap();
    println!("Load verifing key time {:?}", start.elapsed());
    if load_vk != vk {
        panic!("incorrect vk");
    }

    let start = Instant::now();
    let load_pk = load_proving_key::<false>(&"output".into(), &name).unwrap();
    println!("Load proving key time {:?}", start.elapsed());
    if load_pk != pk {
        panic!("incorrect pk");
    }

    std::hint::black_box(load_pk);
    std::hint::black_box(load_vk);
}
