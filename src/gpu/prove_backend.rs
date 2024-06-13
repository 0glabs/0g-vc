use ark_bn254::{Bn254, Fr};
use ark_circom::{CircomBuilder, CircomCircuit, CircomConfig};
use num_bigint::BigInt;
use std::collections::HashMap;
use std::path::Path;

use ark_groth16::{prepare_verifying_key, Groth16, Proof, ProvingKey};
use ark_std::rand::{thread_rng, Rng};

pub fn cal_witness(
    wtns: impl AsRef<Path>,
    r1cs: impl AsRef<Path>,
    inputs: HashMap<String, Vec<BigInt>>,
) -> Result<(CircomCircuit<Bn254>, Vec<Fr>), Box<dyn std::error::Error>>
{
    let cfg = CircomConfig::<Bn254>::new(wtns, r1cs).unwrap();
    let mut builder = CircomBuilder::new(cfg);

    // 遍历输入参数的 HashMap，并为每个输入调用 push_input 函数
    for (name, values) in inputs {
        for value in values {
            builder.push_input(&name, value);
        }
    }
    // println!("builder inputs:{:?}", builder.inputs);
    let circom = builder.build()?;
    let pub_in = circom.get_public_inputs().unwrap();
    Ok((circom, pub_in))
}

pub fn gen_proof(
    circuit: CircomCircuit<Bn254>,
    pk: &ProvingKey<Bn254>,
    rng: &mut impl Rng,
) -> Proof<Bn254> {
    Groth16::create_random_proof_with_reduction(circuit, pk, rng).unwrap()
}

pub fn ver_proof(pk: &ProvingKey<Bn254>, proof: &Proof<Bn254>, public_inputs: &Vec<Fr>) -> bool {
    let pvk = prepare_verifying_key(&pk.vk);
    Groth16::verify_proof(&pvk, proof, public_inputs).unwrap()
}