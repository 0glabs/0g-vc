use ark_bn254::Bn254;
use ark_circom::CircomBuilder;
use ark_groth16::{prepare_verifying_key, PreparedVerifyingKey, Proof, ProvingKey};
use rand::thread_rng;

use crate::types::{Input, PublicInput};

#[cfg(feature = "cuda")]
type Groth16 = ark_groth16::Groth16<ark_bn254::Bn254, ark_groth16::gpu::GpuDomain<ark_bn254::Fr>>;
#[cfg(not(feature = "cuda"))]
type Groth16 = ark_groth16::Groth16<ark_bn254::Bn254>;

pub fn setup(builder: &CircomBuilder<Bn254>) -> (ProvingKey<Bn254>, PreparedVerifyingKey<Bn254>) {
    let circuit = builder.setup();

    let mut rng = thread_rng();
    let pk = Groth16::generate_random_parameters_with_reduction(circuit, &mut rng).unwrap();

    let vk = prepare_verifying_key(&pk.vk);
    (pk, vk)
}

pub fn prove(pk: &ProvingKey<Bn254>, circom: &CircomBuilder<Bn254>, input: Input) -> Proof<Bn254> {
    let mut circom = circom.clone();
    circom.inputs = input.to_inputs();

    Groth16::create_random_proof_with_reduction(circom.build().unwrap(), pk, &mut thread_rng())
        .unwrap()
}

pub fn verify(
    vk: &PreparedVerifyingKey<Bn254>,
    proof: &Proof<Bn254>,
    public_inputs: &PublicInput,
) -> bool {
    Groth16::verify_proof(&vk, proof, &public_inputs.to_public_inputs()).unwrap()
}
