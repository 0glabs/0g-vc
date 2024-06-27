use std::{fs, path::PathBuf};

use ark_bn254::Bn254;
use ark_circom::{CircomBuilder, CircomConfig};

pub fn check_file(file_path: &PathBuf) {
    if !fs::metadata(file_path).is_ok() {
        eprintln!("Error: File '{:?}' does not exist.", file_path);
        eprintln!("Run `./build_circuit.sh` to construct circuit parameters");
        std::process::exit(1);
    }
}

pub fn circom_builder(current_dir: &PathBuf, name: &str) -> CircomBuilder<Bn254> {
    let wtns = current_dir.join(format!("{name}_js/{name}.wasm", name = name));
    let r1cs = current_dir.join(format!("{name}.r1cs", name = name));
    check_file(&wtns);
    check_file(&r1cs);

    let circom_config = CircomConfig::<Bn254>::new(wtns, r1cs).expect("Cannot parse circom");
    CircomBuilder::new(circom_config)
}
