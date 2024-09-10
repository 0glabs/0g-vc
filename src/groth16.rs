use ark_bn254::Bn254;
use ark_circom::{CircomBuilder, CircomCircuit};
use ark_ff::BigInt;
use ark_groth16::{PreparedVerifyingKey, Proof, ProvingKey};
use rand::thread_rng;

use crate::signal::{ProveInput, VerifyInput};

#[cfg(feature = "cuda")]
type Groth16 = ark_groth16::Groth16<ark_bn254::Bn254, ark_groth16::gpu::GpuDomain<ark_bn254::Fr>>;
#[cfg(not(feature = "cuda"))]
type Groth16 = ark_groth16::Groth16<ark_bn254::Bn254>;

pub fn setup(builder: &CircomBuilder<Bn254>) -> Result<ProvingKey<Bn254>, String> {
    let circuit = builder.setup();

    let mut rng = thread_rng();
    Groth16::generate_random_parameters_with_reduction(circuit, &mut rng)
        .map_err(|e| format!("Cannot generate params: {:?}", e))
}

pub fn prove(
    pk: &ProvingKey<Bn254>,
    circom: &CircomBuilder<Bn254>,
    input: impl ProveInput,
) -> Result<Proof<Bn254>, String> {
    let mut circom = circom.clone();
    circom.inputs = input.to_prove_input();

    let circuit = circom
        .build()
        .map_err(|e| format!("Cannot build circuit: {:?}", e))?;

    // debug_vc_public_inputs(&circuit);

    Groth16::create_random_proof_with_reduction(circuit, pk, &mut thread_rng())
        .map_err(|e| format!("Cannot prove: {:?}", e))
}

pub fn verify(
    vk: &PreparedVerifyingKey<Bn254>,
    proof: &Proof<Bn254>,
    public_inputs: &impl VerifyInput,
) -> Result<bool, String> {
    Groth16::verify_proof(&vk, proof, &public_inputs.to_verify_input())
        .map_err(|e| format!("Cannot verify: {:?}", e))
}

#[allow(dead_code)]
pub fn debug_vc_public_inputs(circuit: &CircomCircuit<Bn254>) {
    use ark_ff::PrimeField;
    let inputs: Vec<BigInt<4>> = circuit
        .get_public_inputs()
        .unwrap()
        .iter()
        .map(|x| x.into_bigint())
        .collect();

    let mut slice = &inputs[..];

    fn pull_slice<'a>(slice: &mut &'a [BigInt<4>], len: usize) -> &'a [BigInt<4>] {
        let ans = &(*slice)[..len];
        *slice = &slice[len..];
        ans
    }

    fn pull_hash(slice: &mut &[BigInt<4>]) -> String {
        pull_slice(slice, 2)
            .iter()
            .flat_map(|x| x.as_ref().iter().take(2))
            .map(|x| hex::encode(x.to_le_bytes()))
            .collect()
    }

    fn pull_bytes(slice: &mut &[BigInt<4>], len: usize) -> String {
        let x: Vec<u8> = pull_slice(slice, len)
            .iter()
            .map(|x| x.as_ref()[0] as u8)
            .collect();
        hex::encode(x)
    }

    println!("=====Circom build result ======");
    println!("root: {}", pull_hash(&mut slice));
    println!("leafhash: {}", pull_hash(&mut slice));
    println!("plaintext: {}", pull_bytes(&mut slice, 111));
    println!("ciphertext: {}", pull_bytes(&mut slice, 111));
    println!("paddedleaf: {}", pull_bytes(&mut slice, 256));
    println!("");
}
