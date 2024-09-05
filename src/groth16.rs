use ark_bn254::Bn254;
use ark_circom::CircomBuilder;
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
