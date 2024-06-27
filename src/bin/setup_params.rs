use std::{fs::File, time::Instant};

use ark_bn254::Bn254;
use ark_groth16::{PreparedVerifyingKey, ProvingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use vc_prove::{circuit::circom_builder, groth16::setup};

fn main() {
    let circom = circom_builder(&"output".into(), "check_vc");

    let (pk, vk) = setup(&circom);

    let writer = File::create("output/check_vc.pk").unwrap();
    pk.serialize_uncompressed(writer).unwrap();

    let writer = File::create("output/check_vc.vk").unwrap();
    vk.serialize_uncompressed(writer).unwrap();

    println!("Start load");

    let start = Instant::now();
    let reader = File::open("output/check_vc.vk").unwrap();
    let load_vk: PreparedVerifyingKey<Bn254> =
        CanonicalDeserialize::deserialize_uncompressed(reader).unwrap();
    println!("Load verifing key time {:?}", start.elapsed());

    let start = Instant::now();
    let reader = File::open("output/check_vc.pk").unwrap();
    let load_pk: ProvingKey<Bn254> =
        CanonicalDeserialize::deserialize_uncompressed_unchecked(reader).unwrap();
    println!("Load proving key time {:?}", start.elapsed());

    std::hint::black_box(load_pk);
    std::hint::black_box(load_vk);
}
