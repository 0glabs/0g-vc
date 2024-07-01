pub mod circuit;
pub mod groth16;
pub mod libsnark;
pub mod params;
pub mod sample;
mod signal;
pub mod types;
mod utils;

pub fn warmup_current_thread() {
    #[cfg(feature = "cuda")]
    ark_groth16::init_local_workspace();
}
