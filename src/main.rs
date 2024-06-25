pub mod crypro;
mod prove_backend;
mod prove_frontend;
mod utils;
mod vc;

use ark_bn254::{Bn254, Fr};
use ark_ff::PrimeField;
use ark_groth16::Groth16;
use ark_std::rand::thread_rng;
use ark_ff::Fp;
use chrono::NaiveDate;
use num_bigint::{BigInt, Sign};
use prove_backend::calculate_pub_input_without_witness;
use prove_backend::{cal_witness, gen_proof, ver_proof};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use vc::{pub_input_without_witness, VC};
type GrothBn = Groth16<Bn254, ark_groth16::gpu::GpuDomain>;
use std::time::Instant;

fn check_file(file_path: &PathBuf) {
    if !fs::metadata(file_path).is_ok() {
        eprintln!("Error: File '{:?}' does not exist.", file_path);
        eprintln!("Run `./build_circuit.sh` to construct circuit parameters");
        std::process::exit(1);
    }
}

// use crate::crypro::{sha, keccak};
// use rand as rn;
// use rand::Rng;
// fn sha_example_cal_witness_work() {
//     let current_dir = env::current_dir().expect("Failed to get current directory");

//     let mut rng = rn::thread_rng();
//     let random_bytes: Vec<u8> = (0..128).map(|i| i as u8).collect();
//     println!("hash_input: {:?}", random_bytes);

//     // cal sha outof circuit
//     let out_hash = keccak(&random_bytes.as_slice());
//     println!("out_hash: {:?}", out_hash);

//     let mut inputs = HashMap::new();
//     inputs.insert(
//         "pre".to_string(),
//         random_bytes.iter().map(|&x| BigInt::from(x)).collect(),
//     );

//     // 2. 检查电路
//     // check_file(&current_dir.join("output/hasher_example_js/hasher_example.wasm"));
//     // check_file(&current_dir.join("output/hasher_example.r1cs"));

//     let (_circuit, pub_in) = cal_witness(
//         current_dir.join("output/hasher_example_js/hasher_example.wasm"),
//         current_dir.join("output/hasher_example.r1cs"),
//         inputs,
//     )
//     .unwrap();
    
//     println!("in_hash: {:?}", pub_in_to_u8_array(pub_in));
// }


// fn pub_in_to_u8_array(pub_in: Vec<Fr>) -> Option<Vec<u8>> {
//     let mut result = Vec::new();
//     let mut current_byte = 0u8;
//     let mut bit_index = 0;

//     for value in pub_in {
//         // println!("{:?}", value.0.0);
//         if is_zero(&value.0.0) {
//             // 是0，不需要修改 current_byte
//         } else if is_one(&value.0.0) {
//             // 是1
//             current_byte |= 1 << bit_index;
//         } else {
//             // 既不是0也不是1，返回 None
//             return None;
//         }

//         bit_index += 1;

//         // 如果已经处理了 8 个 bit，生成一个 byte 并重置
//         if bit_index == 8 {
//             result.push(current_byte);
//             current_byte = 0;
//             bit_index = 0;
//         }
//     }

//     // 如果剩余的 bit 不足 8 个，最后一个 byte 也要加入结果
//     if bit_index > 0 {
//         result.push(current_byte);
//     }

//     Some(result)
// }

// fn is_zero(value: &[u64; 4]) -> bool {
//     value.iter().all(|&x| x == 0)
// }

// fn is_one(value: &[u64; 4]) -> bool {
//     value[0] == 12436184717236109307 && value[1] == 3962172157175319849 && value[2] == 7381016538464732718 && value[3] == 1011752739694698287
// }

// fn main() {
//     sha_example_cal_witness_work();
// }

fn main() {
    #[cfg(feature = "cuda")]
    ark_groth16::init_local_workspace();
    // 1. 解析VC Json并计算编码和哈希
    let vc_json = r#"{"name": "Alice", "age": 25, "birth_date": "20000101", "edu_level": 4, "serial_no": "1234567890"}"#;
    let vc: VC = serde_json::from_str(vc_json).unwrap();
    // 1.1. 计算vc编码和哈希
    let (encoded_vc, hash) = vc.hash();
    // println!("leafHash: {:?}", hash);
    let circuit_input = encoded_vc.join();
    // 1.2. 计算birthDateThreshold的编码
    let encoded_birth_date = vc.birth_date();
    println!(
        "encoded VC: {:?}, encoded_birth_date: {}",
        encoded_vc,
        // hex::encode(hash),
        encoded_birth_date
    );

    // compute leafHash outof circuit
    // let mut verifier_inputs = HashMap::new();
    // let leaf_hash = BigInt::from_bytes_le(Sign::Plus, &hash);
    // verifier_inputs.insert(
    //     "leafHash".to_string(),
    //     vec![leaf_hash],
    // );
    // let verifier_pub_in = calculate_pub_input_without_witness(verifier_inputs.clone()).unwrap();
    // println!("verifier_pub_in: {:?}", verifier_pub_in);

    let birth_data_threshold_user_input = String::from("20000304");
    let birth_date_threshold = NaiveDate::parse_from_str(birth_data_threshold_user_input.as_str(), "%Y%m%d")
        .expect("Invalid birth date string")
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp() as u64;
    // println!("birth_date_threshold: {}", birth_date_threshold);
    // verifier calculate public input
    let verifier_pub_in = pub_input_without_witness(birth_data_threshold_user_input, hash);
    println!("verifier public inputs: {:?}", verifier_pub_in);

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
    println!("BigInt::from(birth_date_threshold): {:?}", BigInt::from(birth_date_threshold));
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
    println!("public inputs:{:?}", pub_in);

    // 3. 生成证明
    let mut rng = thread_rng();
    let params =
        GrothBn::generate_random_parameters_with_reduction(circuit.clone(), &mut rng).unwrap();
    let now = Instant::now();
    let proof = gen_proof(circuit, &params, &mut rng);
    let dur = now.elapsed().as_millis();
    #[cfg(not(feature = "cuda"))]
    println!("CPU took {}ms.", dur);
    #[cfg(feature = "cuda")]
    println!("GPU took {}ms.", dur);

    println!("Proof generated: {:?}", proof);

    // 4. 验证证明
    let result = ver_proof(&params, &proof, &pub_in);
    assert!(result == true);
}
