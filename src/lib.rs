pub mod args;
pub mod circuit;
pub mod groth16;
#[cfg(feature = "libsnark")]
pub mod libsnark;
pub mod params;
pub mod sample;
mod signal;
pub mod types;
mod utils;

pub use args::{get_zk_task_input, get_zk_task_name};

pub fn warmup_current_thread() {
    #[cfg(feature = "cuda")]
    ark_groth16::init_local_workspace();
}
