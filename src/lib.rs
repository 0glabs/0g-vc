pub mod circuit;
pub mod groth16;
mod signal;
pub mod types;
mod utils;

pub fn warmup_current_thread() {
    #[cfg(feature = "cuda")]
    ark_groth16::init_local_workspace();
}
