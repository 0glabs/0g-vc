mod crypro;
mod prove_backend;
mod prove_frontend;
mod utils;
mod vc;

use ark_bn254::Bn254;
use ark_groth16::Groth16;
use ark_std::rand::thread_rng;
use chrono::NaiveDate;
use num_bigint::BigInt;
use prove_backend::{cal_witness, gen_proof, ver_proof};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use vc::VC;
type GrothBn = Groth16<Bn254>;

fn check_file(file_path: &PathBuf) {
    if !fs::metadata(file_path).is_ok() {
        eprintln!("Error: File '{:?}' does not exist.", file_path);
        eprintln!("Run `./build_circuit.sh` to construct circuit parameters");
        std::process::exit(1);
    }
}

fn main() {
    // 1. 解析VC Json并计算编码和哈希
    let vc_json = r#"{"name": "Alice", "age": 25, "birth_date": "20000101", "edu_level": 4, "serial_no": "1234567890"}"#;
    let vc: VC = serde_json::from_str(vc_json).unwrap();
    // 1.1. 计算vc编码和哈希
    let (encoded_vc, hash) = vc.hash();
    let circuit_input = encoded_vc.join();
    // 1.2. 计算birthDateThreshold的编码
    let encoded_birth_date = vc.birth_date();
    println!(
        "encoded VC: {:?}, encoded_birth_date: {}",
        encoded_vc,
        // hex::encode(hash),
        encoded_birth_date
    );

    let birth_date_threshold = NaiveDate::parse_from_str("20000304", "%Y%m%d")
        .expect("Invalid birth date string")
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp() as u64;
    println!("birth_date_threshold: {}", birth_date_threshold);

    let current_dir = env::current_dir().expect("Failed to get current directory");

    // 2. 检查电路
    check_file(&current_dir.join("output/check_vc_js/check_vc.wasm"));
    check_file(&current_dir.join("output/check_vc.r1cs"));

    // 3. 计算witness
    let mut inputs = HashMap::new();
    inputs.insert(
        "encodedVC".to_string(),
        circuit_input.iter().map(|&x| BigInt::from(x)).collect(),
    );
    inputs.insert(
        "birthDateThreshold".to_string(),
        vec![BigInt::from(birth_date_threshold)],
    );
    inputs.insert(
        "pathElements".to_string(),
        vec![BigInt::from(0), BigInt::from(0), BigInt::from(0)],
    );
    inputs.insert(
        "pathIndices".to_string(),
        vec![BigInt::from(0), BigInt::from(0), BigInt::from(0)],
    );

    let (circuit, pub_in) = cal_witness(
        current_dir.join("output/check_vc_js/check_vc.wasm"),
        current_dir.join("output/check_vc.r1cs"),
        inputs,
    )
    .unwrap();

    // 3. 生成证明
    let mut rng = thread_rng();
    let params =
        GrothBn::generate_random_parameters_with_reduction(circuit.clone(), &mut rng).unwrap();
    let proof = gen_proof(circuit, &params, &mut rng);

    println!("Proof generated: {:?}", proof);

    // 4. 验证证明
    let result = ver_proof(&params, &proof, &pub_in);
    assert!(result == true);
}
