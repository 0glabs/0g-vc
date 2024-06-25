// use ark_bn254::Bn254;
// use ark_circom::CircomCircuit;
// use ark_groth16::{create_random_proof, Proof, ProvingKey};
// use ark_ff::ToBytes;

use ark_bn254::{Bn254, Fr};
use ark_circom::{CircomBuilder, CircomCircuit, CircomConfig};
use eyre::Result;
use num_bigint::BigInt;
use std::collections::HashMap;
use std::path::Path;

use ark_groth16::{verifier::prepare_verifying_key, Proof, ProvingKey};
use ark_std::rand::Rng;

use crate::GrothBn;

pub fn calculate_pub_input_without_witness(
    inputs: HashMap<String, Vec<BigInt>>,
) -> Result<HashMap<String, Vec<Fr>>> {
    use ark_ff::PrimeField;
    use num_bigint::BigUint;
    let modulus = <Fr as PrimeField>::MODULUS;

    use num_traits::Signed;
    let pub_inputs = inputs
        .into_iter()
        .map(|(key, values)| {
            let converted_values = values
                .into_iter()
                .map(|pub_in| {
                    let pub_in = if pub_in.sign() == num_bigint::Sign::Minus {
                        let pub_in_abs = pub_in.abs().to_biguint().unwrap();
                        BigUint::from(modulus) - pub_in_abs
                    } else {
                        pub_in.to_biguint().unwrap()
                    };
                    Fr::from(pub_in)
                })
                .collect::<Vec<_>>();
            (key, converted_values)
        })
        .collect::<HashMap<_, _>>();

    Ok(pub_inputs)
}

pub fn cal_witness(
    wtns: impl AsRef<Path>,
    r1cs: impl AsRef<Path>,
    inputs: HashMap<String, Vec<BigInt>>,
) -> Result<(CircomCircuit<Bn254>, Vec<Fr>)> {
    let cfg = CircomConfig::<Bn254>::new(wtns, r1cs)?;
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
    GrothBn::create_random_proof_with_reduction(circuit, pk, rng).unwrap()
}

pub fn ver_proof(pk: &ProvingKey<Bn254>, proof: &Proof<Bn254>, public_inputs: &Vec<Fr>) -> bool {
    let pvk = prepare_verifying_key(&pk.vk);
    GrothBn::verify_proof(&pvk, proof, public_inputs).unwrap()
}

#[cfg(test)]
mod test {
    use crate::crypro::sha;

    use super::*;
    use rand::thread_rng;
    use std::env;

    // #[test]
    // fn simple_example_cal_witness_work() {
    //     let current_dir = env::current_dir().expect("Failed to get current directory");

    //     let mut inputs = HashMap::new();

    //     inputs.insert("in".to_string(), vec![BigInt::from(3), BigInt::from(11)]);

    //     let (_circuit, pub_in) = cal_witness(
    //         current_dir.join("output/example_js/example.wasm"),
    //         current_dir.join("output/example.r1cs"),
    //         inputs,
    //     )
    //     .unwrap();

    //     println!("pub in: {:?}: ", pub_in);
    // }

    // #[test]
    // fn simple_example_gen_proof_work() {
    //     let current_dir = env::current_dir().expect("Failed to get current directory");

    //     let mut inputs = HashMap::new();
    //     inputs.insert("in".to_string(), vec![BigInt::from(3), BigInt::from(11)]);

    //     let (circuit, _) = cal_witness(
    //         current_dir.join("output/example_js/example.wasm"),
    //         current_dir.join("output/example.r1cs"),
    //         inputs,
    //     )
    //     .unwrap();

    //     let mut rng = thread_rng();
    //     let params =
    //         GrothBn::generate_random_parameters_with_reduction(circuit.clone(), &mut rng).unwrap();
    //     let proof = gen_proof(circuit, &params, &mut rng);

    //     println!("proof: {:?}: ", proof);
    // }

    // #[test]
    // fn simple_example_ver_proof_work() {
    //     let current_dir = env::current_dir().expect("Failed to get current directory");

    //     let mut inputs = HashMap::new();
    //     inputs.insert("in".to_string(), vec![BigInt::from(3), BigInt::from(11)]);

    //     let (circuit, public_inputs) = cal_witness(
    //         current_dir.join("output/example_js/example.wasm"),
    //         current_dir.join("output/example.r1cs"),
    //         inputs,
    //     )
    //     .unwrap();

    //     let mut rng = thread_rng();
    //     let params =
    //         GrothBn::generate_random_parameters_with_reduction(circuit.clone(), &mut rng).unwrap();
    //     let proof = gen_proof(circuit, &params, &mut rng);

    //     let result = ver_proof(&params, &proof, &public_inputs);
    //     assert!(result == true);
    // }

    #[test]
    fn sha_example_cal_witness_work() {
        let current_dir = env::current_dir().expect("Failed to get current directory");

        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();

        // cal sha outof circuit
        let out_hash = sha(&random_bytes);
        println!("out_hash: {:?}", out_hash);

        let mut inputs = HashMap::new();
        inputs.insert(
            "pre".to_string(),
            random_bytes.iter().map(|&x| BigInt::from(x)).collect(),
        );

        // 2. 检查电路
        // check_file(&current_dir.join("output/hasher_example_js/hasher_example.wasm"));
        // check_file(&current_dir.join("output/hasher_example.r1cs"));

        let (_circuit, pub_in) = cal_witness(
            current_dir.join("output/hasher_example_js/hasher_example.wasm"),
            current_dir.join("output/hasher_example.r1cs"),
            inputs,
        )
        .unwrap();

        println!("in_hash: {:?}: ", pub_in);
    }
}
